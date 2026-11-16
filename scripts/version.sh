#!/usr/bin/env bash
# 查看当前版本号和仓库打包版本号
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT}"

# 读取 desktop/Cargo.toml 版本
CARGO_TOML="${ROOT}/desktop/Cargo.toml"
if [[ ! -f "${CARGO_TOML}" ]]; then
    echo "❌ 未找到 ${CARGO_TOML}" >&2
    exit 1
fi

VERSION="$(
    awk '
        /^\[package\]/ { in_pkg = 1; next }
        /^\[/ { in_pkg = 0 }
        in_pkg && $0 ~ /^version[[:space:]]*=/ {
            gsub(/"/, "", $0)
            sub(/^version[[:space:]]*=[[:space:]]*/, "", $0)
            gsub(/^[[:space:]]+|[[:space:]]+$/, "", $0)
            print $0
            exit
        }
    ' "${CARGO_TOML}"
)"

if [[ ! "${VERSION}" =~ ^[0-9]+\.[0-9]+\.[0-9]+([.-].*)?$ ]]; then
    echo "❌ 无法从 desktop/Cargo.toml 解析 version（得到：${VERSION:-空}）" >&2
    exit 1
fi

echo "当前版本号: ${VERSION}"
echo "打包 tag: desktop-v${VERSION}"

# 检查 GitHub Release
if command -v gh >/dev/null 2>&1; then
    TAG="desktop-v${VERSION}"
    if gh release view "${TAG}" >/dev/null 2>&1; then
        echo "✓ 已发布到 GitHub Release: ${TAG}"
        echo ""
        echo "Release 资产:"
        gh release view "${TAG}" --json assets -q '.assets[] | "  - \(.name) (\(.size // 0 | . / 1024 / 1024 | floor)MB)"'
    else
        echo "✗ 未发布到 GitHub Release: ${TAG}"
    fi
else
    echo "（未安装 gh CLI，无法检查 GitHub Release）"
fi
