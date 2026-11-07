//! Validated, per-iframe host configuration. No transport or networking lives in the UI layer.
use crate::{i18n, theme::Tokens};
use egui::{Color32, Context, Id};
use serde_json::{Value, json};
use std::collections::BTreeMap;

pub const ACTIVE: bool = cfg!(all(feature = "embedded", target_arch = "wasm32"));
#[derive(Clone, Default)]
pub struct Config {
    pub color_scheme: Option<String>,
    pub locale: Option<String>,
    pub branding: Option<String>,
    pub ack_save: bool,
    theme: BTreeMap<String, Value>,
    chrome: BTreeMap<String, bool>,
}
fn id() -> Id {
    Id::unique("embedded-host-config")
}
pub fn current(ctx: &Context) -> Config {
    ctx.data(|d| d.get_temp::<Config>(id())).unwrap_or_default()
}

/// Unspecified controls remain visible. Native builds preserve their original behavior.
pub fn visible(ctx: &Context, key: &str) -> bool {
    !ACTIVE || current(ctx).chrome.get(key).copied().unwrap_or(true)
}

/// Only canonical #RGB, #RRGGBB and #RRGGBBAA strings are accepted; no CSS or executable text.
fn color(value: &Value) -> Option<Color32> {
    let text = value.as_str()?.strip_prefix('#')?;
    if !text.is_ascii() || !text.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    let expanded = if text.len() == 3 { text.chars().flat_map(|c| [c, c]).collect::<String>() } else { text.to_string() };
    if !matches!(expanded.len(), 6 | 8) {
        return None;
    }
    let n = u32::from_str_radix(&expanded, 16).ok()?;
    let rgba = if expanded.len() == 6 { (n << 8) | 255 } else { n };
    Some(Color32::from_rgba_unmultiplied((rgba >> 24) as u8, (rgba >> 16) as u8, (rgba >> 8) as u8, rgba as u8))
}
fn canonical(key: &str) -> String {
    let mut out = String::new();
    for c in key.chars() {
        if c.is_ascii_uppercase() {
            out.push('_');
            out.push(c.to_ascii_lowercase());
        } else {
            out.push(c);
        }
    }
    out
}
fn valid_token(key: &str, value: &Value) -> bool {
    match key {
        "dialog" | "popup" | "toast" | "tooltip" | "toolstrip" | "scrollbar" | "scrollbar_thumb" | "backstage_nav" | "backstage_content" | "menu"
        | "on_accent" | "success" | "warning" | "danger" => color(value).is_some(),
        "titlebar" | "chrome" | "panel" | "pasteboard" | "card" | "border" | "divider" | "text" | "text_muted" | "text_faint" | "icon" | "hover"
        | "pressed" | "selected" | "accent" | "accent_text" | "accent_soft" | "field" | "badge_new" | "page_shadow" => color(value).is_some(),
        "radius" => value.as_u64().is_some_and(|n| n <= 32),
        _ => false,
    }
}
/// Parse a bounded object and report every invalid/unknown field without changing its old value.
/// Configurations merge, so a later message can change one control or restore it with true.
pub fn configure(ctx: &Context, input: &Value) -> Value {
    let mut config = current(ctx);
    let mut applied = Vec::<String>::new();
    let mut ignored = Vec::<String>::new();
    if let Some(fields) = input.as_object().filter(|o| o.len() <= 128) {
        for (key, value) in fields {
            let good = match key.as_str() {
                "type" => continue,
                "ackSave" => {
                    if let Some(on) = value.as_bool() {
                        config.ack_save = on;
                        true
                    } else {
                        false
                    }
                }
                "branding" => {
                    if let Some(entries) = value.as_object() {
                        for (k, v) in entries {
                            if k == "name"
                                && let Some(name) = crate::host_state::branding(v)
                            {
                                config.branding = Some(name);
                                applied.push("branding.name".into());
                                continue;
                            }
                            ignored.push(format!("branding.{k}"));
                        }
                        continue;
                    } else {
                        false
                    }
                }
                "colorScheme" => {
                    if let Some(v) = value.as_str().filter(|v| matches!(*v, "light" | "dark" | "system")) {
                        config.color_scheme = Some(v.into());
                        true
                    } else {
                        false
                    }
                }
                "locale" => {
                    if let Some(v) = value.as_str().filter(|v| i18n::Lang::from_code(v).is_some()) {
                        config.locale = Some(v.into());
                        true
                    } else {
                        false
                    }
                }
                "theme" | "chrome" => {
                    if let Some(entries) = value.as_object().filter(|o| o.len() <= 128) {
                        for (name, v) in entries {
                            let path = format!("{key}.{name}");
                            let token = canonical(name);
                            let ok = if key == "theme" {
                                if valid_token(&token, v) {
                                    config.theme.insert(token, v.clone());
                                    true
                                } else {
                                    false
                                }
                            } else if CHROME_KEYS.contains(&name.as_str()) {
                                if let Some(on) = v.as_bool() {
                                    config.chrome.insert(name.clone(), on);
                                    true
                                } else {
                                    false
                                }
                            } else {
                                false
                            };
                            if ok { applied.push(path) } else { ignored.push(path) }
                        }
                        continue;
                    } else {
                        false
                    }
                }
                _ => false,
            };
            if good { applied.push(key.clone()) } else { ignored.push(key.clone()) }
        }
    } else {
        ignored.push("message".into());
    }
    ctx.data_mut(|d| {
        d.insert_temp(id(), config);
        if !applied.is_empty() {
            d.insert_temp(Id::unique("embedded-config-changed"), true);
        }
    });
    ctx.request_repaint();
    json!({"applied":applied,"ignored":ignored})
}
pub fn tokens(ctx: &Context, t: Tokens) -> Tokens {
    if !ACTIVE {
        return t;
    }
    configured_tokens(t, current(ctx))
}
fn configured_tokens(mut t: Tokens, config: Config) -> Tokens {
    {
        for (key, value) in config.theme.clone() {
            match key.as_str() {
                "titlebar" => {
                    if let Some(c) = color(&value) {
                        t.titlebar = c;
                    }
                }
                "chrome" => {
                    if let Some(c) = color(&value) {
                        t.chrome = c;
                    }
                }
                "panel" => {
                    if let Some(c) = color(&value) {
                        t.panel = c;
                    }
                }
                "pasteboard" => {
                    if let Some(c) = color(&value) {
                        t.pasteboard = c;
                    }
                }
                "card" => {
                    if let Some(c) = color(&value) {
                        t.card = c;
                    }
                }
                "border" => {
                    if let Some(c) = color(&value) {
                        t.border = c;
                    }
                }
                "divider" => {
                    if let Some(c) = color(&value) {
                        t.divider = c;
                    }
                }
                "text" => {
                    if let Some(c) = color(&value) {
                        t.text = c;
                    }
                }
                "text_muted" => {
                    if let Some(c) = color(&value) {
                        t.text_muted = c;
                    }
                }
                "text_faint" => {
                    if let Some(c) = color(&value) {
                        t.text_faint = c;
                    }
                }
                "icon" => {
                    if let Some(c) = color(&value) {
                        t.icon = c;
                    }
                }
                "hover" => {
                    if let Some(c) = color(&value) {
                        t.hover = c;
                    }
                }
                "pressed" => {
                    if let Some(c) = color(&value) {
                        t.pressed = c;
                    }
                }
                "selected" => {
                    if let Some(c) = color(&value) {
                        t.selected = c;
                    }
                }
                "accent" => {
                    if let Some(c) = color(&value) {
                        t.accent = c;
                    }
                }
                "accent_text" => {
                    if let Some(c) = color(&value) {
                        t.accent_text = c;
                    }
                }
                "accent_soft" => {
                    if let Some(c) = color(&value) {
                        t.accent_soft = c;
                    }
                }
                "field" => {
                    if let Some(c) = color(&value) {
                        t.field = c;
                    }
                }
                "badge_new" => {
                    if let Some(c) = color(&value) {
                        t.badge_new = c;
                    }
                }
                "page_shadow" => {
                    if let Some(c) = color(&value) {
                        t.page_shadow = c;
                    }
                }
                "radius" => {
                    if let Some(n) = value.as_u64().filter(|n| *n <= 32) {
                        t.radius = n as u8;
                    }
                }
                _ => {}
            }
        }
    }
    {
        t.dialog = t.card;
        t.popup = t.card;
        t.toast = t.card;
        t.tooltip = t.card;
        t.toolstrip = t.chrome;
        t.scrollbar = t.panel;
        t.scrollbar_thumb = t.hover;
        t.backstage_nav = t.chrome;
        t.backstage_content = t.panel;
        t.menu = t.card;
        t.on_accent = t.text;
        t.success = t.accent;
        t.warning = t.accent;
        t.danger = t.accent;
        for (key, value) in config.theme.clone() {
            if let Some(c) = color(&value) {
                match key.as_str() {
                    "dialog" => t.dialog = c,
                    "popup" => t.popup = c,
                    "toast" => t.toast = c,
                    "tooltip" => t.tooltip = c,
                    "toolstrip" => t.toolstrip = c,
                    "scrollbar" => t.scrollbar = c,
                    "scrollbar_thumb" => t.scrollbar_thumb = c,
                    "backstage_nav" => t.backstage_nav = c,
                    "backstage_content" => t.backstage_content = c,
                    "menu" => t.menu = c,
                    "on_accent" => t.on_accent = c,
                    "success" => t.success = c,
                    "warning" => t.warning = c,
                    "danger" => t.danger = c,
                    _ => {}
                }
            }
        }
    }
    t
}
const CHROME_KEYS: &[&str] = &["fileNew", "fileClose", "titlebar", "homeTab", "openTab", "modebar"];
pub fn apply(app: &mut crate::PdfCraftApp, ctx: &Context) {
    app.ctx = Some(ctx.clone());
    let brand = current(ctx).branding.unwrap_or_else(|| "Editor".into());
    BRAND.with(|n| *n.borrow_mut() = brand);
    process_actions(app, ctx);
    if ACTIVE {
        if let crate::QuickTool::Comment(tool) = app.quick_tool
            && !pdfcraft_engine::catalog::embedded_command_ready(tool.command())
        {
            app.quick_tool = crate::QuickTool::Select;
        }
        for (index, tool) in app.comment_prefs.group_tool.iter_mut().enumerate() {
            if !pdfcraft_engine::catalog::embedded_command_ready(tool.command())
                && let Some(replacement) =
                    crate::comments::GROUPS.get(index).and_then(|g| g.iter().find(|t| pdfcraft_engine::catalog::embedded_command_ready(t.command())))
            {
                *tool = *replacement;
            }
        }
    }
    let c = current(ctx);
    if let Some(s) = c.color_scheme {
        app.theme_preference = match s.as_str() {
            "dark" => crate::theme::ThemePreference::Dark,
            "light" => crate::theme::ThemePreference::Light,
            _ => crate::theme::ThemePreference::System,
        };
    }
    if let Some(l) = c.locale {
        app.language = l;
    }
    if ctx.data_mut(|d| d.remove_temp::<bool>(Id::unique("embedded-config-changed"))).unwrap_or(false) {
        crate::theme::apply(ctx, app.theme);
    }
}
pub fn command_allowed(ctx: &Context, id: &str) -> bool {
    if !ACTIVE {
        return true;
    }
    command_policy(ctx, id)
}
fn command_policy(ctx: &Context, id: &str) -> bool {
    if id.starts_with("ai.") || id.starts_with("view.theme") || matches!(id, "ui.theme" | "ui.darkMode" | "references.researcher") {
        return false;
    }
    if (id.starts_with("file.new") || id == "create.blank") && !configured_visible(ctx, "fileNew") {
        return false;
    }
    if id.starts_with("file.close") && !configured_visible(ctx, "fileClose") {
        return false;
    }
    if !pdfcraft_engine::catalog::embedded_command_ready(id) {
        return false;
    }

    if matches!(id, "help.discord" | "help.github" | "help.website" | "help.app_page" | "help.check_updates") {
        return false;
    }
    if id == "file.open" && !configured_visible(ctx, "openTab") {
        return false;
    }
    true
}

fn configured_visible(ctx: &Context, key: &str) -> bool {
    current(ctx).chrome.get(key).copied().unwrap_or(true)
}

thread_local! {static BRAND: std::cell::RefCell<String> = std::cell::RefCell::new("Editor".into());}
pub fn brand_text(text: &str) -> String {
    BRAND.with(|brand| {
        let brand = brand.borrow();
        let mut out = String::new();
        let mut rest = text;
        let names = ["WordCraft", "PdfCraft", "PrintCraft", "ArtCraft"];
        while let Some((at, name)) = names.iter().filter_map(|name| rest.find(name).map(|at| (at, *name))).min_by_key(|(at, _)| *at) {
            out.push_str(&rest[..at]);
            out.push_str(&brand);
            rest = &rest[at + name.len()..];
        }
        out.push_str(rest);
        out
    })
}
pub fn name() -> String {
    BRAND.with(|n| n.borrow().clone())
}
pub fn ack_enabled(ctx: &Context) -> bool {
    ACTIVE && current(ctx).ack_save
}
pub fn begin_save(ctx: &Context, document: u64, revision: u64, bytes: Option<std::sync::Arc<Vec<u8>>>) -> Result<bool, String> {
    if !ack_enabled(ctx) {
        return Ok(false);
    }
    let now = crate::host_state::now();
    crate::host_state::state(ctx).lock().unwrap_or_else(|e| e.into_inner()).begin(crate::host_state::Ticket {
        document,
        revision,
        started: now,
        bytes,
    })?;
    ctx.request_repaint_after(std::time::Duration::from_millis(100));
    Ok(true)
}
pub fn unconfirmed(ctx: &Context, document: u64) -> bool {
    crate::host_state::state(ctx).lock().unwrap_or_else(|e| e.into_inner()).unconfirmed.contains(&document)
}
pub fn normal_saved(ctx: &Context, document: u64) {
    let state = crate::host_state::state(ctx);
    let mut state = state.lock().unwrap_or_else(|e| e.into_inner());
    state.stored_normally(document);
}
pub fn force_save(ctx: &Context) -> bool {
    crate::host_state::state(ctx).lock().unwrap_or_else(|e| e.into_inner()).force_save
}

fn process_actions(app: &mut crate::PdfCraftApp, ctx: &Context) {
    use crate::host_state::{Action, state};
    let shared = state(ctx);
    let actions = {
        let mut s = shared.lock().unwrap_or_else(|e| e.into_inner());
        s.expire(crate::host_state::now());
        if s.pending.is_some() {
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        }
        std::mem::take(&mut s.actions)
    };
    for action in actions {
        match action {
            Action::Open { name, bytes } => {
                let existing: Vec<_> = app.views.iter().map(|v| v.id).collect();
                if let Err(error) = app.open_bytes(&name, None, bytes) {
                    app.notify(error);
                } else if let Some(id) = app.active.and_then(|i| app.views.get(i)).map(|v| v.id).filter(|id| !existing.contains(id)) {
                    let _ = app.session.accept_host_baseline(id);
                    ctx.data_mut(|d| d.insert_temp(Id::unique("host-open-dirty-reset"), true));
                }
            }
            Action::Rename(name) => {
                if let Some(id) = app.active.and_then(|i| app.views.get(i)).map(|v| v.id) {
                    let _ = app.session.rename_host_file(id, name);
                    let mut s = shared.lock().unwrap_or_else(|e| e.into_inner());
                    if !s.renamed.contains(&id.0) {
                        s.renamed.push(id.0);
                    }
                }
            }
            Action::RequestSave => {
                if shared.lock().unwrap_or_else(|e| e.into_inner()).pending.is_some() {
                    continue;
                }
                shared.lock().unwrap_or_else(|e| e.into_inner()).force_save = true;
                app.save_active(crate::editing::SaveTarget::InPlace);
                shared.lock().unwrap_or_else(|e| e.into_inner()).force_save = false;
            }
            Action::Saved { ok, error } => {
                let ticket = shared.lock().unwrap_or_else(|e| e.into_inner()).acknowledge(ok, error);
                if let Some(ticket) = ticket {
                    let id = pdfcraft_engine::DocId(ticket.document);
                    let same = app.session.get(id).is_some_and(|d| d.edit_generation() == ticket.revision);
                    if same {
                        if let Some(bytes) = ticket.bytes {
                            match app.session.mark_saved(id, bytes, None) {
                                Ok(()) => shared.lock().unwrap_or_else(|e| e.into_inner()).complete(ticket.document),
                                Err(e) => shared.lock().unwrap_or_else(|e| e.into_inner()).error = Some(e.to_string().chars().take(200).collect()),
                            }
                        }
                    } else {
                        shared.lock().unwrap_or_else(|e| e.into_inner()).complete(ticket.document);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn validates_values_and_merges_without_resetting_good_values() {
        let ctx = Context::default();
        let result = configure(
            &ctx,
            &json!({"colorScheme":"dark","locale":"en","theme":{"accent":"#123456","accentText":"#abc","noSuchToken":"#fff"},"chrome":{"titlebar":false,"openTab":"no","unknown":true},"unknown":true}),
        );
        assert_eq!(result["applied"], json!(["chrome.titlebar", "colorScheme", "locale", "theme.accent", "theme.accentText"]));
        assert_eq!(result["ignored"], json!(["chrome.openTab", "chrome.unknown", "theme.noSuchToken", "unknown"]));
        configure(&ctx, &json!({"colorScheme":false,"locale":"not-a-locale","theme":{"accent":"url(bad)"}}));
        let saved = current(&ctx);
        assert_eq!(saved.color_scheme.as_deref(), Some("dark"));
        assert_eq!(saved.locale.as_deref(), Some("en"));
        assert_eq!(saved.chrome.get("titlebar"), Some(&false));
        assert_eq!(saved.theme.get("accent"), Some(&json!("#123456")));
        configure(&ctx, &json!({"chrome":{"titlebar":true}}));
        assert_eq!(current(&ctx).chrome.get("titlebar"), Some(&true));
    }
    #[test]
    fn rejects_bad_colors_and_nonobjects() {
        for v in [json!("#12"), json!("#12345g"), json!("red"), json!("#１２３"), json!(false), json!(null)] {
            assert!(color(&v).is_none());
        }
        assert!(color(&json!("#aBc")).is_some());
        assert!(color(&json!("#12345678")).is_some());
        assert_eq!(configure(&Context::default(), &json!(null))["ignored"], json!(["message"]));
    }
}

#[cfg(test)]
mod policy_tests {
    use super::*;
    #[test]
    fn hidden_open_and_external_commands_are_blocked() {
        let ctx = Context::default();
        configure(&ctx, &json!({"chrome":{"openTab":false}}));
        for id in ["file.open", "help.discord", "help.github", "help.website", "help.app_page", "help.check_updates"] {
            assert!(!command_policy(&ctx, id), "{id}");
        }
        assert!(command_policy(&ctx, "file.save"));
        configure(&ctx, &json!({"chrome":{"openTab":true}}));
        assert!(command_policy(&ctx, "file.open"));
    }
}

#[cfg(test)]
mod host_open_tests {
    #[test]
    fn opening_generic_bytes_establishes_a_clean_baseline() {
        let mut app = crate::PdfCraftApp::new();
        app.open_bytes("blank.pdf", None, include_bytes!("../../../embedded-test/blank.pdf").to_vec()).unwrap();
        let id = app.views[app.active.unwrap()].id;
        app.session.accept_host_baseline(id).unwrap();
        assert!(!app.session.get(id).unwrap().dirty);
    }
}

#[cfg(test)]
mod host_configuration_tests {
    use super::*;
    #[test]
    fn branding_and_ack_values_are_validated_and_merge_live() {
        let ctx = Context::default();
        let reply = configure(&ctx, &json!({"branding":{"name":"Generic Editor","unknown":true},"ackSave":true}));
        assert_eq!(reply["applied"], json!(["ackSave", "branding.name"]));
        assert_eq!(reply["ignored"], json!(["branding.unknown"]));
        configure(&ctx, &json!({"branding":{"name":"x".repeat(41)},"ackSave":"yes"}));
        assert_eq!(current(&ctx).branding.as_deref(), Some("Generic Editor"));
        assert!(current(&ctx).ack_save);
        configure(&ctx, &json!({"ackSave":false}));
        assert!(!current(&ctx).ack_save);
    }
}

#[cfg(test)]
mod host_ack_revision_tests {
    use super::*;
    use crate::host_state::{Action, Ticket, state};
    #[test]
    fn acknowledgement_clears_only_the_saved_revision() {
        let ctx = Context::default();
        let mut app = crate::PdfCraftApp::new();
        let bytes = std::sync::Arc::new(include_bytes!("../../../embedded-test/blank.pdf").to_vec());
        app.open_bytes("blank.pdf", None, bytes.as_ref().clone()).unwrap();
        let id = app.views[app.active.unwrap()].id;
        let shared = state(&ctx);
        let ticket = Ticket {
            document: id.0,
            revision: app.session.get(id).unwrap().edit_generation(),
            started: crate::host_state::now(),
            bytes: Some(bytes),
        };
        shared.lock().unwrap().begin(ticket.clone()).unwrap();
        shared.lock().unwrap().actions.push_back(Action::Saved { ok: true, error: None });
        process_actions(&mut app, &ctx);
        assert!(!app.session.get(id).unwrap().dirty);
        assert!(!unconfirmed(&ctx, id.0));
        shared.lock().unwrap().begin(ticket).unwrap();
        app.session.apply(id, pdfcraft_engine::Edit::InsertBlankPage { at: 1, width: 200.0, height: 300.0 }).unwrap();
        shared.lock().unwrap().actions.push_back(Action::Saved { ok: true, error: None });
        process_actions(&mut app, &ctx);
        assert!(app.session.get(id).unwrap().dirty, "an older stored snapshot cannot clear a newer edit");
    }
}

#[cfg(test)]
mod round3_tests {
    use super::*;
    #[test]
    fn validates_every_surface_slot() {
        let ctx = Context::default();
        for key in ["dialog", "popup", "toast", "tooltip", "toolstrip", "scrollbar", "scrollbarThumb", "backstageNav", "backstageContent"] {
            let result = configure(&ctx, &json!({"theme":{key:"#fd12ab"}}));
            assert_eq!(result["applied"], json!([format!("theme.{key}")]));
            let result = configure(&ctx, &json!({"theme":{key:"invalid"}}));
            assert_eq!(result["ignored"], json!([format!("theme.{key}")]));
        }
    }
    #[test]
    fn new_close_and_removed_preferences_are_not_dispatchable() {
        let ctx = Context::default();
        configure(&ctx, &json!({"chrome":{"fileNew":false,"fileClose":false}}));
        for id in ["create.blank", "file.close", "file.close_all", "view.theme", "ai.summary", "export.xlsx", "js.console"] {
            assert!(!command_policy(&ctx, id), "{id}");
        }
        assert!(command_policy(&ctx, "file.save"));
    }
}

#[cfg(test)]
mod host_palette_tests {
    use super::*;
    #[test]
    fn each_new_region_uses_its_own_validated_host_color() {
        let ctx = Context::default();
        configure(
            &ctx,
            &json!({"theme":{"dialog":"#ffb2ed","popup":"#b7ffce","toast":"#ffcc85","tooltip":"#e7c7ff","toolstrip":"#95f4f0","scrollbar":"#ffbc7d","scrollbarThumb":"#6c138c","backstageNav":"#79edaa","backstageContent":"#fcdf95"}}),
        );
        let t = configured_tokens(Tokens::for_kind(crate::theme::ThemeKind::Light), current(&ctx));
        for (actual, expected) in [
            (t.dialog, "#ffb2ed"),
            (t.popup, "#b7ffce"),
            (t.toast, "#ffcc85"),
            (t.tooltip, "#e7c7ff"),
            (t.toolstrip, "#95f4f0"),
            (t.scrollbar, "#ffbc7d"),
            (t.scrollbar_thumb, "#6c138c"),
            (t.backstage_nav, "#79edaa"),
            (t.backstage_content, "#fcdf95"),
        ] {
            assert_eq!(Some(actual), color(&json!(expected)), "{expected}");
        }
    }
}
