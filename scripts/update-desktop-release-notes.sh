#!/usr/bin/env bash
# 更新 GitHub Release 说明：双平台一键下载按钮 + 文件表。
# 默认 tag 取自 desktop/Cargo.toml → desktop-vX.Y.Z（可用 TAG= 覆盖）。
#
# 用法：
#   just release-notes
#   TAG=desktop-v1.1.2 bash scripts/update-desktop-release-notes.sh
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT}"
# shellcheck source=desktop-version.sh
source "${ROOT}/scripts/desktop-version.sh"

TAG="$(desktop_release_tag "${ROOT}")"
if [[ ! "${TAG}" =~ ^desktop-v([0-9]+\.[0-9]+\.[0-9]+)$ ]]; then
	echo "❌ TAG 格式不对：${TAG}（应为 desktop-vX.Y.Z）" >&2
	exit 1
fi
VERSION="${BASH_REMATCH[1]}"
REPO_URL="${VPK_REPO_URL:-https://github.com/1usdc/pusa}"
REPO="${REPO_URL#https://github.com/}"
REPO="${REPO%/}"
BASE="https://github.com/${REPO}/releases/download/${TAG}"

command -v gh >/dev/null 2>&1 || {
	echo "❌ 未找到 gh CLI" >&2
	exit 1
}

gh release view "${TAG}" >/dev/null

mapfile -t ASSETS < <(gh release view "${TAG}" --json assets -q '.assets[].name' 2>/dev/null || true)
has() {
	local n="$1" a
	for a in "${ASSETS[@]+"${ASSETS[@]}"}"; do
		[[ "${a}" == "${n}" ]] && return 0
	done
	return 1
}

MAC_DMG="Pusa-${VERSION}-macos-arm64.dmg"
WIN_SETUP="Pusa-win-x64-Setup.exe"
WIN_SETUP_ALT="Pusa_${VERSION}_x64-setup.exe"

MAC_FILE=""
WIN_FILE=""
has "${MAC_DMG}" && MAC_FILE="${MAC_DMG}"
if has "${WIN_SETUP}"; then
	WIN_FILE="${WIN_SETUP}"
elif has "${WIN_SETUP_ALT}"; then
	WIN_FILE="${WIN_SETUP_ALT}"
fi

NOTES="$(mktemp)"
trap 'rm -f "${NOTES}"' EXIT

{
	echo "## Pusa ${VERSION}"
	echo
	echo "Pusa 是本地优先的 AI 编程与办公助手：把大模型、文件系统、终端与内置浏览器放进同一个桌面窗口，Agent 可以直接读写项目文件、执行命令并在应用内预览结果。"
	echo
	echo "### 一键下载"
	echo
	if [[ -n "${MAC_FILE}" ]]; then
		echo "[⬇️ 下载 macOS（Apple Silicon）](${BASE}/${MAC_FILE})"
		echo
	fi
	if [[ -n "${WIN_FILE}" ]]; then
		echo "[⬇️ 下载 Windows（x64）](${BASE}/${WIN_FILE})"
		echo
	fi
	if [[ -z "${MAC_FILE}" && -z "${WIN_FILE}" ]]; then
		echo "_暂无安装包资产，请查看下方 Assets 列表。_"
		echo
	fi
	echo "### 文件说明"
	echo
	echo "| 文件 | 平台 | 说明 |"
	echo "|---|---|---|"
	if [[ -n "${MAC_FILE}" ]]; then
		echo "| \`${MAC_FILE}\` | macOS | 推荐。打开后把 Pusa 拖进 Applications |"
	fi
	if [[ -n "${WIN_FILE}" ]]; then
		echo "| \`${WIN_FILE}\` | Windows | 推荐。双击安装（缺 WebView2 时安装器会引导） |"
	fi
	echo
	echo "应用内更新依赖同 Release 中的 \`releases.*.json\` 与 \`*-full.nupkg\`（无需手动下载）。"
} >"${NOTES}"

gh release edit "${TAG}" --notes-file "${NOTES}"
echo "✓ Release ${TAG} 说明已更新（mac=${MAC_FILE:-无}；win=${WIN_FILE:-无}）"
echo "  https://github.com/${REPO}/releases/tag/${TAG}"
