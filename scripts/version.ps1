# 查看当前版本号和仓库打包版本号
$ErrorActionPreference = "Stop"

$ROOT = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
Set-Location $ROOT

$CARGO_TOML = Join-Path $ROOT "desktop\Cargo.toml"
if (-not (Test-Path $CARGO_TOML)) {
    Write-Error "未找到 $CARGO_TOML"
    exit 1
}

# 读取版本号
$content = Get-Content $CARGO_TOML -Raw
if ($content -match 'version\s*=\s*"(\d+\.\d+\.\d+)(?:[.-].*)?"') {
    $VERSION = $matches[1]
} else {
    Write-Error "无法从 desktop/Cargo.toml 解析 version"
    exit 1
}

Write-Host "当前版本号: $VERSION"
Write-Host "打包 tag: desktop-v$VERSION"

# 检查 GitHub Release
if (Get-Command gh -ErrorAction SilentlyContinue) {
    $TAG = "desktop-v$VERSION"
    $result = gh release view $TAG 2>&1
    if ($LASTEXITCODE -eq 0) {
        Write-Host "✓ 已发布到 GitHub Release: $TAG"
        Write-Host ""
        Write-Host "Release 资产:"
        gh release view $TAG --json assets -q '.assets[] | "  - \(.name) (\(.size // 0 | . / 1024 / 1024 | floor)MB)"'
    } else {
        Write-Host "✗ 未发布到 GitHub Release: $TAG"
    }
} else {
    Write-Host "（未安装 gh CLI，无法检查 GitHub Release）"
}
