# 本机桌面：dx serve 热重载（Windows）。联调：
#   $env:ANOTHERME_BASE_URL='http://127.0.0.1:8881'; just desktop
$ErrorActionPreference = 'Stop'
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8

$Root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
Set-Location $Root

if (-not $env:ANOTHERME_BASE_URL) {
    $env:ANOTHERME_BASE_URL = 'https://api.anotherme.co'
}

if (-not (Get-Command dx -ErrorAction SilentlyContinue)) {
    throw '未找到 dx，请先安装 dioxus-cli（版本与 crate 对齐：dx --version）'
}

# 默认 always-on-top=true 会挡住编辑器；开发时关掉
dx serve --desktop -p desktop --always-on-top false
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
