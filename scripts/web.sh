#!/usr/bin/env bash
# 本地 Web：nginx 代理 /llm /relay /v1 → 上游，其余 → dx serve（same-origin，避 CORS）
# 浏览器 :${WEB_PORT:-8080}；dx :${DX_PORT:-8081}
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT}"

web_port="${WEB_PORT:-8080}"
dx_port="${DX_PORT:-8081}"
proxy_container="${WEB_PROXY_CONTAINER:-anotherclaw-web-dev-proxy}"
host_gateway_ipv4="${HOST_DOCKER_INTERNAL_IPV4:-192.168.65.254}"
anotherme_base="${ANOTHERME_BASE_URL:-https://api.anotherme.co}"
anotherme_base="${anotherme_base%/}"
anotherme_proxy_base="${anotherme_base//host.docker.internal/${host_gateway_ipv4}}"
anotherme_authority="${anotherme_base#http://}"
anotherme_authority="${anotherme_authority#https://}"
anotherme_authority="${anotherme_authority%%/*}"
anotherme_host="${anotherme_authority%%:*}"
relay_base="${ANOTHERME_RELAY_BASE_URL:-https://relay.anotherme.co}"
relay_base="${relay_base%/}"
relay_proxy_base="${relay_base//host.docker.internal/${host_gateway_ipv4}}"
relay_authority="${relay_base#http://}"
relay_authority="${relay_authority#https://}"
relay_authority="${relay_authority%%/*}"
relay_host="${relay_authority%%:*}"
server_base="${CLAW_SERVER_URL:-http://host.docker.internal:8787}"
server_base="${server_base%/}"
server_proxy_base="${server_base//host.docker.internal/${host_gateway_ipv4}}"
server_authority="${server_base#http://}"
server_authority="${server_authority#https://}"
server_authority="${server_authority%%/*}"
dx_proxy_base="http://${host_gateway_ipv4}:${dx_port}"
nginx_conf="${TMPDIR:-/tmp}/anotherclaw-nginx-dev-${web_port}.conf"
dx_log="${TMPDIR:-/tmp}/anotherclaw-dx-web-${dx_port}.log"

cleanup() {
	docker rm -f "${proxy_container}" >/dev/null 2>&1 || true
	if [[ -n "${dx_pid:-}" ]]; then
		kill "${dx_pid}" >/dev/null 2>&1 || true
	fi
}
trap cleanup EXIT INT TERM

{
	printf '%s\n' 'server {'
	printf '%s\n' '    listen 80;'
	printf '%s\n' '    server_name localhost;'
	printf '%s\n' ''
	printf '%s\n' '    location /v1/ {'
	printf '%s\n' "        proxy_pass ${server_proxy_base};"
	printf '%s\n' '        proxy_http_version 1.1;'
	printf '%s\n' "        proxy_set_header Host ${server_authority};"
	printf '%s\n' '        proxy_set_header X-Real-IP $remote_addr;'
	printf '%s\n' '        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;'
	printf '%s\n' '        proxy_set_header X-Forwarded-Proto $scheme;'
	printf '%s\n' '        proxy_buffering off;'
	printf '%s\n' '        proxy_cache off;'
	printf '%s\n' '        proxy_connect_timeout 5s;'
	printf '%s\n' '        proxy_send_timeout 10s;'
	printf '%s\n' '        proxy_read_timeout 86400s;'
	printf '%s\n' '    }'
	printf '%s\n' ''
	printf '%s\n' '    location /llm/ {'
	printf '%s\n' "        proxy_pass ${anotherme_proxy_base}/;"
	printf '%s\n' '        proxy_http_version 1.1;'
	printf '%s\n' "        proxy_set_header Host ${anotherme_authority};"
	printf '%s\n' '        proxy_ssl_server_name on;'
	printf '%s\n' "        proxy_ssl_name ${anotherme_host};"
	printf '%s\n' '        proxy_set_header X-Real-IP $remote_addr;'
	printf '%s\n' '        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;'
	printf '%s\n' '        proxy_set_header X-Forwarded-Proto $scheme;'
	printf '%s\n' '        proxy_buffering off;'
	printf '%s\n' '        proxy_cache off;'
	printf '%s\n' '        proxy_connect_timeout 5s;'
	printf '%s\n' '        proxy_send_timeout 5s;'
	printf '%s\n' '        proxy_read_timeout 5s;'
	printf '%s\n' '    }'
	printf '%s\n' ''
	printf '%s\n' '    location /relay/ {'
	printf '%s\n' "        proxy_pass ${relay_proxy_base}/;"
	printf '%s\n' '        proxy_http_version 1.1;'
	printf '%s\n' "        proxy_set_header Host ${relay_authority};"
	printf '%s\n' '        proxy_ssl_server_name on;'
	printf '%s\n' "        proxy_ssl_name ${relay_host};"
	printf '%s\n' '        proxy_set_header X-Real-IP $remote_addr;'
	printf '%s\n' '        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;'
	printf '%s\n' '        proxy_set_header X-Forwarded-Proto $scheme;'
	printf '%s\n' '        proxy_buffering off;'
	printf '%s\n' '        proxy_cache off;'
	printf '%s\n' '        proxy_connect_timeout 5s;'
	printf '%s\n' '        proxy_send_timeout 10s;'
	printf '%s\n' '        proxy_read_timeout 86400s;'
	printf '%s\n' '    }'
	printf '%s\n' ''
	printf '%s\n' '    location / {'
	printf '%s\n' "        proxy_pass ${dx_proxy_base};"
	printf '%s\n' '        proxy_http_version 1.1;'
	printf '%s\n' '        proxy_set_header Host $host;'
	printf '%s\n' '        proxy_set_header X-Real-IP $remote_addr;'
	printf '%s\n' '        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;'
	printf '%s\n' '        proxy_set_header X-Forwarded-Proto $scheme;'
	printf '%s\n' '        proxy_set_header Upgrade $http_upgrade;'
	printf '%s\n' '        proxy_set_header Connection "upgrade";'
	printf '%s\n' '        proxy_buffering off;'
	printf '%s\n' '        proxy_cache off;'
	# dx hot-reload WebSocket 需长超时，否则空闲 ~60s 被掐断导致整页刷新
	printf '%s\n' '        proxy_read_timeout 86400s;'
	printf '%s\n' '        proxy_send_timeout 86400s;'
	printf '%s\n' '    }'
	printf '%s\n' '}'
} >"${nginx_conf}"

docker rm -f "${proxy_container}" >/dev/null 2>&1 || true
rm -f "${dx_log}"

# 端口被占时 fail-fast（常见：上次僵尸 dx serve）
if lsof -iTCP:"${dx_port}" -sTCP:LISTEN >/dev/null 2>&1; then
	echo "错误: 端口 ${dx_port} 已被占用（多半是上次没退干净的 dx serve），请先释放：" >&2
	lsof -iTCP:"${dx_port}" -sTCP:LISTEN >&2
	echo "  快速清理: lsof -tiTCP:${dx_port} -sTCP:LISTEN | xargs -r kill" >&2
	exit 1
fi

echo "== dx serve: http://127.0.0.1:${dx_port}（same-origin-api） =="
(
	cd web
	env -u ANOTHERME_BASE_URL -u ANOTHERME_RELAY_BASE_URL \
		dx serve --port "${dx_port}" --addr "0.0.0.0" --features same-origin-api
) >"${dx_log}" 2>&1 &
dx_pid="$!"

echo "== waiting for dx serve ready on 127.0.0.1:${dx_port} =="
for i in {1..90}; do
	if ! kill -0 "${dx_pid}" >/dev/null 2>&1; then
		echo "错误: dx serve 已退出，日志如下:"
		sed -n '1,160p' "${dx_log}" || true
		exit 1
	fi
	if curl -fsS "http://127.0.0.1:${dx_port}/" >/dev/null 2>&1; then
		echo "== dx serve ready: http://127.0.0.1:${dx_port} =="
		break
	fi
	if (( i % 10 == 0 )); then
		echo "  ... waiting (${i}s), latest dx log:"
		tail -n 20 "${dx_log}" || true
	fi
	sleep 1
done

if ! curl -fsS "http://127.0.0.1:${dx_port}/" >/dev/null 2>&1; then
	echo "错误: dx serve 90s 内未就绪，日志如下:"
	sed -n '1,220p' "${dx_log}" || true
	exit 1
fi

echo "== local nginx proxy: http://127.0.0.1:${web_port}  /v1 → ${server_base}  /llm → ${anotherme_base}  /relay → ${relay_base} =="
docker run --rm --name "${proxy_container}" \
	-p "${web_port}:80" \
	-v "${nginx_conf}:/etc/nginx/conf.d/default.conf:ro" \
	nginx:1.27-alpine
