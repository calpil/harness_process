#requires -Version 5.1
<#
.SYNOPSIS
Installs Harness Process from Windows PowerShell while keeping setup_harness.sh
as the Unix installer.

.DESCRIPTION
The default layout is Subdir: this repository is the harness directory and its
parent is the multi-repository root. Use -Root when the harness lives directly
in the multi-repository root.
#>
[CmdletBinding()]
param(
    [switch]$Root,
    [switch]$Subdir,
    [switch]$NoSubagents,
    [switch]$NoGraphify,
    [switch]$NoGraphifySkills,
    [switch]$NoAntigravity,
    [switch]$Force,
    [Alias("Preview")]
    [switch]$DryRun,
    [switch]$Reset,
    [switch]$Version,
    [switch]$Help,
    [switch]$Json,
    [string]$LogFile,
    [string]$Config,
    [string]$CargoTargetDir
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$script:HarnessVersion = "2026.06-harness-process"
$script:WithSubagents = -not $NoSubagents
$script:InstallGraphify = -not $NoGraphify
$script:InstallGraphifySkills = -not $NoGraphifySkills
$script:InstallAntigravity = -not $NoAntigravity
$script:Layout = "subdir"
if ($Root) {
    $script:Layout = "root"
}
if ($Subdir) {
    $script:Layout = "subdir"
}
if ($Root -and $Subdir) {
    throw "Use only one layout option: -Root or -Subdir."
}

if ($Version) {
    Write-Output $script:HarnessVersion
    exit 0
}
if ($Help) {
    Get-Help $MyInvocation.MyCommand.Path -Detailed
    exit 0
}

$script:HarnessDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$script:HarnessDir = [IO.Path]::GetFullPath($script:HarnessDir)
if ($script:Layout -eq "subdir") {
    $script:RepoRoot = Split-Path -Parent $script:HarnessDir
    $script:HarnessSubdir = Split-Path -Leaf $script:HarnessDir
    $script:Hrel = "$($script:HarnessSubdir)/"
}
else {
    $script:RepoRoot = $script:HarnessDir
    $script:HarnessSubdir = ""
    $script:Hrel = ""
}
$script:SurfaceDir = $script:RepoRoot
$script:ProjectName = if ($env:HARNESS_PROJECT) {
    $env:HARNESS_PROJECT
}
else {
    Split-Path -Leaf $script:RepoRoot
}
$script:BackupDir = if ($env:HARNESS_BKP_DIR) {
    $env:HARNESS_BKP_DIR
}
else {
    Join-Path $script:HarnessDir "bkp"
}
$script:AssetDir = if (Test-Path -LiteralPath (Join-Path $script:HarnessDir "templates/harness_cli")) {
    Join-Path $script:HarnessDir "templates"
}
else {
    $script:HarnessDir
}

$script:Counters = [ordered]@{
    backed_up = 0
    created = 0
    skipped = 0
    installed = 0
    removed = 0
}
$script:LockStream = $null
$script:LockAcquired = $false
$script:LockPath = Join-Path ([IO.Path]::GetTempPath()) "harness-process-setup.lock"

function Write-HarnessLog {
    param(
        [ValidateSet("INFO", "WARN", "ERROR", "OK")]
        [string]$Level,
        [string]$Message
    )

    $line = "[{0}] [{1}] {2}" -f (Get-Date -Format "yyyy-MM-ddTHH:mm:ssK"), $Level, $Message
    switch ($Level) {
        "WARN" { Write-Warning $Message }
        "ERROR" { Write-Error $Message }
        default { Write-Host $line }
    }
    if ($LogFile) {
        $parent = Split-Path -Parent $LogFile
        if ($parent -and -not (Test-Path -LiteralPath $parent)) {
            New-Item -ItemType Directory -Path $parent -Force | Out-Null
        }
        Add-Content -LiteralPath $LogFile -Value $line
    }
}

function Get-EnvValue {
    param([string]$Name)
    [Environment]::GetEnvironmentVariable($Name, "Process")
}

function Set-EnvDefault {
    param(
        [string]$Name,
        [string]$Value
    )
    if (-not (Get-EnvValue $Name)) {
        [Environment]::SetEnvironmentVariable($Name, $Value, "Process")
    }
}

function Import-HarnessEnvFile {
    param([string]$Path)
    if (-not $Path -or -not (Test-Path -LiteralPath $Path -PathType Leaf)) {
        return
    }
    foreach ($rawLine in Get-Content -LiteralPath $Path) {
        $line = $rawLine.Trim()
        if (-not $line -or $line.StartsWith("#") -or -not $line.Contains("=")) {
            continue
        }
        $parts = $line.Split(@("="), 2, [StringSplitOptions]::None)
        $name = $parts[0].Trim()
        $value = $parts[1].Trim().Trim("'").Trim('"')
        Set-EnvDefault -Name $name -Value $value
    }
    Write-HarnessLog INFO "Configuration loaded from: $Path"
}

function Import-HarnessConfiguration {
    $candidate = $Config
    if (-not $candidate -and $env:HARNESS_CONFIG) {
        $candidate = $env:HARNESS_CONFIG
    }
    if (-not $candidate) {
        $localConfig = Join-Path $script:HarnessDir ".harness.env"
        $userConfig = Join-Path $HOME ".config/harness/config"
        $legacyConfig = Join-Path $HOME ".harnessrc"
        foreach ($path in @($localConfig, $userConfig, $legacyConfig)) {
            if (Test-Path -LiteralPath $path -PathType Leaf) {
                $candidate = $path
                break
            }
        }
    }
    Import-HarnessEnvFile -Path $candidate

    $hubDir = if ($env:HARNESS_HUB) {
        $env:HARNESS_HUB
    }
    else {
        Join-Path $HOME ".harness-hub"
    }
    Import-HarnessEnvFile -Path (Join-Path $hubDir ".env")
}

function Enter-HarnessLock {
    if ($DryRun) {
        return
    }
    try {
        $script:LockStream = [IO.File]::Open(
            $script:LockPath,
            [IO.FileMode]::CreateNew,
            [IO.FileAccess]::Write,
            [IO.FileShare]::None
        )
        $pidBytes = [Text.Encoding]::UTF8.GetBytes([string]$PID)
        $script:LockStream.Write($pidBytes, 0, $pidBytes.Length)
        $script:LockStream.Flush()
        $script:LockAcquired = $true
    }
    catch {
        if (-not $Force -and -not $Reset) {
            throw "Another setup_harness.ps1 process appears to be running. Use -Force only after verifying the stale lock: $($script:LockPath)"
        }
        Write-HarnessLog WARN "Continuing despite setup lock because -Force or -Reset is active."
    }
}

function Exit-HarnessLock {
    if ($script:LockStream) {
        $script:LockStream.Dispose()
        $script:LockStream = $null
    }
    if ($script:LockAcquired -and (Test-Path -LiteralPath $script:LockPath)) {
        Remove-Item -LiteralPath $script:LockPath -Force -ErrorAction SilentlyContinue
    }
    $script:LockAcquired = $false
}

function Get-RelativeBackupName {
    param([string]$Target)
    $full = [IO.Path]::GetFullPath($Target)
    if ($full.StartsWith($script:HarnessDir, [StringComparison]::OrdinalIgnoreCase)) {
        return $full.Substring($script:HarnessDir.Length).TrimStart([char[]]@("\", "/"))
    }
    if ($full.StartsWith($script:SurfaceDir, [StringComparison]::OrdinalIgnoreCase)) {
        $relative = $full.Substring($script:SurfaceDir.Length).TrimStart([char[]]@("\", "/"))
        return Join-Path "surface" $relative
    }
    $driveSafe = $full.Replace(":", "").TrimStart([char[]]@("\", "/"))
    Join-Path "external" $driveSafe
}

function Backup-HarnessPath {
    param([string]$Target)
    if ($Force -or -not (Test-Path -LiteralPath $Target)) {
        $script:Counters.skipped++
        return
    }
    if ($DryRun) {
        Write-HarnessLog INFO "[DRY-RUN] Backup: $Target"
        $script:Counters.backed_up++
        return
    }
    $relative = Get-RelativeBackupName -Target $Target
    $destination = Join-Path $script:BackupDir ("{0}.bak.{1}" -f $relative, (Get-Date -Format "yyyyMMddHHmmssfff"))
    $parent = Split-Path -Parent $destination
    New-Item -ItemType Directory -Path $parent -Force | Out-Null
    Copy-Item -LiteralPath $Target -Destination $destination -Recurse -Force
    Write-HarnessLog INFO "Backup created: $destination"
    $script:Counters.backed_up++
}

function Ensure-Directory {
    param([string]$Path)
    if ($DryRun) {
        Write-HarnessLog INFO "[DRY-RUN] Create directory: $Path"
        $script:Counters.created++
        return
    }
    New-Item -ItemType Directory -Path $Path -Force | Out-Null
    $script:Counters.created++
}

function Write-HarnessText {
    param(
        [string]$Path,
        [string]$Content
    )
    if ($DryRun) {
        Write-HarnessLog INFO "[DRY-RUN] Write: $Path"
        $script:Counters.created++
        return
    }
    $parent = Split-Path -Parent $Path
    if ($parent) {
        New-Item -ItemType Directory -Path $parent -Force | Out-Null
    }
    [IO.File]::WriteAllText($Path, $Content, [Text.UTF8Encoding]::new($false))
    $script:Counters.created++
}

function Write-HarnessJson {
    param(
        [string]$Path,
        [object]$Value
    )
    $content = $Value | ConvertTo-Json -Depth 20
    Write-HarnessText -Path $Path -Content ($content + [Environment]::NewLine)
}

function Ensure-HarnessGitIgnore {
    $ignoreName = if ($script:Layout -eq "subdir") {
        "$($script:HarnessSubdir)/"
    }
    else {
        "$(Split-Path -Leaf $script:HarnessDir)/"
    }
    $gitIgnore = Join-Path $script:RepoRoot ".gitignore"
    $existing = @()
    if (Test-Path -LiteralPath $gitIgnore) {
        $existing = Get-Content -LiteralPath $gitIgnore
    }
    if ($existing -contains $ignoreName) {
        $script:Counters.skipped++
        return
    }
    if ($DryRun) {
        Write-HarnessLog INFO "[DRY-RUN] Add '$ignoreName' to $gitIgnore"
        $script:Counters.created++
        return
    }
    Backup-HarnessPath -Target $gitIgnore
    $block = @(
        "",
        "# Harness Process - never commit the installed harness directory",
        $ignoreName,
        "# Local Harness backups",
        "bkp/"
    ) -join [Environment]::NewLine
    Add-Content -LiteralPath $gitIgnore -Value $block
    $script:Counters.created++
}

function Assert-HarnessAssets {
    $required = @(
        "init.sh",
        "validate_ui.sh",
        "debug_ui.js",
        "commit_guard.sh",
        "harness_status.sh",
        "harness_check.sh",
        "harness_cli",
        "harness_cli.ps1",
        "UPDATING.md"
    )
    if ($script:WithSubagents) {
        $required += @(
            "CHECKPOINTS.md",
            "feature_list.json",
            "progress/current.md",
            "progress/history.md",
            "docs/architecture.md",
            "docs/conventions.md",
            "docs/verification.md",
            "roles/README.md",
            "roles/leader.md",
            "roles/implementer.md",
            "roles/reviewer.md"
        )
    }
    foreach ($asset in $required) {
        $source = Join-Path $script:AssetDir $asset
        if (-not (Test-Path -LiteralPath $source -PathType Leaf)) {
            throw "Required asset is missing: $asset (searched in $($script:AssetDir))"
        }
    }
}

function Install-HarnessAsset {
    param(
        [string]$Asset,
        [string]$Destination
    )
    if (-not $Destination) {
        $Destination = Join-Path $script:HarnessDir $Asset
    }
    $source = Join-Path $script:AssetDir $Asset
    if ($DryRun) {
        Write-HarnessLog INFO "[DRY-RUN] Install asset: $Asset -> $Destination"
        $script:Counters.created++
        return
    }
    $parent = Split-Path -Parent $Destination
    New-Item -ItemType Directory -Path $parent -Force | Out-Null
    if ([IO.Path]::GetFullPath($source) -ne [IO.Path]::GetFullPath($Destination)) {
        Copy-Item -LiteralPath $source -Destination $Destination -Force
    }
    $script:Counters.created++
}

function Install-HarnessAssetIfMissing {
    param([string]$Asset)
    $destination = Join-Path $script:HarnessDir $Asset
    if (Test-Path -LiteralPath $destination) {
        $script:Counters.skipped++
        return
    }
    Install-HarnessAsset -Asset $Asset -Destination $destination
}

# Get-PythonCommand removido (feature #2, solo Rust). harness.exe es obligatorio.

function Assert-PostgresConfiguration {
    $missing = @()
    foreach ($name in @("DB_HOST", "DB_USER", "DB_PASSWORD")) {
        if (-not (Get-EnvValue $name)) {
            $missing += $name
        }
    }
    if ($missing.Count -gt 0) {
        $hubDir = if ($env:HARNESS_HUB) { $env:HARNESS_HUB } else { Join-Path $HOME ".harness-hub" }
        $envFile = Join-Path $hubDir ".env"
        Write-HarnessLog ERROR "PostgreSQL is the required Hub. Missing variables: $($missing -join ', ')."
        Write-HarnessLog INFO "Option A (this session only):"
        Write-HarnessLog INFO '    $env:DB_HOST = "postgres.example.com"; $env:DB_USER = "user"; $env:DB_PASSWORD = "secret"'
        Write-HarnessLog INFO "Option B (persistent, recommended): create $envFile with one VAR=value per line:"
        Write-HarnessLog INFO "    DB_HOST=postgres.example.com"
        Write-HarnessLog INFO "    DB_USER=user"
        Write-HarnessLog INFO "    DB_PASSWORD=secret"
        Write-HarnessLog INFO "    DB_NAME=harness_db        # optional (default: postgres)"
        Write-HarnessLog INFO "    DB_SSL_MODE=require       # optional (default: require)"
        Write-HarnessLog INFO "Then re-run: .\setup_harness.ps1"
        exit 2
    }
}

function Initialize-CargoEnvironment {
    if ($CargoTargetDir) {
        $env:CARGO_TARGET_DIR = $CargoTargetDir
    }
    $cargo = Get-Command cargo -ErrorAction SilentlyContinue
    if ($cargo) {
        return $cargo.Source
    }

    $cargoHome = if ($env:CARGO_HOME) {
        $env:CARGO_HOME
    }
    else {
        Join-Path $HOME ".cargo"
    }
    $cargoBin = Join-Path $cargoHome "bin"
    $cargoExe = Join-Path $cargoBin "cargo.exe"
    if (Test-Path -LiteralPath $cargoExe -PathType Leaf) {
        $pathEntries = $env:PATH -split [IO.Path]::PathSeparator
        if ($pathEntries -notcontains $cargoBin) {
            $env:PATH = $cargoBin + [IO.Path]::PathSeparator + $env:PATH
            Write-HarnessLog INFO "Cargo configured for this PowerShell process from: $cargoBin"
        }
        return $cargoExe
    }
    return $null
}

function Build-HarnessBinary {
    $cargo = Initialize-CargoEnvironment
    $manifest = Join-Path $script:HarnessDir "rust/Cargo.toml"
    if (-not $cargo -or -not (Test-Path -LiteralPath $manifest -PathType Leaf)) {
        $existing = Join-Path $script:HarnessDir "harness.exe"
        if (Test-Path -LiteralPath $existing) {
            Write-HarnessLog WARN "Cargo is unavailable; the existing harness.exe may be stale."
        }
        else {
            Write-HarnessLog WARN "Cargo unavailable and no harness.exe present; harness_cli.ps1 will not work. Install rustup."
        }
        return
    }

    if ($DryRun) {
        Write-HarnessLog INFO "[DRY-RUN] Run cargo build --release --locked and copy harness.exe"
        return
    }

    $rustDir = Split-Path -Parent $manifest
    Push-Location $rustDir
    try {
        & $cargo build --release --locked
        if ($LASTEXITCODE -ne 0) {
            Write-HarnessLog ERROR "Cargo build failed; no harness.exe produced. harness_cli will be unusable."
            return
        }
    }
    finally {
        Pop-Location
    }

    $targetRoot = if ($env:CARGO_TARGET_DIR) {
        if ([IO.Path]::IsPathRooted($env:CARGO_TARGET_DIR)) {
            $env:CARGO_TARGET_DIR
        }
        else {
            Join-Path $rustDir $env:CARGO_TARGET_DIR
        }
    }
    else {
        Join-Path $rustDir "target"
    }
    $builtBinary = Join-Path $targetRoot "release/harness.exe"
    if (-not (Test-Path -LiteralPath $builtBinary -PathType Leaf)) {
        Write-HarnessLog WARN "Cargo succeeded but harness.exe was not found at: $builtBinary"
        return
    }
    Copy-Item -LiteralPath $builtBinary -Destination (Join-Path $script:HarnessDir "harness.exe") -Force
    $script:Counters.installed++
    Write-HarnessLog OK "Native harness.exe built and installed."
}

function ConvertTo-PowerShellCommandPath {
    param([string]$Path)
    '"' + $Path.Replace('"', '""') + '"'
}

function Write-AgentSurface {
    param([string]$Target)
    $content = @'
# Harness Process

This repository uses the Harness Process with Claude Code, Codex, Gemini,
Grok, Antigravity, and other agent CLIs.

Before changing code:

1. Run `powershell -NoProfile -ExecutionPolicy Bypass -File "__HREL__harness_cli.ps1" graph mapa`.
2. Check affected services with `... harness_cli.ps1 graph impacto --microservicio <project/service>`.
3. Query `graphify-out/graph.json` when it exists.
4. Run `... harness_cli.ps1 check-plan`.
5. Keep plans and review evidence in `docs/`; keep live state in `__HREL__progress/`.
6. Close through `... harness_cli.ps1 close --feature <id> --status <status>`.

The Unix entry points remain available through `setup_harness.sh` and
`sh "__HREL__harness_cli"`. On Windows, install with `setup_harness.ps1`;
Git for Windows Bash remains required by the existing POSIX project hooks.

__ROLES__

Never commit the installed harness directory into a target project.
'@
    $rolesSection = if ($script:WithSubagents) {
        @'
Agent roles:

- Leader: `__HREL__roles/leader.md`
- Implementer: `__HREL__roles/implementer.md`
- Reviewer: `__HREL__roles/reviewer.md`
'@
    }
    else {
        "Subagents are disabled for this installation."
    }
    $content = $content.Replace("__ROLES__", $rolesSection)
    Write-HarnessText -Path $Target -Content $content.Replace("__HREL__", $script:Hrel)
}

function Write-AgentDefinitions {
    if (-not $script:WithSubagents) {
        return
    }
    $rolesReadme = Join-Path $script:HarnessDir "roles/README.md"
    $rolesReadmeBody = (Get-Content -LiteralPath $rolesReadme -Raw).Replace("__HREL__", $script:Hrel)
    Write-HarnessText -Path $rolesReadme -Content $rolesReadmeBody

    $descriptions = @{
        leader = "Coordinates scope, impact, and the durable plan. Does not implement code."
        implementer = "Implements one concrete unit from the plan and records durable evidence."
        reviewer = "Verifies tests, impact, checkpoints, and Git state before closure."
    }
    foreach ($role in @("leader", "implementer", "reviewer")) {
        $rolePath = Join-Path $script:HarnessDir "roles/$role.md"
        $body = (Get-Content -LiteralPath $rolePath -Raw).Replace("__HREL__", $script:Hrel)
        Write-HarnessText -Path $rolePath -Content $body

        $tools = if ($role -eq "implementer") {
            "Read, Edit, Write, Bash, Grep, Glob"
        }
        else {
            "Read, Grep, Glob, Bash"
        }
        $claude = @"
---
name: $role
description: $($descriptions[$role])
tools: $tools
model: claude-fable-5
effort: max
---

$body
"@
        Write-HarnessText -Path (Join-Path $script:SurfaceDir ".claude/agents/$role.md") -Content $claude

        $sandbox = if ($role -eq "implementer") { "workspace-write" } else { "read-only" }
        $codex = @"
name = "$role"
description = "$($descriptions[$role])"
sandbox_mode = "$sandbox"
model_reasoning_effort = "high"
developer_instructions = '''
$body
'''
"@
        Write-HarnessText -Path (Join-Path $script:SurfaceDir ".codex/agents/$role.toml") -Content $codex

        $gemini = @"
---
name: $role
description: $($descriptions[$role])
---

$body
"@
        Write-HarnessText -Path (Join-Path $script:SurfaceDir ".gemini/agents/$role.md") -Content $gemini
    }
}

function Write-PowerShellHookRuntime {
    $hookPath = Join-Path $script:SurfaceDir "bin/harness-hook.ps1"
    $content = @'
#requires -Version 5.1
[CmdletBinding()]
param(
    [ValidateSet("plain", "gemini-json", "codex-json")]
    [string]$Mode = "plain",
    [string]$Event = "unknown"
)

$ErrorActionPreference = "Stop"
$root = if ($env:HARNESS_REPO_ROOT) {
    $env:HARNESS_REPO_ROOT
}
else {
    Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
}
$harnessRelative = "__HREL_NOSLASH__"
$harnessDir = if ($harnessRelative) {
    Join-Path $root $harnessRelative
}
else {
    $root
}
$cli = Join-Path $harnessDir "harness_cli.ps1"

function Invoke-HarnessEvent {
    switch -Regex ($Event) {
        "^(session-start|SessionStart|InstructionsLoaded|BeforeAgent)$" {
            & $cli graph mapa
            & $cli status
        }
        "^(post-tool|PostToolUse|AfterTool|Tool)$" {
            if (__WITH_SUBAGENTS__ -eq 1) {
                & $cli nudge
            }
            & $cli status
        }
        "^(stop|Stop|AfterAgent|SessionEnd|SessionStop)$" {
            if (__WITH_SUBAGENTS__ -eq 1) {
                & $cli autocheck
                & $cli check-plan
            }
            & $cli status
        }
    }
}

try {
    Invoke-HarnessEvent
    if ($Mode -eq "gemini-json") {
        @{ suppressOutput = $true } | ConvertTo-Json -Compress
    }
}
catch {
    if ($Mode -eq "codex-json" -and $Event -match "stop|Stop|AfterAgent|SessionEnd|SessionStop") {
        @{ decision = "block"; reason = $_.Exception.Message } | ConvertTo-Json -Compress
        exit 0
    }
    if ($Mode -eq "gemini-json") {
        @{ "continue" = $false; stopReason = $_.Exception.Message } | ConvertTo-Json -Compress
        exit 0
    }
    Write-Error $_
    exit 1
}
'@
    $content = $content.Replace("__HREL_NOSLASH__", $script:Hrel.TrimEnd("/"))
    $content = $content.Replace("__WITH_SUBAGENTS__", $(if ($script:WithSubagents) { "1" } else { "0" }))
    Write-HarnessText -Path $hookPath -Content $content
}

function Get-HookCommand {
    param(
        [string]$Mode,
        [string]$Event
    )
    $hook = Join-Path $script:SurfaceDir "bin/harness-hook.ps1"
    "powershell.exe -NoProfile -ExecutionPolicy Bypass -File {0} {1} {2}" -f `
        (ConvertTo-PowerShellCommandPath $hook), $Mode, $Event
}

function Write-AgentHooks {
    $sessionCommand = Get-HookCommand -Mode "codex-json" -Event "session-start"
    $postCommand = Get-HookCommand -Mode "codex-json" -Event "post-tool"
    $stopCommand = Get-HookCommand -Mode "codex-json" -Event "stop"
    $codex = [ordered]@{
        hooks = [ordered]@{
            SessionStart = @(
                [ordered]@{
                    matcher = "startup|resume|clear|compact"
                    hooks = @([ordered]@{
                        type = "command"
                        command = $sessionCommand
                        timeout = 120
                        statusMessage = "Initializing Harness"
                    })
                }
            )
            PostToolUse = @(
                [ordered]@{
                    matcher = "Bash|Edit|Write|apply_patch"
                    hooks = @([ordered]@{
                        type = "command"
                        command = $postCommand
                        timeout = 30
                        statusMessage = "Updating Harness"
                    })
                }
            )
            Stop = @(
                [ordered]@{
                    hooks = @([ordered]@{
                        type = "command"
                        command = $stopCommand
                        timeout = 120
                        statusMessage = "Checking Harness"
                    })
                }
            )
        }
    }
    Write-HarnessJson -Path (Join-Path $script:SurfaceDir ".codex/hooks.json") -Value $codex

    $gemini = [ordered]@{
        hooksConfig = [ordered]@{ enabled = $true; notifications = $true }
        hooks = [ordered]@{
            SessionStart = @([ordered]@{
                hooks = @([ordered]@{
                    type = "command"
                    name = "harness-session-start"
                    command = (Get-HookCommand -Mode "gemini-json" -Event "session-start")
                    timeout = 120000
                })
            })
            AfterTool = @([ordered]@{
                hooks = @([ordered]@{
                    type = "command"
                    name = "harness-status"
                    command = (Get-HookCommand -Mode "gemini-json" -Event "post-tool")
                    timeout = 30000
                })
            })
            AfterAgent = @([ordered]@{
                hooks = @([ordered]@{
                    type = "command"
                    name = "harness-check"
                    command = (Get-HookCommand -Mode "gemini-json" -Event "stop")
                    timeout = 120000
                })
            })
        }
    }
    Write-HarnessJson -Path (Join-Path $script:SurfaceDir ".gemini/settings.json") -Value $gemini
    $geminiCheck = @'
description = "Run the Harness Process closure checks."
prompt = """
Run this command and fix any blocking result before closing:

```powershell
!{powershell.exe -NoProfile -ExecutionPolicy Bypass -File "bin/harness-hook.ps1" plain stop}
```
"""
'@
    Write-HarnessText -Path (Join-Path $script:SurfaceDir ".gemini/commands/harness/check.toml") -Content $geminiCheck
    $geminiStatus = @'
description = "Show the current Harness Process status."
prompt = """
Summarize the Harness Process using this output:

```powershell
!{powershell.exe -NoProfile -ExecutionPolicy Bypass -File "bin/harness-hook.ps1" plain session-start}
```
"""
'@
    Write-HarnessText -Path (Join-Path $script:SurfaceDir ".gemini/commands/harness/status.toml") -Content $geminiStatus

    $claude = [ordered]@{
        attribution = [ordered]@{ commit = ""; pr = "" }
        hooks = [ordered]@{
            SessionStart = @([ordered]@{
                hooks = @([ordered]@{
                    type = "command"
                    command = (Get-HookCommand -Mode "plain" -Event "session-start")
                })
            })
            Stop = @([ordered]@{
                hooks = @([ordered]@{
                    type = "command"
                    command = (Get-HookCommand -Mode "plain" -Event "stop")
                })
            })
        }
    }
    Write-HarnessJson -Path (Join-Path $script:SurfaceDir ".claude/settings.json") -Value $claude

    $grokHook = @'
#requires -Version 5.1
$root = if ($env:GROK_WORKSPACE_ROOT) {
    $env:GROK_WORKSPACE_ROOT
}
else {
    Split-Path -Parent (Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path))
}
$env:HARNESS_REPO_ROOT = $root
$eventName = if ($env:GROK_HOOK_EVENT) { $env:GROK_HOOK_EVENT } else { "unknown" }
& (Join-Path $root "bin/harness-hook.ps1") plain $eventName
exit $LASTEXITCODE
'@
    Write-HarnessText -Path (Join-Path $script:SurfaceDir ".grok/hooks/harness.ps1") -Content $grokHook
    Write-HarnessText -Path (Join-Path $script:SurfaceDir ".grok/GROK.md") -Content @'
# Harness Process for Grok

Windows hooks are installed under `.grok/hooks/`. Trust them when Grok asks,
or start through `bin/harness-grok.ps1`.
'@
}

function Write-AgentLaunchers {
    foreach ($agent in @("claude", "codex", "gemini", "grok", "antigravity")) {
        $content = @'
#requires -Version 5.1
[CmdletBinding()]
param(
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$Arguments
)
$ErrorActionPreference = "Stop"
$agent = "__AGENT__"
$root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$env:HARNESS_REPO_ROOT = $root
& (Join-Path $root "bin/harness-hook.ps1") plain session-start
$command = Get-Command $agent -ErrorAction SilentlyContinue
if (-not $command) {
    Write-Error "[Harness] Command '$agent' was not found in PATH."
    exit 127
}
Push-Location $root
try {
    & $command.Source @Arguments
    exit $LASTEXITCODE
}
finally {
    Pop-Location
}
'@
        Write-HarnessText -Path (Join-Path $script:SurfaceDir "bin/harness-$agent.ps1") -Content $content.Replace("__AGENT__", $agent)
    }
}

function Ensure-Graphify {
    $graphify = Get-Command graphify -ErrorAction SilentlyContinue
    if ($graphify) {
        Write-HarnessLog OK "graphify is already available."
        $script:Counters.installed++
        return
    }
    if (-not $script:InstallGraphify) {
        Write-HarnessLog INFO "graphify installation disabled."
        return
    }
    if ($DryRun) {
        Write-HarnessLog INFO "[DRY-RUN] Install graphifyy with uv or pipx"
        return
    }

    $uv = Get-Command uv -ErrorAction SilentlyContinue
    if ($uv) {
        & $uv.Source tool install --upgrade graphifyy
    }
    else {
        $pipx = Get-Command pipx -ErrorAction SilentlyContinue
        if ($pipx) {
            & $pipx.Source install graphifyy
        }
        else {
            # python pip fallback for graphifyy removed; only uv/pipx attempted above
        }
    }
    if ($LASTEXITCODE -eq 0) {
        $script:Counters.installed++
        Write-HarnessLog OK "graphify installed."
    }
    else {
        Write-HarnessLog WARN "graphify could not be installed automatically."
    }
}

function Install-GraphifyAgentSkills {
    if (-not $script:InstallGraphifySkills) {
        return
    }
    $graphify = Get-Command graphify -ErrorAction SilentlyContinue
    if (-not $graphify) {
        Write-HarnessLog WARN "Skipping agent graphify skills because graphify is unavailable."
        return
    }
    if ($DryRun) {
        Write-HarnessLog INFO "[DRY-RUN] Install graphify skills for claude, codex, and antigravity"
        return
    }
    $temp = Join-Path ([IO.Path]::GetTempPath()) ("harness-graphify-" + [Guid]::NewGuid().ToString("N"))
    New-Item -ItemType Directory -Path $temp -Force | Out-Null
    Push-Location $temp
    try {
        foreach ($platform in @("claude", "codex", "antigravity")) {
            & $graphify.Source install --platform $platform
            if ($LASTEXITCODE -ne 0) {
                Write-HarnessLog WARN "graphify skill installation failed for: $platform"
            }
        }
    }
    finally {
        Pop-Location
        Remove-Item -LiteralPath $temp -Recurse -Force -ErrorAction SilentlyContinue
    }
}

function Ensure-Antigravity {
    if (Get-Command antigravity -ErrorAction SilentlyContinue) {
        Write-HarnessLog OK "antigravity is already available."
        return
    }
    if (-not $script:InstallAntigravity) {
        return
    }
    Write-HarnessLog WARN "Antigravity automatic installation remains POSIX-only. Install it separately or rerun setup_harness.sh from Git Bash."
}

# Ensure-Psycopg + Invoke-PostgresMigration + heredoc py migration REMOVED (feature #2 pure Rust).
# The harness.exe binary owns hub init, schema creation and any legacy data load.
function Invoke-PostgresMigration { param([string]$Python) Write-HarnessLog INFO "[psycopg] skipped (Rust only)"; }

function Archive-LegacyHub {
    $hubDir = if ($env:HARNESS_HUB) {
        $env:HARNESS_HUB
    }
    else {
        Join-Path $HOME ".harness-hub"
    }
    $graphFile = Join-Path $hubDir "graph_db.json"
    $progressDir = Join-Path $hubDir "progress"
    if (-not (Test-Path -LiteralPath $graphFile) -and -not (Test-Path -LiteralPath $progressDir)) {
        return
    }
    $destination = Join-Path $script:BackupDir ("memory-hub/{0}-{1}" -f (Get-Date -Format "yyyyMMddHHmmss"), $PID)
    if ($DryRun) {
        Write-HarnessLog INFO "[DRY-RUN] Archive legacy Hub memory to: $destination"
        return
    }
    New-Item -ItemType Directory -Path $destination -Force | Out-Null
    if (Test-Path -LiteralPath $graphFile) {
        Copy-Item -LiteralPath $graphFile -Destination (Join-Path $destination "graph_db.json") -Force
        Remove-Item -LiteralPath $graphFile -Force
    }
    if (Test-Path -LiteralPath $progressDir) {
        Copy-Item -LiteralPath $progressDir -Destination (Join-Path $destination "progress") -Recurse -Force
        Remove-Item -LiteralPath $progressDir -Recurse -Force
    }
    $script:Counters.backed_up++
}

function Invoke-HarnessReset {
    Ensure-HarnessGitIgnore
    $targets = @(
        "CLAUDE.md",
        "AGENTS.md",
        "GEMINI.md",
        "LLM.md",
        ".claude/settings.json",
        ".claude/agents",
        ".codex/hooks.json",
        ".codex/agents",
        ".gemini/settings.json",
        ".gemini/commands",
        ".gemini/agents",
        ".grok/hooks",
        ".grok/GROK.md",
        "bin/harness-hook",
        "bin/harness-hook.ps1",
        "bin/harness-claude",
        "bin/harness-codex",
        "bin/harness-gemini",
        "bin/harness-grok",
        "bin/harness-antigravity",
        "bin/harness-claude.ps1",
        "bin/harness-codex.ps1",
        "bin/harness-gemini.ps1",
        "bin/harness-grok.ps1",
        "bin/harness-antigravity.ps1"
    )
    $targets += @(
        (Join-Path $script:HarnessDir "roles"),
        (Join-Path $script:HarnessDir "docs"),
        (Join-Path $script:HarnessDir "progress"),
        (Join-Path $script:HarnessDir "CHECKPOINTS.md"),
        (Join-Path $script:HarnessDir "feature_list.json")
    )
    foreach ($relative in $targets) {
        $target = if ([IO.Path]::IsPathRooted($relative)) {
            $relative
        }
        else {
            Join-Path $script:SurfaceDir $relative
        }
        if (Test-Path -LiteralPath $target) {
            Backup-HarnessPath -Target $target
            if (-not $DryRun) {
                Remove-Item -LiteralPath $target -Recurse -Force
            }
            $script:Counters.removed++
        }
        else {
            $script:Counters.skipped++
        }
    }
    foreach ($relative in @(".harness_layout", ".harness_backend")) {
        $target = Join-Path $script:HarnessDir $relative
        if (Test-Path -LiteralPath $target) {
            Backup-HarnessPath -Target $target
            if (-not $DryRun) {
                Remove-Item -LiteralPath $target -Force
            }
            $script:Counters.removed++
        }
    }
}

function Write-FinalReport {
    $status = if ($DryRun) { "dry-run" } else { "success" }
    Write-HarnessLog OK "Harness Process setup complete ($($script:Layout), $status)."
    Write-HarnessLog INFO "Actions: backups=$($script:Counters.backed_up), created=$($script:Counters.created), skipped=$($script:Counters.skipped), installed=$($script:Counters.installed), removed=$($script:Counters.removed)"
    if ($Json) {
        [ordered]@{
            version = $script:HarnessVersion
            layout = $script:Layout
            dry_run = [bool]$DryRun
            with_subagents = [bool]$script:WithSubagents
            actions = $script:Counters
            status = $status
        } | ConvertTo-Json -Depth 5
    }
}

Import-HarnessConfiguration
Enter-HarnessLock

try {
    if ($Reset) {
        Invoke-HarnessReset
        Write-FinalReport
        exit 0
    }

    Assert-PostgresConfiguration
    Assert-HarnessAssets
    Ensure-HarnessGitIgnore
    if (-not (Get-Command bash -ErrorAction SilentlyContinue)) {
        Write-HarnessLog WARN "Git for Windows Bash was not found. Direct PowerShell commands work, but existing POSIX hooks and scripts require Bash."
    }

    if ($DryRun) {
        Write-HarnessLog INFO "[DRY-RUN] Install Harness Process in: $($script:HarnessDir)"
        Build-HarnessBinary
        Ensure-Graphify
        Write-FinalReport
        exit 0
    }

    foreach ($directory in @(
        $script:HarnessDir,
        (Join-Path $script:SurfaceDir ".claude"),
        (Join-Path $script:SurfaceDir ".codex"),
        (Join-Path $script:SurfaceDir ".gemini"),
        (Join-Path $script:SurfaceDir ".grok"),
        (Join-Path $script:SurfaceDir "bin")
    )) {
        Ensure-Directory -Path $directory
    }
    if ($script:WithSubagents) {
        foreach ($directory in @(
            (Join-Path $script:HarnessDir "roles"),
            (Join-Path $script:HarnessDir "docs"),
            (Join-Path $script:HarnessDir "progress"),
            (Join-Path $script:SurfaceDir ".claude/agents"),
            (Join-Path $script:SurfaceDir ".codex/agents"),
            (Join-Path $script:SurfaceDir ".gemini/agents")
        )) {
            Ensure-Directory -Path $directory
        }
    }

    $layoutMarker = Join-Path $script:HarnessDir ".harness_layout"
    $backendMarker = Join-Path $script:HarnessDir ".harness_backend"
    Backup-HarnessPath -Target $layoutMarker
    Backup-HarnessPath -Target $backendMarker
    Write-HarnessText -Path $layoutMarker -Content ($script:Layout + [Environment]::NewLine)
    Write-HarnessText -Path $backendMarker -Content ("postgres" + [Environment]::NewLine)

    $generatedAssets = @(
        "init.sh",
        "validate_ui.sh",
        "debug_ui.js",
        "commit_guard.sh",
        "harness_status.sh",
        "harness_check.sh",
        "harness_cli",
        "harness_cli.ps1",
        "UPDATING.md"
    )
    if ($script:WithSubagents) {
        $generatedAssets += @(
            "CHECKPOINTS.md",
            "docs/architecture.md",
            "docs/conventions.md",
            "docs/verification.md",
            "roles/README.md",
            "roles/leader.md",
            "roles/implementer.md",
            "roles/reviewer.md"
        )
    }
    foreach ($asset in $generatedAssets) {
        $destination = Join-Path $script:HarnessDir $asset
        Backup-HarnessPath -Target $destination
        Install-HarnessAsset -Asset $asset -Destination $destination
    }
    if ($script:WithSubagents) {
        foreach ($asset in @("feature_list.json", "progress/current.md", "progress/history.md")) {
            Install-HarnessAssetIfMissing -Asset $asset
        }
    }

    Build-HarnessBinary
    # Python postgres migration skipped (pure Rust harness.exe owns hub init/migration)
    Archive-LegacyHub

    $surfaceBackups = @(
        "CLAUDE.md",
        "AGENTS.md",
        "GEMINI.md",
        "LLM.md",
        ".claude/settings.json",
        ".codex/hooks.json",
        ".gemini/settings.json",
        ".gemini/commands/harness/check.toml",
        ".gemini/commands/harness/status.toml",
        ".grok/GROK.md",
        ".grok/hooks/harness.ps1",
        "bin/harness-hook.ps1",
        "bin/harness-claude.ps1",
        "bin/harness-codex.ps1",
        "bin/harness-gemini.ps1",
        "bin/harness-grok.ps1",
        "bin/harness-antigravity.ps1"
    )
    if ($script:WithSubagents) {
        foreach ($role in @("leader", "implementer", "reviewer")) {
            $surfaceBackups += ".claude/agents/$role.md"
            $surfaceBackups += ".codex/agents/$role.toml"
            $surfaceBackups += ".gemini/agents/$role.md"
        }
    }
    foreach ($relative in $surfaceBackups) {
        Backup-HarnessPath -Target (Join-Path $script:SurfaceDir $relative)
    }

    Write-AgentDefinitions
    foreach ($surface in @("CLAUDE.md", "AGENTS.md", "GEMINI.md", "LLM.md")) {
        $target = Join-Path $script:SurfaceDir $surface
        Write-AgentSurface -Target $target
    }
    Write-PowerShellHookRuntime
    Write-AgentHooks
    Write-AgentLaunchers

    Ensure-Graphify
    Install-GraphifyAgentSkills
    Ensure-Antigravity

    Write-HarnessLog INFO "PowerShell entry point: $($script:Hrel)harness_cli.ps1"
    Write-HarnessLog INFO "Unix entry point remains: sh $($script:Hrel)harness_cli"
    Write-FinalReport
}
finally {
    Exit-HarnessLock
}
