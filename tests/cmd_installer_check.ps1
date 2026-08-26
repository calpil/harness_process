#requires -Version 5.1
<#
.SYNOPSIS
Ejecuta el contrato runtime de setup_harness.cmd en Windows.

.DESCRIPTION
El check Bash homonimo cubre el launcher de forma estatica fuera de Windows.
Este smoke corre solo en Windows y usa cmd.exe de verdad: comprueba que el
wrapper abre el setup_harness.ps1 real, traduce flags Unix, conserva flags de
PowerShell y devuelve los errores del delegado sin alterarlos.
#>
[CmdletBinding()]
param()

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$tempRoot = Join-Path ([IO.Path]::GetTempPath()) ("harness-cmd-" + [Guid]::NewGuid().ToString("N"))

function Assert-True {
    param(
        [bool]$Condition,
        [string]$Message
    )
    if (-not $Condition) {
        throw $Message
    }
}

function Text-Of {
    param([object[]]$Value)
    return ($Value | Out-String).Trim()
}

function Invoke-CmdSetup {
    param(
        [string]$Directory,
        [string[]]$Arguments
    )
    $launcher = Join-Path $Directory "setup_harness.cmd"
    $output = & $env:ComSpec /d /c $launcher @Arguments 2>&1
    return [pscustomobject]@{
        ExitCode = $LASTEXITCODE
        Output = Text-Of -Value $output
    }
}

function New-DelegationSandbox {
    $path = Join-Path $tempRoot ("sandbox-" + [Guid]::NewGuid().ToString("N"))
    New-Item -ItemType Directory -Path $path -Force | Out-Null
    Copy-Item -LiteralPath (Join-Path $repoRoot "setup_harness.cmd") -Destination $path
    @'
param([Parameter(ValueFromRemainingArguments = $true)][string[]]$Rest)
$code = 0
foreach ($argument in $Rest) {
    if ($argument -eq "-Salir3") { $code = 3 }
}
"ARGS:" + ($Rest -join " ")
exit $code
'@ | Set-Content -LiteralPath (Join-Path $path "setup_harness.ps1") -Encoding Ascii
    return $path
}

try {
    Assert-True ($env:OS -eq "Windows_NT") "Este smoke exige Windows: no convierte la ausencia de cmd.exe en un verde."
    Assert-True (-not [string]::IsNullOrWhiteSpace($env:ComSpec)) "Windows no declaro COMSPEC."
    Assert-True (Test-Path -LiteralPath $env:ComSpec) "No existe cmd.exe en COMSPEC: $env:ComSpec"

    # AC-2: el wrapper real tiene que abrir el setup_harness.ps1 real. `--version`
    # no instala ni compila, pero ejercita el launcher y el parser del delegado.
    $real = Invoke-CmdSetup -Directory $repoRoot -Arguments @("--version")
    Assert-True ($real.ExitCode -eq 0) "El setup_harness.cmd real devolvio $($real.ExitCode): $($real.Output)"
    Assert-True ($real.Output -match "harness-process") "El launcher real no devolvio la version del PS1: $($real.Output)"

    # AC-3: las opciones Unix pasan a PascalCase y las nativas no se alteran.
    $sandbox = New-DelegationSandbox
    try {
        $translated = Invoke-CmdSetup -Directory $sandbox -Arguments @("--dry-run", "--no-subagents", "-Force")
        Assert-True ($translated.ExitCode -eq 0) "El sandbox de traduccion devolvio $($translated.ExitCode): $($translated.Output)"
        Assert-True ($translated.Output -match "-DryRun") "--dry-run no llego como -DryRun: $($translated.Output)"
        Assert-True ($translated.Output -match "-NoSubagents") "--no-subagents no llego como -NoSubagents: $($translated.Output)"
        Assert-True ($translated.Output -match "-Force") "La opcion PowerShell no paso intacta: $($translated.Output)"

        # AC-4: el error del delegado debe llegar intacto al caller CMD.
        $exitCode = Invoke-CmdSetup -Directory $sandbox -Arguments @("-Salir3")
        Assert-True ($exitCode.ExitCode -eq 3) "CMD devolvio $($exitCode.ExitCode), esperaba el exit code 3 del delegado."
    }
    finally {
        if (Test-Path -LiteralPath $sandbox) {
            Remove-Item -LiteralPath $sandbox -Recurse -Force
        }
    }

    # AC-4: sin el delegado, CMD promete 127 y una explicacion accionable.
    $missing = Join-Path $tempRoot "missing-ps1"
    New-Item -ItemType Directory -Path $missing -Force | Out-Null
    Copy-Item -LiteralPath (Join-Path $repoRoot "setup_harness.cmd") -Destination $missing
    $withoutPs1 = Invoke-CmdSetup -Directory $missing -Arguments @("-Version")
    Assert-True ($withoutPs1.ExitCode -eq 127) "Sin setup_harness.ps1 CMD devolvio $($withoutPs1.ExitCode), esperaba 127: $($withoutPs1.Output)"
    Assert-True ($withoutPs1.Output -match "setup_harness\.ps1") "CMD no nombro el PS1 faltante: $($withoutPs1.Output)"

    Write-Host "[OK] CMD installer runtime: launcher real, traduccion, exit code y delegado faltante."
}
finally {
    if (Test-Path -LiteralPath $tempRoot) {
        Remove-Item -LiteralPath $tempRoot -Recurse -Force
    }
}
