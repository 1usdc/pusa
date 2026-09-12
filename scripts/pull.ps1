# 同时拉取开源仓 pusa 与私有仓 pusa-core\（固定在仓库根下），不打 tag / 不发版。
$ErrorActionPreference = 'Stop'
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8

$Root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
Set-Location $Root

function Invoke-RepoPull {
    param(
        [Parameter(Mandatory = $true)][string]$Dir,
        [Parameter(Mandatory = $true)][string]$Label
    )
    Write-Host "== pull $Label ($Dir) =="
    $gitDir = Join-Path $Dir '.git'
    if (-not (Test-Path $gitDir)) {
        throw "$Label`: 不是 git 仓库（期望路径：$Dir）"
    }
    Push-Location $Dir
    try {
        $branch = (git rev-parse --abbrev-ref HEAD).Trim()
        if ($branch -eq 'HEAD') {
            throw "$Label`: detached HEAD，请先切到分支再拉"
        }
        git pull --ff-only
        if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
        $sha = (git rev-parse --short HEAD).Trim()
        Write-Host "✓ ${Label}: $sha @ $branch"
    }
    finally {
        Pop-Location
    }
}

Invoke-RepoPull -Dir $Root -Label 'pusa'
Invoke-RepoPull -Dir (Join-Path $Root 'pusa-core') -Label 'pusa-core'
Write-Host '✓ pull 完成（pusa + pusa-core）'
