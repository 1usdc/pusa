#!/usr/bin/env bash
# 本机桌面：dx serve 热重载。
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT}"

# 默认 always-on-top=true 会挡住编辑器；开发时关掉
exec dx serve --desktop -p desktop --always-on-top false
