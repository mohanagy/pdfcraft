#!/usr/bin/env bash
# Build a self-contained embedded site. See CHANGES.md for the host protocol.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
: "${CRAFT_I18N_DIR:?Set CRAFT_I18N_DIR to the checkout pinned in craft-i18n.rev}"
export CRAFT_I18N_REQUIRED=1
# Optional CRAFT_FONTS_DIR must point to the licensed craft-fonts checkout used by the caller.
(cd "$ROOT/apps/pdfcraft-web" && trunk build --skip-version-check --release --locked --features "${1:-embedded}" --html-output index.html index-embedded.html)
DIST="$ROOT/dist/embedded-release" bash "$ROOT/packaging/web/package.sh" --skip-build
