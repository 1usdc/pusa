#!/usr/bin/env bash
# macOS：Developer ID 签名 + notarytool 公证 + staple，并重打 DMG。
# 凭证见 .env.signing（MACOS_SIGNING_IDENTITY / APPLE_ID / APPLE_APP_SPECIFIC_PASSWORD / APPLE_TEAM_ID）。
# 用法：dx bundle --release 后跑本脚本；APP=... 指定 .app；NOTARIZE=0 仅签名。
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIST="${ROOT}/desktop/dist"
ENTITLEMENTS="${ROOT}/desktop/entitlements.plist"
NOTARIZE="${NOTARIZE:-1}"

log() { printf '%s\n' "$*"; }
die() { printf '错误: %s\n' "$*" >&2; exit 1; }

[[ "$(uname -s)" == "Darwin" ]] || die "本脚本仅适用于 macOS"

if [[ -f "${ROOT}/.env.signing" ]]; then
	set -a
	# shellcheck source=/dev/null
	source "${ROOT}/.env.signing"
	set +a
fi

resolve_signing_identity() {
	if [[ -n "${MACOS_SIGNING_IDENTITY:-}" ]]; then
		printf '%s' "${MACOS_SIGNING_IDENTITY}"
		return 0
	fi
	security find-identity -v -p codesigning 2>/dev/null \
		| awk -F'"' '/Developer ID Application/ { print $2; exit }'
}

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

sign_app() {
	local app="$1"
	local identity="$2"
	local exe="${app}/Contents/MacOS"

	[[ -f "${ENTITLEMENTS}" ]] || die "缺少 entitlements: ${ENTITLEMENTS}"

	log "== 签名: $(basename "${app}") =="
	log "   identity: ${identity}"

	# 先签内部 Mach-O，再签 .app（避免 --deep 的已知问题）
	while IFS= read -r -d '' bin; do
		codesign --force --timestamp --options runtime \
			--entitlements "${ENTITLEMENTS}" \
			--sign "${identity}" "${bin}"
	done < <(find "${app}" -type f \( -perm -111 -o -name '*.dylib' \) -print0 2>/dev/null)

	if [[ -d "${exe}" ]]; then
		while IFS= read -r -d '' bin; do
			codesign --force --timestamp --options runtime \
				--entitlements "${ENTITLEMENTS}" \
				--sign "${identity}" "${bin}"
		done < <(find "${exe}" -type f -print0)
	fi

	codesign --force --timestamp --options runtime \
		--entitlements "${ENTITLEMENTS}" \
		--sign "${identity}" "${app}"

	codesign --verify --verbose=2 "${app}"
	spctl -a -vv "${app}" 2>&1 || true
}

build_dmg() {
	local app="$1"
	local app_name ver dmg
	app_name="$(basename "${app}" .app)"
	ver="$(plutil -extract CFBundleShortVersionString raw "${app}/Contents/Info.plist")"
	rm -f "${DIST}"/*.dmg
	# 顺带清 dx 直出的未签名 DMG，避免和公证成品混淆
	if [[ -d "${ROOT}/target/dx" ]]; then
		find "${ROOT}/target/dx" -type f -name '*.dmg' -delete 2>/dev/null || true
	fi
	dmg="${DIST}/${app_name// /_}_${ver}_aarch64.dmg"
	# hdiutil 会往 stdout 打 created:，须重定向以免污染命令替换
	hdiutil create -volname "${app_name}" -srcfolder "${app}" -ov -format UDZO "${dmg}" >&2
	printf '%s' "${dmg}"
}

notarize_dmg() {
	local dmg="$1"
	[[ "${NOTARIZE}" == "1" ]] || {
		log "NOTARIZE=0，跳过公证"
		return 0
	}

	[[ -n "${APPLE_ID:-}" ]] || die "请设置 APPLE_ID"
	[[ -n "${APPLE_APP_SPECIFIC_PASSWORD:-}" ]] || die "请设置 APPLE_APP_SPECIFIC_PASSWORD"
	[[ -n "${APPLE_TEAM_ID:-}" ]] || die "请设置 APPLE_TEAM_ID"

	log "== 提交公证: $(basename "${dmg}") =="
	# 不用 --wait（runner 偶发网络错误）；改为 submit 后自轮询
	local submit_json submission_id
	submit_json="$(
		xcrun notarytool submit "${dmg}" \
			--apple-id "${APPLE_ID}" \
			--password "${APPLE_APP_SPECIFIC_PASSWORD}" \
			--team-id "${APPLE_TEAM_ID}" \
			--output-format json
	)"
	submission_id="$(printf '%s' "${submit_json}" | python3 -c 'import json,sys; print(json.load(sys.stdin).get("id",""))')"
	[[ -n "${submission_id}" ]] || die "submit 没拿到 id: ${submit_json}"
	log "submission id = ${submission_id}"

	# 默认每 15s 轮询，最长 1h（按墙钟 deadline）
	local poll_max_seconds="${NOTARIZE_POLL_MAX_SECONDS:-3600}"
	local poll_interval="${NOTARIZE_POLL_INTERVAL:-15}"
	local soft_fail="${NOTARIZE_SOFT_FAIL:-0}"
	local started_at deadline now elapsed info_json status
	started_at="$(date +%s)"
	deadline=$((started_at + poll_max_seconds))
	status=""
	while :; do
		now="$(date +%s)"
		elapsed=$((now - started_at))
		(( now < deadline )) || break
		info_json="$(
			xcrun notarytool info "${submission_id}" \
				--apple-id "${APPLE_ID}" \
				--password "${APPLE_APP_SPECIFIC_PASSWORD}" \
				--team-id "${APPLE_TEAM_ID}" \
				--output-format json 2>/dev/null
		)" || info_json=""
		status="$(printf '%s' "${info_json}" | python3 -c 'import json,sys
try:
    print(json.load(sys.stdin).get("status",""))
except Exception:
    print("")')"
		case "${status}" in
			Accepted)
				log "公证通过 (Accepted) - 用时 ${elapsed}s"
				break
				;;
			Invalid|Rejected)
				log "公证被拒：${status}，拉日志看原因："
				xcrun notarytool log "${submission_id}" \
					--apple-id "${APPLE_ID}" \
					--password "${APPLE_APP_SPECIFIC_PASSWORD}" \
					--team-id "${APPLE_TEAM_ID}" || true
				die "notarytool ${status}"
				;;
			"")
				log "  … 第 ${elapsed}s：info 调用失败（多半网络抖动），${poll_interval}s 后重试"
				;;
			*)
				log "  … 第 ${elapsed}s：status=${status}"
				;;
		esac
		sleep "${poll_interval}"
	done

	if [[ "${status}" != "Accepted" ]]; then
		local hint
		hint="Apple 可能仍在排队，可手动查询：
  xcrun notarytool info ${submission_id} --apple-id \$APPLE_ID --password \$APPLE_APP_SPECIFIC_PASSWORD --team-id \$APPLE_TEAM_ID
处理完毕后再补 staple：
  xcrun stapler staple '${dmg}'"
		if [[ "${soft_fail}" == "1" ]]; then
			log "::warning::公证轮询超时（${poll_max_seconds}s），最后状态=${status:-unknown}。NOTARIZE_SOFT_FAIL=1，跳过 staple 继续发布。"
			log "${hint}"
			return 0
		fi
		die "公证轮询超时（${poll_max_seconds}s），最后状态=${status:-unknown}。${hint}"
	fi

	log "== staple DMG =="
	xcrun stapler staple "${dmg}"
	xcrun stapler validate "${dmg}"
}

main() {
	local app identity dmg
	identity="$(resolve_signing_identity)"
	[[ -n "${identity}" ]] || die "未找到 Developer ID Application 证书；请设置 MACOS_SIGNING_IDENTITY 或在钥匙串安装证书"

	app="$(find_app_bundle)"
	bash "${ROOT}/scripts/embed-pusa-core-macos.sh"
	sign_app "${app}" "${identity}"
	dmg="$(build_dmg "${app}")"
	log "已生成 DMG: ${dmg}"

	# 公证前对 DMG 签名（可选但推荐）
	if [[ "${NOTARIZE}" == "1" ]]; then
		codesign --force --timestamp --sign "${identity}" "${dmg}" || true
	fi
	notarize_dmg "${dmg}"

	log "完成。分发文件: ${dmg}"
}

main "$@"
