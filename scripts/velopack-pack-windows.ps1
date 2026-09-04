# Windows：把 dx bundle 的程序目录（desktop.exe + pusa_core.dll + assets）打成 Velopack 发布包。
#
# 输出 desktop\dist\velopack\：
#   Pusa-win-x64-Setup.exe        安装器（内含 WebView2 引导，--framework webview2）
#   Pusa-win-x64-Portable.zip     免安装版
#   Pusa-<ver>-win-x64-full.nupkg / -delta.nupkg
#   releases.win-x64.json         客户端更新清单
#
# 用法：
#   .\scripts\velopack-pack-windows.ps1              # 不签名
#   $env:SIGN='1'; .\scripts\velopack-pack-windows.ps1
#       → Azure Trusted Signing（vpk --azureTrustedSignFile），凭证见 .env.signing：
#         AZURE_TS_ENDPOINT / AZURE_TS_ACCOUNT / AZURE_TS_PROFILE，并需 az login 或 AZURE_CLIENT_ID 等
#   $env:RELEASE_NOTES_FILE='CHANGELOG.md'           # 可选发布说明
$ErrorActionPreference = 'Stop'

$Root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$Desktop = Join-Path $Root 'desktop'
$Dist = Join-Path $Desktop 'dist'
$Out = if ($env:VPK_OUT_DIR) { $env:VPK_OUT_DIR } else { Join-Path $Dist 'velopack' }
$EnvSigning = Join-Path $Root '.env.signing'

# 与 scripts/velopack-common.sh、ui/src/desktop/updater.rs 保持一致
$RepoUrl = if ($env:VPK_REPO_URL) { $env:VPK_REPO_URL } else { 'https://github.com/1usdc/pusa' }
$PackId = if ($env:VPK_PACK_ID) { $env:VPK_PACK_ID } else { 'Pusa' }
$Channel = if ($env:VPK_CHANNEL) { $env:VPK_CHANNEL } else { 'win-x64' }
$MainExe = 'desktop.exe'

function Load-EnvSigning {
    if (-not (Test-Path $EnvSigning)) { return }
    Get-Content $EnvSigning | ForEach-Object {
        $line = $_.Trim()
        if ($line -eq '' -or $line.StartsWith('#')) { return }
        if ($line.StartsWith('export ')) { $line = $line.Substring(7).Trim() }
        $eq = $line.IndexOf('=')
        if ($eq -lt 1) { return }
        $name = $line.Substring(0, $eq).Trim()
        $value = $line.Substring($eq + 1).Trim()
        if ($value.StartsWith('"') -and $value.EndsWith('"')) { $value = $value.Substring(1, $value.Length - 2) }
        if (-not [Environment]::GetEnvironmentVariable($name, 'Process')) {
            [Environment]::SetEnvironmentVariable($name, $value, 'Process')
        }
    }
}

function Get-DesktopVersion {
    $toml = Get-Content (Join-Path $Desktop 'Cargo.toml')
    $inPkg = $false
    foreach ($l in $toml) {
        $t = $l.Trim()
        if ($t -eq '[package]') { $inPkg = $true; continue }
        if ($t.StartsWith('[')) { $inPkg = $false; continue }
        if ($inPkg -and $t -match '^version\s*=\s*"([^"]+)"') { return $Matches[1] }
    }
    throw 'desktop/Cargo.toml 里没有 [package].version'
}

# dx bundle 装进安装器的完整程序目录：优先 NSIS _staging（含 exe + resources），其次 windows\app
function Find-PackDir {
    if ($env:PACK_DIR -and (Test-Path (Join-Path $env:PACK_DIR $MainExe))) { return $env:PACK_DIR }
    $targetDx = Join-Path $Root 'target\dx'
    $candidates = @()
    if (Test-Path $targetDx) {
        $candidates += @(Get-ChildItem -Path $targetDx -Recurse -Filter $MainExe -ErrorAction SilentlyContinue |
            Where-Object { $_.DirectoryName -match '\\nsis\\_staging$' -or $_.DirectoryName -match '\\windows\\app$' } |
            Sort-Object LastWriteTime -Descending |
            Select-Object -ExpandProperty DirectoryName)
    }
    foreach ($dir in $candidates) {
        if (Test-Path (Join-Path $dir 'pusa_core.dll')) { return $dir }
    }
    if ($candidates.Count -gt 0) {
        throw "找到 $MainExe 但同目录缺 pusa_core.dll：$($candidates[0])。检查 embed-pusa-core-windows.ps1 / Dioxus.toml [bundle].resources"
    }
    throw "未找到 dx bundle 的程序目录（target\dx\**\nsis\_staging 或 windows\app），请先 dx bundle --release"
}

function Get-GithubToken {
    if ($env:GITHUB_TOKEN) { return $env:GITHUB_TOKEN }
    if (Get-Command gh -ErrorAction SilentlyContinue) {
        try { $t = (gh auth token 2>$null); if ($LASTEXITCODE -eq 0 -and $t) { return $t.Trim() } } catch {}
    }
    return $null
}

# Azure Trusted Signing 的 metadata.json（vpk --azureTrustedSignFile）
function New-AzureSignMetadata {
    foreach ($n in 'AZURE_TS_ENDPOINT', 'AZURE_TS_ACCOUNT', 'AZURE_TS_PROFILE') {
        if (-not [Environment]::GetEnvironmentVariable($n, 'Process')) { throw "SIGN=1 需要 .env.signing 里的 $n" }
    }
    $path = Join-Path $Out 'azure-sign-metadata.json'
    @{
        Endpoint               = $env:AZURE_TS_ENDPOINT
        CodeSigningAccountName = $env:AZURE_TS_ACCOUNT
        CertificateProfileName = $env:AZURE_TS_PROFILE
    } | ConvertTo-Json | Set-Content -Path $path -Encoding UTF8
    return $path
}

if (-not (Get-Command vpk -ErrorAction SilentlyContinue)) {
    throw @'
未找到 vpk（Velopack CLI）。安装：
  winget install Microsoft.DotNet.SDK.8
  dotnet tool install -g vpk
'@
}

Load-EnvSigning
$Version = Get-DesktopVersion
$PackDir = Find-PackDir
New-Item -ItemType Directory -Force -Path $Out | Out-Null

Write-Host "== Velopack pack（Windows）=="
Write-Host "   packDir: $PackDir"
Write-Host "   version: $Version"
Write-Host "   channel: $Channel"

# 清掉本机同版本残留，保留 vpk download 拉的旧版（生成 delta 用）
Get-ChildItem -Path $Out -Filter "$PackId-$Version-*" -File -ErrorAction SilentlyContinue | Remove-Item -Force

Write-Host "== vpk download github（上一版 $Channel，用于生成增量包）=="
$token = Get-GithubToken
$dlArgs = @('download', 'github', '--repoUrl', $RepoUrl, '--channel', $Channel, '--outputDir', $Out)
if ($token) { $dlArgs += @('--token', $token) }
& vpk @dlArgs
if ($LASTEXITCODE -ne 0) {
    Write-Warning '未能下载上一版（首次发版或网络问题）；本次只生成全量包，无 delta'
}

$args = @(
    'pack',
    '--packId', $PackId,
    '--packVersion', $Version,
    '--packDir', $PackDir,
    '--mainExe', $MainExe,
    '--packTitle', 'Pusa',
    '--packAuthors', 'Pusa',
    '--icon', (Join-Path $Desktop 'assets\Pusa.ico'),
    '--channel', $Channel,
    '--outputDir', $Out,
    # 系统 WebView 依赖：安装时缺 WebView2 运行时则由 Setup 引导安装
    '--framework', 'webview2',
    '--shortcuts', 'Desktop,StartMenuRoot'
)

if ($env:SIGN -eq '1') {
    $meta = New-AzureSignMetadata
    Write-Host "   sign: Azure Trusted Signing ($env:AZURE_TS_ACCOUNT / $env:AZURE_TS_PROFILE)"
    $args += @('--azureTrustedSignFile', $meta)
} else {
    Write-Host '   未签名（需要时 $env:SIGN=1）'
}

if ($env:RELEASE_NOTES_FILE -and (Test-Path $env:RELEASE_NOTES_FILE)) {
    $args += @('--releaseNotes', $env:RELEASE_NOTES_FILE)
}

& vpk @args --yes
if ($LASTEXITCODE -ne 0) { throw "vpk pack 失败（exit $LASTEXITCODE）" }

Write-Host "✓ Velopack 产物：$Out"
Get-ChildItem $Out | Format-Table Name, Length -AutoSize
