# 本机 Windows 打包：dx bundle --release → desktop/dist/*-setup.exe（默认不签名）
#
# 用法：
#   just desktop-windows
#   SIGN=1 just desktop-windows    # 打包后走 Azure Artifact Signing（见 scripts/windows-sign.ps1）
#   ANOTHERME_BASE_URL=... just desktop-windows
$ErrorActionPreference = 'Stop'

$Root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$Desktop = Join-Path $Root 'desktop'
$Dist = Join-Path $Desktop 'dist'
$DoSign = $env:SIGN -eq '1'

if (-not (Get-Command dx -ErrorAction SilentlyContinue)) {
    throw '未找到 dx，请先安装 dioxus-cli（版本与 CI 对齐：dx --version）'
}

if (-not $env:ANOTHERME_BASE_URL) {
    $env:ANOTHERME_BASE_URL = 'https://api.anotherme.co'
}

Write-Host '== 1) 清空 desktop/dist =='
if (Test-Path $Dist) { Remove-Item -Recurse -Force $Dist }
New-Item -ItemType Directory -Force -Path $Dist | Out-Null

Write-Host '== 2) dx bundle --release（desktop/）=='
Push-Location $Desktop
try {
    dx bundle --release
} finally {
    Pop-Location
}

Write-Host '== 3) 收集 NSIS 安装器到 desktop/dist =='
$inDist = @(Get-ChildItem -Path $Dist -Filter '*-setup.exe' -ErrorAction SilentlyContinue)
if ($inDist.Count -eq 0) {
    $targetDx = Join-Path $Root 'target\dx'
    if (Test-Path $targetDx) {
        $inTarget = @(Get-ChildItem -Path $targetDx -Recurse -Filter '*-setup.exe' -ErrorAction SilentlyContinue)
        foreach ($f in ($inTarget | Sort-Object FullName -Unique)) {
            Copy-Item -Force $f.FullName -Destination $Dist
            Write-Host "staged: $($f.FullName) -> desktop\dist\$($f.Name)"
        }
    }
}

$installers = @(Get-ChildItem -Path $Dist -Filter '*-setup.exe' -ErrorAction SilentlyContinue)
if ($installers.Count -eq 0) {
    throw 'dx bundle 未产出 *-setup.exe'
}

# 删掉文件名带空格的裸产物，避免误分发
Get-ChildItem -Path $Dist, (Join-Path $Root 'target\dx') -Recurse -ErrorAction SilentlyContinue |
    Where-Object { $_.Name -match '\.(exe|msi)$' -and $_.Name -match ' ' } |
    ForEach-Object { Write-Host "remove raw artifact: $($_.FullName)"; Remove-Item -Force $_.FullName }

if ($DoSign) {
    Write-Host '== 4) Azure Artifact Signing（SIGN=1）=='
    powershell.exe -NoProfile -ExecutionPolicy Bypass -File (Join-Path $Root 'scripts\windows-sign.ps1')
} else {
    Write-Host '== 4) 未签名（需要时 SIGN=1 just desktop-windows）=='
}

Write-Host "✓ 本机 Windows 包已就绪：$Dist"
Get-ChildItem $Dist -Filter '*-setup.exe' | Format-Table Name, Length -AutoSize
