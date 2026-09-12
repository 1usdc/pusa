#!/usr/bin/env bash
# macOS：把 vpk pack 产出的 Portable.zip（内含已签名+公证、带 UpdateMac 的 Pusa.app）改封装成 DMG，
# 并从 Velopack 产物里移除 Portable.zip（assets.<channel>.json 同步去掉 Portable 条目，vpk upload 不再上传它）。
#
# 产物：desktop/dist/velopack/Pusa-<ver>-<channel>.dmg（带 Applications 拖拽快捷方式，已签名、公证、staple）
# DMG 里的 .app 就是 Velopack 处理过的那份，拖进 Applications 后同样支持应用内增量更新。
#
# 用法（velopack-pack-mac.sh 末尾自动调用；也可单独跑）：
#   bash scripts/velopack-dmg-mac.sh
#   NOTARIZE=0 bash scripts/velopack-dmg-mac.sh   # 不公证 DMG（内部 .app 本身已公证）
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
NOTARIZE="${NOTARIZE:-1}"
NOTARY_PROFILE="${NOTARY_PROFILE:-pusa-velopack}"

# shellcheck source=velopack-common.sh
source "${ROOT}/scripts/velopack-common.sh"
# shellcheck source=desktop-version.sh
source "${ROOT}/scripts/desktop-version.sh"

log() { printf '%s\n' "$*"; }
die() { printf '错误: %s\n' "$*" >&2; exit 1; }

[[ "$(uname -s)" == "Darwin" ]] || die "本脚本仅适用于 macOS"

if [[ -f "${ROOT}/.env.signing" ]]; then
	set -a
	# shellcheck source=/dev/null
	source "${ROOT}/.env.signing"
	set +a
fi

resolve_app_identity() {
	if [[ -n "${MACOS_SIGNING_IDENTITY:-}" ]]; then
		printf '%s' "${MACOS_SIGNING_IDENTITY}"
		return 0
	fi
	security find-identity -v -p codesigning 2>/dev/null \
		| awk -F'"' '/Developer ID Application/ { print $2; exit }'
}

# 从 assets.<channel>.json 删掉 Portable 条目（文件已被 DMG 取代）
drop_portable_asset() {
	local assets_json="$1"
	[[ -f "${assets_json}" ]] || return 0
	python3 - "${assets_json}" <<'PY'
import json, sys
p = sys.argv[1]
with open(p, encoding="utf-8") as f:
    items = json.load(f)
items = [a for a in items if a.get("Type") != "Portable"]
with open(p, "w", encoding="utf-8") as f:
    json.dump(items, f, ensure_ascii=False)
PY
}

# 临时工作目录；必须是全局变量——EXIT trap 在 main 返回后才跑，local 变量此时已不存在（set -u 会报 unbound）
WORK_DIR=""
cleanup() { [[ -n "${WORK_DIR}" && -d "${WORK_DIR}" ]] && rm -rf "${WORK_DIR}"; return 0; }
trap cleanup EXIT

main() {
	local out channel ver zip dmg identity app
	out="$(vpk_out_dir "${ROOT}")"
	channel="$(vpk_channel)"
	ver="$(desktop_release_tag "${ROOT}")"
	ver="${ver#desktop-v}"
	zip="${out}/${VPK_PACK_ID}-${channel}-Portable.zip"
	dmg="${out}/${VPK_PACK_ID}-${ver}-${channel}.dmg"

	[[ -f "${zip}" ]] || die "未找到 ${zip}，请先 bash scripts/velopack-pack-mac.sh"
	identity="$(resolve_app_identity)"
	[[ -n "${identity}" ]] || die "未找到 Developer ID Application 证书"

	WORK_DIR="$(mktemp -d "${TMPDIR:-/tmp}/pusa-dmg.XXXXXX")"
	local work="${WORK_DIR}"

	log "== 解包 Portable.zip → ${work} =="
	# ditto 保留权限 / 符号链接 / 资源分叉，unzip 会丢
	ditto -x -k "${zip}" "${work}/stage"
	app="$(find "${work}/stage" -maxdepth 1 -name '*.app' -type d | head -n1)"
	[[ -n "${app}" ]] || die "Portable.zip 里没有 .app"
	rm -rf "${work}/stage/__MACOSX"
	[[ -f "${app}/Contents/MacOS/UpdateMac" ]] || die "${app} 缺 UpdateMac，不是 Velopack 处理过的包"

	log "== 校验 .app 签名 =="
	codesign --verify --deep --strict --verbose=1 "${app}"
	spctl -a -vv --type exec "${app}" 2>&1 | sed 's/^/   /'

	log "== 生成 DMG: $(basename "${dmg}") =="
	ln -s /Applications "${work}/stage/Applications"
	rm -f "${dmg}"
	hdiutil create -volname "${VPK_PACK_TITLE}" -srcfolder "${work}/stage" -ov -format UDZO "${dmg}" >/dev/null

	log "== 签名 DMG =="
	codesign --force --timestamp --sign "${identity}" "${dmg}"

	if [[ "${NOTARIZE}" == "1" ]]; then
		log "== 公证 DMG（profile=${NOTARY_PROFILE}）=="
		xcrun notarytool submit "${dmg}" --keychain-profile "${NOTARY_PROFILE}" --wait
		xcrun stapler staple "${dmg}"
		xcrun stapler validate "${dmg}"
	else
		log "NOTARIZE=0：跳过 DMG 公证"
	fi

	log "== 用 DMG 取代 Portable.zip =="
	rm -f "${zip}"
	drop_portable_asset "${out}/assets.${channel}.json"

	log "✓ DMG：${dmg}"
	ls -lh "${dmg}"
}

main "$@"
