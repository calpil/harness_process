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
        # (fake python block removed - Rust only)
@echo off
echo %*> "%CD%\cargo-args.txt"
if not exist "%CARGO_TARGET_DIR%\release" mkdir "%CARGO_TARGET_DIR%\release"
echo fake harness> "%CARGO_TARGET_DIR%\release\harness.exe"
exit /b 0
'@
        Set-Content -LiteralPath (Join-Path $fakeBin "cargo.cmd") -Value $fakeCargo -Encoding Ascii
    }
    else {
        # (fake python block removed - Rust only)
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

    # SDD: la constitution es un required asset y el instalador la siembra en el
    # docs/ de la RAIZ (en root layout, RAIZ == fixture). Paridad con el smoke sh;
    # la ejecucion real en Windows queda pendiente de entorno (como en feature #1).
    $installerText = Get-Content -LiteralPath (Join-Path $fixture "setup_harness.ps1") -Raw
    Assert-True ($installerText -match '"docs/constitution\.md"') "Installer does not declare docs/constitution.md as a required asset."
    Assert-True (Test-Path -LiteralPath (Join-Path $fixture "templates/docs/constitution.md")) "Constitution template asset is missing from the distribution."
    Assert-True (Test-Path -LiteralPath (Join-Path $fixture "docs/constitution.md")) "Constitution was not seeded by the installer."

    # Feature #4 / AC-2: en layout root los tres docs del arnes se siembran en el
    # docs/ de la RAIZ (que aqui es el propio fixture).
    foreach ($harnessDoc in @("architecture.md", "conventions.md", "verification.md")) {
        Assert-True (Test-Path -LiteralPath (Join-Path $fixture "docs/$harnessDoc")) "Harness doc $harnessDoc was not seeded into the root docs/."
    }
    # Feature #4 / AC-4 (reinstall): comparten carpeta con la documentacion del
    # equipo, asi que se siembran solo-si-faltan y un reinstall NO los pisa.
    $docsSentinel = "SENTINEL-DOCS-ARNES-NO-PISA-PS"
    Add-Content -LiteralPath (Join-Path $fixture "docs/conventions.md") -Value "<!-- $docsSentinel -->"
    & (Join-Path $fixture "setup_harness.ps1") `
        -Root -NoGraphify -NoGraphifySkills -NoAntigravity -CargoTargetDir $cargoTarget
    Assert-True ((Get-Content -LiteralPath (Join-Path $fixture "docs/conventions.md") -Raw) -match $docsSentinel) "Reinstall overwrote a harness doc already present in the root docs/."

    # Feature #4 / AC-6: los artefactos de feature comparten carpeta con los docs
    # generados y el reset NO puede llevarselos por delante.
    Set-Content -LiteralPath (Join-Path $fixture "docs/spec-feature-1-demo.md") -Value "# spec" -Encoding utf8NoBOM
    Set-Content -LiteralPath (Join-Path $fixture "docs/plan-feature-1-demo.md") -Value "# plan" -Encoding utf8NoBOM

    & (Join-Path $fixture "setup_harness.ps1") `
        -Root -NoGraphify -NoGraphifySkills -NoAntigravity -Reset
    Assert-True (-not (Test-Path -LiteralPath (Join-Path $fixture ".harness_layout"))) "Reset did not remove the layout marker."
    Assert-True (Test-Path -LiteralPath (Join-Path $fixture "docs/constitution.md")) "Reset removed the user's constitution."
    foreach ($artifact in @("spec-feature-1-demo.md", "plan-feature-1-demo.md")) {
        Assert-True (Test-Path -LiteralPath (Join-Path $fixture "docs/$artifact")) "Reset removed the feature artifact $artifact."
    }
    foreach ($harnessDoc in @("architecture.md", "conventions.md", "verification.md")) {
        Assert-True (-not (Test-Path -LiteralPath (Join-Path $fixture "docs/$harnessDoc"))) "Reset did not clean the generated doc $harnessDoc."
    }

    # --- Feature #4 / AC-1 + AC-3 + AC-4: layout subdir y migracion -----------
    # El arnes vive en <raiz>/harness_process y los docs deben quedar en
    # <raiz>/docs. Los que ya estaban en la ubicacion vieja se MUEVEN si faltan
    # en la raiz; el que el equipo ya tiene en la raiz NO se pisa.
    $subdirRoot = Join-Path $tempRoot "subdir-layout"
    $subdirHarness = Join-Path $subdirRoot "harness_process"
    Copy-Fixture -Target $subdirHarness
    New-Item -ItemType Directory -Path (Join-Path $subdirHarness "docs") -Force | Out-Null
    New-Item -ItemType Directory -Path (Join-Path $subdirRoot "docs") -Force | Out-Null
    Set-Content -LiteralPath (Join-Path $subdirHarness "docs/architecture.md") -Value "VIEJO-ARCHITECTURE" -Encoding utf8NoBOM
    Set-Content -LiteralPath (Join-Path $subdirHarness "docs/verification.md") -Value "VIEJO-VERIFICATION" -Encoding utf8NoBOM
    Set-Content -LiteralPath (Join-Path $subdirHarness "docs/conventions.md") -Value "VIEJO-CONVENTIONS" -Encoding utf8NoBOM
    $teamSentinel = "SENTINEL-CONVENTIONS-DEL-EQUIPO-PS"
    Set-Content -LiteralPath (Join-Path $subdirRoot "docs/conventions.md") -Value $teamSentinel -Encoding utf8NoBOM
    & (Join-Path $subdirHarness "setup_harness.ps1") `
        -NoGraphify -NoGraphifySkills -NoAntigravity -CargoTargetDir $cargoTarget

    # AC-1: destino raiz, y la subcarpeta del arnes ya no tiene esos docs.
    Assert-True (Test-Path -LiteralPath (Join-Path $subdirRoot "docs/constitution.md")) "Constitution was not seeded into the multi-repo root docs/."
    # AC-3: se movio el contenido viejo (no se regenero desde la plantilla).
    Assert-True ((Get-Content -LiteralPath (Join-Path $subdirRoot "docs/architecture.md") -Raw).Trim() -eq "VIEJO-ARCHITECTURE") "architecture.md was not migrated with its content."
    Assert-True ((Get-Content -LiteralPath (Join-Path $subdirRoot "docs/verification.md") -Raw).Trim() -eq "VIEJO-VERIFICATION") "verification.md was not migrated with its content."
    Assert-True (-not (Test-Path -LiteralPath (Join-Path $subdirHarness "docs/architecture.md"))) "architecture.md is still in the harness subfolder."
    Assert-True (-not (Test-Path -LiteralPath (Join-Path $subdirHarness "docs/verification.md"))) "verification.md is still in the harness subfolder."
    # AC-4: el doc del equipo queda intacto y la copia vieja se conserva.
    Assert-True ((Get-Content -LiteralPath (Join-Path $subdirRoot "docs/conventions.md") -Raw) -match $teamSentinel) "Migration overwrote the team's conventions.md in the root docs/."
    Assert-True ((Get-Content -LiteralPath (Join-Path $subdirHarness "docs/conventions.md") -Raw).Trim() -eq "VIEJO-CONVENTIONS") "Migration removed the old copy instead of keeping it."

    Write-Host "[OK] PowerShell setup smoke: dry-run, root layout, hooks, shim, constitution seed, harness docs in root docs/ (seed, migration, no-overwrite), and reset."
}
finally {
    Remove-Item -LiteralPath $tempRoot -Recurse -Force -ErrorAction SilentlyContinue
}
