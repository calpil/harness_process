#requires -Version 5.1
[CmdletBinding()]
param()

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$tempRoot = Join-Path ([IO.Path]::GetTempPath()) ("harness-setup-ps-" + [Guid]::NewGuid().ToString("N"))

# El binario real, compilado UNA vez y sembrado en las fixtures que despues le
# piden trabajo de verdad al CLI (paridad con PREBUILT_BIN del smoke de sh).
# Sin esto el smoke sembraba un harness.exe FALSO —un archivo de texto que
# escribe el cargo de mentira— y mas adelante le pedia `prd add`: Windows
# contestaba "no es una aplicacion valida para esta plataforma" y el smoke moria
# ahi. Es la segunda mitad de por que nunca se lo vio pasar.
function Get-CargoPath {
    $command = Get-Command cargo.exe -ErrorAction SilentlyContinue
    if ($command) { return $command.Source }
    $candidatos = @()
    if ($env:CARGO_HOME) { $candidatos += (Join-Path $env:CARGO_HOME "bin/cargo.exe") }
    $candidatos += (Join-Path $HOME ".cargo/bin/cargo.exe")
    foreach ($c in $candidatos) {
        if ($c -and (Test-Path -LiteralPath $c -PathType Leaf)) { return $c }
    }
    return $null
}

function Get-PrebuiltBinary {
    $targetDir = if ($env:CARGO_TARGET_DIR) { $env:CARGO_TARGET_DIR } else { Join-Path $repoRoot "rust/target" }
    $bin = Join-Path $targetDir "release/harness.exe"
    if (Test-Path -LiteralPath $bin -PathType Leaf) { return $bin }
    $cargo = Get-CargoPath
    if (-not $cargo) {
        throw "cargo es requerido para el smoke (harness Rust-only) y no esta en PATH ni en ~/.cargo/bin."
    }
    Push-Location (Join-Path $repoRoot "rust")
    try { & $cargo build --release --quiet } finally { Pop-Location }
    if (-not (Test-Path -LiteralPath $bin -PathType Leaf)) {
        throw "cargo build no dejo el binario en $bin"
    }
    return $bin
}

# Mismo motivo que en el instalador: `Get-Content` en Windows PowerShell 5.1 lee
# con la codepage ANSI y devuelve `â€”` donde el archivo dice `—`. Como
# `[IO.File]::ReadAllLines` SI lee UTF-8, el smoke comparaba un texto contra su
# version mojibake y reportaba espejos "desincronizados" que estaban bien.
function Get-Utf8Text {
    param([string]$Path)
    return [IO.File]::ReadAllText($Path, [Text.UTF8Encoding]::new($false))
}

# En Windows PowerShell 5.1, redirigir el stderr de un EJECUTABLE nativo con
# `2>&1` envuelve cada linea en un ErrorRecord, y con $ErrorActionPreference =
# "Stop" la primera de ellas LANZA. harness_check.sh escribe sus avisos por
# stderr —incluso los informativos, incluso cuando sale 0—, asi que el smoke se
# caia por leer la salida que venia a revisar. Se baja la preferencia solo
# alrededor de la llamada.
function Invoke-CapturandoStderr {
    param([scriptblock]$Bloque)
    $previo = $ErrorActionPreference
    $ErrorActionPreference = "Continue"
    try { return (& $Bloque | Out-String) }
    finally { $ErrorActionPreference = $previo }
}

function Assert-True {
    param(
        [bool]$Condition,
        [string]$Message
    )
    if (-not $Condition) {
        throw $Message
    }
}

# `utf8NoBOM` es un valor de -Encoding que SOLO existe en PowerShell 7. Este
# smoke declara `#requires -Version 5.1`, pero con esa palabra adentro en 5.1 no
# llegaba ni a la primera asercion: moria con "Cannot convert value utf8NoBOM".
# Es la razon concreta por la que once features seguidas escribieron "esta
# maquina no tiene pwsh" y la paridad de instaladores nunca se ejecuto. UTF-8
# sin BOM escrito a mano funciona igual en 5.1 y en 7.
function Set-TextUtf8NoBom {
    param(
        [string]$Path,
        [string]$Value
    )
    if (-not [IO.Path]::IsPathRooted($Path)) {
        $Path = Join-Path (Get-Location).Path $Path
    }
    [IO.File]::WriteAllText($Path, $Value + [Environment]::NewLine, (New-Object System.Text.UTF8Encoding($false)))
}

function Copy-Fixture {
    param([string]$Target)
    New-Item -ItemType Directory -Path $Target -Force | Out-Null
    Copy-Item -LiteralPath (Join-Path $repoRoot "setup_harness.ps1") -Destination $Target
    Copy-Item -LiteralPath (Join-Path $repoRoot "templates") -Destination $Target -Recurse
}

$prebuilt = Get-PrebuiltBinary

try {
    $env:DB_HOST = "postgres.example"
    $env:DB_USER = "harness"
    $env:DB_PASSWORD = "secret"
    $env:DB_NAME = "harness"
    $env:DB_SSL_MODE = "require"
    # Aislamiento Kimi (feature #8): este smoke NO overridea HOME, asi que sin
    # esto una maquina con kimi en PATH escribiria el bloque global de hooks en
    # el ~/.kimi-code REAL. Toda corrida usa una fixture; los bloques Kimi
    # re-apuntan la variable a sus propias fixtures.
    $env:KIMI_CODE_HOME = Join-Path $tempRoot "kimi-home-default"

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
    # El manifiesto de mentira vive en una variable y no en la fixture: mas
    # abajo hay otra fixture que lo necesita, y leerlo de la primera ataba las
    # dos (la primera lo borra cuando termina de usarlo).
    $cargoTomlStub = @'
[package]
name = "harness-smoke"
version = "0.0.0"
edition = "2021"
'@
    Set-TextUtf8NoBom -Path (Join-Path $fixture "rust/Cargo.toml") -Value $cargoTomlStub

    $runningOnWindows = $env:OS -eq "Windows_NT"
    if ($runningOnWindows) {
        # (fake python block removed - Rust only)
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
        # (fake python block removed - Rust only)
        $fakeCargo = @'
#!/bin/sh
printf '%s\n' "$*" > "$PWD/cargo-args.txt"
mkdir -p "$CARGO_TARGET_DIR/release"
printf 'fake harness\n' > "$CARGO_TARGET_DIR/release/harness.exe"
exit 0
'@
        $cargoPath = Join-Path $fakeBin "cargo"
        Set-TextUtf8NoBom -Path $cargoPath -Value $fakeCargo
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
    $cargoArgs = Get-Utf8Text -Path (Join-Path $fixture "rust/cargo-args.txt")
    Assert-True ($cargoArgs -match "build --release --locked") "Cargo was not invoked with build --release --locked."

    # Feature #14 / AC-12 + AC-13 (paridad con el bloque del smoke sh): UPDATING
    # no puede escribir sobre el binario vivo. La segunda instalacion tiene que
    # reemplazar harness.exe via temporal + move y no dejar temporales colgados.
    $secondCargo = if ($runningOnWindows) {
        @'
@echo off
echo %*> "%CD%\cargo-args.txt"
if not exist "%CARGO_TARGET_DIR%\release" mkdir "%CARGO_TARGET_DIR%\release"
echo fake harness v2> "%CARGO_TARGET_DIR%\release\harness.exe"
exit /b 0
'@
    }
    else {
        @'
#!/bin/sh
printf '%s\n' "$*" > "$PWD/cargo-args.txt"
mkdir -p "$CARGO_TARGET_DIR/release"
printf 'fake harness v2\n' > "$CARGO_TARGET_DIR/release/harness.exe"
exit 0
'@
    }
    if ($runningOnWindows) {
        Set-Content -LiteralPath (Join-Path $fakeBin "cargo.cmd") -Value $secondCargo -Encoding Ascii
    }
    else {
        $cargoPath = Join-Path $fakeBin "cargo"
        Set-TextUtf8NoBom -Path $cargoPath -Value $secondCargo
        & chmod +x $cargoPath
    }
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
    $reinstalled = (Get-Utf8Text -Path (Join-Path $fixture "harness.exe"))
    Assert-True ($reinstalled -match "fake harness v2") "Re-running the installer did not replace harness.exe with the freshly built one."
    $leftovers = @(Get-ChildItem -LiteralPath $fixture -Force -Filter ".harness.exe.*" -ErrorAction SilentlyContinue)
    # El mensaje se arma SIEMPRE, incluso cuando la asercion pasa. En PowerShell
    # 5.1 con Set-StrictMode -Version Latest, pedirle .Name a un array VACIO
    # tira PropertyNotFoundStrict: el smoke moria justo cuando no habia nada
    # que reportar. El pipe devuelve vacio sin quejarse en las dos versiones.
    Assert-True ($leftovers.Count -eq 0) "The atomic install left temporary files behind: $(($leftovers | ForEach-Object { $_.Name }) -join ', ')"
    Assert-True (Test-Path -LiteralPath (Join-Path $fixture ".codex/hooks.json")) "Codex hooks were not generated."
    Assert-True (Test-Path -LiteralPath (Join-Path $fixture "bin/harness-hook.ps1")) "PowerShell hook runtime was not generated."
    Assert-True (Test-Path -LiteralPath (Join-Path $fixture ".gemini/commands/harness/check.toml")) "Gemini check command was not generated."
    Assert-True ((Get-Utf8Text -Path (Join-Path $fixture ".harness_layout")).Trim() -eq "root") "Root layout marker is incorrect."
    Get-Utf8Text -Path (Join-Path $fixture ".codex/hooks.json") | ConvertFrom-Json | Out-Null
    Get-Utf8Text -Path (Join-Path $fixture ".gemini/settings.json") | ConvertFrom-Json | Out-Null

    # Hasta aca el harness.exe de la fixture es el que escribio el cargo falso:
    # eso es lo que hace verificable la instalacion atomica. De aca en adelante
    # el smoke le pide trabajo REAL al CLI (`prd add`, `prd tree`, `start`), asi
    # que se siembra el binario de verdad, igual que copy_fixture en el smoke sh.
    Copy-Item -LiteralPath $prebuilt -Destination (Join-Path $fixture "harness.exe") -Force
    # Y se va el `rust/` de mentira: existe solo para que el cargo falso tenga
    # un manifiesto que "compilar". Si queda, la proxima corrida del instalador
    # encuentra un cargo REAL en la maquina, intenta compilar un paquete sin
    # fuentes y falla. Las fixtures del smoke de sh no llevan `rust/` por la
    # misma razon: sin manifiesto, el instalador conserva el binario que ya esta.
    Remove-Item -Recurse -Force -LiteralPath (Join-Path $fixture "rust")

    # SDD: la constitution es un required asset y el instalador la siembra en el
    # docs/ de la RAIZ (en root layout, RAIZ == fixture). Paridad con el smoke sh;
    # la ejecucion real en Windows queda pendiente de entorno (como en feature #1).
    $installerText = Get-Utf8Text -Path (Join-Path $fixture "setup_harness.ps1")
    Assert-True ($installerText -match '"docs/constitution\.md"') "Installer does not declare docs/constitution.md as a required asset."
    Assert-True (Test-Path -LiteralPath (Join-Path $fixture "templates/docs/constitution.md")) "Constitution template asset is missing from the distribution."
    Assert-True (Test-Path -LiteralPath (Join-Path $fixture "docs/constitution.md")) "Constitution was not seeded by the installer."

    # Feature #6 / AC-10 + AC-11: la superficie sembrada describe el ritual de
    # aprobacion (mostrar el spec + preguntar + approve-spec --yes) y ya no la
    # edicion manual de `Estado:`. Paridad con el bloque AC-10 del smoke sh.
    $constitutionText = Get-Utf8Text -Path (Join-Path $fixture "docs/constitution.md")
    Assert-True ($constitutionText -match 'approve-spec --yes') "Seeded constitution does not describe the approve-spec flow."
    Assert-True (-not ($constitutionText -match 'auto-aprobar')) "Seeded constitution still carries the old manual-approval wording."
    $checkText = Get-Utf8Text -Path (Join-Path $fixture "harness_check.sh")
    Assert-True ($checkText -match 'approve-spec --yes') "Seeded harness_check.sh does not mention approve-spec."
    $implementerText = Get-Utf8Text -Path (Join-Path $fixture "roles/implementer.md")
    Assert-True ($implementerText -match 'approve-spec --yes') "Seeded implementer role does not describe the approval ritual."

    # Feature #4 / AC-2: en layout root los tres docs del arnes se siembran en el
    # docs/ de la RAIZ (que aqui es el propio fixture). Feature #11: + la guia.
    foreach ($harnessDoc in @("architecture.md", "conventions.md", "verification.md", "kimi-cli-uso-eficiente.md", "prd/COMO-ESCRIBIR-UN-PRD.md", "lecciones/COMO-ESCRIBIR-UNA-LECCION.md")) {
        Assert-True (Test-Path -LiteralPath (Join-Path $fixture "docs/$harnessDoc")) "Harness doc $harnessDoc was not seeded into the root docs/."
    }
    # Feature #19 / AC-1: el perfil se siembra VACIO en el docs/ de la RAIZ, y
    # AC-12: sin entradas no se inyecta bloque en ninguna superficie.
    $perfilPath = Join-Path $fixture "docs/perfil-usuario.md"
    Assert-True (Test-Path -LiteralPath $perfilPath) "perfil-usuario.md was not seeded into the root docs/."
    $perfilText = Get-Utf8Text -Path $perfilPath
    Assert-True ($perfilText -match '^# Perfil de usuario') "The seeded profile is missing its header."
    Assert-True (-not ($perfilText -match '(?m)^- ')) "The installer seeded profile entries; it must start empty."
    foreach ($perfilSurface in @("CLAUDE.md", "AGENTS.md", "GEMINI.md", "LLM.md")) {
        $surfaceText = Get-Utf8Text -Path (Join-Path $fixture $perfilSurface)
        Assert-True (-not ($surfaceText -match 'harness:perfil:inicio')) "$perfilSurface has a profile block while the profile is empty."
    }

    # Feature #17 / AC-1 + AC-14: la carpeta nace SOLO con la guia, y la guia trae
    # el orden de preferencia y la lista de que NO capturar (paridad con el smoke sh).
    $leccionFiles = @(Get-ChildItem -LiteralPath (Join-Path $fixture "docs/lecciones") -Filter "*.md" -File |
        Where-Object { $_.Name -ne "COMO-ESCRIBIR-UNA-LECCION.md" })
    Assert-True ($leccionFiles.Count -eq 0) "The installer seeded lessons; it must seed only the guide."
    $leccionGuideText = Get-Utf8Text -Path (Join-Path $fixture "docs/lecciones/COMO-ESCRIBIR-UNA-LECCION.md")
    Assert-True ($leccionGuideText -match 'primero patchear, crear al final') "The lessons guide is missing the preference order."
    Assert-True ($leccionGuideText -match '## El nombre tiene que ser de CLASE') "The lessons guide is missing the class-name rule."
    Assert-True ($leccionGuideText -match '## Que NO capturar') "The lessons guide is missing the do-not-capture list."
    foreach ($regla in @('Fallas dependientes del entorno', 'Afirmaciones negativas sobre herramientas', 'Errores transitorios', 'Narrativas de una tarea unica', 'Fracasos no resueltos')) {
        Assert-True ($leccionGuideText -match [regex]::Escape($regla)) "The lessons guide does not list '$regla'."
    }
    # Feature #17 / AC-17: la superficie instalada explica el comando y el gate.
    $agentsSurface = Get-Utf8Text -Path (Join-Path $fixture "AGENTS.md")
    Assert-True ($agentsSurface -match 'docs/lecciones/') "The installed AGENTS.md does not link docs/lecciones/."
    Assert-True ($agentsSurface -match 'require_leccion') "The installed AGENTS.md does not mention the require_leccion rule."
    # Feature #11 (companion KimiDotfiles): .kimiignore/.kimirules se siembran en
    # la RAIZ (paridad con KIMI_DOTFILES del smoke sh).
    foreach ($kimiDotfile in @(".kimiignore", ".kimirules")) {
        Assert-True (Test-Path -LiteralPath (Join-Path $fixture $kimiDotfile)) "Kimi dotfile $kimiDotfile was not seeded into the project root."
    }
    # Feature #5 / AC-2: las planillas maestras PRD y SDD se siembran en docs/prd/.
    Assert-True (Test-Path -LiteralPath (Join-Path $fixture "docs/prd/PRD-master.md")) "PRD-master.md was not seeded into docs/prd/."
    Assert-True (Test-Path -LiteralPath (Join-Path $fixture "docs/prd/SDD-master.md")) "SDD-master.md was not seeded into docs/prd/."
    # Feature #12 / AC-5: la guia del metodo PRD acompana a las planillas.
    Assert-True (Test-Path -LiteralPath (Join-Path $fixture "docs/prd/COMO-ESCRIBIR-UN-PRD.md")) "COMO-ESCRIBIR-UN-PRD.md was not seeded into docs/prd/."
    # Feature #12 / AC-4: la guia trae el metodo (historia, tamano, sin codigo final).
    $prdGuideText = Get-Utf8Text -Path (Join-Path $fixture "docs/prd/COMO-ESCRIBIR-UN-PRD.md")
    Assert-True ($prdGuideText -match '## 2\. Todo empieza con una historia') "The PRD guide is missing the story section."
    Assert-True ($prdGuideText -match '## 3\. El tamano lo decide el cambio') "The PRD guide is missing the sizing table."
    Assert-True ($prdGuideText -match 'NUNCA CONTIENE') "The PRD guide does not state what a PRD never contains."
    # Feature #5 / AC-7 + AC-8 + Feature #12 / AC-1..AC-3: las planillas traen las
    # secciones que las hacen utiles, ya con la anatomia del metodo.
    $prdText = Get-Utf8Text -Path (Join-Path $fixture "docs/prd/PRD-master.md")
    Assert-True ($prdText -match '## 2\. La historia') "PRD-master.md is missing the story section."
    Assert-True ($prdText -match '## 8\. Pseudo-codigo \(el acuerdo\)') "PRD-master.md is missing the pseudo-code agreement section."
    Assert-True ($prdText -match '## 10\. Hitos -> features') "PRD-master.md is missing the milestones-to-features table."
    Assert-True ($prdText -match 'harness_cli add') "PRD-master.md does not link milestones to the backlog command."
    $sddText = Get-Utf8Text -Path (Join-Path $fixture "docs/prd/SDD-master.md")
    Assert-True ($sddText -match '## 4\. Decisiones tecnicas') "SDD-master.md is missing the technical decisions section."
    Assert-True ($sddText -match 'docs/architecture\.md') "SDD-master.md does not distinguish itself from docs/architecture.md."

    # Feature #13 / AC-11: el maestro declara donde se cuelgan los PRDs anidados
    # y como se cierra el ciclo (bitacora del cierre).
    Assert-True ($prdText -match '## PRDs anidados') "PRD-master.md is missing the nested-PRD section."
    Assert-True ($prdText -match '## Bitacora') "PRD-master.md is missing the close log section."
    Assert-True ($prdText -match '--prd <ruta>') "PRD-master.md does not link milestones to their nested PRD."
    # Feature #13 / AC-10: la guia documenta los comandos reales del arbol.
    Assert-True ($prdGuideText -match 'prd add --name cobranza') "The PRD guide does not document 'prd add'."
    Assert-True ($prdGuideText -match 'harness_cli prd tree') "The PRD guide does not document 'prd tree'."
    Assert-True ($prdGuideText -match 'PRD-cobranza-mora\.md') "The PRD guide does not show the nested folder layout."

    # Feature #13 / AC-1 + AC-4 + AC-7: el arbol de PRDs anidados de punta a
    # punta con el binario ya sembrado (paridad con el bloque PRD_E2E del sh).
    & (Join-Path $fixture "harness_cli.ps1") prd add --name cobranza | Out-Null
    & (Join-Path $fixture "harness_cli.ps1") prd add --name mora --parent cobranza | Out-Null
    Assert-True (Test-Path -LiteralPath (Join-Path $fixture "docs/prd/cobranza/PRD-cobranza.md")) "prd add did not create the child PRD folder."
    Assert-True (Test-Path -LiteralPath (Join-Path $fixture "docs/prd/cobranza/mora/PRD-cobranza-mora.md")) "prd add did not nest the grandchild PRD."
    $childText = Get-Utf8Text -Path (Join-Path $fixture "docs/prd/cobranza/mora/PRD-cobranza-mora.md")
    Assert-True ($childText -match '(?m)^Padre: cobranza') "The nested PRD does not declare its parent."
    Assert-True ($childText -match '## 10\. Hitos -> features') "The nested PRD is missing the milestones table."
    Assert-True ((Get-Utf8Text -Path (Join-Path $fixture "docs/prd/PRD-master.md")) -match '\| cobranza \| \[cobranza/PRD-cobranza\.md\]') "The master PRD does not link its child."
    $treeOut = (& (Join-Path $fixture "harness_cli.ps1") prd tree | Out-String)
    Assert-True ($treeOut -match 'PRD-cobranza-mora') "prd tree did not draw the nested PRD."
    # Feature #13 / AC-5 + AC-6: la cadena PRD hoja -> feature -> spec.
    & (Join-Path $fixture "harness_cli.ps1") add --name avisar_mora --service cobranza --acceptance "llega el aviso" --prd mora | Out-Null
    & (Join-Path $fixture "harness_cli.ps1") start --feature 1 | Out-Null
    Assert-True ((Get-Utf8Text -Path (Join-Path $fixture "docs/spec-feature-1-avisar-mora.md")) -match '(?m)^PRD: docs/prd/cobranza/mora/PRD-cobranza-mora\.md') "The generated spec does not cite its source PRD."

    # Feature #11 / AC-4: la superficie que genera el ps1 referencia la guia de
    # uso eficiente de Kimi CLI (paridad con el grep del smoke sh).
    $agentsText = Get-Utf8Text -Path (Join-Path $fixture "AGENTS.md")
    Assert-True ($agentsText -match 'kimi-cli-uso-eficiente') "Seeded AGENTS.md does not reference the efficient Kimi CLI usage guide."
    # Feature #12 / AC-8: y tambien el metodo para escribir PRDs.
    Assert-True ($agentsText -match 'COMO-ESCRIBIR-UN-PRD') "Seeded AGENTS.md does not reference the PRD writing method."
    # Feature #13 / AC-12: y el arbol de PRDs anidados con sus comandos.
    Assert-True ($agentsText -match 'prd add --name') "Seeded AGENTS.md does not document the nested-PRD commands."

    # Feature #4 / AC-4 (reinstall): comparten carpeta con la documentacion del
    # equipo, asi que se siembran solo-si-faltan y un reinstall NO los pisa.
    $docsSentinel = "SENTINEL-DOCS-ARNES-NO-PISA-PS"
    Add-Content -LiteralPath (Join-Path $fixture "docs/conventions.md") -Value "<!-- $docsSentinel -->"
    # Feature #5 / AC-3: el PRD del proyecto es del USUARIO; el reinstall no lo pisa.
    $prdSentinel = "SENTINEL-PRD-NO-PISA-PS"
    Add-Content -LiteralPath (Join-Path $fixture "docs/prd/PRD-master.md") -Value "<!-- $prdSentinel -->"
    & (Join-Path $fixture "setup_harness.ps1") `
        -Root -NoGraphify -NoGraphifySkills -NoAntigravity -CargoTargetDir $cargoTarget
    Assert-True ((Get-Utf8Text -Path (Join-Path $fixture "docs/conventions.md")) -match $docsSentinel) "Reinstall overwrote a harness doc already present in the root docs/."
    Assert-True ((Get-Utf8Text -Path (Join-Path $fixture "docs/prd/PRD-master.md")) -match $prdSentinel) "Reinstall overwrote the project's PRD."

    # Feature #4 / AC-6: los artefactos de feature comparten carpeta con los docs
    # generados y el reset NO puede llevarselos por delante.
    Set-TextUtf8NoBom -Path (Join-Path $fixture "docs/spec-feature-1-demo.md") -Value "# spec"
    Set-TextUtf8NoBom -Path (Join-Path $fixture "docs/plan-feature-1-demo.md") -Value "# plan"

    & (Join-Path $fixture "setup_harness.ps1") `
        -Root -NoGraphify -NoGraphifySkills -NoAntigravity -Reset
    Assert-True (-not (Test-Path -LiteralPath (Join-Path $fixture ".harness_layout"))) "Reset did not remove the layout marker."
    Assert-True (Test-Path -LiteralPath (Join-Path $fixture "docs/constitution.md")) "Reset removed the user's constitution."
    foreach ($artifact in @("spec-feature-1-demo.md", "plan-feature-1-demo.md")) {
        Assert-True (Test-Path -LiteralPath (Join-Path $fixture "docs/$artifact")) "Reset removed the feature artifact $artifact."
    }
    foreach ($harnessDoc in @("architecture.md", "conventions.md", "verification.md", "kimi-cli-uso-eficiente.md", "prd/COMO-ESCRIBIR-UN-PRD.md")) {
        Assert-True (-not (Test-Path -LiteralPath (Join-Path $fixture "docs/$harnessDoc"))) "Reset did not clean the generated doc $harnessDoc."
    }
    # Feature #11 (companion KimiDotfiles): los dotfiles son documentos del
    # USUARIO y sobreviven al reset (mismo criterio que PRD/SDD).
    foreach ($kimiDotfile in @(".kimiignore", ".kimirules")) {
        Assert-True (Test-Path -LiteralPath (Join-Path $fixture $kimiDotfile)) "Reset removed the user's $kimiDotfile."
    }
    # Feature #5 / AC-4: las planillas maestras NO son superficie generada y
    # sobreviven al reset con el contenido que escribio el usuario.
    Assert-True ((Get-Utf8Text -Path (Join-Path $fixture "docs/prd/PRD-master.md")) -match $prdSentinel) "Reset removed or overwrote the project's PRD."
    Assert-True (Test-Path -LiteralPath (Join-Path $fixture "docs/prd/SDD-master.md")) "Reset removed the project's SDD master."

    # --- Feature #4 / AC-1 + AC-3 + AC-4: layout subdir y migracion -----------
    # El arnes vive en <raiz>/harness_process y los docs deben quedar en
    # <raiz>/docs. Los que ya estaban en la ubicacion vieja se MUEVEN si faltan
    # en la raiz; el que el equipo ya tiene en la raiz NO se pisa.
    $subdirRoot = Join-Path $tempRoot "subdir-layout"
    $subdirHarness = Join-Path $subdirRoot "harness_process"
    Copy-Fixture -Target $subdirHarness
    New-Item -ItemType Directory -Path (Join-Path $subdirHarness "docs") -Force | Out-Null
    New-Item -ItemType Directory -Path (Join-Path $subdirRoot "docs") -Force | Out-Null
    Set-TextUtf8NoBom -Path (Join-Path $subdirHarness "docs/architecture.md") -Value "VIEJO-ARCHITECTURE"
    Set-TextUtf8NoBom -Path (Join-Path $subdirHarness "docs/verification.md") -Value "VIEJO-VERIFICATION"
    Set-TextUtf8NoBom -Path (Join-Path $subdirHarness "docs/conventions.md") -Value "VIEJO-CONVENTIONS"
    $teamSentinel = "SENTINEL-CONVENTIONS-DEL-EQUIPO-PS"
    Set-TextUtf8NoBom -Path (Join-Path $subdirRoot "docs/conventions.md") -Value $teamSentinel
    & (Join-Path $subdirHarness "setup_harness.ps1") `
        -NoGraphify -NoGraphifySkills -NoAntigravity -CargoTargetDir $cargoTarget

    # AC-1: destino raiz, y la subcarpeta del arnes ya no tiene esos docs.
    Assert-True (Test-Path -LiteralPath (Join-Path $subdirRoot "docs/constitution.md")) "Constitution was not seeded into the multi-repo root docs/."
    # Feature #5 / AC-1: las planillas maestras van al docs/prd/ de la RAIZ, no a
    # la subcarpeta del arnes.
    Assert-True (Test-Path -LiteralPath (Join-Path $subdirRoot "docs/prd/PRD-master.md")) "PRD-master.md was not seeded into the multi-repo root docs/prd/."
    Assert-True (Test-Path -LiteralPath (Join-Path $subdirRoot "docs/prd/SDD-master.md")) "SDD-master.md was not seeded into the multi-repo root docs/prd/."
    Assert-True (-not (Test-Path -LiteralPath (Join-Path $subdirHarness "docs/prd"))) "docs/prd/ was created inside the harness subfolder."
    # Feature #11 (companion KimiDotfiles): los dotfiles se siembran en la RAIZ
    # multi-repo, no en la subcarpeta del arnes.
    foreach ($kimiDotfile in @(".kimiignore", ".kimirules")) {
        Assert-True (Test-Path -LiteralPath (Join-Path $subdirRoot $kimiDotfile)) "Kimi dotfile $kimiDotfile was not seeded into the multi-repo root."
        Assert-True (-not (Test-Path -LiteralPath (Join-Path $subdirHarness $kimiDotfile))) "Kimi dotfile $kimiDotfile was created inside the harness subfolder."
    }
    # AC-3: se movio el contenido viejo (no se regenero desde la plantilla).
    Assert-True ((Get-Utf8Text -Path (Join-Path $subdirRoot "docs/architecture.md")).Trim() -eq "VIEJO-ARCHITECTURE") "architecture.md was not migrated with its content."
    Assert-True ((Get-Utf8Text -Path (Join-Path $subdirRoot "docs/verification.md")).Trim() -eq "VIEJO-VERIFICATION") "verification.md was not migrated with its content."
    Assert-True (-not (Test-Path -LiteralPath (Join-Path $subdirHarness "docs/architecture.md"))) "architecture.md is still in the harness subfolder."
    Assert-True (-not (Test-Path -LiteralPath (Join-Path $subdirHarness "docs/verification.md"))) "verification.md is still in the harness subfolder."
    # AC-4: el doc del equipo queda intacto y la copia vieja se conserva.
    Assert-True ((Get-Utf8Text -Path (Join-Path $subdirRoot "docs/conventions.md")) -match $teamSentinel) "Migration overwrote the team's conventions.md in the root docs/."
    Assert-True ((Get-Utf8Text -Path (Join-Path $subdirHarness "docs/conventions.md")).Trim() -eq "VIEJO-CONVENTIONS") "Migration removed the old copy instead of keeping it."

    # --- Feature #7: gate de espejo de roles + resolucion robusta de raiz ------
    # Paridad con los bloques nuevos de tests/setup_smoke.sh. El gate vive en
    # harness_check.sh (bash); aqui se valida (1) que el script sembrado trae el
    # gate y el guardrail, (2) la MISMA extraccion de cuerpos portada a
    # PowerShell contra los espejos que genera el instalador ps1 (cero falsos
    # positivos, AC-2/AC-11), (3) que esa extraccion detecta un espejo stale
    # inyectado, y (4) si hay bash disponible, el harness_check.sh REAL sobre un
    # checkout fuente simulado (AC-6). La ejecucion completa en Windows queda
    # pendiente de entorno, como en las features #1/#4/#5/#6.

    function Get-AgentBody {
        param([string]$Path)
        $lines = [IO.File]::ReadAllLines($Path, [Text.UTF8Encoding]::new($false))
        $fm = 0
        $inBody = $false
        $body = New-Object System.Collections.Generic.List[string]
        foreach ($line in $lines) {
            if (-not $inBody) {
                if ($line -match '^---\s*$') {
                    $fm++
                    if ($fm -eq 2) { $inBody = $true }
                }
                continue
            }
            $body.Add($line)
        }
        while ($body.Count -gt 0 -and $body[0].Trim() -eq '') { $body.RemoveAt(0) }
        return ($body -join "`n").TrimEnd()
    }

    function Get-CodexBody {
        param([string]$Path)
        $lines = [IO.File]::ReadAllLines($Path, [Text.UTF8Encoding]::new($false))
        $inBlock = $false
        $body = New-Object System.Collections.Generic.List[string]
        foreach ($line in $lines) {
            if (-not $inBlock) {
                if ($line -match '^developer_instructions\s*=') { $inBlock = $true }
                continue
            }
            if ($line -eq "'''") { break }
            $body.Add($line)
        }
        while ($body.Count -gt 0 -and $body[0].Trim() -eq '') { $body.RemoveAt(0) }
        return ($body -join "`n").TrimEnd()
    }

    function Get-NormalizedText {
        param([string]$Path)
        return ((Get-Utf8Text -Path $Path) -replace "`r`n", "`n").TrimEnd()
    }

    $checkRobust = Join-Path $tempRoot "check-robust-ps"
    Copy-Fixture -Target $checkRobust
    New-Item -ItemType Directory -Path (Join-Path $checkRobust "rust") -Force | Out-Null
    Set-TextUtf8NoBom -Path (Join-Path $checkRobust "rust/Cargo.toml") -Value $cargoTomlStub
    $env:PATH = $fakeBin + [IO.Path]::PathSeparator + $env:PATH
    try {
        & (Join-Path $checkRobust "setup_harness.ps1") `
            -Root -NoGraphify -NoGraphifySkills -NoAntigravity `
            -CargoTargetDir (Join-Path $checkRobust "cargo-target")
    }
    finally {
        $env:PATH = $oldPath
    }

    # (1) El harness_check.sh sembrado trae el gate de espejo y el guardrail.
    $seededCheck = Get-Utf8Text -Path (Join-Path $checkRobust "harness_check.sh")
    Assert-True ($seededCheck -match 'Espejo desincronizado') "Seeded harness_check.sh does not carry the role-mirror gate."
    Assert-True ($seededCheck -match 'extract_agent_body') "Seeded harness_check.sh does not carry the frontmatter body extractor."
    Assert-True ($seededCheck -match 'Checkout fuente del arnes detectado') "Seeded harness_check.sh does not carry the source-checkout guardrail."
    Assert-True ($seededCheck -match 'Divergencia roles/') "Seeded harness_check.sh does not carry the roles/ vs templates/roles/ sub-gate."

    # (2) AC-2/AC-3/AC-11: los espejos generados por el instalador ps1 llevan el
    # MISMO cuerpo que roles/<rol>.md en los tres formatos.
    foreach ($role in @("leader", "implementer", "reviewer")) {
        $roleBody = Get-NormalizedText -Path (Join-Path $checkRobust "roles/$role.md")
        $claudeBody = Get-AgentBody -Path (Join-Path $checkRobust ".claude/agents/$role.md")
        Assert-True ($claudeBody -eq $roleBody) "Mirror .claude/agents/$role.md is out of sync with roles/$role.md."
        $geminiBody = Get-AgentBody -Path (Join-Path $checkRobust ".gemini/agents/$role.md")
        Assert-True ($geminiBody -eq $roleBody) "Mirror .gemini/agents/$role.md is out of sync with roles/$role.md."
        $codexBody = Get-CodexBody -Path (Join-Path $checkRobust ".codex/agents/$role.toml")
        Assert-True ($codexBody -eq $roleBody) "Mirror .codex/agents/$role.toml is out of sync with roles/$role.md."
    }

    # AC-4: roles/ instalado equivale a templates/roles/ bajo alguna de las dos
    # expansiones de __HREL__ (root => prefijo vacio).
    $hrelPrefix = (Split-Path -Leaf $checkRobust) + "/"
    foreach ($roleFile in @("leader", "implementer", "reviewer", "README")) {
        $srcBody = Get-NormalizedText -Path (Join-Path $checkRobust "roles/$roleFile.md")
        $tplRaw = Get-NormalizedText -Path (Join-Path $checkRobust "templates/roles/$roleFile.md")
        $expSubdir = $tplRaw.Replace('__HREL__', $hrelPrefix)
        $expFlat = $tplRaw.Replace('__HREL__', '')
        Assert-True (($srcBody -eq $expSubdir) -or ($srcBody -eq $expFlat)) "roles/$roleFile.md diverges from templates/roles/$roleFile.md modulo __HREL__."
    }

    # (3) AC-1/AC-3: la misma extraccion DETECTA un espejo stale inyectado.
    $staleMirror = Join-Path $checkRobust ".claude/agents/implementer.md"
    $mirrorBackup = [IO.File]::ReadAllText($staleMirror)
    Add-Content -LiteralPath $staleMirror -Value "PROTOCOLO VIEJO INYECTADO"
    $roleBody = Get-NormalizedText -Path (Join-Path $checkRobust "roles/implementer.md")
    Assert-True ((Get-AgentBody -Path $staleMirror) -ne $roleBody) "Injected stale mirror was not detected by the gate extraction logic."
    [IO.File]::WriteAllText($staleMirror, $mirrorBackup)
    Assert-True ((Get-AgentBody -Path $staleMirror) -eq $roleBody) "Restored mirror should match roles/implementer.md again."

    # (4) AC-6: harness_check.sh REAL sobre un checkout fuente simulado (marker
    # subdir + senales de fuente, padre sin huella). Requiere bash (Git Bash en
    # Windows); sin bash se deja constancia y se omite.
    $bashCmd = Get-Command bash -ErrorAction SilentlyContinue
    if ($bashCmd) {
        $sourceParent = Join-Path $tempRoot "source-sim"
        $sourceClone = Join-Path $sourceParent "harness_process"
        New-Item -ItemType Directory -Path $sourceClone -Force | Out-Null
        foreach ($f in @("harness_check.sh", "commit_guard.sh", "CHECKPOINTS.md")) {
            Copy-Item -LiteralPath (Join-Path $repoRoot $f) -Destination (Join-Path $sourceClone $f)
        }
        Copy-Item -LiteralPath (Join-Path $repoRoot "templates") -Destination $sourceClone -Recurse
        Copy-Item -LiteralPath (Join-Path $repoRoot "roles") -Destination $sourceClone -Recurse
        New-Item -ItemType Directory -Path (Join-Path $sourceClone "rust") -Force | Out-Null
        New-Item -ItemType Directory -Path (Join-Path $sourceClone "docs") -Force | Out-Null
        New-Item -ItemType Directory -Path (Join-Path $sourceClone "progress") -Force | Out-Null
        New-Item -ItemType Directory -Path (Join-Path $sourceClone ".claude/agents") -Force | Out-Null
        Copy-Item -LiteralPath (Join-Path $repoRoot "rust/Cargo.toml") -Destination (Join-Path $sourceClone "rust/Cargo.toml")
        Copy-Item -LiteralPath (Join-Path $repoRoot "docs/constitution.md") -Destination (Join-Path $sourceClone "docs/constitution.md")
        Copy-Item -Path (Join-Path $repoRoot ".claude/agents/*.md") -Destination (Join-Path $sourceClone ".claude/agents")
        Copy-Item -LiteralPath (Join-Path $repoRoot "templates/progress/current.md") -Destination (Join-Path $sourceClone "progress/current.md")
        # Sin feature_list.json: el check omite los subcomandos del binario (el
        # fixture ps1 solo tiene el harness.exe fake del cargo simulado).
        Set-Content -LiteralPath (Join-Path $sourceClone ".harness_layout") -Value "subdir" -Encoding Ascii

        $oldRepoRootEnv = $env:HARNESS_REPO_ROOT
        $oldClaudeProjectDir = $env:CLAUDE_PROJECT_DIR
        $env:HARNESS_REPO_ROOT = $null
        $env:CLAUDE_PROJECT_DIR = $null
        Push-Location $sourceClone
        try {
            # $null en el pipe cierra stdin (commit_guard.sh hace cat de stdin).
            $checkOutput = Invoke-CapturandoStderr { $null | & $bashCmd.Source "harness_check.sh" 2>&1 }
            $checkExit = $LASTEXITCODE
        }
        finally {
            Pop-Location
            $env:HARNESS_REPO_ROOT = $oldRepoRootEnv
            $env:CLAUDE_PROJECT_DIR = $oldClaudeProjectDir
        }
        Assert-True ($checkExit -eq 0) "harness_check.sh should pass on the simulated source checkout (exit=$checkExit): $checkOutput"
        Assert-True ($checkOutput -match 'Checkout fuente del arnes detectado') "Source-checkout guardrail did not emit its informative notice."
        Assert-True (-not ($checkOutput -match 'Falta docs/constitution\.md')) "harness_check.sh resolved to the parent (false constitution failure)."
    }
    else {
        Write-Host "[INFO] bash not available: skipping the live harness_check.sh run on the simulated source checkout (static parity only)."
    }

    # --- Feature #10: layout inferido por huella cuando falta el marker -------
    # Paridad con el bloque "Feature #10" de tests/setup_smoke.sh: sin
    # .harness_layout (el estado en que queda toda instalacion que hizo
    # 'git pull' tras la feature #7) la raiz se infiere del padre con huella.

    # (1) Los CUATRO scripts sembrados por el instalador traen la regla nueva.
    foreach ($seededScript in @("harness_check.sh", "harness_status.sh", "init.sh", "commit_guard.sh")) {
        $seeded = Get-Utf8Text -Path (Join-Path $checkRobust $seededScript)
        Assert-True ($seeded -match '\.harness_layout ausente') "Seeded $seededScript does not carry the marker-inference notice."
        Assert-True ($seeded -match 'harness_parent_footprint') "Seeded $seededScript does not carry the parent-footprint probe."
        Assert-True ($seeded -match 'elif \[ ! -f "\$harness_marker" \]') "Seeded $seededScript does not gate the inference on the marker being ABSENT."
        Assert-True ($seeded -match 'Checkout fuente del arnes detectado') "Seeded $seededScript lost the feature #7 source-checkout guardrail."
    }

    # (2) Ejecucion real con bash (Git Bash en Windows); sin bash se informa.
    if ($bashCmd) {
        function New-LostMarkerCase {
            param([string]$Name, [string]$Marker)
            $caseProj = Join-Path (Join-Path $tempRoot "lost-marker-ps") $Name
            $caseHarness = Join-Path $caseProj "harness_process"
            New-Item -ItemType Directory -Path $caseHarness -Force | Out-Null
            New-Item -ItemType Directory -Path (Join-Path $caseHarness "progress") -Force | Out-Null
            New-Item -ItemType Directory -Path (Join-Path $caseHarness "rust") -Force | Out-Null
            foreach ($f in @("harness_check.sh", "harness_status.sh", "init.sh", "commit_guard.sh", "CHECKPOINTS.md")) {
                Copy-Item -LiteralPath (Join-Path $repoRoot $f) -Destination (Join-Path $caseHarness $f)
            }
            Copy-Item -LiteralPath (Join-Path $repoRoot "templates") -Destination $caseHarness -Recurse
            Copy-Item -LiteralPath (Join-Path $repoRoot "roles") -Destination $caseHarness -Recurse
            Copy-Item -LiteralPath (Join-Path $repoRoot "rust/Cargo.toml") -Destination (Join-Path $caseHarness "rust/Cargo.toml")
            Copy-Item -LiteralPath (Join-Path $repoRoot "templates/progress/current.md") -Destination (Join-Path $caseHarness "progress/current.md")
            # Huella de instalacion en el PADRE (el proyecto), no en el arnes.
            New-Item -ItemType Directory -Path (Join-Path $caseProj "docs") -Force | Out-Null
            Copy-Item -LiteralPath (Join-Path $repoRoot "docs/constitution.md") -Destination (Join-Path $caseProj "docs/constitution.md")
            Set-Content -LiteralPath (Join-Path $caseProj "CLAUDE.md") -Value "# proyecto" -Encoding Ascii
            if ($Marker) {
                Set-Content -LiteralPath (Join-Path $caseHarness ".harness_layout") -Value $Marker -Encoding Ascii
            }
            return $caseHarness
        }

        function Invoke-HarnessCheck {
            param([string]$HarnessDir)
            $oldRepoRootEnv = $env:HARNESS_REPO_ROOT
            $oldClaudeProjectDir = $env:CLAUDE_PROJECT_DIR
            $env:HARNESS_REPO_ROOT = $null
            $env:CLAUDE_PROJECT_DIR = $null
            Push-Location $HarnessDir
            try {
                return (Invoke-CapturandoStderr { $null | & $bashCmd.Source "harness_check.sh" 2>&1 })
            }
            finally {
                Pop-Location
                $env:HARNESS_REPO_ROOT = $oldRepoRootEnv
                $env:CLAUDE_PROJECT_DIR = $oldClaudeProjectDir
            }
        }

        # (a) AC-1/AC-2: sin marker + huella en el padre -> raiz al PROYECTO. La
        # constitution vive en el proyecto: si la resolucion cayera en el arnes,
        # el check reportaria "Falta docs/constitution.md".
        $lostHarness = New-LostMarkerCase -Name "sin-marker" -Marker ""
        $lostOutput = Invoke-HarnessCheck -HarnessDir $lostHarness
        Assert-True ($lostOutput -match '\.harness_layout ausente: layout subdir inferido por la huella de instalacion del padre') "Missing marker did not trigger the subdir inference notice."
        Assert-True ($lostOutput -match 'para regenerar el marker') "Inference notice does not name the remedy (re-run the installer)."
        Assert-True (-not ($lostOutput -match 'Falta docs/constitution\.md')) "Resolution fell back to the harness dir instead of the project."

        # (c) AC-3: marker EXPLICITO 'root' con la misma huella -> sin
        # inferencia; la raiz es el arnes y ahi SI falta la constitution (esa
        # linea es justamente la evidencia de que resolvio local).
        $rootMarkerHarness = New-LostMarkerCase -Name "marker-root" -Marker "root"
        $rootMarkerOutput = Invoke-HarnessCheck -HarnessDir $rootMarkerHarness
        Assert-True (-not ($rootMarkerOutput -match '\.harness_layout ausente')) "An explicit 'root' marker must never go through the inference."
        Assert-True ($rootMarkerOutput -match 'Falta docs/constitution\.md') "Explicit 'root' marker did not resolve to the harness dir."
    }
    else {
        Write-Host "[INFO] bash not available: skipping the live marker-inference runs (static parity only)."
    }

    # --- Feature #8: Kimi Code CLI como backend (paridad con setup_smoke.sh) ---
    # (a) AC-9a/AC-10: espejos Kimi generados con frontmatter valido (allowlist
    # de tools por rol, decision usuario 2026-07-28) y cuerpo == roles/<rol>.md.
    foreach ($role in @("leader", "implementer", "reviewer")) {
        $kimiMirror = Join-Path $checkRobust ".kimi-code/agents/$role.md"
        Assert-True (Test-Path -LiteralPath $kimiMirror) "Kimi mirror $role.md was not generated."
        $kimiLines = [IO.File]::ReadAllLines($kimiMirror)
        Assert-True ($kimiLines[0] -eq "---") "Kimi mirror $role.md lacks YAML frontmatter."
        Assert-True (@($kimiLines | Where-Object { $_ -eq "name: $role" }).Count -eq 1) "Kimi mirror $role.md lacks name: in the frontmatter."
        Assert-True (@($kimiLines | Where-Object { $_ -match '^description: ' }).Count -ge 1) "Kimi mirror $role.md lacks description: in the frontmatter."
        $kimiBody = Get-AgentBody -Path $kimiMirror
        $kimiRoleBody = Get-NormalizedText -Path (Join-Path $checkRobust "roles/$role.md")
        Assert-True ($kimiBody -eq $kimiRoleBody) "Mirror .kimi-code/agents/$role.md is out of sync with roles/$role.md."
    }
    $kimiLeaderTools = @([IO.File]::ReadAllLines((Join-Path $checkRobust ".kimi-code/agents/leader.md")) | Where-Object { $_ -match '^tools: ' })[0]
    Assert-True ($kimiLeaderTools -eq "tools: Read, Grep, Glob, Bash") "Kimi leader allowlist must be read-only (Read, Grep, Glob, Bash)."
    $kimiImplTools = @([IO.File]::ReadAllLines((Join-Path $checkRobust ".kimi-code/agents/implementer.md")) | Where-Object { $_ -match '^tools: ' })[0]
    Assert-True ($kimiImplTools -match 'Edit' -and $kimiImplTools -match 'Write') "Kimi implementer allowlist must include Edit and Write."
    # El harness_check.sh sembrado trae el gate de espejo extendido a Kimi.
    $seededCheckKimi = Get-Utf8Text -Path (Join-Path $checkRobust "harness_check.sh")
    Assert-True ($seededCheckKimi -match '\.kimi-code/agents') "Seeded harness_check.sh does not cover the Kimi mirrors."
    # Launcher generado como los demas.
    Assert-True (Test-Path -LiteralPath (Join-Path $checkRobust "bin/harness-kimi.ps1")) "Kimi launcher bin/harness-kimi.ps1 was not generated."

    # (b) AC-9c/AC-9d/AC-9e/AC-10: bloque GLOBAL de hooks contra un
    # KIMI_CODE_HOME de fixture con un kimi falso (doctor OK), NUNCA el real.
    $kimiHomeOn = Join-Path $tempRoot "kimi-home-on"
    New-Item -ItemType Directory -Path (Join-Path $kimiHomeOn "bin") -Force | Out-Null
    if ($runningOnWindows) {
        $fakeKimi = @'
@echo off
exit /b 0
'@
        Set-Content -LiteralPath (Join-Path $kimiHomeOn "bin/kimi.cmd") -Value $fakeKimi -Encoding Ascii
    }
    else {
        $fakeKimi = @'
#!/bin/sh
exit 0
'@
        $fakeKimiPath = Join-Path $kimiHomeOn "bin/kimi"
        Set-TextUtf8NoBom -Path $fakeKimiPath -Value $fakeKimi
        & chmod +x $fakeKimiPath
    }
    $kimiSentinel = "SENTINEL-KIMI-USER-CONFIG-PS"
    $kimiConfigPath = Join-Path $kimiHomeOn "config.toml"
    $kimiUserConfig = @'
# __KIMI_SENTINEL__

[[hooks]]
event = "UserPromptSubmit"
command = "echo hook-del-usuario"
'@
    Set-TextUtf8NoBom -Path $kimiConfigPath -Value ($kimiUserConfig.Replace("__KIMI_SENTINEL__", $kimiSentinel))
    $kimiBeginMarker = "# >>> harness-process hooks >>>"
    $oldKimiHomeEnv = $env:KIMI_CODE_HOME
    $env:KIMI_CODE_HOME = $kimiHomeOn
    $env:PATH = $fakeBin + [IO.Path]::PathSeparator + $env:PATH
    try {
        & (Join-Path $checkRobust "setup_harness.ps1") `
            -Root -NoGraphify -NoGraphifySkills -NoAntigravity `
            -CargoTargetDir (Join-Path $checkRobust "cargo-target")
        $kimiConfigText = Get-Utf8Text -Path $kimiConfigPath
        Assert-True ($kimiConfigText -match [regex]::Escape($kimiSentinel)) "User content in the global Kimi config did not survive."
        Assert-True ($kimiConfigText -match 'hook-del-usuario') "The user's own hook in the global Kimi config did not survive."
        Assert-True (([regex]::Matches($kimiConfigText, [regex]::Escape($kimiBeginMarker))).Count -eq 1) "The harness block marker count must be exactly 1."
        Assert-True (([regex]::Matches($kimiConfigText, [regex]::Escape("[[hooks]]"))).Count -eq 4) "Expected the user hook plus the 3 harness hooks."
        Assert-True ($kimiConfigText -match 'event = "SessionStart"') "SessionStart hook missing from the harness block."
        Assert-True ($kimiConfigText -match 'matcher = "Edit\|Write"') "PostToolUse matcher Edit|Write missing from the harness block."
        Assert-True ($kimiConfigText -match 'event = "Stop"') "Stop hook missing from the harness block."
        Assert-True (-not ($kimiConfigText -match 'SessionEnd')) "SessionEnd must not be registered (it would double the Stop check)."

        # Re-instalar NO duplica el bloque (reemplazo idempotente entre marcadores).
        & (Join-Path $checkRobust "setup_harness.ps1") `
            -Root -NoGraphify -NoGraphifySkills -NoAntigravity `
            -CargoTargetDir (Join-Path $checkRobust "cargo-target")
        $kimiConfigText = Get-Utf8Text -Path $kimiConfigPath
        Assert-True (([regex]::Matches($kimiConfigText, [regex]::Escape($kimiBeginMarker))).Count -eq 1) "Reinstall duplicated the harness block."
        Assert-True (([regex]::Matches($kimiConfigText, [regex]::Escape("[[hooks]]"))).Count -eq 4) "Reinstall changed the hook count."

        # -Reset limpia el artefacto de proyecto pero NO toca el config global
        # (decision usuario 2026-07-28: es compartido entre proyectos).
        & (Join-Path $checkRobust "setup_harness.ps1") `
            -Root -NoGraphify -NoGraphifySkills -NoAntigravity -Reset
        Assert-True (-not (Test-Path -LiteralPath (Join-Path $checkRobust ".kimi-code/agents"))) "-Reset did not clean the project .kimi-code/agents."
        $kimiConfigAfterReset = Get-Utf8Text -Path $kimiConfigPath
        Assert-True ($kimiConfigAfterReset -eq $kimiConfigText) "-Reset must not touch the global Kimi hooks block."

        # -NoKimi: con Kimi detectable, el bloque global NO se escribe (los
        # artefactos de proyecto se regeneran igual).
        $kimiHomeFlag = Join-Path $tempRoot "kimi-home-flag"
        New-Item -ItemType Directory -Path (Join-Path $kimiHomeFlag "bin") -Force | Out-Null
        Get-ChildItem -LiteralPath (Join-Path $kimiHomeOn "bin") | ForEach-Object {
            Copy-Item -LiteralPath $_.FullName -Destination (Join-Path $kimiHomeFlag "bin")
        }
        $env:KIMI_CODE_HOME = $kimiHomeFlag
        & (Join-Path $checkRobust "setup_harness.ps1") `
            -Root -NoGraphify -NoGraphifySkills -NoAntigravity -NoKimi `
            -CargoTargetDir (Join-Path $checkRobust "cargo-target")
        Assert-True (-not (Test-Path -LiteralPath (Join-Path $kimiHomeFlag "config.toml"))) "-NoKimi must not write the global Kimi config."
        Assert-True (Test-Path -LiteralPath (Join-Path $checkRobust ".kimi-code/agents/leader.md")) "-NoKimi must still generate the project Kimi mirrors."
    }
    finally {
        $env:PATH = $oldPath
        $env:KIMI_CODE_HOME = $oldKimiHomeEnv
    }

    # -----------------------------------------------------------------------
    # Feature #15 (AC-1/AC-2/AC-3/AC-13): binding de Atlassian del instalador.
    # -----------------------------------------------------------------------
    $atlassianOff = Join-Path $tempRoot "atlassian-off"
    Copy-Fixture -Target $atlassianOff
    & (Join-Path $atlassianOff "setup_harness.ps1") `
        -Root -NoGraphify -NoGraphifySkills -NoAntigravity -NoKimi `
        -CargoTargetDir (Join-Path $atlassianOff "cargo-target")
    Assert-True (-not (Test-Path -LiteralPath (Join-Path $atlassianOff "atlassian.json"))) "Without flags no atlassian.json must be written (AC-3)."

    $atlassianOn = Join-Path $tempRoot "atlassian-on"
    Copy-Fixture -Target $atlassianOn
    & (Join-Path $atlassianOn "setup_harness.ps1") `
        -Root -NoGraphify -NoGraphifySkills -NoAntigravity -NoKimi `
        -AtlassianSite "calpil.atlassian.net" -JiraProject "ADR" -ConfluenceSpace "SD" `
        -CargoTargetDir (Join-Path $atlassianOn "cargo-target")
    $bindingPath = Join-Path $atlassianOn "atlassian.json"
    Assert-True (Test-Path -LiteralPath $bindingPath) "The installer must write atlassian.json with the flags (AC-1)."
    $binding = Get-Utf8Text -Path $bindingPath
    Assert-True ($binding -match '"project_key": "ADR"') "atlassian.json must carry the Jira project (AC-1)."
    Assert-True ($binding -match '"space_key": "SD"') "atlassian.json must carry the Confluence space (AC-1)."
    Assert-True ($binding -match '"feature": "Story"') "Story is the default issue type for a feature (OBS-6)."
    Assert-True ($binding -match '"blocked_flag": "Impediment"') "blocked maps to the Impediment flag (OBS-7)."

    Write-Host "[OK] PowerShell setup smoke: dry-run, root layout, hooks, shim, constitution seed, interactive spec approval surface (approve-spec), harness docs in root docs/ (seed, migration, no-overwrite; incl. efficient Kimi CLI usage guide linked from the surface), PRD/SDD master templates in docs/prd/, reset, role-mirror gate parity, source-checkout guardrail, subdir layout inferred from the parent footprint when the marker is missing (explicit 'root' marker never inferred), Kimi Code backend (mirrors, guarded global hooks block, -NoKimi, -Reset keeps global), and Atlassian binding (off without flags, written with flags, Story/Impediment defaults)."
}
finally {
    Remove-Item -LiteralPath $tempRoot -Recurse -Force -ErrorAction SilentlyContinue
}
