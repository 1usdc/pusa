# 打包闭源核心 → vendor/pusa-core/<triple>/pusa_core.dll
#
# pusa-core/justfile 的 pack 配方用了 `#!/usr/bin/env bash` shebang，
# Windows 上 just 需要 cygpath 才能翻译解释器路径。Git for Windows 自带
# cygpath.exe（usr\bin），但默认不在 PATH，这里临时补进去。
$ErrorActionPreference = 'Stop'
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8

$Root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
Set-Location $Root

if (-not (Test-Path (Join-Path $Root 'pusa-core'))) {
    Write-Host 'error: 缺少 pusa-core/ 源码目录，无法打包'
    exit 1
}

# MSVC 预检查必须在改 PATH 之前：Git\usr\bin 里的 GNU link.exe 会被误认成链接器。
function Test-MsvcLinker {
    $vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio\Installer\vswhere.exe'
    if (Test-Path $vswhere) { return $true }
    foreach ($cmd in @(Get-Command link.exe -All -ErrorAction SilentlyContinue)) {
        if ($cmd.Source -match 'Microsoft Visual Studio|\\MSVC\\') { return $true }
    }
    return $false
}

if (-not (Test-MsvcLinker)) {
    Write-Host @"
error: 未检测到 MSVC 编译工具（link.exe / Visual Studio 安装器）。
Rust 工具链是 x86_64-pc-windows-msvc，编译必须有「使用 C++ 的桌面开发」工作负载：
  winget install --id Microsoft.VisualStudio.2022.BuildTools --override "--quiet --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"
装完重开终端再跑 just pusa-core。
"@
    exit 1
}

if (-not (Get-Command cygpath -ErrorAction SilentlyContinue)) {
    $git = Get-Command git -ErrorAction SilentlyContinue
    if (-not $git) {
        Write-Host 'error: 未找到 git；需要 Git for Windows 提供的 bash / cygpath'
        exit 1
    }
    $gitRoot = Split-Path -Parent (Split-Path -Parent $git.Source)
    $usrBin = Join-Path $gitRoot 'usr\bin'
    if (-not (Test-Path (Join-Path $usrBin 'cygpath.exe'))) {
        Write-Host "error: 未找到 cygpath.exe（找过 $usrBin）；请安装 Git for Windows"
        exit 1
    }
    # usr\bin 里的 GNU link.exe 会顶掉 MSVC 链接器，所以只能追加到 PATH 末尾；
    # bash 取自 Git\bin（只有 bash/git/sh，不会遮蔽 MSVC 工具）。
    $env:PATH = "$(Join-Path $gitRoot 'bin');$env:PATH;$usrBin"
    Write-Host "PATH: +Git\bin（bash）, +Git\usr\bin 末尾（cygpath/env）"
}

just --justfile pusa-core/justfile --working-directory pusa-core pack
exit $LASTEXITCODE
