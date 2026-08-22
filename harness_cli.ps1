#requires -Version 5.1
# Punto de entrada unico del Harness en Windows (Rust only, post-migracion).
# Despacha SIEMPRE al binario nativo (harness.exe). Sin binario: error.
#
# Homologado con `harness_cli` (superficie sh), feature #25: este lanzador cubre
# la mitad que `harness_cli doctor` NO puede cubrir por definicion — un doctor
# que vive en el binario no puede diagnosticar un binario ausente, ni uno tan
# viejo que no conozca el subcomando. Ese caso se traduce aca al mismo remedio
# en vez de dejar salir el error de clap, que dice que algo no existe pero no
# que hacer.
#
# stdout y stderr pasan SIN bufferear: capturarlos para inspeccionarlos dejaria
# mudos los comandos que tardan (con el hub sin responder, `close` tarda ~90s y
# el usuario no veria nada). Por eso se mira el exit code y, solo cuando es el
# codigo de uso invalido de clap, se le pregunta al binario si conoce el
# subcomando. Un subcomando conocido nunca dispara el aviso.
[CmdletBinding()]
param(
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$Arguments
)

$ErrorActionPreference = "Stop"
$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path

$nativeBinary = Join-Path $scriptDir "harness.exe"
if (-not (Test-Path -LiteralPath $nativeBinary -PathType Leaf)) {
    [Console]::Error.WriteLine("[harness_cli] Binario 'harness.exe' no encontrado en $scriptDir.")
    [Console]::Error.WriteLine("              Remedio: powershell -ExecutionPolicy Bypass -File setup_harness.ps1")
    [Console]::Error.WriteLine("              (requiere rust/cargo disponible para compilar el binario)")
    exit 127
}

& $nativeBinary @Arguments
$rc = $LASTEXITCODE

# 2 es el codigo de uso invalido de clap. Solo entonces vale la pena preguntar,
# y la pregunta es barata: `help <sub>` no ejecuta el subcomando.
$sub = if ($Arguments) { $Arguments[0] } else { $null }
if ($rc -eq 2 -and $sub) {
    & $nativeBinary help $sub *> $null
    if ($LASTEXITCODE -ne 0) {
        [Console]::Error.WriteLine("")
        [Console]::Error.WriteLine("[harness_cli] El binario instalado no conoce el subcomando '$sub': es mas viejo")
        [Console]::Error.WriteLine("              que los scripts que lo invocan (tipico de 'git pull' sin")
        [Console]::Error.WriteLine("              re-correr el instalador).")
        [Console]::Error.WriteLine("              Remedio: powershell -ExecutionPolicy Bypass -File setup_harness.ps1")
    }
}

exit $rc
