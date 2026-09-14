# 从 docker/.env 注入环境（compose / just server 共用）

set dotenv-path := "docker/.env"
set shell := ["bash", "-euo", "pipefail", "-c"]
# Windows：避免 shebang 配方依赖 cygpath；Bypass 绕过本机 ExecutionPolicy
set windows-shell := ["powershell.exe", "-NoProfile", "-ExecutionPolicy", "Bypass", "-Command"]

default:
    @just --list

# LAN=1 just web 局域网访问（依赖 bash；Windows 需 Git Bash 在 PATH）
[unix]
web:
    bash scripts/web.sh

# Agent API；Web 默认连 http://127.0.0.1:8787（依赖 bash）
[unix]
server:
    bash scripts/server.sh

# 清空 data/（请先停服务；依赖 bash）
[unix]
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
# Windows 不能直接跑 bash 脚本：用 [unix]/[windows] 分支。
[unix]
desktop: ensure-pusa-core
    bash scripts/desktop.sh

[windows]
desktop: ensure-pusa-core
    powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\desktop.ps1

# 本机 macOS 打包（patch 版号 +1 后 dx bundle → vpk 签名+公证+增量包 → desktop/dist/velopack/）
# BUMP=0 不改版号；NOTARIZE=0 跳过公证；VELOPACK=0 回退旧 DMG 流程（无应用内更新）
[unix]
desktop-mac: ensure-pusa-core
    bash scripts/desktop-mac.sh

# 本机 Windows 打包（dx bundle → vpk pack → desktop/dist/velopack/，默认不签名）
# SIGN=1 由 vpk 走 Azure Trusted Signing；VELOPACK=0 回退旧 NSIS *-setup.exe 流程（无应用内更新）
[windows]
desktop-windows: ensure-pusa-core
    powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\desktop-windows.ps1

# macOS 发版（tag 默认 desktop-v + desktop/Cargo.toml version）；BUILD=1 先打包
# 有 desktop/dist/velopack/ 产物时 vpk upload github（含 releases.<channel>.json），否则上传旧 *.dmg
[unix]
release-mac:
    bash scripts/release-mac.sh

# Windows 发版；BUILD=1 先打包；有 Velopack 产物时 vpk upload github，否则上传旧 *-setup.exe
[windows]
release-windows:
    powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\release-windows.ps1

# 合并上传 Windows 到已有 Release（不 push/不打 tag；BUILD=1 先打包；TAG=desktop-v1.1.2 可覆盖）
[windows]
release-windows-merge:
    $env:MERGE = '1'; powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\release-windows.ps1

# 按当前 Release 资产重写说明（双平台一键下载按钮）；TAG= 可覆盖
[windows]
release-notes:
    powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\update-desktop-release-notes.ps1

[unix]
release-notes:
    bash scripts/update-desktop-release-notes.sh

# 同时拉取 pusa + pusa-core/（私有仓固定在仓库根下），不打 tag / 不发版
[unix]
pull:
    bash scripts/pull.sh

[windows]
pull:
    powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\pull.ps1

# 同时推送 pusa + pusa-core/：有未提交改动则先 add+commit，再 push；不打 tag / 不发版
# COMMIT_MSG="feat: ..." just push   # 可选，覆盖默认提交说明
[unix]
push:
    bash scripts/push.sh

[windows]
push:
    powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\push.ps1
