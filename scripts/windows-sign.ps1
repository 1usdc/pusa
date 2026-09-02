# Windows：用 Azure Artifact Signing 给 NSIS 安装器盖 Authenticode 签名。
#
# 前置（本机一次性）：
#   winget install -e --id Microsoft.Azure.ArtifactSigningClientTools
#   az login   # 或 .env.signing 里配 AZURE_CLIENT_ID / AZURE_TENANT_ID / AZURE_CLIENT_SECRET
#
# 凭证见 .env.signing（由 desktop-windows.ps1 加载）：
#   AZURE_TS_ENDPOINT、AZURE_TS_ACCOUNT、AZURE_TS_PROFILE
#   可选 AZURE_SIGNING_DLIB（Azure.CodeSigning.Dlib.dll 绝对路径）
#
# 用法：.\scripts\windows-sign.ps1 [-Path desktop\dist\*-setup.exe]
param(
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$Paths = @()
)

$ErrorActionPreference = 'Stop'

$Root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$Dist = Join-Path $Root 'desktop\dist'
$EnvSigning = Join-Path $Root '.env.signing'
$TimestampUrl = 'http://timestamp.acs.microsoft.com'
$MinSdkVersion = [version]'10.0.22621.755'

function Load-EnvSigning {
    if (-not (Test-Path $EnvSigning)) { return }
    Get-Content $EnvSigning | ForEach-Object {
        $line = $_.Trim()
        if ($line -eq '' -or $line.StartsWith('#')) { return }
        $eq = $line.IndexOf('=')
        if ($eq -lt 1) { return }
        $name = $line.Substring(0, $eq).Trim()
        $value = $line.Substring($eq + 1).Trim()
        if ($value.StartsWith('"') -and $value.EndsWith('"')) {
            $value = $value.Substring(1, $value.Length - 2)
        }
        [Environment]::SetEnvironmentVariable($name, $value, 'Process')
    }
}

function Find-Signtool {
    $candidates = @()

    $cmd = Get-Command signtool.exe -ErrorAction SilentlyContinue
    if ($cmd) { $candidates += $cmd.Source }

    $kits = 'C:\Program Files (x86)\Windows Kits\10\bin'
    if (Test-Path $kits) {
        $candidates += @(
            Get-ChildItem -Path $kits -Recurse -Filter signtool.exe -ErrorAction SilentlyContinue |
                Where-Object { $_.FullName -match '\\x64\\signtool\.exe$' } |
                Sort-Object FullName -Descending |
                Select-Object -ExpandProperty FullName
        )
    }

    $sdkBuild = 'C:\Program Files\Microsoft\Windows Kits\10\bin'
    if (Test-Path $sdkBuild) {
        $candidates += @(
            Get-ChildItem -Path $sdkBuild -Recurse -Filter signtool.exe -ErrorAction SilentlyContinue |
                Where-Object { $_.FullName -match '\\x64\\signtool\.exe$' } |
                Sort-Object FullName -Descending |
                Select-Object -ExpandProperty FullName
        )
    }

    foreach ($path in ($candidates | Select-Object -Unique)) {
        if ($path -match '\\10\.0\.(\d+\.\d+)\\') {
            $ver = [version]$Matches[1]
            if ($ver -ge $MinSdkVersion) { return $path }
        }
    }

    if ($candidates.Count -gt 0) {
        Write-Warning "signtool 版本可能低于 ${MinSdkVersion}，仍尝试：$($candidates[0])"
        return $candidates[0]
    }

    throw @"
未找到 signtool.exe（需 Windows SDK >= 10.0.22621.755）。
安装 Visual Studio Build Tools / Windows SDK，或：
  winget install Microsoft.Windows.SDK.BuildTools
"@
}

function Find-Dlib {
    if ($env:AZURE_SIGNING_DLIB -and (Test-Path $env:AZURE_SIGNING_DLIB)) {
        return (Resolve-Path $env:AZURE_SIGNING_DLIB).Path
    }

    $searchRoots = @(
        'C:\Program Files\Microsoft\Azure Artifact Signing Client Tools',
        'C:\Program Files (x86)\Microsoft\Azure Artifact Signing Client Tools',
        (Join-Path $Root '.azure-signing'),
        (Join-Path $env:LOCALAPPDATA 'Microsoft\ArtifactSigning')
    )

    foreach ($root in $searchRoots) {
        if (-not (Test-Path $root)) { continue }
        $hit = Get-ChildItem -Path $root -Recurse -Filter 'Azure.CodeSigning.Dlib.dll' -ErrorAction SilentlyContinue |
            Where-Object { $_.FullName -match '\\x64\\Azure\.CodeSigning\.Dlib\.dll$' } |
            Select-Object -First 1
        if ($hit) { return $hit.FullName }
    }

    throw @"
未找到 Azure.CodeSigning.Dlib.dll。请安装 Artifact Signing Client Tools：
  winget install -e --id Microsoft.Azure.ArtifactSigningClientTools
或从 NuGet 解压 Microsoft.ArtifactSigning.Client 到 .azure-signing/，
或在 .env.signing 设置 AZURE_SIGNING_DLIB=完整路径
"@
}

function New-MetadataFile {
    param(
        [string]$Endpoint,
        [string]$Account,
        [string]$Profile
    )

    $endpoint = $Endpoint.TrimEnd('/')
    $metaDir = Join-Path $Root '.azure-signing'
    New-Item -ItemType Directory -Force -Path $metaDir | Out-Null
    $metaPath = Join-Path $metaDir 'metadata.json'

    $payload = [ordered]@{
        Endpoint               = $endpoint
        CodeSigningAccountName = $Account
        CertificateProfileName = $Profile
    }
    ($payload | ConvertTo-Json -Compress) + "`n" | Set-Content -Path $metaPath -Encoding utf8NoBOM
    return $metaPath
}

function Resolve-InstallerPaths {
    param([string[]]$InputPaths)
    if ($InputPaths.Count -gt 0) {
        return $InputPaths | ForEach-Object { (Resolve-Path $_).Path }
    }
    $files = @(Get-ChildItem -Path $Dist -Filter '*-setup.exe' -ErrorAction SilentlyContinue)
    if ($files.Count -eq 0) {
        throw '未找到安装器：先在 desktop/ 跑 dx bundle --release，或指定 -Path'
    }
    return $files.FullName
}

function Test-AzureAuthHint {
    if ($env:AZURE_CLIENT_ID -and $env:AZURE_TENANT_ID -and $env:AZURE_CLIENT_SECRET) {
        return
    }
    $az = Get-Command az -ErrorAction SilentlyContinue
    if (-not $az) {
        Write-Warning '未检测到 az CLI；请在 .env.signing 配置 AZURE_CLIENT_ID/TENANT_ID/CLIENT_SECRET，或安装 Azure CLI 后 az login'
        return
    }
    $account = az account show 2>$null | ConvertFrom-Json
    if (-not $account) {
        throw 'Azure 未登录。请先 az login，或在 .env.signing 配置服务主体 AZURE_CLIENT_ID / AZURE_TENANT_ID / AZURE_CLIENT_SECRET'
    }
    Write-Host "Azure 账号：$($account.user.name)（subscription: $($account.name)）"
}

Load-EnvSigning

$endpoint = $env:AZURE_TS_ENDPOINT
$account = $env:AZURE_TS_ACCOUNT
$profile = $env:AZURE_TS_PROFILE

if (-not $endpoint -or -not $account -or -not $profile) {
    throw @"
缺少 Azure Artifact Signing 配置。在 .env.signing 中设置：
  AZURE_TS_ENDPOINT=https://eus.codesigning.azure.net
  AZURE_TS_ACCOUNT=你的 Trusted Signing 账户名
  AZURE_TS_PROFILE=你的 Certificate Profile 名
"@
}

Test-AzureAuthHint

$signtool = Find-Signtool
$dlib = Find-Dlib
$metadata = New-MetadataFile -Endpoint $endpoint -Account $account -Profile $profile
$targets = @(Resolve-InstallerPaths -InputPaths $Paths)

Write-Host "signtool: $signtool"
Write-Host "dlib:     $dlib"
Write-Host "metadata: $metadata"

foreach ($file in $targets) {
    Write-Host "==> 签名 $file"
    & $signtool sign /v /fd SHA256 /tr $TimestampUrl /td SHA256 `
        /dlib $dlib /dmdf $metadata $file

    if ($LASTEXITCODE -ne 0) {
        throw @"
signtool 失败（exit $LASTEXITCODE）。
常见原因：az login 过期、无 Certificate Profile Signer 角色、Endpoint 区域与账户不一致。
"@
    }

    $sig = Get-AuthenticodeSignature $file
    $sig | Format-List Status, StatusMessage, SignerCertificate | Out-String | Write-Host
    if ($sig.Status -ne 'Valid') {
        throw "签名无效：$($sig.Status) — $($sig.StatusMessage)"
    }
}

Write-Host "✓ Azure Artifact Signing 完成（$($targets.Count) 个文件）"
