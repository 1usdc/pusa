#!/usr/bin/env bash
# 本机桌面：dx serve 热重载。联调：ANOTHERME_BASE_URL=http://127.0.0.1:8881
# ANOTHERME_BASE_URL 经 option_env! 在编译期写入，须出现在 dx/cargo 环境中。
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT}"

export ANOTHERME_BASE_URL="${ANOTHERME_BASE_URL:-https://api.anotherme.co}"
# 默认 always-on-top=true 会挡住编辑器；开发时关掉
exec dx serve --desktop -p desktop --always-on-top false
