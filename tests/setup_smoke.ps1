#requires -Version 5.1
[CmdletBinding()]
param()

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$tempRoot = Join-Path ([IO.Path]::GetTempPath()) ("harness-setup-ps-" + [Guid]::NewGuid().ToString("N"))

function Assert-True {
    param(
        [bool]$Condition,
        [string]$Message
    )
    if (-not $Condition) {
        throw $Message
    }
}

function Copy-Fixture {
    param([string]$Target)
    New-Item -ItemType Directory -Path $Target -Force | Out-Null
    Copy-Item -LiteralPath (Join-Path $repoRoot "setup_harness.ps1") -Destination $Target
    Copy-Item -LiteralPath (Join-Path $repoRoot "templates") -Destination $Target -Recurse
}

try {
    $env:DB_HOST = "postgres.example"
    $env:DB_USER = "harness"
    $env:DB_PASSWORD = "secret"
    $env:DB_NAME = "harness"
    $env:DB_SSL_MODE = "require"

    $dryRun = Join-Path $tempRoot "dry-run"
    Copy-Fixture -Target $dryRun
    $dryJson = & (Join-Path $dryRun "setup_harness.ps1") `
        -Root -NoGraphify -NoGraphifySkills -NoAntigravity -DryRun -Json 6>&1 |
        Out-String
    Assert-True ($dryJson -match '"dry_run":\s*true') "Dry-run JSON report was not emitted."
    Assert-True (-not (Test-Path -LiteralPath (Join-Path $dryRun ".harness_layout"))) "Dry-run wrote the layout marker."

    $fixture = Join-Path $tempRoot "root-layout"
    Copy-Fixture -Target $fixture

    $fakeBin = Join-Path $tempRoot "fake-bin"
    New-Item -ItemType Directory -Path $fakeBin -Force | Out-Null
    $cargoTarget = Join-Path $fixture "cargo-target"
    New-Item -ItemType Directory -Path (Join-Path $fixture "rust") -Force | Out-Null
    Set-Content -LiteralPath (Join-Path $fixture "rust/Cargo.toml") -Value @'
[package]
name = "harness-smoke"
version = "0.0.0"
edition = "2021"
'@ -Encoding utf8NoBOM

    $runningOnWindows = $env:OS -eq "Windows_NT"
    if ($runningOnWindows) {
        $fakePython = @'
@echo off
if "%1"=="-c" exit /b 0
if "%1"=="-" (
  more >nul
  exit /b 0
)
exit /b 0
'@
        Set-Content -LiteralPath (Join-Path $fakeBin "python.cmd") -Value $fakePython -Encoding Ascii
        Set-Content -LiteralPath (Join-Path $fakeBin "python3.cmd") -Value $fakePython -Encoding Ascii
        $fakeCargo = @'
@echo off
echo %*> "%CD%\cargo-args.txt"
if not exist "%CARGO_TARGET_DIR%\release" mkdir "%CARGO_TARGET_DIR%\release"
echo fake harness> "%CARGO_TARGET_DIR%\release\harness.exe"
exit /b 0
'@
        Set-Content -LiteralPath (Join-Path $fakeBin "cargo.cmd") -Value $fakeCargo -Encoding Ascii
    }
    else {
        $fakePython = @'
#!/bin/sh
if [ "$1" = "-c" ]; then
  exit 0
fi
if [ "$1" = "-" ]; then
  cat >/dev/null
  exit 0
fi
exit 0
'@
        foreach ($name in @("python", "python3")) {
            $path = Join-Path $fakeBin $name
            Set-Content -LiteralPath $path -Value $fakePython -Encoding utf8NoBOM
            & chmod +x $path
        }
        $fakeCargo = @'
#!/bin/sh
printf '%s\n' "$*" > "$PWD/cargo-args.txt"
mkdir -p "$CARGO_TARGET_DIR/release"
printf 'fake harness\n' > "$CARGO_TARGET_DIR/release/harness.exe"
exit 0
'@
        $cargoPath = Join-Path $fakeBin "cargo"
        Set-Content -LiteralPath $cargoPath -Value $fakeCargo -Encoding utf8NoBOM
        & chmod +x $cargoPath
    }
    $oldPath = $env:PATH
    $oldCargoTarget = $env:CARGO_TARGET_DIR
    $env:PATH = $fakeBin + [IO.Path]::PathSeparator + $env:PATH
    try {
        & (Join-Path $fixture "setup_harness.ps1") `
            -Root -NoGraphify -NoGraphifySkills -NoAntigravity `
            -CargoTargetDir $cargoTarget
    }
    finally {
        $env:PATH = $oldPath
        $env:CARGO_TARGET_DIR = $oldCargoTarget
    }

    Assert-True (Test-Path -LiteralPath (Join-Path $fixture "harness_cli.ps1")) "PowerShell CLI shim was not installed."
    Assert-True (Test-Path -LiteralPath (Join-Path $fixture "harness.exe")) "Cargo output harness.exe was not installed."
    $cargoArgs = Get-Content -LiteralPath (Join-Path $fixture "rust/cargo-args.txt") -Raw
    Assert-True ($cargoArgs -match "build --release --locked") "Cargo was not invoked with build --release --locked."
    Assert-True (Test-Path -LiteralPath (Join-Path $fixture ".codex/hooks.json")) "Codex hooks were not generated."
    Assert-True (Test-Path -LiteralPath (Join-Path $fixture "bin/harness-hook.ps1")) "PowerShell hook runtime was not generated."
    Assert-True (Test-Path -LiteralPath (Join-Path $fixture ".gemini/commands/harness/check.toml")) "Gemini check command was not generated."
    Assert-True ((Get-Content -LiteralPath (Join-Path $fixture ".harness_layout") -Raw).Trim() -eq "root") "Root layout marker is incorrect."
    Get-Content -LiteralPath (Join-Path $fixture ".codex/hooks.json") -Raw | ConvertFrom-Json | Out-Null
    Get-Content -LiteralPath (Join-Path $fixture ".gemini/settings.json") -Raw | ConvertFrom-Json | Out-Null

    & (Join-Path $fixture "setup_harness.ps1") `
        -Root -NoGraphify -NoGraphifySkills -NoAntigravity -Reset
    Assert-True (-not (Test-Path -LiteralPath (Join-Path $fixture ".harness_layout"))) "Reset did not remove the layout marker."

    Write-Host "[OK] PowerShell setup smoke: dry-run, root layout, hooks, shim, and reset."
}
finally {
    Remove-Item -LiteralPath $tempRoot -Recurse -Force -ErrorAction SilentlyContinue
}
