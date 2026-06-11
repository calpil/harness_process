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

$python = Get-Command python3 -ErrorAction SilentlyContinue
if (-not $python) {
    $python = Get-Command python -ErrorAction SilentlyContinue
}
if (-not $python) {
    Write-Error "[harness_cli] Neither harness.exe nor Python is available."
    exit 127
}

if ($Arguments.Count -gt 0 -and $Arguments[0] -eq "graph") {
    $graphArgs = @()
    if ($Arguments.Count -gt 1) {
        $graphArgs = $Arguments[1..($Arguments.Count - 1)]
    }
    & $python.Source (Join-Path $scriptDir "graph_memory.py") @graphArgs
    exit $LASTEXITCODE
}

& $python.Source (Join-Path $scriptDir "harness.py") @Arguments
exit $LASTEXITCODE
