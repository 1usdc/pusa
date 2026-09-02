# 把 vendor 里的 pusa_core.dll 拷到 desktop/bundle-resources/，供 dx bundle
# 打进 NSIS 安装器（安装后与 exe 同目录，运行时 libloading 从这里加载）。
#
# 在 dx bundle 之前调用。SKIP_CORE_CHECK=1 时跳过（安装器运行时无法加载核心）。
$ErrorActionPreference = 'Stop'
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8

$Root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$DestDir = Join-Path $Root 'desktop\bundle-resources'
$Dest = Join-Path $DestDir 'pusa_core.dll'

if ($env:SKIP_CORE_CHECK -eq '1') {
    Write-Host 'embed pusa-core: 跳过（SKIP_CORE_CHECK=1）；安装器运行时可能无法加载核心'
    exit 0
}

$hostTriple = (rustc -vV | Select-String '^host:').ToString() -replace '^host:\s*', ''
$Lib = Join-Path $Root "vendor\pusa-core\$hostTriple\pusa_core.dll"
if (-not (Test-Path $Lib)) {
    Write-Host @"
error: 缺少 $Lib
请先 just pusa-core，或把预编译 DLL 放到上述路径。
仅验证打包流程：SKIP_CORE_CHECK=1 just desktop-windows
"@
    exit 1
}

New-Item -ItemType Directory -Force -Path $DestDir | Out-Null
Copy-Item -Force $Lib $Dest
Write-Host "已暂存: $Dest（来自 $Lib）"
