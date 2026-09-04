#!/usr/bin/env bash
# macOS：把 dx bundle 产出的 Pusa.app 打成 Velopack 发布包（签名 + 公证由 vpk 完成）。
#
# 输入：desktop/dist/Pusa.app（已嵌入 libpusa_core.dylib，**未签名**——vpk 会在注入 UpdateMac 后统一深签）
# 输出：desktop/dist/velopack/（.pkg 安装器、Portable.zip、full/delta .nupkg、releases.<channel>.json）
#
# 用法：
#   bash scripts/velopack-pack-mac.sh            # 签名 + 公证
#   NOTARIZE=0 bash scripts/velopack-pack-mac.sh # 只签名
#   RELEASE_NOTES_FILE=CHANGELOG.md ...          # 可选：写进 releases.json 的发布说明（markdown）
#
# 凭证：.env.signing（MACOS_SIGNING_IDENTITY / APPLE_ID / APPLE_APP_SPECIFIC_PASSWORD / APPLE_TEAM_ID）
#   .pkg 需要 “Developer ID Installer” 证书；钥匙串里没有时自动 --noInst，只出 Portable.zip。
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIST="${ROOT}/desktop/dist"
ENTITLEMENTS="${ROOT}/desktop/entitlements.plist"
NOTARIZE="${NOTARIZE:-1}"
NOTARY_PROFILE="${NOTARY_PROFILE:-pusa-velopack}"

# shellcheck source=velopack-common.sh
source "${ROOT}/scripts/velopack-common.sh"
# shellcheck source=desktop-version.sh
source "${ROOT}/scripts/desktop-version.sh"

log() { printf '%s\n' "$*"; }
die() { printf '错误: %s\n' "$*" >&2; exit 1; }

[[ "$(uname -s)" == "Darwin" ]] || die "本脚本仅适用于 macOS"
require_vpk

if [[ -f "${ROOT}/.env.signing" ]]; then
	set -a
	# shellcheck source=/dev/null
	source "${ROOT}/.env.signing"
	set +a
fi

find_app_bundle() {
	if [[ -n "${APP:-}" && -d "${APP}" ]]; then
		printf '%s' "${APP}"
		return 0
	fi
	local candidate
	candidate="$(find "${DIST}" -maxdepth 1 -name '*.app' -type d ! -name 'Desktop.app' 2>/dev/null | head -n1)"
	[[ -n "${candidate}" ]] || candidate="$(find "${DIST}" -maxdepth 1 -name '*.app' -type d 2>/dev/null | head -n1)"
	[[ -n "${candidate}" ]] || die "未找到 .app，请先 dx bundle --release（输出目录 ${DIST}）"
	printf '%s' "${candidate}"
}

resolve_app_identity() {
	if [[ -n "${MACOS_SIGNING_IDENTITY:-}" ]]; then
		printf '%s' "${MACOS_SIGNING_IDENTITY}"
		return 0
	fi
	security find-identity -v -p codesigning 2>/dev/null \
		| awk -F'"' '/Developer ID Application/ { print $2; exit }'
}

resolve_installer_identity() {
	if [[ -n "${MACOS_INSTALLER_IDENTITY:-}" ]]; then
		printf '%s' "${MACOS_INSTALLER_IDENTITY}"
		return 0
	fi
	# Installer 证书不在 codesigning policy 里，直接列全部
	security find-identity -v 2>/dev/null \
		| awk -F'"' '/Developer ID Installer/ { print $2; exit }'
}

# vpk 只接受 notarytool 的 keychain profile 名；用 .env.signing 里的 Apple ID 凭证（重新）写一份
ensure_notary_profile() {
	[[ -n "${APPLE_ID:-}" ]] || die "请设置 APPLE_ID（或 NOTARIZE=0 跳过公证）"
	[[ -n "${APPLE_APP_SPECIFIC_PASSWORD:-}" ]] || die "请设置 APPLE_APP_SPECIFIC_PASSWORD"
	[[ -n "${APPLE_TEAM_ID:-}" ]] || die "请设置 APPLE_TEAM_ID"
	xcrun notarytool store-credentials "${NOTARY_PROFILE}" \
		--apple-id "${APPLE_ID}" \
		--team-id "${APPLE_TEAM_ID}" \
		--password "${APPLE_APP_SPECIFIC_PASSWORD}" >/dev/null
}

main() {
	local app ver channel out app_identity inst_identity main_exe
	local -a args

	app="$(find_app_bundle)"
	ver="$(desktop_release_tag "${ROOT}")"
	ver="${ver#desktop-v}"
	channel="$(vpk_channel)"
	out="$(vpk_out_dir "${ROOT}")"
	main_exe="$(plutil -extract CFBundleExecutable raw "${app}/Contents/Info.plist")"

	[[ -f "${app}/Contents/MacOS/libpusa_core.dylib" ]] \
		|| die "${app} 里没有 libpusa_core.dylib，请先 bash scripts/embed-pusa-core-macos.sh"
	[[ -f "${ENTITLEMENTS}" ]] || die "缺少 entitlements: ${ENTITLEMENTS}"

	app_identity="$(resolve_app_identity)"
	[[ -n "${app_identity}" ]] || die "未找到 Developer ID Application 证书；请设置 MACOS_SIGNING_IDENTITY 或在钥匙串安装证书"
	inst_identity="$(resolve_installer_identity)"

	log "== Velopack pack（macOS）=="
	log "   app:      ${app}"
	log "   version:  ${ver}"
	log "   channel:  ${channel}"
	log "   mainExe:  ${main_exe}"
	log "   sign app: ${app_identity}"

	mkdir -p "${out}"
	# 清掉本机上次同版本残留，但保留 vpk download 拉下来的旧版（用于 delta）
	find "${out}" -maxdepth 1 -type f -name "${VPK_PACK_ID}-${ver}-*" -delete 2>/dev/null || true

	# vpk 强制要求 --signEntitlements 文件以 .entitlements 结尾；仓库里是 .plist，复制一份改名
	# （放临时目录，避免混进产物目录被 vpk upload 一起上传）
	local entitlements_vpk="${TMPDIR:-/tmp}/${VPK_PACK_ID}.entitlements"
	cp "${ENTITLEMENTS}" "${entitlements_vpk}"

	vpk_download_previous "${out}" "${channel}"

	args=(
		pack
		--packId "${VPK_PACK_ID}"
		--packVersion "${ver}"
		--packDir "${app}"
		--mainExe "${main_exe}"
		--packTitle "${VPK_PACK_TITLE}"
		--packAuthors "${VPK_PACK_AUTHORS}"
		--icon "${ROOT}/desktop/assets/Pusa.icns"
		--channel "${channel}"
		--outputDir "${out}"
		--signAppIdentity "${app_identity}"
		--signEntitlements "${entitlements_vpk}"
	)

	if [[ -n "${inst_identity}" ]]; then
		log "   sign pkg: ${inst_identity}"
		args+=(--signInstallIdentity "${inst_identity}")
	else
		log "   ⚠ 钥匙串无 “Developer ID Installer” 证书：跳过 .pkg，只出 Portable.zip（--noInst）"
		args+=(--noInst)
	fi

	if [[ "${NOTARIZE}" == "1" ]]; then
		ensure_notary_profile
		log "   notarize: profile=${NOTARY_PROFILE}"
		args+=(--notaryProfile "${NOTARY_PROFILE}")
	else
		log "   NOTARIZE=0：跳过公证"
	fi

	if [[ -n "${RELEASE_NOTES_FILE:-}" && -f "${RELEASE_NOTES_FILE}" ]]; then
		args+=(--releaseNotes "${RELEASE_NOTES_FILE}")
	fi

	# 与 CI 一致：非交互
	vpk "${args[@]}" --yes

	log "✓ Velopack 产物：${out}"
	ls -lh "${out}"
}

main "$@"
