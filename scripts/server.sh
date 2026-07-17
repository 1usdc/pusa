#!/usr/bin/env bash
# 本地 Agent API（SQLite + SSE）。默认 RUST_LOG=info；可覆盖如 RUST_LOG=debug。
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT}"

export RUST_LOG="${RUST_LOG:-info}"
anotherme_base="${ANOTHERME_BASE_URL:-http://127.0.0.1:8881}"
export ANOTHERME_BASE_URL="${anotherme_base//host.docker.internal/127.0.0.1}"
exec cargo run -p server
