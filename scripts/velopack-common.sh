#!/usr/bin/env bash
# Velopack 公共配置（被 velopack-pack-mac.sh / release-*.sh source）。
#
# 产物目录：desktop/dist/velopack/
#   Pusa-<ver>-<channel>-full.nupkg     全量包（客户端首次 / 回退）
#   Pusa-<ver>-<channel>-delta.nupkg    相对上一版的增量包（需先 vpk download 上一版）
#   releases.<channel>.json             客户端 GithubSource 读取的清单
#   macOS: Pusa-<ver>-<channel>.dmg（始终 --noInst，不出 .pkg；channel 形如 macos-arm64 / macos-x64）
#   Windows: Pusa-<channel>-Setup.exe + Pusa-<channel>-Portable.zip（channel win-x64）
#
# 客户端（ui/src/desktop/updater.rs）用 GithubSource 读 Release 资产里的 releases.<channel>.json，
# channel 由安装包自带的 sq.version 决定，客户端代码无需感知。

# 与 ui/src/desktop/updater.rs::GITHUB_REPO_URL 保持一致
VPK_REPO_URL="${VPK_REPO_URL:-https://github.com/1usdc/pusa}"
# Velopack packId：安装目录名 / 缓存目录名 / Windows 注册表键，发布后不要再改
VPK_PACK_ID="${VPK_PACK_ID:-Pusa}"
VPK_PACK_TITLE="${VPK_PACK_TITLE:-Pusa}"
VPK_PACK_AUTHORS="${VPK_PACK_AUTHORS:-Pusa}"

vpk_out_dir() {
	local root="$1"
	printf '%s' "${VPK_OUT_DIR:-${root}/desktop/dist/velopack}"
}

# 按 host 选 channel；一台机器只出一种架构，channel 名带架构便于以后并存。
# VPK_CHANNEL 可覆盖（如在 mac 上给 Windows 产物发版：VPK_CHANNEL=win-x64 just release-windows）。
vpk_channel() {
	if [[ -n "${VPK_CHANNEL:-}" ]]; then
		printf '%s' "${VPK_CHANNEL}"
		return 0
	fi
	case "$(uname -s)" in
		Darwin)
			case "$(uname -m)" in
				arm64) printf 'macos-arm64' ;;
				*) printf 'macos-x64' ;;
			esac
			;;
		MINGW* | MSYS* | CYGWIN*) printf 'win-x64' ;;
		*)
			if [[ "${OS:-}" == "Windows_NT" ]]; then printf 'win-x64'; else printf 'linux-x64'; fi
			;;
	esac
}

require_vpk() {
	if command -v vpk >/dev/null 2>&1; then
		return 0
	fi
	cat >&2 <<'EOF'
❌ 未找到 vpk（Velopack CLI）。安装：
   1) .NET 8 SDK：macOS `brew install --cask dotnet-sdk`；Windows `winget install Microsoft.DotNet.SDK.8`
   2) dotnet tool install -g vpk
   3) 确认 ~/.dotnet/tools 在 PATH 里（macOS: export PATH="$HOME/.dotnet/tools:$PATH"）
EOF
	return 1
}

# gh 已登录时复用其 token（vpk download/upload github 需要；公开仓库 download 可匿名但有限速）
vpk_github_token() {
	if [[ -n "${GITHUB_TOKEN:-}" ]]; then
		printf '%s' "${GITHUB_TOKEN}"
		return 0
	fi
	if command -v gh >/dev/null 2>&1; then
		gh auth token 2>/dev/null || true
	fi
}

# 拉上一版全量包到产物目录，供 vpk pack 生成 delta。首次发版 / 离线时软失败。
vpk_download_previous() {
	local out="$1" channel="$2" token
	token="$(vpk_github_token)"
	echo "== vpk download github（上一版 ${channel}，用于生成增量包）=="
	if vpk download github \
		--repoUrl "${VPK_REPO_URL}" \
		--channel "${channel}" \
		--outputDir "${out}" \
		${token:+--token "${token}"}; then
		return 0
	fi
	echo "⚠ 未能下载上一版（首次发版或网络问题）；本次只生成全量包，无 delta" >&2
	return 0
}

# 上传本 channel 的产物到 GitHub Release；--merge 允许 mac / windows 挂到同一个 Release
vpk_upload_github() {
	local out="$1" channel="$2" tag="$3" sha="$4" ver="$5" token
	token="$(vpk_github_token)"
	[[ -n "${token}" ]] || {
		echo "❌ 需要 GITHUB_TOKEN 或已登录的 gh（gh auth login）" >&2
		return 1
	}
	vpk upload github \
		--repoUrl "${VPK_REPO_URL}" \
		--channel "${channel}" \
		--outputDir "${out}" \
		--token "${token}" \
		--tag "${tag}" \
		--targetCommitish "${sha}" \
		--releaseName "${VPK_PACK_TITLE} ${ver}" \
		--merge \
		--publish
}
