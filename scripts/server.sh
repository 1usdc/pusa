#!/usr/bin/env bash
# 本地 Agent API（SQLite + SSE）。默认 RUST_LOG=info；可覆盖如 RUST_LOG=debug。
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT}"

export RUST_LOG="${RUST_LOG:-info}"
exec cargo run -p server
