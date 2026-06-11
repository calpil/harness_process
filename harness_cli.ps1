#requires -Version 5.1
[CmdletBinding()]
param(
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$Arguments
)

$ErrorActionPreference = "Stop"
$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path

$nativeBinary = Join-Path $scriptDir "harness.exe"
if (Test-Path -LiteralPath $nativeBinary -PathType Leaf) {
    & $nativeBinary @Arguments
    exit $LASTEXITCODE
}

Write-Error "[harness_cli] Binario 'harness.exe' no encontrado en $scriptDir. Ejecuta setup_harness.ps1 con rust/cargo disponible para compilarlo."
exit 127
