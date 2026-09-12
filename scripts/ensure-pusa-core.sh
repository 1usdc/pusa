#!/usr/bin/env bash
# 无 vendor 产物，或 pusa-core/protocol 源码比产物新 → 自动 just pusa-core
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT}"

if [[ "${SKIP_CORE_CHECK:-0}" == "1" ]]; then
	echo "pusa-core: 跳过检查（SKIP_CORE_CHECK=1）；产物运行时可能无法加载核心"
	exit 0
fi

HOST="$(rustc -vV | awk '/^host:/{print $2}')"
case "$HOST" in
	*windows*) LIB="pusa_core.dll" ;;
	*apple*) LIB="libpusa_core.dylib" ;;
	*) LIB="libpusa_core.so" ;;
esac
DEST="vendor/pusa-core/${HOST}/${LIB}"

if [[ ! -d pusa-core ]]; then
	if [[ -f "$DEST" ]]; then
		echo "pusa-core: 无源码目录，沿用已有产物 ($DEST)"
		exit 0
	fi
	echo "error: 缺少 pusa-core/ 源码，且无 vendor 产物 ($DEST)" >&2
	exit 1
fi

stale=""
if [[ -f "$DEST" ]]; then
	# 不用 find | head：pipefail 下 head 关闭管道会让 find 收到 SIGPIPE（exit 141）
	stale="$(find pusa-core/src protocol/src -type f \( -name '*.rs' -o -name '*.inc.rs' \) -newer "$DEST" -print -quit 2>/dev/null || true)"
	if [[ -z "$stale" ]]; then
		for f in pusa-core/Cargo.toml pusa-core/Cargo.lock protocol/Cargo.toml protocol/Cargo.lock; do
			if [[ -f "$f" && "$f" -nt "$DEST" ]]; then
				stale="$f"
				break
			fi
		done
	fi
fi

if [[ -f "$DEST" && -z "$stale" ]]; then
	echo "pusa-core: 已是最新 ($DEST)"
	exit 0
fi

if [[ ! -f "$DEST" ]]; then
	echo "pusa-core: 未找到产物，开始打包… ($DEST)"
else
	echo "pusa-core: 源码新于产物，开始打包… (newer: $stale)"
fi
just pusa-core
