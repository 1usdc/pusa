#!/usr/bin/env bash
# 本机 macOS 打包发版：dx bundle --release → Developer ID 签名 + 公证 → desktop/dist/*.dmg
#
# 用法：
#   just desktop-mac
#   NOTARIZE=0 just desktop-mac          # 只签名，跳过 Apple 公证
#   ANOTHERME_BASE_URL=... just desktop-mac
#
# 凭证：.env.signing（MACOS_SIGNING_IDENTITY / APPLE_ID / APPLE_APP_SPECIFIC_PASSWORD / APPLE_TEAM_ID）
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT}"

if [[ "$(uname -s)" != "Darwin" ]]; then
	echo "❌ just desktop-mac 只能在 macOS 上跑" >&2
	exit 1
fi

if ! command -v dx >/dev/null 2>&1; then
	echo "❌ 未找到 dx，请先安装 dioxus-cli（与 CI 对齐：dx --version）" >&2
	exit 1
fi

export ANOTHERME_BASE_URL="${ANOTHERME_BASE_URL:-https://api.anotherme.co}"

echo "== 1) 清空 desktop/dist =="
rm -rf "${ROOT}/desktop/dist"
mkdir -p "${ROOT}/desktop/dist"

echo "== 2) dx bundle --release（desktop/）=="
# 与 CI 一致：在 desktop crate 目录 bundle，不传 --target，用 host 默认目标
(
	cd "${ROOT}/desktop"
	dx bundle --release
)

echo "== 3) 签名 + 公证 =="
bash "${ROOT}/scripts/macos-sign-notarize.sh"

echo "✓ 本机 macOS 包已就绪：${ROOT}/desktop/dist/"
ls -lh "${ROOT}/desktop/dist/"*.dmg 2>/dev/null || ls -lh "${ROOT}/desktop/dist/"
