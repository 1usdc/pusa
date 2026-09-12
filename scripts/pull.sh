#!/usr/bin/env bash
# 同时拉取开源仓 pusa 与私有仓 pusa-core/（固定在仓库根下），不打 tag / 不发版。
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT}"

pull_one() {
	local dir="$1"
	local label="$2"
	echo "== pull ${label} (${dir}) =="
	if [[ ! -d "${dir}/.git" ]]; then
		echo "❌ ${label}: 不是 git 仓库（期望路径：${dir}）" >&2
		return 1
	fi
	(
		cd "${dir}"
		local branch
		branch="$(git rev-parse --abbrev-ref HEAD)"
		if [[ "${branch}" == "HEAD" ]]; then
			echo "❌ ${label}: detached HEAD，请先切到分支再拉" >&2
			exit 1
		fi
		git pull --ff-only
		echo "✓ ${label}: $(git rev-parse --short HEAD) @ ${branch}"
	)
}

pull_one "${ROOT}" "pusa"
pull_one "${ROOT}/pusa-core" "pusa-core"
echo "✓ pull 完成（pusa + pusa-core）"
