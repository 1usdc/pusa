#!/usr/bin/env bash
# 把 vendor 里的 libpusa_core.dylib 拷进 .app/Contents/MacOS/，供安装用户运行时加载。
# 在 codesign / 公证之前调用。APP=... 指定 .app。
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIST="${ROOT}/desktop/dist"

die() { printf '错误: %s\n' "$*" >&2; exit 1; }

find_app_bundle() {
	if [[ -n "${APP:-}" && -d "${APP}" ]]; then
		printf '%s' "${APP}"
		return 0
	fi
	local candidate
	candidate="$(find "${DIST}" -maxdepth 1 -name '*.app' -type d ! -name 'Desktop.app' 2>/dev/null | head -n1)"
	if [[ -n "${candidate}" ]]; then
		printf '%s' "${candidate}"
		return 0
	fi
	candidate="$(find "${DIST}" -maxdepth 1 -name '*.app' -type d 2>/dev/null | head -n1)"
	[[ -n "${candidate}" ]] || die "未找到 .app，请先 dx bundle --release（输出目录 ${DIST}）"
	printf '%s' "${candidate}"
}

HOST="$(rustc -vV | awk '/^host:/{print $2}')"
LIB="${ROOT}/vendor/pusa-core/${HOST}/libpusa_core.dylib"
[[ -f "${LIB}" ]] || die "缺少 ${LIB}，请先 just pusa-core"

app="$(find_app_bundle)"
dest="${app}/Contents/MacOS/libpusa_core.dylib"
mkdir -p "${app}/Contents/MacOS"
cp -f "${LIB}" "${dest}"
chmod +x "${dest}"
echo "已嵌入: ${dest}"
