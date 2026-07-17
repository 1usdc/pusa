#!/usr/bin/env bash
# 清空本机 data/（`just server` 用）。请先停服务。
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT}"

mkdir -p data
shopt -s nullglob
rm -rf data/*
shopt -u nullglob
echo "已清空: data/*"
