#!/usr/bin/env bash
# 同时推送开源仓 pusa 与私有仓 pusa-core/（固定在仓库根下），不打 tag / 不发版。
# 工作区有未提交改动时：git add -A → commit → push。
# 提交说明：COMMIT_MSG（默认 chore: sync local changes before push）。
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT}"

commit_message() {
	if [[ -n "${COMMIT_MSG:-}" ]]; then
		printf '%s' "${COMMIT_MSG}"
	else
		printf '%s' 'chore: sync local changes before push'
	fi
}

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
		local branch upstream ahead msg
		branch="$(git rev-parse --abbrev-ref HEAD)"
		if [[ "${branch}" == "HEAD" ]]; then
			echo "❌ ${label}: detached HEAD，请先切到分支再推" >&2
			exit 1
		fi

		if [[ -n "$(git status --porcelain)" ]]; then
			echo "→ ${label}: 检测到未提交改动，自动 add + commit"
			git add -A
			if [[ -n "$(git diff --cached --name-only)" ]]; then
				msg="$(commit_message)"
				git commit -m "${msg}"
				echo "✓ ${label}: committed — ${msg}"
			else
				echo "→ ${label}: add 后无已跟踪变更（可能全是 ignored），跳过 commit"
			fi
		fi

		upstream="$(git rev-parse --abbrev-ref --symbolic-full-name '@{u}' 2>/dev/null || true)"
		if [[ -n "${upstream}" ]]; then
			ahead="$(git rev-list --count "${upstream}..HEAD")"
			if [[ "${ahead}" -eq 0 ]]; then
				echo "✓ ${label}: 已与 ${upstream} 同步，跳过 push"
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
