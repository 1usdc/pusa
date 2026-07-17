#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SRC="$ROOT_DIR/Polyclaw/SKILL.md"
DEST="$ROOT_DIR/apps/web/public/skill.md"

cp "$SRC" "$DEST"
echo "Synced $SRC -> $DEST"
