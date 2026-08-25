#!/usr/bin/env bash
# 桌面发版：推代码并触发 CI 签名 + GitHub Release。
#
# 用法：
#   TAG=desktop-vX.Y.Z just desktop-ship          # 双平台（推 tag）
#   TAG=desktop-vX.Y.Z just release-mac           # 仅 macOS
#   TAG=desktop-vX.Y.Z just release-windows       # 仅 Windows
#
# 环境变量：
#   TAG        必填，desktop-vX.Y.Z
#   PLATFORM   all|mac|windows（默认 all）
#   WATCH=1    等待 CI 结束
#   VERBOSE=1  详细日志
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT}"

: "${TAG:?用法: TAG=desktop-vX.Y.Z just release-mac|release-windows|desktop-ship}"
PLATFORM="${PLATFORM:-all}"
verbose="${VERBOSE:-0}"
watch="${WATCH:-0}"
say() { [[ "${verbose}" == "1" ]] && echo "$@" || true; }
git_q() { if [[ "${verbose}" == "1" ]]; then git "$@"; else git "$@" --quiet; fi; }

if [[ ! "${TAG}" =~ ^desktop-v[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
	echo "❌ TAG 格式不对：${TAG}（应为 desktop-vX.Y.Z，如 desktop-v0.2.3）" >&2
	exit 1
fi

case "${PLATFORM}" in
	all|mac|windows) ;;
	*)
		echo "❌ PLATFORM 无效：${PLATFORM}（应为 all|mac|windows）" >&2
		exit 1
		;;
esac

if [[ -n "$(git status --porcelain)" ]]; then
	echo "❌ 工作区不干净，请先提交或 stash" >&2
	git status --short >&2
	exit 1
fi

say "== 1) git push（推送本地 commits） =="
git_q push

BRANCH="$(git rev-parse --abbrev-ref HEAD)"
SHA="$(git rev-parse HEAD)"

if [[ "${PLATFORM}" == "all" ]]; then
	# 双平台：推 tag，由 push:tags 触发完整 matrix + Release
	if git rev-parse "${TAG}" >/dev/null 2>&1; then
		echo "❌ 本地已有 tag ${TAG}，请 bump 或先删：git tag -d ${TAG}" >&2
		exit 1
	fi
	if [[ -n "$(git ls-remote --tags origin "refs/tags/${TAG}" 2>/dev/null)" ]]; then
		echo "❌ remote 已有 tag ${TAG}，请 bump 或先删：git push origin :refs/tags/${TAG}" >&2
		exit 1
	fi
	say "== 2) git tag ${TAG} && git push origin ${TAG}（双平台 CI） =="
	git tag "${TAG}"
	git_q push origin "${TAG}"
	trigger="tag"
else
	# 单平台：workflow_dispatch，避免再推 tag 触发双平台
	say "== 2) gh workflow run desktop-bundle.yml (platform=${PLATFORM}, tag=${TAG}) =="
	gh workflow run desktop-bundle.yml \
		--ref "${BRANCH}" \
		-f "platform=${PLATFORM}" \
		-f "release_tag=${TAG}"
	trigger="dispatch"
fi

if [[ "${watch}" != "1" ]]; then
	if [[ "${trigger}" == "tag" ]]; then
		echo "✓ 已触发双平台 CI（tag ${TAG}，约 10-20 min 出 Release）"
	else
		echo "✓ 已触发 ${PLATFORM} CI（release_tag=${TAG}，约 10-20 min 出 Release）"
		echo "  查看：gh run list --workflow=desktop-bundle.yml --limit 5"
	fi
	exit 0
fi

say "== 3) 等 CI run 注册并跟踪（Ctrl-C 安全退出，CI 在云端继续跑）=="
sleep 6
run_id=""
for _ in 1 2 3 4 5 6 7 8; do
	if [[ "${trigger}" == "tag" ]]; then
		run_id="$(gh run list --workflow=desktop-bundle.yml --limit 10 \
			--json databaseId,headBranch \
			--jq ".[] | select(.headBranch == \"${TAG}\") | .databaseId" | head -n1)"
	else
		run_id="$(gh run list --workflow=desktop-bundle.yml --limit 10 \
			--json databaseId,headSha,event,status \
			--jq ".[] | select(.event == \"workflow_dispatch\" and .headSha == \"${SHA}\") | .databaseId" | head -n1)"
	fi
	[[ -n "${run_id}" ]] && break
	sleep 4
done
if [[ -z "${run_id}" ]]; then
	echo "⚠️  没抓到 run id，去 Actions 看：gh run list --workflow=desktop-bundle.yml"
	exit 1
fi

if [[ "${verbose}" == "1" ]]; then
	gh run watch "${run_id}" --exit-status || {
		echo "⚠️  CI 未通过；详情：gh run view ${run_id} --log-failed" >&2
		exit 1
	}
else
	gh run watch "${run_id}" --exit-status >/dev/null 2>&1 || {
		echo "⚠️  CI 未通过（run ${run_id}）；详情：gh run view ${run_id} --log-failed" >&2
		exit 1
	}
fi
echo "✓ Release ${TAG}（${PLATFORM}）已就绪"
