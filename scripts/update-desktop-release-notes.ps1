# 更新 GitHub Release 说明：双平台一键下载按钮 + 文件表。
# 默认 tag 取自 desktop/Cargo.toml → desktop-vX.Y.Z（可用 $env:TAG 覆盖）。
#
# 用法：
#   just release-notes
#   $env:TAG='desktop-v1.1.2'; .\scripts\update-desktop-release-notes.ps1
$ErrorActionPreference = 'Stop'
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8

$Root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
Set-Location $Root

function Get-DesktopReleaseTag {
    if ($env:TAG) { return $env:TAG.Trim() }
    $cargo = Join-Path $Root 'desktop\Cargo.toml'
    if (-not (Test-Path $cargo)) { throw "未找到 $cargo" }
    $inPkg = $false
    foreach ($line in Get-Content $cargo) {
        $t = $line.Trim()
        if ($t -eq '[package]') { $inPkg = $true; continue }
        if ($t.StartsWith('[')) { $inPkg = $false; continue }
        if ($inPkg -and $t -match '^version\s*=\s*"([^"]+)"') {
            return "desktop-v$($Matches[1])"
        }
    }
    throw '无法从 desktop/Cargo.toml 解析 version'
}

if (-not (Get-Command gh -ErrorAction SilentlyContinue)) {
    throw '未找到 gh CLI，请先安装并 gh auth login'
}

$Tag = Get-DesktopReleaseTag
if ($Tag -notmatch '^desktop-v([0-9]+\.[0-9]+\.[0-9]+)$') {
    throw "TAG 格式不对：$Tag（应为 desktop-vX.Y.Z）"
}
$Version = $Matches[1]
$Repo = if ($env:VPK_REPO_URL) {
    ($env:VPK_REPO_URL -replace 'https://github.com/', '').TrimEnd('/')
} else {
    '1usdc/pusa'
}
$Base = "https://github.com/$Repo/releases/download/$Tag"

& gh release view $Tag 2>$null | Out-Null
if ($LASTEXITCODE -ne 0) {
    throw "GitHub Release $Tag 不存在"
}

$assetNames = @(& gh release view $Tag --json assets -q '.assets[].name' 2>$null)
$has = { param($n) $assetNames -contains $n }

$MacDmg = "Pusa-$Version-macos-arm64.dmg"
$WinSetup = 'Pusa-win-x64-Setup.exe'
# 兼容旧 NSIS 命名
$WinSetupAlt = "Pusa_${Version}_x64-setup.exe"

$winFile = if (& $has $WinSetup) { $WinSetup } elseif (& $has $WinSetupAlt) { $WinSetupAlt } else { $null }
$macFile = if (& $has $MacDmg) { $MacDmg } else { $null }

$lines = New-Object System.Collections.Generic.List[string]
$lines.Add("## Pusa $Version")
$lines.Add('')
$lines.Add('Pusa 是本地优先的 AI 编程与办公助手：把大模型、文件系统、终端与内置浏览器放进同一个桌面窗口，Agent 可以直接读写项目文件、执行命令并在应用内预览结果。')
$lines.Add('')
$lines.Add('### 一键下载')
$lines.Add('')

if ($macFile) {
    $lines.Add("[⬇️ 下载 macOS（Apple Silicon）]($Base/$macFile)")
    $lines.Add('')
}
if ($winFile) {
    $lines.Add("[⬇️ 下载 Windows（x64）]($Base/$winFile)")
    $lines.Add('')
}
if (-not $macFile -and -not $winFile) {
    $lines.Add('_暂无安装包资产，请查看下方 Assets 列表。_')
    $lines.Add('')
}

$lines.Add('### 文件说明')
$lines.Add('')
$lines.Add('| 文件 | 平台 | 说明 |')
$lines.Add('|---|---|---|')
if ($macFile) {
    $lines.Add("| ``$macFile`` | macOS | 推荐。打开后把 Pusa 拖进 Applications |")
}
if ($winFile) {
    $lines.Add("| ``$winFile`` | Windows | 推荐。双击安装（缺 WebView2 时安装器会引导） |")
}
$lines.Add('')
$lines.Add('应用内更新依赖同 Release 中的 `releases.*.json` 与 `*-full.nupkg`（无需手动下载）。')

$notesFile = Join-Path $env:TEMP "pusa-release-notes-$Tag.md"
# GitHub 要 LF；UTF-8 无 BOM
$utf8NoBom = New-Object System.Text.UTF8Encoding $false
[System.IO.File]::WriteAllText($notesFile, (($lines -join "`n") + "`n"), $utf8NoBom)

& gh release edit $Tag --notes-file $notesFile
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host "✓ Release $Tag 说明已更新（mac=$(if ($macFile) { $macFile } else { '无' })；win=$(if ($winFile) { $winFile } else { '无' })）"
Write-Host "  https://github.com/$Repo/releases/tag/$Tag"
