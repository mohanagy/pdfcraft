# Embedded web profile — rounds 1 and 2

Local branch `embed/profile`, based on `8bb0528b`. No pushes. Application version 0.18.1; host protocol remains **1**.

The normal feature-off application keeps its existing behavior. Embedded-specific UI, host and resource changes are gated to wasm32; selecting the feature on native does not activate the embedded interface. `embedded-lite` is a separate opt-in resource variant and implies `embedded`. No font binaries were added to either app repository. Package licence/notice files are retained; generic fixtures are attributed.

## Build and deliverables

Use Rust installed through rustup, the wasm32-unknown-unknown target, Trunk and wasm-bindgen 0.2.129 (matching Cargo.lock). The toolchain used here is Rust 1.99.1, Trunk 0.22.1 and Binaryen 133. The HTML requests wasm-opt `z` (`-Oz`).

```sh
# From apps/pdfcraft-web; use absolute paths for these build inputs.
export CRAFT_I18N_DIR=/absolute/path/to/the-checkout-pinned-in-craft-i18n.rev
export CRAFT_I18N_REQUIRED=1
export CRAFT_FONTS_DIR=/absolute/path/to/licensed-craft-fonts
trunk build --skip-version-check --release --locked --features embedded \
  --html-output index.html index-embedded.html
# Optional, separate output; keeps dist/web as the full embedded build.
trunk build --skip-version-check --release --locked --features embedded-lite \
  --dist ../../dist/web-lite --html-output index.html index-embedded.html
```

`packaging/web/build-embedded.sh` builds/packages the full embedded site. Default `index.html` and normal Trunk commands still build the standard app; the explicit `index-embedded.html` is necessary to white-label the pre-WASM loading/title. Fonts input is pinned at `b076417b5880bd4154bd22f00695b1d28c6bea83`; translation pin is the repository's `craft-i18n.rev`. Full builds retain the previous round's font/data inputs.

Full site: `dist/web`. Optional measured variant: `dist/web-lite`. Full release ZIP retains the previous name/layout, `dist/embedded-release/pdfcraft-web-0.18.1.zip`. Serve the whole extracted folder with WASM/JavaScript MIME types; relative assets support subpaths. `embedded-test/index.html` in the ZIP points to the app root. In the source checkout it points to `../dist/web/`. Serve only generated files and generic fixtures, on loopback for local checks.

## Host protocol and validation

Use a **same-origin** iframe with `?host=parent`. Incoming messages must come from the exact parent window and origin; replies target that origin, never `*`. Existing ready/open/dirty/save messages remain. The protocol's app prefix is unchanged even though visible branding is neutral.

```js
// App -> host
{type:"pdfcraft:ready", version:"0.18.1", protocol:1}
// Host -> app; valid fields merge live, omitted fields retain their values
{type:"pdfcraft:configure",
  colorScheme:"light", // light | dark | system
  locale:"en", // available exact locale, including ar
  branding:{name:"Sample Editor"}, // optional, 1..40 Unicode characters
  ackSave:true, // optional; default false
  theme:{accent:"#146c43", accentText:"#ffffff", accentSoft:"#dbf2e4"},
  chrome:{titlebar:false, homeTab:false, openTab:false}}
// App -> host; dotted paths; invalid/unknown values do not reset prior values
{type:"pdfcraft:configured", applied:["branding.name","ackSave"], ignored:["unknown"]}
// Host -> app; existing bytes contract
{type:"pdfcraft:open", name:"sample.pdf", bytes:ArrayBuffer /* or Uint8Array */}
{type:"pdfcraft:rename", name:"Renamed.pdf"}
{type:"pdfcraft:requestSave"}
{type:"pdfcraft:saved", ok:true}
{type:"pdfcraft:saved", ok:false, error:"Generic storage failure"}
// App -> host; existing bytes contract, including Save As from the editor
{type:"pdfcraft:dirty", dirty:true}
{type:"pdfcraft:save", name:"Renamed.pdf", saveAs:false, bytes:ArrayBuffer}
```

Configuration/rename/ack payloads are bounded to 64 KiB. Configuration objects are capped at 128 keys. Unknown keys are ignored. Booleans must be actual booleans. Colour strings accept only `#RGB`, `#RRGGBB`, `#RRGGBBAA`; colour arrays require exactly five valid colours. Known snake_case tokens and camelCase aliases work. Computed palette selectors cannot be overridden. Available colour tokens: `titlebar`, `chrome`, `panel`, `pasteboard`, `card`, `border`, `divider`, `text`, `textMuted`, `textFaint`, `icon`, `hover`, `pressed`, `selected`, `accent`, `accentText`, `accentSoft`, `field`, `badgeNew`, `pageShadow`.
The existing `radius` token accepts an integer from 0 to 32.

`branding.name` rejects empty/whitespace-only strings, controls, and more than 40 Unicode characters. It changes UI application-name labels, About content, welcome text and document/browser titles live. Before configuration the visible name is **Editor**. Application logos and branded credits are omitted in embedded wasm; package legal notices remain. Host names are rendered as text; they are never interpreted as HTML. Document content and supplied filenames are not rewritten as branding.

`rename` accepts 1..200 Unicode characters, rejects controls and both path separators (also blank-only and dot/dot-dot names), and changes the displayed name/window title and suggested save filename without marking document content dirty. There is no new rename response message. Invalid names are ignored.

`requestSave` directly invokes saving and emits the existing save payload with `saveAs:false`, even for a clean document and while keyboard focus is in the host. It does not rely on synthetic key events. Existing user Save As still emits `saveAs:true`.

With `ackSave:false`/omitted, saves retain their prior behavior. With `ackSave:true`, a delivered save remains unsaved until `saved {ok:true}`. Failed acknowledgements and a 30-second missing acknowledgement retain dirty state and show a dismissible **Not saved** notice. Host error text is sanitized and capped at 200 Unicode characters. Pending saves request periodic repaint checks, so an idle editor still displays the timeout. Acknowledgements clear only the stored document revision; edits made after that snapshot remain dirty.

Protocol 1 has no request IDs. Saves are serialized while an acknowledgement is pending. A timed-out acknowledgement is treated conservatively: one subsequent acknowledgement is consumed as stale instead of being allowed to confirm a newer save. Hosts should acknowledge every outgoing save in order, including failures. If an acknowledgement was permanently lost, reload/reopen after retaining the host's bytes to reset ambiguous acknowledgement state. No claim of arbitrary overlapping/out-of-order save correlation is made.

## Chrome switches

Supported booleans: `titlebar`, `homeTab`, `openTab`, `modebar`. Unspecified controls stay visible. Hidden controls free their layout space. Disabled dedicated navigation/actions are filtered before dispatch; host-supplied bytes still open when the picker is hidden.

Exactly what `titlebar:false` hides: **The whole top document-tab strip: Home/Open entries, all document tabs and their close/overflow controls, Combine tab, Settings, display-theme control, and Keyboard shortcuts button. The menu/mode/search/save/print row, tool panes, canvas and document keyboard shortcuts remain. This is the existing `titlebar:false` behavior; it was not broadened in round 2. `modebar:false` separately controls the menu/mode row.**

Undo, redo, Save and Find remain keyboard commands when titlebar is hidden. Word's existing Find-field focus rules still apply to document shortcuts; host `requestSave` works from any focus. Cmd/Ctrl+Shift+S opens Save As.

## Opening and dirty state

A completed host open establishes the imported/canonicalized document as the clean baseline. Load-time normalization is retained as baseline data, not reported as a user edit. Initial clean opens emit `dirty:false`, including clean-to-clean switches. Invalid opens/password prompts must not clear an existing document's edits. Subsequent real edits report dirty normally.

The generic blank fixtures and blank-document unit tests do not reproduce the original host's exact bytes; the precise normalization/style upgrade in that original document therefore remains unknown. The implementation explicitly accepts the successfully loaded baseline rather than guessing an Office-style rule. Regression tests cover blank opening and revision-aware acknowledgements.

## Embedded removals retained from round 1

Discord UI, Help/Backstage/keytip entries and handlers; promotional catalog rows; product/community/repository/contributor hyperlinks; update checks; language/model/data downloads; external document-link navigation and startup ?file= URL loading. No empty control placeholders are left. Local open/save/export/printing remain. Trademark graphics are compiled out of the embedded renderer.

## Fonts and the optional lite variant

The synthetic Word test document requests **Aptos, Calibri and Cambria** by family name and embeds
no font programs. These proprietary Office families are absent from the bundled font set. Other Office
fonts, such as Arial, Times New Roman and Courier New, likewise require an available licensed face
or substitution. A document's own embedded/licensed fonts and fonts explicitly imported by the user
can change availability; naming a font in a document does not grant redistribution rights.

Suggested licensed fallbacks: **Source Sans 3** for Aptos/Calibri/Arial and **Source Serif 4** for
Cambria/Times New Roman (OFL-1.1, already bundled by WordCraft); **JetBrains Mono** for Courier New
(OFL-1.1). They are not guaranteed metric-compatible with every Office face. Carlito/Caladea and
Liberation are existing resolver candidates when available; none were downloaded or added here.
PDFs with embedded font programs retain them. Missing unembedded PDF font families need per-document
inspection; the generic PDF fixture uses a standard-14 Helvetica reference, with no proprietary font file.

The full embedded build does not drop fonts or language resources in round 2. The optional `embedded-lite` feature removes selected non-Latin/Arabic/Hebrew shared faces and selected locale catalogs: `zh-hans`, `zh-hant`, `ja`, `ko`, `uk`, `ru`, `bg`, `sr`, `th`, `hi`, `te`, `ka` (where present). Latin-script resources and Arabic remain. A Hebrew face from the same licensed shared font checkout is retained. It is off by default. This is a resource experiment, not an assertion that every other script is gone. The pinned proofing crate has small unconditional supplements (including Russian) that remain; renderer/runtime Unicode tables remain as well.

## Verification, measurements and audit

- Full workspace unit run with embedded parser/state tests: **3,988 passed, 0 failed, 5 ignored**. The final host-focused regression run passed **5** tests; these overlap existing unit coverage and are not added to the workspace total.
- Strict embedded-feature Clippy (`--all-targets --features embedded -- -D warnings`): **PASS**. Formatting and `git diff --check`: **PASS**.
- Layers, asset attribution and default WASM checks: **PASS**.
- Default full CI: **not green**. One native integration test, `control::preferences_menu_and_shortcut_allow_switching_interface_languages`, cannot find the Preferences row at `tests/it/control.rs:1088`. The exact same test fails on the unmodified pinned base `8bb0528b` with the same build inputs. No default-behavior repair was made. The CI stages after that failure were not run; layers/assets/default WASM checks were completed separately.
- Full embedded browser protocol checks: **PASS** for ready/version/protocol, branding, rename, clean and real-dirty requestSave, positive/negative ack, visible 30-second timeout, blank clean open, and Save As. Titlebar-hidden Undo/Redo/Save/Find, theme/Arabic, invalid fields, and hide/restore were observed. File signatures were checked; this is not an exhaustive fidelity test.
- Final full/lite release builds: **PASS**. Lite was measured and its Latin/Arabic/Hebrew resource retention checked; the complete browser run above uses the full build.
- Detailed machine-readable evidence: [verification.json](embedded-test/verification.json) and [browser-results.json](embedded-test/browser-results.json).

### WASM sizes

| Build | Bytes | MiB |
|---|---:|---:|
| Unmodified pinned default baseline | 60,907,398 | 58.086 |
| Full embedded | 60,910,375 | 58.089 |
| Optional embedded-lite | 51,895,736 | 49.492 |
| Full embedded change from baseline | +2,977 | +0.003 |
| Lite reduction from full | 9,014,639 | 8.597 |

Full WASM SHA-256: `b6d2f25e9645c1012a7dce9d7d4f6ec558c4d98a9c895cae32eeddfe0ebcb86d`. Lite SHA-256: `9787502247cbb22b7d53e3d4e107956d80d0165e026225fcb05914176da5ff1f`.

| Byte category | Full | Lite |
|---|---:|---:|
| code | 34,451,537 | 34,426,199 |
| fonts | 9,383,424 | 4,228,160 |
| dictionaries | 0 | 0 |
| locales | 13,208,640 | 9,378,312 |
| other | 3,866,774 | 3,863,065 |

Code is the complete WASM code section. Resources are exact source-byte matches against reconstructed active linear-memory data, counted only where bytes are stored in the WASM (wasm-opt omits zero runs). Identical resources are deduplicated. Other is all remaining bytes, including section framing, runtime/static tables, compressed or transformed resources and any unmatched data. Categories are byte accounting, not independent removable or compressed download sizes.
Dictionaries here means identifiable spelling/hyphenation source data; PDF CMaps/encoding tables and transformed/unmatched resources belong to Other. These are uncompressed WASM file bytes, not HTTP transfer sizes. Detailed resource matches and hashes: [full breakdown](embedded-test/size-web.json), [lite breakdown](embedded-test/size-web-lite.json).

### Built-artifact string audit

Case-insensitive raw-byte searches over every generated `.wasm` and `.js`, including helper modules. Every hit has a zero-based byte offset, context and reason in [artifact-search.json](embedded-test/artifact-search.json). A substring is not evidence of an executable URL or request.

| Needle | Remaining hits |
|---|---:|
| `discord` | 0 |
| `getartcraft` | 0 |
| `storytold` | 0 |
| `github.com` | 12 |
| `http://` | 484 |
| `https://` | 73 |

Remaining text is document/XML/SVG/PDF namespaces, local icon markup, parser scheme tokens, dependency diagnostics/shader provenance, translation text, document-action placeholders, licences/proofing data and the verified Brotli dictionary. There are no Discord/product/community destination hits.


## Files touched against the pinned base

- `ATTRIBUTION.md`
- `ATTRIBUTION.toml`
- `CHANGES.md`
- `Cargo.lock`
- `apps/pdfcraft-web/Cargo.toml`
- `apps/pdfcraft-web/index-embedded.html`
- `apps/pdfcraft-web/src/main.rs`
- `crates/engine/Cargo.toml`
- `crates/engine/src/commands.rs`
- `crates/engine/src/links.rs`
- `crates/engine/src/session/view.rs`
- `crates/fonts/Cargo.toml`
- `crates/fonts/build.rs`
- `crates/ui-egui/Cargo.toml`
- `crates/ui-egui/build.rs`
- `crates/ui-egui/src/app.rs`
- `crates/ui-egui/src/app/files.rs`
- `crates/ui-egui/src/app/links.rs`
- `crates/ui-egui/src/canvas.rs`
- `crates/ui-egui/src/canvas/bars.rs`
- `crates/ui-egui/src/chrome.rs`
- `crates/ui-egui/src/commands.rs`
- `crates/ui-egui/src/dialogs.rs`
- `crates/ui-egui/src/dialogs/credits.rs`
- `crates/ui-egui/src/dialogs/preferences.rs`
- `crates/ui-egui/src/editing.rs`
- `crates/ui-egui/src/editing/save.rs`
- `crates/ui-egui/src/embedded.rs`
- `crates/ui-egui/src/home.rs`
- `crates/ui-egui/src/host_state.rs`
- `crates/ui-egui/src/lib.rs`
- `crates/ui-egui/src/menus.rs`
- `crates/ui-egui/src/platform/updates.rs`
- `crates/ui-egui/src/theme.rs`
- `crates/ui-egui/src/tools/content_ui.rs`
- `crates/ui-egui/src/tools/fill_sign/scan.rs`
- `crates/ui-egui/src/tools/sign_ui/dialog.rs`
- `crates/ui-egui/src/widgets.rs`
- `crates/ui-egui/src/workflows/ocr_ui.rs`
- `crates/ui-egui/src/workflows/web_print.rs`
- `embedded-test/artifact-search.json`
- `embedded-test/blank.pdf`
- `embedded-test/browser-results.json`
- `embedded-test/index.html`
- `embedded-test/sample.pdf`
- `embedded-test/size-web-lite.json`
- `embedded-test/size-web.json`
- `embedded-test/verification.json`
- `packaging/web/build-embedded.sh`
