//! Protocol-1 host actions and conservative save acknowledgement state.
use egui::{Context, Id};
use serde_json::Value;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug)]
pub enum Action {
    Open { name: String, bytes: Vec<u8> },
    Rename(String),
    RequestSave,
    Saved { ok: bool, error: Option<String> },
}
#[derive(Clone, Debug)]
pub struct Ticket {
    pub document: u64,
    pub revision: u64,
    pub started: f64,
    pub bytes: Option<Arc<Vec<u8>>>,
}
#[derive(Default, Debug)]
pub struct State {
    pub actions: VecDeque<Action>,
    pub pending: Option<Ticket>,
    pub unconfirmed: Vec<u64>,
    pub error: Option<String>,
    pub force_save: bool,
    pub renamed: Vec<u64>,
    // Protocol 1 has no save identifier. An expired acknowledgement must never confirm a later save.
    stale_acks: usize,
}
impl State {
    pub fn begin(&mut self, ticket: Ticket) -> Result<(), String> {
        if self.pending.is_some() {
            return Err("Waiting for the host to confirm the previous save".into());
        }
        if !self.unconfirmed.contains(&ticket.document) {
            self.unconfirmed.push(ticket.document);
        }
        self.error = None;
        self.pending = Some(ticket);
        Ok(())
    }
    pub fn acknowledge(&mut self, ok: bool, error: Option<String>) -> Option<Ticket> {
        if self.stale_acks > 0 {
            self.stale_acks -= 1;
            return None;
        }
        let ticket = self.pending.take()?;
        if ok {
            Some(ticket)
        } else {
            self.error = Some(error.unwrap_or_else(|| "The host could not store the document".into()));
            None
        }
    }
    pub fn complete(&mut self, document: u64) {
        self.unconfirmed.retain(|id| *id != document);
    }
    pub fn stored_normally(&mut self, document: u64) {
        if self.pending.as_ref().is_some_and(|p| p.document == document) {
            self.pending = None;
            self.stale_acks = self.stale_acks.saturating_add(1);
        }
        self.complete(document);
        self.error = None;
    }
    pub fn expire(&mut self, now: f64) {
        if self.pending.as_ref().is_some_and(|p| now - p.started >= 30.0) {
            self.pending = None;
            self.stale_acks = self.stale_acks.saturating_add(1);
            self.error = Some("The host did not confirm the save within 30 seconds".into());
        }
    }
}
fn key() -> Id {
    Id::unique("embedded-host-runtime")
}
pub fn state(ctx: &Context) -> Arc<Mutex<State>> {
    ctx.data_mut(|d| {
        if let Some(s) = d.get_temp::<Arc<Mutex<State>>>(key()) {
            s
        } else {
            let s = Arc::new(Mutex::new(State::default()));
            d.insert_temp(key(), s.clone());
            s
        }
    })
}
pub fn rename(value: &Value) -> Option<String> {
    let s = value.as_str()?;
    (1..=200).contains(&s.chars().count()).then_some(())?;
    if (s.trim().is_empty() || matches!(s, "." | "..")) || s.chars().any(|c| c.is_control() || matches!(c, '/' | '\\')) {
        return None;
    }
    Some(s.to_string())
}
pub fn branding(value: &Value) -> Option<String> {
    let s = value.as_str()?;
    if !(1..=40).contains(&s.chars().count()) || s.trim().is_empty() || s.chars().any(char::is_control) {
        return None;
    }
    Some(s.to_string())
}
pub fn parse(kind: &str, value: &Value) -> Option<Action> {
    match kind {
        "rename" => rename(value.get("name")?).map(Action::Rename),
        "requestSave" => Some(Action::RequestSave),
        "saved" => {
            let ok = value.get("ok")?.as_bool()?;
            let error = match value.get("error") {
                None | Some(Value::Null) => None,
                Some(v) => Some(v.as_str()?.chars().filter(|c| !c.is_control()).take(200).collect()),
            };
            Some(Action::Saved { ok, error })
        }
        _ => None,
    }
}
pub fn enqueue(ctx: &Context, kind: &str, input: &Value) {
    if let Some(action) = parse(kind, input) {
        let state = state(ctx);
        let mut s = state.lock().unwrap_or_else(|e| e.into_inner());
        if s.actions.len() < 128 {
            s.actions.push_back(action);
        }
        ctx.request_repaint();
    }
}
pub fn notice(ctx: &Context) {
    let shared = state(ctx);
    let message = shared.lock().unwrap_or_else(|e| e.into_inner()).error.clone();
    if let Some(message) = message {
        let mut open = true;
        egui::Window::new(tl!("Not saved")).id(Id::unique("host-not-saved")).collapsible(false).open(&mut open).show(ctx, |ui| {
            ui.label(message);
        });
        if !open {
            shared.lock().unwrap_or_else(|e| e.into_inner()).error = None;
        }
    }
}

pub fn enqueue_open(ctx: &Context, name: String, bytes: Vec<u8>) {
    let shared = state(ctx);
    let mut s = shared.lock().unwrap_or_else(|e| e.into_inner());
    if s.actions.len() < 128 {
        s.actions.push_back(Action::Open { name, bytes });
    }
    ctx.request_repaint();
}

thread_local! {static START:web_time::Instant=web_time::Instant::now();}
pub fn now() -> f64 {
    START.with(|start| start.elapsed().as_secs_f64())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn ticket(revision: u64) -> Ticket {
        Ticket { document: 1, revision, started: 0.0, bytes: None }
    }
    #[test]
    fn validation_is_bounded_and_strict() {
        for s in ["", "../file", "a/b", "a\\b", "a\0b", "   "] {
            assert!(rename(&Value::from(s)).is_none());
        }
        assert!(rename(&Value::from("a".repeat(201))).is_none());
        assert!(branding(&Value::from("a".repeat(41))).is_none());
        assert_eq!(rename(&Value::from("مرحبا.docx")).as_deref(), Some("مرحبا.docx"));
        assert!(parse("saved", &serde_json::json!({"ok":"true"})).is_none());
        assert!(parse("unknown", &Value::Null).is_none());
    }
    #[test]
    fn ack_failure_timeout_and_stale_acks_never_confirm_newer_work() {
        let mut s = State::default();
        s.begin(ticket(4)).unwrap();
        assert!(s.begin(ticket(5)).is_err());
        assert_eq!(s.acknowledge(true, None).unwrap().revision, 4);
        assert!(s.acknowledge(true, None).is_none());
        s.begin(ticket(6)).unwrap();
        assert!(s.acknowledge(false, Some("storage failed".into())).is_none());
        assert_eq!(s.error.as_deref(), Some("storage failed"));
        s.begin(ticket(7)).unwrap();
        s.expire(29.99);
        assert!(s.pending.is_some());
        s.expire(30.0);
        assert!(s.pending.is_none());
        assert!(s.error.is_some());
        s.begin(ticket(8)).unwrap();
        assert!(s.acknowledge(true, None).is_none());
        assert_eq!(s.pending.as_ref().unwrap().revision, 8);
    }
}
