# Embedded web profile — rounds 1–3

Local branch `embed/profile`, based on `8bb0528b`. No pushes. Application version 0.18.1; host protocol remains **1**.

The normal feature-off application keeps its existing behavior. Embedded-specific UI, host and resource changes are gated to wasm32; selecting the feature on native does not activate the embedded interface. `embedded-lite` is a separate opt-in resource variant and implies `embedded`. No font binaries were added to either app repository. Package licence/notice files are retained; generic fixtures are attributed.


## Round 3: host surfaces and available tools

Protocol remains **1**. The two additional `configure.chrome` booleans are `fileNew` and `fileClose`; both default to **true**. Sending false removes the File entry and blocks its associated command/shortcut and command-search paths. These values merge and can be restored with true, just like the previous switches.

```js
{type:"pdfcraft:configure", chrome:{fileNew:false,fileClose:false},
 theme:{dialog:"#ffb2ed",popup:"#b7ffce",toast:"#ffcc85",tooltip:"#e7c7ff",
        toolstrip:"#95f4f0",scrollbar:"#ffbc7d",scrollbarThumb:"#6c138c",
        backstageNav:"#79edaa",backstageContent:"#fcdf95"}}
```

The color grammar and validation are unchanged: #RGB, #RRGGBB or #RRGGBBAA, including alpha; camelCase and snake_case token names are accepted. Unknown keys and invalid values appear in `configured.ignored` and retain the previous value. No CSS, JavaScript or URL color values are accepted. New slots inherit already configured legacy slots if omitted, and independent region overrides win. Host colors apply only on the embedded wasm target. Theme/appearance controls are removed from that UI so the host remains the authority.

### Region → host slot

| Region | Slot(s) |
|---|---|
| Title bar / mode bar | `titlebar / chrome` |
| Side panels / Home / cards | `panel, card` |
| Dialog bodies / Preferences cards | `dialog` |
| File menu and other menus, command palette, popups | `popup` |
| File backstage navigation/content | `No separate backstage screen in this app; File is a menu. backstageNav/backstageContent are accepted common-palette slots but have no separate PDF region.` |
| Floating / docked tool strips and Find bar | `toolstrip` |
| Toast / progress notices | `toast, text, onAccent` |
| Not saved acknowledgement notice | `dialog` |
| Pointer and disabled tooltips | `tooltip` |
| Scroll area tracks / thumbs, including document canvas | `scrollbar / scrollbarThumb` |
| Fields, hover, pressed, borders | `field, hover, pressed, border, divider` |
| Text / icons / accent foreground | `text, textMuted, textFaint, icon, onAccent` |
| Semantic warning / error / password strength | `success, warning, danger` |
| Pasteboard / selection / accents | `pasteboard, selected, accent, accentText, accentSoft, badgeNew`; editing/crop/form outlines use accent and their wash uses accentSoft |
| Shadows / modal backdrop | `pageShadow` |

Hover/pressed blends, outlines, shadows, stock widget fills and IME underlines use host tokens. Scroll-area styling is scoped: the scrollbar gets its own track/thumb palette while the area contents retain their original host widget palette. Dropdowns supplied by shared widgets inherit host foreground/field colors. Transparent layout frames, texture tint identity colors and measurement-only colors are not independent visible surfaces.

Document paper, actual document text/graphics, imported images, swatches, document style/chart previews, print thumbnails and signature previews retain document colors. No font or other asset binary was added. Licensing and attribution text stays intact.

### Removed or hidden in this round

- AI assistant group and its bring-your-own-model text; all `ai.*` commands are blocked. The provider-backed catalog group is compiled out of embedded WASM. No provider upload implementation was present at this pin.
- Secondary Organize placeholders (Page transitions and Page templates); unavailable comment tools in quick-tool flyouts and saved/custom hotbar choices.
- Unsupported formatting placeholders: bulleted/numbered lists, paragraph spacing, unavailable alignment contexts, underline/superscript/subscript when unsupported, and added-text line-spacing/horizontal-scale controls. Working equivalents remain in contexts where they are implemented. Gaps disappear with the controls.
- The wholly unfinished search-index and JavaScript tool groups. Individual Planned/provider items are compiled out of the catalog, and placeholder milestone/provider fallback toasts are compiled out.
- Appearance card in Preferences (theme tiles, page-edge appearance and reading-color section), title-bar theme cycle/context menu, Display theme submenu and `view.theme*` commands/search entries.
- On host request: File New blank PDF and Close/Close all (`create.blank`, `file.close`, `file.close_all`). Menus, search and keyboard dispatch obey the same host policy.
- Network/update/download/community controls removed in round 1 remain excluded. No network/account/telemetry/provider settings are introduced.

The source catalog labels Edit, Protect and Print Production as Planned roadmap groups even though they contain working Ready commands. Embedded builds retain their implemented Ready items and label these three groups Ready; unfinished items disappear. Desktop metadata stays unchanged. This prevents removing working editing/protection commands merely because their parent roadmap milestone is incomplete. A target-independent readiness policy and regression tests require exposed commands to be implemented and Ready. A separate deny list preserves the decision after non-ready catalog metadata has been compiled out.

Non-ready command IDs removed/blocked in this round:

`a11y.reading_order`, `actions.archive`, `actions.make_accessible`, `ai.ask`, `ai.summary`, `ai.translate`, `comment.cloud`, `comment.stamp`, `export.ps`, `export.xlsx`, `js.actions`, `js.console`, `js.document`, `measure.geo`, `ocr.correct`, `ocr.enhance`, `optimize.audit`, `page.transitions`, `preflight.run`, `prepress.convert_colors`, `prepress.hairlines`, `prepress.marks`, `prepress.output_preview`, `protect.certificate`, `sign.timestamp`, `standards.pdfua`, `standards.pdfx`.

### Round 3 verification and release evidence

Workspace embedded-feature unit tests: **3,995 passed, 0 failed, 5 ignored**. Strict all-targets embedded-feature Clippy, formatting, layering, asset and embedded WASM checks passed. The final affected engine/UI rerun passed **1,145 tests, 0 failed, 3 ignored** (overlapping the workspace run). The default CI still fails only the pre-existing native integration test `control::preferences_menu_and_shortcut_allow_switching_interface_languages` at `tests/it/control.rs:1088`, reproduced on the untouched pinned base. Later CI stages were not run after this stop; see `embedded-test/verification.json`.

Both release builds completed with Binaryen `-Oz`. Exact byte accounting follows; categories sum to each file size. Resources are exact source matches in reconstructed WASM memory; `other` includes unmatched/transformed data and runtime/section overhead. These categories are not independent removable/compressed-download estimates.

| Variant | Total bytes | Code | Fonts | Dictionaries | Locale data | Other |
|---|---:|---:|---:|---:|---:|---:|
| web | 60,871,524 | 34,455,787 | 9,383,424 | 0 | 13,168,961 | 3,863,352 |
| web-lite | 51,868,243 | 34,430,440 | 4,228,160 | 0 | 9,349,993 | 3,859,650 |

Full WASM changed from round 2's **60,910,375** to **60,871,524** bytes (-38,851); optional lite saves **9,003,281** bytes relative to this full build. Each file hash and resource attribution is in `embedded-test/size-web.json` / `size-web-lite.json`.

Case-insensitive raw scans cover every final `.wasm` and `.js` in both outputs. Every hit has an offset, context and reason in `embedded-test/artifact-search.json` and `artifact-search-lite.json`; **zero unclassified hits**. Counts include non-actionable data:

| Needle | Full hits | Lite hits |
|---|---:|---:|
| `discord` | 0 | 0 |
| `getartcraft` | 0 | 0 |
| `storytold` | 0 | 0 |
| `github.com` | 12 | 12 |
| `http://` | 484 | 444 |
| `https://` | 73 | 59 |
| `milestone` | 0 | 0 |
| `bring your own` | 0 | 0 |
| `provider` | 311 | 223 |

Provider matches are unused translated catalog strings (including digital-signing/privacy text), local font/ICU diagnostics and Brotli dictionary bytes. They do not expose model/provider actions; unchanged full translation resources have not been byte-scrubbed. URL/hostname matches are document/XML/SVG format identifiers, licences/provenance, dependency diagnostics, parser tokens and Brotli dictionary bytes; the per-hit reports distinguish each source.

The final full artifact passed all **11** harness result flags, including 34 loud theme settings accepted, ready/version/protocol, branding, rename, clean/dirty requestSave, positive/negative acknowledgements, timeout, blank clean open and Save As. Observed UI regions and removals are listed in `embedded-test/browser-results.json`. Shared unit coverage and visited screens support the surface changes; every possible dialog/hover state was not manually enumerated. Cross-browser and exhaustive document-fidelity acceptance remain unverified. Lite was built, measured and string-audited; this browser run used full embedded.


The local test page includes `loud-theme.json` and sends a distinct loud color for every accepted color setting. Its manual checklist covers UI regions, hidden New/Close and their shortcuts, absent placeholder/AI search results, and removed Preferences sections. Protocol, branding, open/dirty/save/ack/timeout checks from round 2 remain available. Only generic fixtures are used.

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

Supported booleans: `fileNew`, `fileClose`, `titlebar`, `homeTab`, `openTab`, `modebar`. Unspecified controls stay visible. Hidden controls free their layout space. Disabled dedicated navigation/actions are filtered before dispatch; host-supplied bytes still open when the picker is hidden.

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

## Files touched against the pinned base

- `ATTRIBUTION.md`
- `ATTRIBUTION.toml`
- `CHANGES.md`
- `Cargo.lock`
- `apps/pdfcraft-web/Cargo.toml`
- `apps/pdfcraft-web/index-embedded.html`
- `apps/pdfcraft-web/src/main.rs`
- `crates/engine/Cargo.toml`
- `crates/engine/src/catalog.rs`
- `crates/engine/src/commands.rs`
- `crates/engine/src/links.rs`
- `crates/engine/src/session/view.rs`
- `crates/fonts/Cargo.toml`
- `crates/fonts/build.rs`
- `crates/ui-egui/Cargo.toml`
- `crates/ui-egui/build.rs`
- `crates/ui-egui/src/app.rs`
- `crates/ui-egui/src/app/commands.rs`
- `crates/ui-egui/src/app/files.rs`
- `crates/ui-egui/src/app/links.rs`
- `crates/ui-egui/src/canvas.rs`
- `crates/ui-egui/src/canvas/area.rs`
- `crates/ui-egui/src/canvas/bars.rs`
- `crates/ui-egui/src/canvas/find.rs`
- `crates/ui-egui/src/canvas/organize.rs`
- `crates/ui-egui/src/chrome.rs`
- `crates/ui-egui/src/commands.rs`
- `crates/ui-egui/src/dialogs.rs`
- `crates/ui-egui/src/dialogs/credits.rs`
- `crates/ui-egui/src/dialogs/palette.rs`
- `crates/ui-egui/src/dialogs/preferences.rs`
- `crates/ui-egui/src/dialogs/prompts.rs`
- `crates/ui-egui/src/dialogs/protect.rs`
- `crates/ui-egui/src/dialogs/standards_ui.rs`
- `crates/ui-egui/src/editing.rs`
- `crates/ui-egui/src/editing/save.rs`
- `crates/ui-egui/src/embedded.rs`
- `crates/ui-egui/src/home.rs`
- `crates/ui-egui/src/home/folders_ui.rs`
- `crates/ui-egui/src/host_state.rs`
- `crates/ui-egui/src/hotbar.rs`
- `crates/ui-egui/src/image_engine.rs`
- `crates/ui-egui/src/lib.rs`
- `crates/ui-egui/src/menus.rs`
- `crates/ui-egui/src/panels/all_tools_ui.rs`
- `crates/ui-egui/src/panels/comments_panel.rs`
- `crates/ui-egui/src/panels/edit.rs`
- `crates/ui-egui/src/panels/format_text_ui.rs`
- `crates/ui-egui/src/panels/organize_panel.rs`
- `crates/ui-egui/src/panels/right.rs`
- `crates/ui-egui/src/panels/search_ui.rs`
- `crates/ui-egui/src/panels/tools.rs`
- `crates/ui-egui/src/platform/motion/tests.rs`
- `crates/ui-egui/src/platform/updates.rs`
- `crates/ui-egui/src/platform/window_controls.rs`
- `crates/ui-egui/src/theme.rs`
- `crates/ui-egui/src/tools/bulk_fields.rs`
- `crates/ui-egui/src/tools/calc_order_ui.rs`
- `crates/ui-egui/src/tools/camera_ui.rs`
- `crates/ui-egui/src/tools/comments/menu.rs`
- `crates/ui-egui/src/tools/content_ui.rs`
- `crates/ui-egui/src/tools/field_table_ui.rs`
- `crates/ui-egui/src/tools/fill_sign.rs`
- `crates/ui-egui/src/tools/fill_sign/scan.rs`
- `crates/ui-egui/src/tools/forms_ui.rs`
- `crates/ui-egui/src/tools/link_ui.rs`
- `crates/ui-egui/src/tools/measure_ui.rs`
- `crates/ui-egui/src/tools/prepare/properties.rs`
- `crates/ui-egui/src/tools/prepare/tabs.rs`
- `crates/ui-egui/src/tools/redact_ui.rs`
- `crates/ui-egui/src/tools/sign_ui/dialog.rs`
- `crates/ui-egui/src/tools/stamps_ui.rs`
- `crates/ui-egui/src/tools/tags_ui.rs`
- `crates/ui-egui/src/widgets.rs`
- `crates/ui-egui/src/widgets/icons.rs`
- `crates/ui-egui/src/workflows/a11y_ui.rs`
- `crates/ui-egui/src/workflows/combine_grid.rs`
- `crates/ui-egui/src/workflows/combine_grid/card.rs`
- `crates/ui-egui/src/workflows/combine_ui/table.rs`
- `crates/ui-egui/src/workflows/combine_ui/view.rs`
- `crates/ui-egui/src/workflows/js_ui.rs`
- `crates/ui-egui/src/workflows/ocr_ui.rs`
- `crates/ui-egui/src/workflows/print_ui/body.rs`
- `crates/ui-egui/src/workflows/web_print.rs`
- `embedded-test/artifact-search-lite.json`
- `embedded-test/artifact-search.json`
- `embedded-test/blank.pdf`
- `embedded-test/browser-results.json`
- `embedded-test/index.html`
- `embedded-test/loud-theme.json`
- `embedded-test/sample.pdf`
- `embedded-test/size-web-lite.json`
- `embedded-test/size-web.json`
- `embedded-test/verification.json`
- `packaging/web/build-embedded.sh`
