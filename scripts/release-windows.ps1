# Windows 发版：本机打包（可选）→ 打 tag → 上传 GitHub Release。
# 版本默认取自 desktop/Cargo.toml → desktop-vX.Y.Z（可用 $env:TAG 覆盖）。
# 有 desktop/dist/velopack/releases.win-x64.json 时走 `vpk upload github`，
# 否则退回上传 *-setup.exe / *.msi。
#
# 用法：
#   just release-windows
#   $env:BUILD='1'; just release-windows
#   $env:TAG='desktop-v0.2.8'; just release-windows
#   $env:VPK_CHANNEL='win-x64'; just release-windows
#   MERGE=1：只合并上传到已有 Release（不 push / 不打 tag）；just release-windows-merge
$ErrorActionPreference = 'Stop'
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8

$Root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
Set-Location $Root

$VerboseLog = $env:VERBOSE -eq '1'
$MergeOnly = $env:MERGE -eq '1'
function Say([string]$Message) {
    if ($VerboseLog) { Write-Host $Message }
}

function Get-TagCommitSha([string]$TagName) {
    $local = & git rev-parse --verify --quiet "refs/tags/$TagName" 2>$null
    if ($LASTEXITCODE -eq 0 -and $local) {
        $peeled = (& git rev-parse "${TagName}^{commit}" 2>$null)
        if ($LASTEXITCODE -eq 0 -and $peeled) { return $peeled.Trim() }
    }
    $peeledLine = @(& git ls-remote --tags origin "refs/tags/${TagName}^{}" 2>$null) | Select-Object -First 1
    if ($peeledLine) { return ($peeledLine -split '\s+')[0] }
    $direct = @(& git ls-remote --tags origin "refs/tags/$TagName" 2>$null) |
        Where-Object { $_ -notmatch '\^\{\}$' } |
        Select-Object -First 1
    if ($direct) { return ($direct -split '\s+')[0] }
    return $null
}

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

function Get-VpkChannel {
    if ($env:VPK_CHANNEL) { return $env:VPK_CHANNEL.Trim() }
    return 'win-x64'
}

function Get-GithubToken {
    if ($env:GITHUB_TOKEN) { return $env:GITHUB_TOKEN.Trim() }
    if (Get-Command gh -ErrorAction SilentlyContinue) {
        $token = (& gh auth token 2>$null)
        if ($LASTEXITCODE -eq 0 -and $token) { return $token.Trim() }
    }
    return $null
}

function Require-Vpk {
    if (Get-Command vpk -ErrorAction SilentlyContinue) { return }
    throw @"
未找到 vpk（Velopack CLI）。安装：
  1) .NET 8 SDK：winget install Microsoft.DotNet.SDK.8
  2) dotnet tool install -g vpk
  3) 确认 %USERPROFILE%\.dotnet\tools 在 PATH 里
"@
}

function Update-ReleaseNotes {
    & powershell.exe -NoProfile -ExecutionPolicy Bypass -File (Join-Path $Root 'scripts\update-desktop-release-notes.ps1')
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}

function Invoke-VpkUploadGithub {
    param(
        [string]$OutDir,
        [string]$Channel,
        [string]$Tag,
        [string]$Sha,
        [string]$Version
    )
    $token = Get-GithubToken
    if (-not $token) {
        throw '需要 GITHUB_TOKEN 或已登录的 gh（gh auth login）'
    }
    $repoUrl = if ($env:VPK_REPO_URL) { $env:VPK_REPO_URL } else { 'https://github.com/1usdc/pusa' }
    $title = if ($env:VPK_PACK_TITLE) { $env:VPK_PACK_TITLE } else { 'Pusa' }
    & vpk upload github `
        --repoUrl $repoUrl `
        --channel $Channel `
        --outputDir $OutDir `
        --token $token `
        --tag $Tag `
        --targetCommitish $Sha `
        --releaseName "$title $Version" `
        --merge `
        --publish
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}


$Tag = Get-DesktopReleaseTag
$Build = if ($env:BUILD) { $env:BUILD } else { '0' }
$ReleaseDir = if ($env:RELEASE_DIR) { $env:RELEASE_DIR } else { Join-Path $Root 'desktop\dist' }

if ($Tag -notmatch '^desktop-v[0-9]+\.[0-9]+\.[0-9]+') {
    throw "TAG 格式不对：$Tag（应为 desktop-vX.Y.Z）"
}

if ($MergeOnly) {
    Write-Host "→ Merge 上传到已有 Release：$Tag（不 push / 不打 tag；可用 `$env:TAG 覆盖）"
} else {
    Write-Host "→ Release tag: $Tag（来自 desktop/Cargo.toml，可用 `$env:TAG 覆盖）"
}

if (-not (Get-Command gh -ErrorAction SilentlyContinue)) {
    throw '未找到 gh CLI，请先安装并 gh auth login'
}

if (-not $MergeOnly) {
    $dirty = git status --porcelain
    if ($dirty) {
        Write-Host '工作区不干净，请先提交或 stash' -ForegroundColor Red
        git status --short
        exit 1
    }
}

if ($Build -eq '1') {
    Say '== 0) 本机打包 =='
    & powershell.exe -NoProfile -ExecutionPolicy Bypass -File (Join-Path $Root 'scripts\desktop-windows.ps1')
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
    $ReleaseDir = Join-Path $Root 'desktop\dist'
}

$VpkOut = if ($env:VPK_OUT_DIR) { $env:VPK_OUT_DIR } else { Join-Path $Root 'desktop\dist\velopack' }
$VpkChannel = Get-VpkChannel
if ($VpkChannel -notlike 'win-*') {
    $VpkChannel = 'win-x64'
}

$Mode = 'setup'
$releasesJson = Join-Path $VpkOut "releases.$VpkChannel.json"
if (Test-Path $releasesJson) {
    $Mode = 'velopack'
    Require-Vpk
    Write-Host "→ Velopack 模式：$VpkOut（channel $VpkChannel）"
}

$Stage = Join-Path $Root '.release-upload'
if (Test-Path $Stage) { Remove-Item -Recurse -Force $Stage }
New-Item -ItemType Directory -Force -Path $Stage | Out-Null

if ($Mode -eq 'setup') {
    $found = $false
    foreach ($pattern in @('*-setup.exe', '*.msi')) {
        Get-ChildItem -Path $ReleaseDir -Filter $pattern -ErrorAction SilentlyContinue | ForEach-Object {
            Copy-Item -Force $_.FullName -Destination $Stage
            $found = $true
        }
    }
    if (-not $found) {
        throw "$ReleaseDir 里没有 Velopack 产物也没有 *-setup.exe，请先 just desktop-windows"
    }
}

if ($MergeOnly) {
    & gh release view $Tag 2>$null | Out-Null
    if ($LASTEXITCODE -ne 0) {
        throw "GitHub Release $Tag 不存在。请先用 just release-windows / release-mac 创建，或检查 TAG="
    }
    $Sha = Get-TagCommitSha $Tag
    if (-not $Sha) {
        throw "找不到 tag $Tag 的 commit（本地与 origin 都没有）"
    }
    Write-Host "→ 已有 Release $Tag @ $Sha，合并上传 Windows 资产"

    Say '== 上传 GitHub Release（merge）=='
    if ($Mode -eq 'velopack') {
        $ver = $Tag -replace '^desktop-v', ''
        Invoke-VpkUploadGithub -OutDir $VpkOut -Channel $VpkChannel -Tag $Tag -Sha $Sha -Version $ver
        Write-Host "✓ Windows Velopack 已合并到 Release $Tag（channel $VpkChannel）"
        Get-ChildItem $VpkOut | Format-Table Name, Length -AutoSize
        Update-ReleaseNotes
        exit 0
    }

    $files = @(Get-ChildItem $Stage -File | ForEach-Object { $_.FullName })
    if ($files.Count -gt 0) {
        & gh release upload $Tag @files --clobber
        if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
    }
    Write-Host "✓ Windows 安装包已合并到 Release $Tag"
    Get-ChildItem $Stage | Format-Table Name, Length -AutoSize
    Update-ReleaseNotes
    exit 0
}

Say '== 1) git push =='
git push
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

$Sha = (git rev-parse HEAD).Trim()

Say "== 2) 创建并推送 tag $Tag =="
$localTag = & git rev-parse --verify --quiet "refs/tags/$Tag" 2>$null
if ($LASTEXITCODE -eq 0 -and $localTag) {
    $localTagSha = (& git rev-parse "${Tag}^{commit}").Trim()
    if ($localTagSha -ne $Sha) {
        throw "本地 tag $Tag 指向 $localTagSha，当前 HEAD 是 $Sha（若只想补传资产：just release-windows-merge）"
    }
    Say "本地已有 tag $Tag，跳过 git tag"
} else {
    git tag $Tag
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}

$remoteLines = @(& git ls-remote --tags origin "refs/tags/$Tag" 2>$null)
$hasRemoteTag = $remoteLines | Where-Object { $_ -match [regex]::Escape("refs/tags/$Tag") -and $_ -notmatch '\^\{\}$' }
if ($hasRemoteTag) {
    $localPeeled = (& git rev-parse "${Tag}^{commit}").Trim()
    $peeledLine = @(& git ls-remote --tags origin "refs/tags/${Tag}^{}" 2>$null) | Select-Object -First 1
    $remoteSha = if ($peeledLine) {
        ($peeledLine -split '\s+')[0]
    } else {
        ($hasRemoteTag -split '\s+')[0]
    }
    if ($remoteSha -ne $localPeeled) {
        throw "remote 已有 tag $Tag 但指向不同 commit（remote=$remoteSha local=$localPeeled）"
    }
    Say "remote 已有 tag $Tag"
} else {
    git push origin $Tag
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}

Say '== 3) 上传 GitHub Release =='
if ($Mode -eq 'velopack') {
    $ver = $Tag -replace '^desktop-v', ''
    Invoke-VpkUploadGithub -OutDir $VpkOut -Channel $VpkChannel -Tag $Tag -Sha $Sha -Version $ver
    Write-Host "✓ Windows Velopack Release $Tag 已上传（channel $VpkChannel）"
    Get-ChildItem $VpkOut | Format-Table Name, Length -AutoSize
    Update-ReleaseNotes
    exit 0
}

$releaseExists = $false
& gh release view $Tag 2>$null | Out-Null
if ($LASTEXITCODE -eq 0) { $releaseExists = $true }

if ($releaseExists) {
    $assets = & gh release view $Tag --json assets -q '.assets[].name' 2>$null
    foreach ($name in @($assets)) {
        if (-not $name) { continue }
        if ($name -notmatch '(-setup\.exe|\.msi)$') { continue }
        Say "delete asset: $name"
        & gh release delete-asset $Tag $name --yes 2>$null
    }
    $files = @(Get-ChildItem $Stage -File | ForEach-Object { $_.FullName })
    if ($files.Count -gt 0) {
        & gh release upload $Tag @files --clobber
        if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
    }
} else {
    $files = @(Get-ChildItem $Stage -File | ForEach-Object { $_.FullName })
    & gh release create $Tag --target $Sha --generate-notes @files
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}

Write-Host "✓ Windows Release $Tag 已上传"
Get-ChildItem $Stage | Format-Table Name, Length -AutoSize
Update-ReleaseNotes
