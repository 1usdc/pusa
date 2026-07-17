#!/usr/bin/env bash
# 推 TAG=desktop-vX.Y.Z → CI 签名公证 + Release。
# 默认异步；WATCH=1 等 CI；VERBOSE=1 详细日志。
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT}"

: "${TAG:?用法: TAG=desktop-vX.Y.Z just desktop-ship}"

verbose="${VERBOSE:-0}"
watch="${WATCH:-0}"
say() { [[ "${verbose}" == "1" ]] && echo "$@" || true; }
git_q() { if [[ "${verbose}" == "1" ]]; then git "$@"; else git "$@" --quiet; fi; }

if [[ ! "${TAG}" =~ ^desktop-v[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
	echo "❌ TAG 格式不对：${TAG}（应为 desktop-vX.Y.Z，如 desktop-v0.2.3）" >&2
	exit 1
fi

if git rev-parse "${TAG}" >/dev/null 2>&1; then
	echo "❌ 本地已有 tag ${TAG}，请 bump 或先删：git tag -d ${TAG}" >&2
	exit 1
fi
if [[ -n "$(git ls-remote --tags origin "refs/tags/${TAG}" 2>/dev/null)" ]]; then
	echo "❌ remote 已有 tag ${TAG}，请 bump 或先删：git push origin :refs/tags/${TAG}" >&2
	exit 1
fi

say "== 1) git push（推送本地 commits） =="
git_q push

say "== 2) git tag ${TAG} && git push origin ${TAG}（触发 CI workflow） =="
git tag "${TAG}"
git_q push origin "${TAG}"

if [[ "${watch}" != "1" ]]; then
	echo "✓ 已触发 CI（tag ${TAG} 已推送，约 10-20 min 出 Release）"
	exit 0
fi

say "== 3) 等 CI run 注册并跟踪（Ctrl-C 安全退出，CI 在云端继续跑）=="
sleep 6
run_id=""
for _ in 1 2 3 4 5; do
	run_id="$(gh run list --workflow=desktop-bundle.yml --limit 10 \
		--json databaseId,headBranch \
		--jq ".[] | select(.headBranch == \"${TAG}\") | .databaseId" | head -n1)"
	[[ -n "${run_id}" ]] && break
	sleep 4
done
if [[ -z "${run_id}" ]]; then
	echo "⚠️  没抓到 ${TAG} 的 run id，去 Actions 页面看：gh run list --workflow=desktop-bundle.yml"
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
echo "✓ Release ${TAG} 已就绪"
