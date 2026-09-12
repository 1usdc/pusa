#!/usr/bin/env bash
# 同时推送开源仓 pusa 与私有仓 pusa-core/（固定在仓库根下），不打 tag / 不发版。
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT}"

push_one() {
	local dir="$1"
	local label="$2"
	echo "== push ${label} (${dir}) =="
	if [[ ! -d "${dir}/.git" ]]; then
		echo "❌ ${label}: 不是 git 仓库（期望路径：${dir}）" >&2
		return 1
	fi
	(
		cd "${dir}"
		local branch upstream ahead
		branch="$(git rev-parse --abbrev-ref HEAD)"
		if [[ "${branch}" == "HEAD" ]]; then
			echo "❌ ${label}: detached HEAD，请先切到分支再推" >&2
			exit 1
		fi
		upstream="$(git rev-parse --abbrev-ref --symbolic-full-name '@{u}' 2>/dev/null || true)"
		if [[ -n "${upstream}" ]]; then
			ahead="$(git rev-list --count "${upstream}..HEAD")"
			if [[ "${ahead}" -eq 0 ]]; then
				echo "✓ ${label}: 已与 ${upstream} 同步，跳过"
				exit 0
			fi
		fi
		git push -u origin HEAD
		echo "✓ ${label}: ${branch} → origin（无 tag / 无 release）"
	)
}

push_one "${ROOT}" "pusa"
push_one "${ROOT}/pusa-core" "pusa-core"
echo "✓ push 完成（pusa + pusa-core）"
