# 从 docker/.env 注入环境（compose / just server 共用）

set dotenv-path := "docker/.env"
set shell := ["bash", "-euo", "pipefail", "-c"]

default:
    @just --list

# LAN=1 just web 局域网访问
web:
    bash scripts/web.sh

# Agent API；Web 默认连 http://127.0.0.1:8787
server:
    bash scripts/server.sh

# 清空 data/（请先停服务）
db-clear:
    bash scripts/db-clear.sh

# 打包闭源核心 → vendor/pusa-core/<host-triple>/libpusa_core.*
pusa-core:
    #!/usr/bin/env bash
    if [[ ! -d pusa-core ]]; then
      echo "error: 缺少 pusa-core/ 源码目录，无法打包" >&2
      exit 1
    fi
    just --justfile pusa-core/justfile --working-directory pusa-core pack

# 无 vendor 产物，或 pusa-core/protocol 源码比产物新 → 自动 just pusa-core
# 注意：产物不存在时 find -newer / -nt 不会当成「过期」，必须显式判断 ! -f
ensure-pusa-core:
    #!/usr/bin/env bash
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
      stale="$(
        {
          find pusa-core/src protocol/src -type f \( -name '*.rs' -o -name '*.inc.rs' \) -newer "$DEST" 2>/dev/null || true
          for f in pusa-core/Cargo.toml pusa-core/Cargo.lock protocol/Cargo.toml protocol/Cargo.lock; do
            if [[ -f "$f" && "$f" -nt "$DEST" ]]; then
              printf '%s\n' "$f"
            fi
          done
        } | head -1
      )"
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

# 本机桌面（dx serve 热重载）；联调：ANOTHERME_BASE_URL=http://127.0.0.1:8881 just desktop
desktop: ensure-pusa-core
    bash scripts/desktop.sh

# 本机 macOS 打包发版（dx bundle + 签名公证 → desktop/dist/*.dmg）；NOTARIZE=0 跳过公证
desktop-mac: ensure-pusa-core
    bash scripts/desktop-mac.sh

# 本机 Windows 打包（默认不签名）；SIGN=1 可选 Azure Artifact Signing
desktop-windows: ensure-pusa-core
    pwsh -NoProfile -File scripts/desktop-windows.ps1

# macOS 发版：TAG=desktop-vX.Y.Z BUILD=1 just release-mac
release-mac:
    bash scripts/release-mac.sh

# Windows 发版：TAG=desktop-vX.Y.Z BUILD=1 just release-windows
release-windows:
    bash scripts/release-windows.sh

# 只推仓库到 origin，不打 tag / 不发版
push:
    bash scripts/push.sh
