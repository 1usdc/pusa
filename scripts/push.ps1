# 同时推送开源仓 pusa 与私有仓 pusa-core\（固定在仓库根下），不打 tag / 不发版。
$ErrorActionPreference = 'Stop'
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8

$Root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
Set-Location $Root

function Invoke-RepoPush {
    param(
        [Parameter(Mandatory = $true)][string]$Dir,
        [Parameter(Mandatory = $true)][string]$Label
    )
    Write-Host "== push $Label ($Dir) =="
    $gitDir = Join-Path $Dir '.git'
    if (-not (Test-Path $gitDir)) {
        throw "$Label`: 不是 git 仓库（期望路径：$Dir）"
    }
    Push-Location $Dir
    try {
        $branch = (git rev-parse --abbrev-ref HEAD).Trim()
        if ($branch -eq 'HEAD') {
            throw "$Label`: detached HEAD，请先切到分支再推"
        }
        $upstream = $null
        try {
            $upstream = (git rev-parse --abbrev-ref --symbolic-full-name '@{u}' 2>$null).Trim()
        } catch {
            $upstream = $null
        }
        if ($upstream) {
            $ahead = [int](git rev-list --count "${upstream}..HEAD").Trim()
            if ($ahead -eq 0) {
                Write-Host "✓ ${Label}: 已与 $upstream 同步，跳过"
                return
            }
        }
        git push -u origin HEAD
        if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
        Write-Host "✓ ${Label}: $branch → origin（无 tag / 无 release）"
    }
    finally {
        Pop-Location
    }
}

Invoke-RepoPush -Dir $Root -Label 'pusa'
Invoke-RepoPush -Dir (Join-Path $Root 'pusa-core') -Label 'pusa-core'
Write-Host '✓ push 完成（pusa + pusa-core）'
