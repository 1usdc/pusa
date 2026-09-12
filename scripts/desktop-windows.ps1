# 本机 Windows 打包：嵌入 pusa_core.dll → dx bundle --release → Velopack（vpk pack）
#   → desktop/dist/velopack/（Setup.exe / Portable.zip / full+delta .nupkg / releases.win-x64.json）
#
# 安装后 DLL 与 exe 同目录（shared/src/ffi.rs 从 current_exe 父目录加载）。
#
# 用法：
#   just desktop-windows
#   SIGN=1 just desktop-windows      # vpk 用 Azure Trusted Signing 签 exe / 安装器（凭证 .env.signing）
#   VELOPACK=0 just desktop-windows  # 旧流程：只出 NSIS *-setup.exe（无应用内更新）
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

Write-Host '== 2) 嵌入 pusa_core.dll（供 NSIS 装到程序目录）=='
powershell.exe -NoProfile -ExecutionPolicy Bypass -File (Join-Path $Root 'scripts\embed-pusa-core-windows.ps1')
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host '== 3) dx bundle --release（desktop/）=='
Push-Location $Desktop
try {
    dx bundle --release --windows-subsystem windows
} finally {
    Pop-Location
}

Write-Host '== 4) 收集 NSIS 安装器到 desktop/dist =='
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

if ($env:SKIP_CORE_CHECK -ne '1') {
    $stagedDll = @(Get-ChildItem -Path (Join-Path $Root 'target\dx') -Recurse -Filter 'pusa_core.dll' -ErrorAction SilentlyContinue |
        Where-Object { $_.DirectoryName -match '\\nsis\\_staging$' })
    if ($stagedDll.Count -eq 0) {
        throw 'NSIS _staging 里没有 pusa_core.dll：安装器不会带上核心库。检查 Dioxus.toml [bundle].resources 与 embed-pusa-core-windows.ps1'
    }
    Write-Host "已打进安装器: $($stagedDll[0].FullName)"
}

if ($env:VELOPACK -ne '0') {
    # Velopack：Setup.exe / Portable.zip / full+delta 包 / releases.win-x64.json → desktop\dist\velopack
    # 签名交给 vpk（SIGN=1 时 --azureTrustedSignFile），不再单独给 NSIS 安装器签名。
    Write-Host '== 5) Velopack 打包（vpk pack，含增量包）=='
    powershell.exe -NoProfile -ExecutionPolicy Bypass -File (Join-Path $Root 'scripts\velopack-pack-windows.ps1')
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
    Write-Host "✓ 本机 Windows Velopack 包已就绪：$(Join-Path $Dist 'velopack')"
    exit 0
}

if ($DoSign) {
    Write-Host '== 5) Azure Artifact Signing（SIGN=1）=='
    powershell.exe -NoProfile -ExecutionPolicy Bypass -File (Join-Path $Root 'scripts\windows-sign.ps1')
} else {
    Write-Host '== 5) 未签名（需要时 SIGN=1 just desktop-windows）=='
}

Write-Host "✓ 本机 Windows 包已就绪：$Dist"
Get-ChildItem $Dist -Filter '*-setup.exe' | Format-Table Name, Length -AutoSize
