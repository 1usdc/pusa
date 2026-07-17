# 从 docker/.env 注入环境（compose / just server 共用）

set dotenv-path := "docker/.env"

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

# 本机桌面（dx serve 热重载）；联调：ANOTHERME_BASE_URL=http://127.0.0.1:8881 just desktop
desktop:
    bash scripts/desktop.sh

# TAG=desktop-vX.Y.Z just desktop-ship；WATCH=1 等 CI；VERBOSE=1 详细日志
desktop-ship:
    bash scripts/desktop-ship.sh

# 只推仓库到 origin，不打 tag / 不发版
push:
    bash scripts/push.sh

android:
    cd web && dx serve --port "8082" --addr "0.0.0.0" --platform android

ios:
    cd web && dx serve --port "8083" --addr "0.0.0.0" --platform ios
