# 从 docker/.env 注入环境（compose / just server 共用）

set dotenv-path := "docker/.env"
set shell := ["bash", "-euo", "pipefail", "-c"]
# Windows：避免 shebang 配方依赖 cygpath；Bypass 绕过本机 ExecutionPolicy
set windows-shell := ["powershell.exe", "-NoProfile", "-ExecutionPolicy", "Bypass", "-Command"]

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
[unix]
pusa-core:
    if [[ ! -d pusa-core ]]; then echo "error: 缺少 pusa-core/ 源码目录，无法打包" >&2; exit 1; fi
    just --justfile pusa-core/justfile --working-directory pusa-core pack

[windows]
pusa-core:
    powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\pusa-core-pack.ps1

# 无 vendor 产物，或源码比产物新 → 自动 just pusa-core
[unix]
ensure-pusa-core:
    bash scripts/ensure-pusa-core.sh

[windows]
ensure-pusa-core:
    powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\ensure-pusa-core.ps1

# 本机桌面（dx serve 热重载）；联调：ANOTHERME_BASE_URL=http://127.0.0.1:8881 just desktop
desktop: ensure-pusa-core
    bash scripts/desktop.sh

# 本机 macOS 打包发版（patch 版号 +1 后 dx bundle + 签名公证）；BUMP=0 不改版号；NOTARIZE=0 跳过公证
desktop-mac: ensure-pusa-core
    bash scripts/desktop-mac.sh

# 把本机 desktop/dist/*.dmg 挂到 GitHub Release；TAG 可省略（默认 desktop-v + DMG 版本）
desktop-mac-release:
    bash scripts/desktop-mac-release.sh

# 本机 Windows 打包（默认不签名）；SIGN=1 可选 Azure Artifact Signing
desktop-windows: ensure-pusa-core
    powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\desktop-windows.ps1

# macOS 发版（tag 默认 desktop-v + desktop/Cargo.toml version）；BUILD=1 先打包
release-mac:
    bash scripts/release-mac.sh

# Windows 发版（同上）；BUILD=1 先打包
release-windows:
    bash scripts/release-windows.sh

# 只推仓库到 origin，不打 tag / 不发版
push:
    bash scripts/push.sh
