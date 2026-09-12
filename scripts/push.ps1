# 同时推送开源仓 pusa 与私有仓 pusa-core\（固定在仓库根下），不打 tag / 不发版。
# 工作区有未提交改动时：git add -A → commit → push。
# 提交说明：$env:COMMIT_MSG（默认 chore: sync local changes before push）。
$ErrorActionPreference = 'Stop'
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8

$Root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
Set-Location $Root

function Get-CommitMessage {
    if ($env:COMMIT_MSG -and $env:COMMIT_MSG.Trim()) {
        return $env:COMMIT_MSG.Trim()
    }
    return 'chore: sync local changes before push'
}

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

        $dirty = git status --porcelain
        if ($dirty) {
            Write-Host "→ ${Label}: 检测到未提交改动，自动 add + commit"
            git add -A
            if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
            $staged = git diff --cached --name-only
            if ($staged) {
                $msg = Get-CommitMessage
                git commit -m $msg
                if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
                Write-Host "✓ ${Label}: committed — $msg"
            } else {
                Write-Host "→ ${Label}: add 后无已跟踪变更（可能全是 ignored），跳过 commit"
            }
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
                Write-Host "✓ ${Label}: 已与 $upstream 同步，跳过 push"
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
