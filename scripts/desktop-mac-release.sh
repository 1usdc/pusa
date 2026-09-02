#!/usr/bin/env bash
# 把本机已公证的 DMG 挂到 GitHub Release，不提交仓库、不在本机再打包。
#
# 用法：
#   just desktop-mac-release                 # TAG 默认 desktop-v + DMG/Cargo 版本
#   TAG=desktop-vX.Y.Z just desktop-mac-release
#
# 前置：just desktop-mac 已生成 desktop/dist/*.dmg，且 gh 已登录、对 origin 有写权限。
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT}"

if ! command -v gh >/dev/null 2>&1; then
	echo "❌ 未找到 gh，请先安装 GitHub CLI" >&2
	exit 1
fi

if ! gh auth status >/dev/null 2>&1; then
	echo "❌ gh 未登录，请先 gh auth login" >&2
	exit 1
fi

DIST="${ROOT}/desktop/dist"
shopt -s nullglob
dmgs=( "${DIST}"/*.dmg )
shopt -u nullglob

if [[ ${#dmgs[@]} -eq 0 ]]; then
	echo "❌ 没有 DMG，请先 just desktop-mac（输出目录 ${DIST}）" >&2
	exit 1
fi

dmg=""
for f in "${dmgs[@]}"; do
	base="$(basename "${f}")"
	# 公证脚本打出的下划线文件名；跳过 dx 直出的带空格未公证 DMG
	if [[ "${base}" == *_aarch64.dmg && "${base}" != *" "* ]]; then
		if [[ -n "${dmg}" ]]; then
			echo "❌ 找到多份 aarch64 DMG，请只留一份再上传：" >&2
			printf '  %s\n' "${dmgs[@]}" >&2
			exit 1
		fi
		dmg="${f}"
	fi
done

if [[ -z "${dmg}" ]]; then
	if [[ ${#dmgs[@]} -eq 1 ]]; then
		dmg="${dmgs[0]}"
	else
		echo "❌ 无法判断要上传哪份 DMG：" >&2
		printf '  %s\n' "${dmgs[@]}" >&2
		exit 1
	fi
fi

if [[ -z "${TAG:-}" ]]; then
	ver=""
	if [[ "$(basename "${dmg}")" =~ _([0-9]+\.[0-9]+\.[0-9]+)_aarch64\.dmg$ ]]; then
		ver="${BASH_REMATCH[1]}"
	elif [[ "$(basename "${dmg}")" =~ _([0-9]+\.[0-9]+\.[0-9]+)\.dmg$ ]]; then
		ver="${BASH_REMATCH[1]}"
	else
		ver="$(awk -F'"' '/^version[[:space:]]*=/{print $2; exit}' "${ROOT}/desktop/Cargo.toml")"
	fi
	if [[ ! "${ver}" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
		echo "❌ 无法从 DMG 推断版本，请显式指定：TAG=desktop-vX.Y.Z just desktop-mac-release" >&2
		exit 1
	fi
	TAG="desktop-v${ver}"
	echo "未指定 TAG，使用 ${TAG}"
fi

if [[ ! "${TAG}" =~ ^desktop-v[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
	echo "❌ TAG 格式不对：${TAG}（应为 desktop-vX.Y.Z，如 desktop-v0.1.0）" >&2
	exit 1
fi

SHA="$(git rev-parse HEAD)"
if git rev-parse "${TAG}" >/dev/null 2>&1; then
	tagged="$(git rev-parse "${TAG}^{commit}")"
	if [[ "${tagged}" != "${SHA}" ]]; then
		echo "本地 tag ${TAG} 仍指向 ${tagged:0:12}，移到当前 HEAD ${SHA:0:12}"
		git tag -f "${TAG}" "${SHA}"
	fi
else
	git tag "${TAG}"
	echo "已打本地 tag ${TAG} → ${SHA}"
fi

notes="${RELEASE_NOTES:-macOS Apple Silicon (aarch64)。本机 Developer ID 签名并公证。}"

echo "== 上传 $(basename "${dmg}") → Release ${TAG} =="
if gh release view "${TAG}" >/dev/null 2>&1; then
	gh release upload "${TAG}" "${dmg}" --clobber
	gh release edit "${TAG}" --target "${SHA}" >/dev/null
	echo "✓ 已更新已有 Release 的附件，target → ${SHA:0:12}"
else
	gh release create "${TAG}" "${dmg}" \
		--title "${TAG}" \
		--target "${SHA}" \
		--notes "${notes}"
	echo "✓ 已创建 Release ${TAG}"
fi

url="$(gh release view "${TAG}" --json url -q .url)"
echo "下载页：${url}"
