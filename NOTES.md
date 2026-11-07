# pdfcraft-cli 0.18.1 — offline release notes

Date: 2026-11-08. Local `embed/cli`; no push, publication or upload.

Original base: `8bb0528bed785283452263b0290321ccf95859be`. CLI branch started from embedded-build commit `002b07fbb892be66f0308ba7aaecf6633a9f6e92`, not main. The final source commit/tree and source-input fingerprint are recorded in `SOURCE.json` supplied with the archives. Rust/Cargo source inputs are checked against the exact build snapshot; reports were written afterward.

## Binaries and versions

| Target | Binary bytes | Version output | SHA-256 |
|---|---:|---|---|
| `aarch64-apple-darwin` | 76,636,080 | `pdfcraft-cli 0.18.1` | `f65ca7c21eb6e31d5cf635a046009a0bba9b18fb3c253dda6a86823ac3f364c3` |
| `x86_64-apple-darwin` | 81,107,200 | `pdfcraft-cli 0.18.1` | `e3e56fd3604727b30350a8be1f9bd1b9e986c4fc47609d968c9668894cd35888` |
| `x86_64-pc-windows-msvc` | 90,273,792 | Not executed; Cargo package version 0.18.1 | `57a30127a2b8073c586aa677e716194789ed4af32145e0e5bca7fa8524fa0b3a` |

Windows is cross-compiled, PE/import-inspected and built with static MSVC C runtime. Import inspection found no VCRUNTIME/MSVCP/UCRT runtime DLL dependency. Windows OS DLL imports remain; see `reports/binary-inspection.json`. Windows execution, version output, confinement behavior and performance are **NOT VERIFIED** here. Both macOS binaries were executed; Intel ran through Rosetta 2 on Apple Silicon, not physical Intel hardware. They import only macOS system libraries/frameworks. Cargo strips symbols; inspection found no DWARF/.debug sections. No PDB is packaged.

## Exact build setup

Rust 1.99.1 via rustup; LLVM/LLD 23.1.5; cargo-xwin 0.23.1; Microsoft SDK cache 10.0.26100. Targets: aarch64-apple-darwin, x86_64-apple-darwin, x86_64-pc-windows-msvc. These are MSVC builds, not GNU substitutes.

From the repository root, the release environment was:

```sh
export PATH=/opt/homebrew/opt/lld/bin:/opt/homebrew/opt/llvm/bin:/Users/mohammednaji/.cargo/bin:/opt/homebrew/bin:/usr/bin:/bin
export CARGO_BUILD_JOBS=2
export CARGO_PROFILE_RELEASE_OPT_LEVEL=3
export CARGO_PROFILE_RELEASE_LTO=thin
export CARGO_PROFILE_RELEASE_STRIP=symbols
export CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_RUSTFLAGS='-C target-feature=+crt-static -C debuginfo=0'
export XWIN_ARCH=x86_64
export CRAFT_FONTS_DIR=/Users/mohammednaji/Desktop/projects/craft-build-inputs/fonts
export CRAFT_I18N_DIR=/Users/mohammednaji/Desktop/projects/craft-build-inputs/pdfcraft-i18n
export CRAFT_I18N_REQUIRED=1
cargo build --release --locked --target aarch64-apple-darwin -p pdfcraft-cli --features mcp,offline
cargo build --release --locked --target x86_64-apple-darwin -p pdfcraft-cli --features mcp,offline
cargo xwin build --release --locked --target x86_64-pc-windows-msvc -p pdfcraft-cli --features mcp,offline
```

Release panic/unwind behavior is retained; panic=abort was not enabled. All builds used --locked. Tool-generated compiler/linker defaults remain those of the versions above. Optional Microsoft CRT debug PDBs were not downloaded; the linker emitted missing-PDB warnings while successfully producing stripped release executables. Exact logs and commands are in `reports/logs` and `reports/build-runs.json`.

## Profile and network/process behavior

`offline` is opt-in and is the feature used for these archives. Feature-off defaults retain the original behavior. MCP starts only through the `mcp` command, uses stdin/stdout, replies before EOF, and exits when stdin closes. It is not registered with another application and never starts a listener automatically. No telemetry/update check or model/data download is started by the tested startup/open/edit/save/convert workflows.

The desktop `ui` TCP bridge is compiled out. Version/help/MCP output omit promotional links, and the Help community/product/repository/update command metadata is excluded. OCR uses locally supplied/discovered models; this CLI does not download them. OCR without suitable local model files is unavailable and was not exercised. `check` may spawn this same CLI executable as a bounded worker. Native printing delegates to OS printing (including lp/lpstat/lpoptions or the Windows print backend); it is opt-in and was not exercised.

The offline profile removes the application network integrations listed above; it is not an OS firewall. Optional OS services such as printing/speech, and explicitly mounted remote filesystems, are controlled by the OS and can involve external devices/services. Leave those opt-in commands unused for the strict offline workflow verified here. Runtime checks used an OS sandbox denying network access; no print/speech/model-download operation was part of those checks. `reports/DEPENDENCIES.md` lists network/process-capable source matches, build scripts and proc macros, with full source evidence in dependency-inventory.json. A dependency API match is not proof it is linked or reachable.

## Folder restrictions and end-to-end results

`--root` confines automation tool/script paths to one folder; relative tool paths resolve there. MCP separate read/write flags add independent capabilities, with any --root constraint applied as well. In the offline profile, an explicit --root by itself still grants confined read/write access; with no root of either kind, MCP grants neither. Providing only one capability denies the other. The explicit script file named on the CLI is read by the CLI; --root confines the paths used by its tool steps and outputs. Other standalone explicit-file CLI commands retain their documented option semantics.

Verified on the final Apple Silicon binary: open our original PDF with text and a text form field; add a note; fill generic_name; save; reopen and confirm both note and field value. Inside-root reads/writes work; outside path, .., absolute outside path and outward symlink cases return explicit refusals. Missing-read, missing-write and no-root cases were refused, including saving an in-memory blank PDF with no write authority. Legacy --root worked in both MCP and a CLI script, and rejected outside paths. No forbidden output appeared.

Roots protect application file operations, not against an independent process racing to change the filesystem after validation. They are not an OS-wide filesystem sandbox. Filesystem and workflow acceptance ran on Apple Silicon; Intel was exercised for version/startup/conversion through Rosetta; Windows runtime checks remain unverified.

The exact commands, JSON requests/responses, sandbox profile, working directories and output assertions are in `reports/runtime-checks.json`. Only original generic fixtures were used; the Microsoft licence attachment was a legal document, not an application fixture.

## Timing and memory

Warm filesystem cache; fresh process per run. Five --version launch/exit samples, three interactive stdio MCP initialize-ready/EOF samples, three 30-page conversion samples. RSS is macOS /usr/bin/time -l maximum resident set size, bytes. Other machine workload was not controlled. Intel results include Rosetta 2 and are not physical-Intel performance results.

The benchmark input is 30 A4 pages, 11,280 original words, 12 body paragraphs/page plus headings, with explicit page breaks and Source Sans 3. This is a simple text workload, not an OCR, table/picture stress, cold-start or document-fidelity benchmark. Separate small fixtures cover the requested table/picture/form workflows. Word converts DOCX to PDF; PDF exports that PDF to DOCX. Inputs were confirmed at 30 pages; converted DOCX output includes the page-30 heading.

| Target | --version launch/exit median ms | MCP initialize-ready median ms | 30-page conversion median s | Conversion peak RSS MiB |
|---|---:|---:|---:|---:|
| `aarch64-apple-darwin` | 6.99 | 6.00 | 0.0437 | 16.52 |
| `x86_64-apple-darwin` | 38.92 | 34.80 | 0.1093 | 17.50 |
| `x86_64-pc-windows-msvc` | Not run | Not run | Not run | Not run |

Full samples, commands, host version, input hashes and raw time output are in reports/performance.json.

## Checks and string audit

- offline-tests: PASS — `cargo test --locked -p pdfcraft-cli --bin pdfcraft-cli --features mcp,offline`.
- offline-engine: PASS — `cargo test --locked -p pdfcraft-engine -p pdfcraft-automation --lib --features pdfcraft-engine/offline,pdfcraft-automation/offline`.
- clippy: PASS — `cargo clippy --locked -p pdfcraft-cli -p pdfcraft-engine -p pdfcraft-automation --all-targets --features pdfcraft-cli/offline,mcp -- -D warnings`.
- default-cli: PASS — `cargo test --locked -p pdfcraft-cli --features mcp`.
- layers: PASS — `cargo xtask layers`.
- assets: PASS — `cargo xtask assets`.
- wasm: PASS — `cargo xtask wasm`.
- These are the affected CLI/engine/automation and required layering/asset/WASM checks. A full GUI workspace CI sweep was not rerun for this CLI task; the earlier web round documented an unrelated existing native GUI Preferences integration failure.

Every occurrence of discord, getartcraft, storytold, http:// and https:// was scanned in ASCII/UTF-8, UTF-16LE and UTF-16BE. Offsets, contexts and per-hit reasons are in reports/artifact-search.json; zero unclassified hits. No byte-scrubbing of required format data was performed.

| Target | discord | getartcraft | storytold | http:// | https:// |
|---|---:|---:|---:|---:|---:|
| `aarch64-apple-darwin` | 0 | 0 | 0 | 194 | 94 |
| `x86_64-apple-darwin` | 0 | 0 | 0 | 199 | 94 |
| `x86_64-pc-windows-msvc` | 0 | 0 | 0 | 224 | 94 |

Remaining scheme/URL matches are individually explained namespace/format tokens, licensed font metadata, proofing/Brotli data, dependency references, document-link edit/recognition descriptions or architecture-specific code immediates. None is presented as a promotional navigation action by the offline CLI. Input documents may retain their own links as data; the CLI does not follow them.

## Licences, inventory and delivery

Program MIT/Apache licence and notice files are included unchanged. THIRD-PARTY-LICENSES.txt and reports/dependency-inventory.json cover the conservative production/build dependency graph across targets; dev-only edges are excluded. All identified packages have corresponding licence texts in this generated bundle. Where CMS, fmt-derive/fmt-derive-proc or rustls-cng omit licence files in their original source, the bundle explicitly records the declared Apache-2.0 option and the provenance of the canonical text; it does not label that text as a package-specific upstream file. Declared alternative licences and non-core families (including Unicode-3.0, Zlib, Unlicense, CC0-1.0, BSL-1.0, LLVM exception and, where resolved, CDLA-Permissive-2.0) are called out per package in reports/DEPENDENCIES.md. Font OFL/other notices and the Rust standard-library copyright notice are retained. Windows archives also include the approved Microsoft build-tools terms and source attribution; Microsoft runtime and OS terms are outside the listed open-source families. macOS system frameworks are provided by the OS, not copied into the archive.

Full top-level CLI commands/options: reports/CLI-HELP.txt. Full engine command listing: reports/ENGINE-COMMANDS.json. Full MCP tool list (184 tools), one-line descriptions and standalone limitations: reports/MCP-TOOLS.md and MCP-TOOLS.json. Full cold-build build.rs/proc-macro inventory: reports/DEPENDENCIES.md.

One ZIP per target: `pdfcraft-cli-0.18.1-<target>.zip`, each with the executable, licences, this NOTES.md and reports. SHA256SUMS covers the six delivered ZIPs. `patches-pdfcraft-cli/` contains the ordered git format-patch series against the original base, including the embedded-profile history from which this branch started. Patch application and archive byte checks are recorded in the delivery manifest.

Known limits: no Windows execution/performance acceptance; Intel tested via Rosetta only; no exhaustive document-fidelity or every-command acceptance; OCR/optional OS services not exercised; standalone explicit-file CLI operations retain their original filesystem option scopes. No source behavior was changed beyond this opt-in offline profile, its capability defaults and the documented exclusions.

## Files changed in the CLI task

- `NOTES.md`
- `apps/pdfcraft-cli/Cargo.toml`
- `apps/pdfcraft-cli/src/main.rs`
- `apps/pdfcraft-cli/src/options.rs`
- `apps/pdfcraft-cli/src/run.rs`
- `crates/automation/Cargo.toml`
- `crates/automation/src/file_access.rs`
- `crates/automation/src/mcp.rs`
- `crates/engine/Cargo.toml`
- `crates/engine/src/commands.rs`
- `crates/engine/src/links.rs`
