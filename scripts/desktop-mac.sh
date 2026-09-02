#!/usr/bin/env bash
# 本机 macOS 打包发版：dx bundle --release → Developer ID 签名 + 公证 → desktop/dist/*.dmg
#
# 用法：
#   just desktop-mac                         # 先把 desktop/Cargo.toml patch 位 +1
#   BUMP=0 just desktop-mac                  # 不改版号，按当前 version 打包
#   NOTARIZE=0 just desktop-mac              # 只签名，跳过 Apple 公证
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

bump="${BUMP:-1}"
if [[ "${bump}" == "1" ]]; then
	echo "== 0) desktop/Cargo.toml patch +1 =="
	python3 - "${ROOT}" <<'PY'
import re
import sys
from pathlib import Path

root = Path(sys.argv[1])
toml_path = root / "desktop" / "Cargo.toml"
text = toml_path.read_text()
m = re.search(r'(?m)^version = "(\d+)\.(\d+)\.(\d+)"', text)
if not m:
    sys.exit("desktop/Cargo.toml 没有 version = \"X.Y.Z\"")
major, minor, patch = (int(m.group(1)), int(m.group(2)), int(m.group(3)))
new = f"{major}.{minor}.{patch + 1}"
text, n = re.subn(r'(?m)^version = "\d+\.\d+\.\d+"', f'version = "{new}"', text, count=1)
if n != 1:
    sys.exit("未能改写 desktop/Cargo.toml 的 version")
toml_path.write_text(text)

lock_path = root / "Cargo.lock"
lock = lock_path.read_text()
lock, n = re.subn(
    r'(name = "desktop"\nversion = ")\d+\.\d+\.\d+(")',
    rf"\g<1>{new}\2",
    lock,
    count=1,
)
if n != 1:
    sys.exit("未能改写 Cargo.lock 里 desktop 的 version")
lock_path.write_text(lock)
print(f"version {major}.{minor}.{patch} → {new}")
PY
fi

echo "== 1) 清空 desktop/dist =="
rm -rf "${ROOT}/desktop/dist"
mkdir -p "${ROOT}/desktop/dist"

echo "== 2) dx bundle --release（desktop/）=="
# 与 CI 一致：在 desktop crate 目录 bundle，不传 --target，用 host 默认目标
(
	cd "${ROOT}/desktop"
	dx bundle --release
)

echo "== 3) 嵌入 libpusa_core.dylib =="
bash "${ROOT}/scripts/embed-pusa-core-macos.sh"

echo "== 4) 签名 + 公证 =="
bash "${ROOT}/scripts/macos-sign-notarize.sh"

echo "✓ 本机 macOS 包已就绪：${ROOT}/desktop/dist/"
ls -lh "${ROOT}/desktop/dist/"*.dmg 2>/dev/null || ls -lh "${ROOT}/desktop/dist/"
