#!/usr/bin/env bash
# 本机桌面：dx serve 热重载。
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT}"

# 样式：Tailwind v4 生成 ui/assets/css/styles.css（Rust 侧只挂这个产物）
# 已装好 node_modules 就重新生成；否则用已提交的产物继续。热更新请另开终端 `just tailwind-watch`。
if [[ -x node_modules/.bin/tailwindcss ]]; then
	npx tailwindcss -i ui/assets/css/app.css -o ui/assets/css/styles.css --minify
else
	echo "warn: 缺少 node_modules/.bin/tailwindcss，跳过 Tailwind 生成（使用已提交的 ui/assets/css/styles.css）" >&2
fi

# 默认 always-on-top=true 会挡住编辑器；开发时关掉
exec dx serve --desktop -p desktop --always-on-top false
