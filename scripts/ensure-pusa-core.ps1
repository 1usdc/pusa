# 无 vendor 产物，或 pusa-core/protocol 源码比产物新 → 自动 just pusa-core
$ErrorActionPreference = 'Stop'
# Windows PowerShell 5.1 默认用 GBK 输出，中文提示会乱码
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8

$Root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
Set-Location $Root

if ($env:SKIP_CORE_CHECK -eq '1') {
    Write-Host 'pusa-core: 跳过检查（SKIP_CORE_CHECK=1）；产物运行时可能无法加载核心'
    exit 0
}

$hostTriple = (rustc -vV | Select-String '^host:').ToString() -replace '^host:\s*', ''
if ($hostTriple -match 'windows') {
    $lib = 'pusa_core.dll'
} elseif ($hostTriple -match 'apple') {
    $lib = 'libpusa_core.dylib'
} else {
    $lib = 'libpusa_core.so'
}
$dest = Join-Path $Root "vendor\pusa-core\$hostTriple\$lib"
$pusaCoreDir = Join-Path $Root 'pusa-core'

if (-not (Test-Path $pusaCoreDir)) {
    if (Test-Path $dest) {
        Write-Host "pusa-core: 无源码目录，沿用已有产物 ($dest)"
        exit 0
    }
    Write-Host @"
error: 缺少 pusa-core/ 源码，且无当前平台的 vendor 产物：
  $dest
解决其一：
  1) 把闭源 pusa-core/ 源码放到仓库根，再跑 just pusa-core
  2) 取到预编译库放到上述路径
  3) 仅验证打包流程（产物运行时无法加载核心）：SKIP_CORE_CHECK=1 just desktop-windows
"@
    exit 1
}

function Test-NewerThanDest {
    param([string]$Path)
    if (-not (Test-Path $Path)) { return $false }
    return (Get-Item $Path).LastWriteTime -gt (Get-Item $dest).LastWriteTime
}

$stale = $null
if (Test-Path $dest) {
    $candidates = @()
    foreach ($dir in @('pusa-core\src', 'protocol\src')) {
        $full = Join-Path $Root $dir
        if (Test-Path $full) {
            $candidates += Get-ChildItem -Path $full -Recurse -File -Include *.rs, *.inc.rs -ErrorAction SilentlyContinue
        }
    }
    foreach ($f in @(
            'pusa-core\Cargo.toml', 'pusa-core\Cargo.lock',
            'protocol\Cargo.toml', 'protocol\Cargo.lock'
        )) {
        $candidates += Get-Item (Join-Path $Root $f) -ErrorAction SilentlyContinue
    }
    $stale = $candidates | Where-Object { $_ -and $_.LastWriteTime -gt (Get-Item $dest).LastWriteTime } | Select-Object -First 1
}

if ((Test-Path $dest) -and -not $stale) {
    Write-Host "pusa-core: 已是最新 ($dest)"
    exit 0
}

if (-not (Test-Path $dest)) {
    Write-Host "pusa-core: 未找到产物，开始打包… ($dest)"
} else {
    Write-Host "pusa-core: 源码新于产物，开始打包… (newer: $($stale.FullName))"
}

just pusa-core
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
