#!/usr/bin/env bash
# 只推当前分支到 origin，不打 tag / 不发版。
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT}"

branch="$(git rev-parse --abbrev-ref HEAD)"
if [[ "${branch}" == "HEAD" ]]; then
	echo "❌ 当前处于 detached HEAD，请先切到分支再推" >&2
	exit 1
fi

upstream="$(git rev-parse --abbrev-ref --symbolic-full-name '@{u}' 2>/dev/null || true)"
if [[ -n "${upstream}" ]]; then
	ahead="$(git rev-list --count "${upstream}..HEAD")"
	if [[ "${ahead}" -eq 0 ]]; then
		echo "✓ 已与 ${upstream} 同步，无需推送"
		exit 0
	fi
fi

echo "== git push -u origin ${branch} =="
git push -u origin "HEAD"
echo "✓ 已推送 ${branch} → origin（无 tag / 无 release）"
