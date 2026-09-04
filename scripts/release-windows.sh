#!/usr/bin/env bash
# Windows 发版：本机打包（可选）→ 打 tag → 上传 GitHub Release。
# 版本默认取自 desktop/Cargo.toml → desktop-vX.Y.Z（可用 TAG= 覆盖）。
# 有 desktop/dist/velopack/releases.<channel>.json 时走 `vpk upload github`（应用内增量更新依赖它），
# 否则退回上传 *-setup.exe / *.msi。
#
# 用法：
#   just release-windows
#   BUILD=1 just release-windows   # 先打包再上传
#   TAG=desktop-v0.2.8 just release-windows   # 覆盖 tag
#   VPK_CHANNEL=win-x64 just release-windows  # 在 mac 上给拷过来的 Windows Velopack 产物发版
#
# 环境变量：
#   TAG         可选，默认 desktop-v$(desktop/Cargo.toml version)
#   BUILD       1 时先跑 scripts/desktop-windows.ps1
#   RELEASE_DIR 产物目录（默认 desktop/dist）
#   VPK_CHANNEL Velopack channel（Windows 主机默认 win-x64；非 Windows 主机未设时也按 win-x64）
#   VERBOSE     1 详细日志
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT}"
# shellcheck source=desktop-version.sh
source "${ROOT}/scripts/desktop-version.sh"
# shellcheck source=velopack-common.sh
source "${ROOT}/scripts/velopack-common.sh"

TAG="$(desktop_release_tag "${ROOT}")"
BUILD="${BUILD:-0}"
RELEASE_DIR="${RELEASE_DIR:-${ROOT}/desktop/dist}"
verbose="${VERBOSE:-0}"
say() { [[ "${verbose}" == "1" ]] && echo "$@" || true; }
git_q() { if [[ "${verbose}" == "1" ]]; then git "$@"; else git "$@" --quiet; fi; }

if [[ ! "${TAG}" =~ ^desktop-v[0-9]+\.[0-9]+\.[0-9]+ ]]; then
	echo "❌ TAG 格式不对：${TAG}（应为 desktop-vX.Y.Z）" >&2
	exit 1
fi

echo "→ Release tag: ${TAG}（来自 desktop/Cargo.toml，可用 TAG= 覆盖）"

if ! command -v gh >/dev/null 2>&1; then
	echo "❌ 未找到 gh CLI，请先安装并 gh auth login" >&2
	exit 1
fi

if [[ -n "$(git status --porcelain)" ]]; then
	echo "❌ 工作区不干净，请先提交或 stash" >&2
	git status --short >&2
	exit 1
fi

if [[ "${BUILD}" == "1" ]]; then
	case "$(uname -s)" in
		MINGW* | MSYS* | CYGWIN*) ;;
		*)
			if [[ "${OS:-}" != "Windows_NT" ]]; then
				echo "❌ BUILD=1 需要在 Windows 上执行" >&2
				exit 1
			fi
			;;
	esac
	say "== 0) 本机打包 =="
	powershell.exe -NoProfile -ExecutionPolicy Bypass -File "${ROOT}/scripts/desktop-windows.ps1"
	RELEASE_DIR="${ROOT}/desktop/dist"
fi

# 优先 Velopack 产物（desktop/dist/velopack/releases.<channel>.json 存在即认为是 Velopack 构建）。
# 本脚本只发 Windows 包：非 Windows 主机上 vpk_channel 会按 host 给 osx-*，这里强制回 win-x64（可用 VPK_CHANNEL 覆盖）。
VPK_OUT="$(vpk_out_dir "${ROOT}")"
VPK_CHANNEL="$(vpk_channel)"
if [[ "${VPK_CHANNEL}" != win-* ]]; then
	VPK_CHANNEL="win-x64"
fi
MODE="setup"
if [[ -f "${VPK_OUT}/releases.${VPK_CHANNEL}.json" ]]; then
	MODE="velopack"
	require_vpk
	echo "→ Velopack 模式：${VPK_OUT}（channel ${VPK_CHANNEL}）"
fi

STAGE="${ROOT}/.release-upload"
rm -rf "${STAGE}"
mkdir -p "${STAGE}"

if [[ "${MODE}" == "setup" ]]; then
	shopt -s nullglob
	found=0
	for f in "${RELEASE_DIR}"/*-setup.exe "${RELEASE_DIR}"/*.msi; do
		[[ -e "${f}" ]] || continue
		cp -v "${f}" "${STAGE}/"
		found=1
	done
	shopt -u nullglob

	if [[ "${found}" -eq 0 ]]; then
		echo "❌ ${RELEASE_DIR} 里没有 Velopack 产物也没有 *-setup.exe，请先 just desktop-windows" >&2
		exit 1
	fi
fi

say "== 1) git push =="
git_q push

SHA="$(git rev-parse HEAD)"

say "== 2) 创建并推送 tag ${TAG} =="
if git rev-parse "${TAG}" >/dev/null 2>&1; then
	local_tag_sha="$(git rev-parse "${TAG}^{commit}")"
	if [[ "${local_tag_sha}" != "${SHA}" ]]; then
		echo "❌ 本地 tag ${TAG} 指向 ${local_tag_sha}，当前 HEAD 是 ${SHA}" >&2
		exit 1
	fi
	say "本地已有 tag ${TAG}，跳过 git tag"
else
	git tag "${TAG}"
fi

if git ls-remote --tags origin "refs/tags/${TAG}" | grep -q "${TAG}"; then
	remote_sha="$(git ls-remote --tags origin "refs/tags/${TAG}" | awk '{print $1}')"
	if [[ "${remote_sha}" != "$(git rev-parse "${TAG}^{commit}")" ]]; then
		echo "❌ remote 已有 tag ${TAG} 但指向不同 commit" >&2
		exit 1
	fi
	say "remote 已有 tag ${TAG}"
else
	git_q push origin "${TAG}"
fi

say "== 3) 上传 GitHub Release =="
if [[ "${MODE}" == "velopack" ]]; then
	# vpk 会上传 Setup.exe / Portable.zip / full+delta .nupkg / releases.<channel>.json；
	# --merge 让 macOS 产物能挂到同一个 Release。
	vpk_upload_github "${VPK_OUT}" "${VPK_CHANNEL}" "${TAG}" "${SHA}" "${TAG#desktop-v}"
	echo "✓ Windows Velopack Release ${TAG} 已上传（channel ${VPK_CHANNEL}）"
	ls -lh "${VPK_OUT}"
	exit 0
fi

if gh release view "${TAG}" >/dev/null 2>&1; then
	gh release view "${TAG}" --json assets -q '.assets[].name' \
		| while IFS= read -r name; do
			[[ -n "${name}" ]] || continue
			[[ "${name}" == *-setup.exe || "${name}" == *.msi ]] || continue
			say "delete asset: ${name}"
			gh release delete-asset "${TAG}" "${name}" --yes || true
		done
	gh release upload "${TAG}" "${STAGE}"/* --clobber
else
	gh release create "${TAG}" \
		--target "${SHA}" \
		--generate-notes \
		"${STAGE}"/*
fi

echo "✓ Windows Release ${TAG} 已上传"
ls -lh "${STAGE}"
