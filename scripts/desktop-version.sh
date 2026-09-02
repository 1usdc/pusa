#!/usr/bin/env bash
# 从 desktop/Cargo.toml 读取 [package].version，拼成 desktop-vX.Y.Z。
# 用法：source 后调用 desktop_release_tag；可用 TAG=... 覆盖。
desktop_release_tag() {
	local root="${1:-.}"
	local cargo="${root}/desktop/Cargo.toml"
	if [[ -n "${TAG:-}" ]]; then
		printf '%s' "${TAG}"
		return 0
	fi
	if [[ ! -f "${cargo}" ]]; then
		echo "❌ 未找到 ${cargo}" >&2
		return 1
	fi
	local ver
	ver="$(
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
		' "${cargo}"
	)"
	if [[ ! "${ver}" =~ ^[0-9]+\.[0-9]+\.[0-9]+([.-].*)?$ ]]; then
		echo "❌ 无法从 desktop/Cargo.toml 解析 version（得到：${ver:-空}）" >&2
		return 1
	fi
	printf 'desktop-v%s' "${ver}"
}
