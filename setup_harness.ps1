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
    [switch]$NoKimi,
    [switch]$Force,
    [Alias("Preview")]
    [switch]$DryRun,
    [switch]$Reset,
    [switch]$Version,
    [switch]$Help,
    [switch]$Json,
    [string]$LogFile,
    [string]$Config,
    # Binding de Atlassian (feature #15): paridad con --atlassian-site,
    # --jira-project, --confluence-space y --jira-issue-type de setup_harness.sh.
    [string]$AtlassianSite,
    [string]$JiraProject,
    [string]$ConfluenceSpace,
    [string]$JiraIssueType,
    [switch]$CreateJiraProject,
    [switch]$CreateConfluenceSpace,
    [string]$CargoTargetDir
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$script:HarnessVersion = "2026.07-harness-process"
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

# Docs GENERADOS por el instalador (plantillas de templates/docs/). Viven en el
# docs/ de la RAIZ del proyecto (SurfaceDir), junto a docs/constitution.md y a
# los artefactos SDD (spec-*/plan-*). La constitution NO esta en esta lista: es
# documento del usuario y tiene su propio tratamiento.
# Las rutas pueden traer subdirectorio (p.ej. `prd/...`): la siembra, los reset
# targets y la migracion crean el directorio destino. Paridad con HARNESS_DOCS.
$script:HarnessDocs = @(
    "architecture.md",
    "conventions.md",
    "verification.md",
    "kimi-cli-uso-eficiente.md",
    "atlassian-integracion.md",
    "prd/COMO-ESCRIBIR-UN-PRD.md",
    # Feature #17: la GUIA de lecciones es plantilla del arnes (se refresca al
    # reinstalar y entra en los reset targets). Las lecciones en si
    # (docs/lecciones/*.md) NO se listan en ningun lado a proposito: esa ausencia
    # es lo que las hace sobrevivir a -Reset, porque son conocimiento ganado del
    # proyecto, como el PRD y la constitution.
    "lecciones/COMO-ESCRIBIR-UNA-LECCION.md"
)

# Planillas maestras del proyecto (PRD y SDD), en `docs/prd/` de la RAIZ. Son
# documentos del USUARIO: se siembran una sola vez si faltan, no se respaldan, no
# se regeneran y NO entran en los reset targets (a diferencia de HarnessDocs, que
# son plantillas del arnes y si se limpian con -Reset). Paridad con PRD_DOCS de
# setup_harness.sh.
$script:PrdDocs = @(
    "PRD-master.md",
    "SDD-master.md"
)

# Documentos del USUARIO que viven directamente en el `docs/` de la RAIZ (no bajo
# `docs/prd/`). Mismo trato que PrdDocs: se siembran SOLO si faltan, un reinstall
# NO los pisa y NO entran en los reset targets. Paridad con USER_DOCS de
# setup_harness.sh. Feature #19.
$script:UserDocs = @(
    "perfil-usuario.md"
)

# Dotfiles de contexto para agentes (Kimi y otros): .kimiignore (exclusiones
# de contexto, espejo de .gitignore) y .kimirules (reglas fijas del proyecto,
# referenciado desde el AGENTS.md de la raiz). Documentos del USUARIO en la
# RAIZ: se siembran solo si faltan, no se respaldan, no se regeneran y NO
# entran en los reset targets. Paridad con KIMI_DOTFILES de setup_harness.sh.
$script:KimiDotfiles = @(
    ".kimiignore",
    ".kimirules"
)

# Guardrail: nunca escribir superficies en el HOME del usuario (pisaria
# .claude/settings.json y agentes globales). Escape: HARNESS_ALLOW_HOME_SURFACE=1.
if ($env:HARNESS_ALLOW_HOME_SURFACE -ne "1") {
    $surfaceFull = [IO.Path]::GetFullPath($script:SurfaceDir).TrimEnd('\', '/')
    $homeFull = [IO.Path]::GetFullPath($HOME).TrimEnd('\', '/')
    if ($surfaceFull -eq $homeFull) {
        Write-Host "[ERROR] SurfaceDir is your HOME ($HOME): installing here would overwrite .claude/settings.json and global agents." -ForegroundColor Red
        Write-Host "Move this checkout inside your project (<project>\harness_process) or use -Root for a self-contained install."
        Write-Host "Conscious escape: `$env:HARNESS_ALLOW_HOME_SURFACE = '1'"
        exit 2
    }
}
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

function Initialize-HarnessEnvTemplate {
    # Feature #15: deja `.harness.env` listo en la RAIZ del proyecto con las
    # claves comentadas. Documento del USUARIO: se siembra solo si falta, nunca
    # se pisa (puede tener el token real) y no entra en los targets de -Reset.
    # Paridad con seed_harness_env() de setup_harness.sh.
    $target = Join-Path $script:SurfaceDir ".harness.env"
    if (Test-Path -LiteralPath $target -PathType Leaf) {
        $script:Counters.skipped++
        return
    }
    if ($DryRun) {
        Write-HarnessLog INFO "[DRY-RUN] Would seed $target (local config template)"
        $script:Counters.created++
        return
    }
    $template = @"
# Config local del arnes. NUNCA se commitea: el instalador lo deja en
# .gitignore porque puede llevar credenciales.
#
# Alcance: este archivo vale para ESTE proyecto. Si preferis definirlo una sola
# vez para todos tus proyectos, escribi las mismas claves en
# ~/.config/harness/config (lo local siempre gana sobre lo global).

# --- Atlassian: credenciales del ejecutor REST -----------------------------
# Solo hacen falta para ``atlassian apply``, ``atlassian sprint`` y
# ``atlassian publish``. Sin ellas la integracion igual funciona con un agente
# que tenga MCP de Atlassian (``atlassian drain`` + ``atlassian ack``).
# El API token se genera en:
#   https://id.atlassian.com/manage-profile/security/api-tokens
#HARNESS_ATLASSIAN_EMAIL=tu.correo@empresa.cl
#HARNESS_ATLASSIAN_TOKEN=

# --- Atlassian: a que proyecto y space pertenece este repo -----------------
# Alternativa a los parametros del instalador (-AtlassianSite, -JiraProject,
# -ConfluenceSpace, -JiraIssueType). Lo que pasa por parametro manda sobre esto.
#HARNESS_ATLASSIAN_SITE=acme.atlassian.net
#HARNESS_JIRA_PROJECT=ADR
#HARNESS_CONFLUENCE_SPACE=SD
#HARNESS_JIRA_ISSUE_TYPE=Story
"@
    Write-HarnessText -Path $target -Content ($template + [Environment]::NewLine)
    Write-HarnessLog INFO "Local config seeded: $target (put the Atlassian email and token there; already gitignored)."
}

function Write-AtlassianBinding {
    # Feature #15 (AC-1/AC-2/AC-3/AC-13): a que proyecto Jira y a que space de
    # Confluence pertenece ESTE repo. Precedencia parametro > config file >
    # nada. Sin proyecto y sitio NO se escribe atlassian.json: la integracion
    # queda apagada y el arnes se comporta igual que siempre. Paridad literal
    # con write_atlassian_binding() de setup_harness.sh.
    $site = if ($AtlassianSite) { $AtlassianSite } else { $env:HARNESS_ATLASSIAN_SITE }
    $project = if ($JiraProject) { $JiraProject } else { $env:HARNESS_JIRA_PROJECT }
    $space = if ($ConfluenceSpace) { $ConfluenceSpace } else { $env:HARNESS_CONFLUENCE_SPACE }
    $issueType = if ($JiraIssueType) { $JiraIssueType } else { $env:HARNESS_JIRA_ISSUE_TYPE }
    if (-not $issueType) { $issueType = "Story" }

    $target = Join-Path $script:SurfaceDir "atlassian.json"
    if (-not $project -or -not $site) {
        Write-HarnessLog INFO "Atlassian: sin binding (integracion apagada). Para activarla, preguntale al USUARIO a que proyecto y space pertenece este repo y corre:"
        Write-HarnessLog INFO "    sh harness_cli atlassian bind --site <sitio>.atlassian.net --jira-project <KEY> --confluence-space <KEY>"
        return
    }
    if ($DryRun) {
        Write-HarnessLog INFO "[DRY-RUN] Escribiria $target (Jira $project, Confluence $(if ($space) { $space } else { 'sin space' }))"
        return
    }
    if ((Test-Path -LiteralPath $target -PathType Leaf) -and (-not $Force)) {
        Write-HarnessLog INFO "Atlassian: $target ya existe (no se pisa; usa 'atlassian bind' para cambiarlo)."
        return
    }

    $binding = @"
{
  "site": "$site",
  "enabled": true,
  "jira": {
    "project_key": "$project",
    "issue_types": {
      "epic": "Epic",
      "feature": "$issueType",
      "ac": "Subtask"
    },
    "statuses": {
      "pending": "To Do",
      "in_progress": "In Progress",
      "done": "Done",
      "blocked_flag": "Impediment"
    }
  },
  "confluence": {
    "space_key": "$space"
  }
}
"@
    Write-HarnessText -Path $target -Content ($binding + [Environment]::NewLine)
    Write-HarnessLog OK "Atlassian: binding escrito en $target (Jira $project, Confluence $(if ($space) { $space } else { 'sin space' }))."
    # Feature #16 (AC-18): la verificacion se delega al binario, que sabe buscar
    # las credenciales y hablar con la API. Paridad con setup_harness.sh.
    $harnessBin = Join-Path $script:HarnessDir "harness.exe"
    if (Test-Path -LiteralPath $harnessBin -PathType Leaf) {
        $verifyArgs = @("atlassian", "bind")
        if ($CreateJiraProject) { $verifyArgs += "--create-project" }
        if ($CreateConfluenceSpace) { $verifyArgs += "--create-space" }
        try {
            & $harnessBin @verifyArgs 2>&1 | ForEach-Object { Write-Host "    $_" }
        }
        catch {
            Write-HarnessLog INFO "Atlassian: la verificacion del binding no pudo completarse (se sigue igual)."
        }
    }
    if (-not $space) {
        Write-HarnessLog INFO "Atlassian: sin space de Confluence no se publican PRD/SDD; agregalo con -ConfluenceSpace <KEY>."
    }
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

# Migracion de instalaciones anteriores a la feature #4: los docs del arnes
# vivian en <harness>/docs/. Ahora viven en el docs/ de la RAIZ, junto a la
# constitution y a los artefactos SDD. Regla (decision usuario 2026-07-24): se
# MUEVEN solo si en la raiz no existen; si ya existen, no se pisa nada y se
# avisa. En layout root las dos rutas son la misma y la funcion es un no-op.
# Paridad exacta con migrate_harness_docs() de setup_harness.sh.
function Move-HarnessDocsToRoot {
    if ([IO.Path]::GetFullPath($script:HarnessDir) -eq [IO.Path]::GetFullPath($script:SurfaceDir)) {
        return
    }
    $legacyDir = Join-Path $script:HarnessDir "docs"
    if (-not (Test-Path -LiteralPath $legacyDir)) {
        return
    }
    foreach ($harnessDoc in $script:HarnessDocs) {
        $old = Join-Path $legacyDir $harnessDoc
        $new = Join-Path $script:SurfaceDir "docs/$harnessDoc"
        if (-not (Test-Path -LiteralPath $old -PathType Leaf)) {
            continue
        }
        if (Test-Path -LiteralPath $new) {
            Write-HarnessLog WARN "Migration: $new already exists; kept intact and NOT overwritten (old copy stays at $old)."
            $script:Counters.skipped++
            continue
        }
        if ($DryRun) {
            Write-HarnessLog INFO "[DRY-RUN] Would migrate harness doc: $old -> $new"
            $script:Counters.created++
            continue
        }
        $parent = Split-Path -Parent $new
        New-Item -ItemType Directory -Path $parent -Force | Out-Null
        Move-Item -LiteralPath $old -Destination $new
        Write-HarnessLog INFO "Migrated to the root docs/: docs/$harnessDoc (was at $old)"
        $script:Counters.created++
    }
    # Si docs/ del arnes quedo vacio, se elimina. Si el usuario dejo ahi otros
    # archivos suyos, se conserva tal cual.
    if (-not $DryRun) {
        if (-not (Get-ChildItem -LiteralPath $legacyDir -Force)) {
            Remove-Item -LiteralPath $legacyDir -Force
        }
    }
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
    # Feature #15 / Articulo 4: `.harness.env` puede llevar el email y el API
    # token de Atlassian; se ignora SIEMPRE y aparte, para que tambien lo gane
    # una instalacion vieja que ya tenia su .gitignore. Paridad con
    # setup_harness.sh.
    if (($existing -notcontains ".harness.env") -and (-not $DryRun)) {
        $credBlock = @(
            "",
            "# Local harness config (may hold credentials): never commit",
            ".harness.env"
        ) -join [Environment]::NewLine
        Add-Content -LiteralPath $gitIgnore -Value $credBlock
        Write-HarnessLog INFO ".gitignore updated: .harness.env (credentials) must never be committed."
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
            "docs/kimi-cli-uso-eficiente.md",
            "docs/atlassian-integracion.md",
            "docs/constitution.md",
            "docs/prd/COMO-ESCRIBIR-UN-PRD.md",
            "docs/prd/PRD-master.md",
            "docs/prd/SDD-master.md",
            ".kimiignore",
            ".kimirules",
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

function Test-WindowsGnuToolchainGap {
    # El target *-windows-gnu necesita binutils de MinGW (dlltool.exe) para
    # crates con raw-dylib (windows-sys, getrandom). El estandar en Windows
    # es el toolchain MSVC, que no depende de herramientas externas.
    $rustc = Get-Command rustc -ErrorAction SilentlyContinue
    if (-not $rustc) {
        return $false
    }
    $hostLine = (& $rustc.Source -vV | Where-Object { $_ -like "host:*" }) -join ""
    if ($hostLine -notlike "*windows-gnu*") {
        return $false
    }
    if (Get-Command dlltool -ErrorAction SilentlyContinue) {
        return $false
    }
    Write-HarnessLog ERROR "Rust toolchain is *-windows-gnu but MinGW 'dlltool.exe' is not in PATH; the build would fail (getrandom/windows-sys need it)."
    Write-HarnessLog INFO "Recommended fix - switch to the MSVC toolchain:"
    Write-HarnessLog INFO "    rustup default stable-x86_64-pc-windows-msvc"
    Write-HarnessLog INFO "    winget install Microsoft.VisualStudio.2022.BuildTools --override `"--quiet --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended`""
    Write-HarnessLog INFO "    (reopen the terminal and re-run .\setup_harness.ps1)"
    Write-HarnessLog INFO "Alternative - stay on GNU: install MSYS2/MinGW-w64 and add its bin (dlltool.exe) to PATH."
    return $true
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

    if (Test-WindowsGnuToolchainGap) {
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
    if (-not (Install-BinaryAtomic -Source $builtBinary -Destination (Join-Path $script:HarnessDir "harness.exe"))) {
        return
    }
    $script:Counters.installed++
    Write-HarnessLog OK "Native harness.exe built and installed."
}

# Installs a BINARY without ever writing over the live one: copy to a sibling
# temp file (same directory = same volume, required for the move to replace the
# entry in one step) and only then move it onto the destination. Overwriting the
# running binary in place is what leaves a half-written harness.exe when the old
# one is locked (and, on macOS, what makes the installed binary die with
# SIGKILL). On any failure the temp file is removed and the previous binary is
# left intact.
function Install-BinaryAtomic {
    param(
        [Parameter(Mandatory = $true)][string]$Source,
        [Parameter(Mandatory = $true)][string]$Destination
    )
    $destinationDir = Split-Path -Parent $Destination
    $leaf = Split-Path -Leaf $Destination
    $temporary = Join-Path $destinationDir (".{0}.new.{1}" -f $leaf, $PID)
    $displaced = Join-Path $destinationDir (".{0}.old.{1}" -f $leaf, $PID)
    try {
        Copy-Item -LiteralPath $Source -Destination $temporary -Force
        try {
            Move-Item -LiteralPath $temporary -Destination $Destination -Force
        }
        catch {
            # Windows keeps a running .exe locked: move the live one aside (that
            # IS allowed) and put the new one in its place, instead of failing
            # or leaving the destination half written.
            Move-Item -LiteralPath $Destination -Destination $displaced -Force
            Move-Item -LiteralPath $temporary -Destination $Destination -Force
            Remove-Item -LiteralPath $displaced -Force -ErrorAction SilentlyContinue
        }
        return $true
    }
    catch {
        Remove-Item -LiteralPath $temporary -Force -ErrorAction SilentlyContinue
        Write-HarnessLog ERROR "Could not install $leaf atomically: $($_.Exception.Message). The previous binary was left untouched."
        return $false
    }
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
Grok, Kimi Code, Antigravity, and other agent CLIs.

Before changing code:

1. Run `powershell -NoProfile -ExecutionPolicy Bypass -File "__HREL__harness_cli.ps1" graph mapa`.
2. Check affected services with `... harness_cli.ps1 graph impacto --microservicio <project/service>`.
3. Query `graphify-out/graph.json` when it exists.
4. Run `... harness_cli.ps1 check-plan`.
5. Run `... harness_cli.ps1 check-spec`: `start` generates the spec next to the
   plan (`docs/spec-feature-<id>-<slug>.md`) and both are watched against edits
   by other LLMs. If the spec is still `Estado: draft`, STOP: SHOW the spec to the
   USER (in the chat and opened in their editor), ASK whether they approve it, and
   only with their explicit YES record it with
   `... harness_cli.ps1 approve-spec --yes`; never approve on your own.
   Specs and plans must comply with `docs/constitution.md`.
6. Check the plan section "Observaciones (decisiones pendientes)": if any
   observation has no decision yet, ASK THE USER which decision to apply
   BEFORE implementing that feature/phase/task, then record it with
   `... harness_cli.ps1 advance --nota "Decision usuario: <...>"`.
7. Keep plans and review evidence in `docs/`; keep live state in `__HREL__progress/`.
8. Close through `... harness_cli.ps1 close --feature <id> --status <status>`.

Lessons (`docs/lecciones/<class>.md`) are the project's procedural memory,
ordered by CLASS of work instead of by feature id. Check them BEFORE designing
(`... harness_cli.ps1 leccion list`), leave a trace when one helps you
(`leccion usar <class>`), and when you learn something PATCH the lesson that was
in play before creating another (`leccion nueva` rejects session names: with
`feature`, with `#`, with a `fix-`/`debug-` prefix, with a date or with long
numbers, and there is no `--force`). The method and the list of what NOT to
capture live in `docs/lecciones/COMO-ESCRIBIR-UNA-LECCION.md`. With the
`require_leccion` rule enabled, `close --status done` demands `--leccion <class>`
or `--leccion ninguna --leccion-motivo "<why>"`. The harness reminds you on its
own: every 25 writes (`rules.leccion_nudge_interval`, `0` turns it off) and when
you close without declaring, where it prints the full CONTRACT on stderr, read
from that same guide. When you see it, don't ignore it: check the catalog and
patch.

`harness_cli doctor` diagnoses the INSTALLATION (binary, hooks, surfaces,
marker, hub, tools, graphify) and prints the EXACT remedy command for each
problem. Run it first whenever the harness does not behave as expected. It exits
2 only when something prevents work; an unreachable hub or a missing graphify are
warnings and exit 0. It never fixes anything: it prints the command and you run
it. Different from `harness_check.sh`, which checks the PROCESS (spec, plan,
PRDs, lessons, profile, conventions); they do not overlap.

`harness_cli verify --feature <id>` runs the commands your spec's ACs declare
below them (`Comando: <shell>`) and writes `docs/verify-<id>.md`. An AC with no
command is MANUAL and never counts as a failure, so existing specs keep working.
It is the ONLY command that executes shell: it demands `Estado: approved` (on a
draft it refuses without running anything), no hook calls it, and every command
is printed before it runs. With `rules.require_verify_green`, closing demands a
green report newer than the spec -- but CLOSING NEVER EXECUTES: it reads the
report. When declaring a command, beware of ones that cannot fail
(`cargo test <name>` with no matches exits 0; so does `|| true`).

`harness_cli journey` maps what the project has learned (closed features,
lessons and profile, with their links) and points out the GAPS. It is read-only:
for each gap it prints the command that fixes it, and pruning goes through each
store's own command.

`harness_cli lecciones status` shows the health of the lesson library. The
curator pass (`lecciones curar`) only REPORTS; moving anything requires
`--aplicar`, and you tell the user before doing it. Nothing is ever deleted:
archiving moves the file to `docs/lecciones/archivo/`, with a backup and
`lecciones rollback`.

Before proposing something or reconstructing how it was done before, ASK the
repo: `harness_cli buscar "<terms>"` searches specs, plans, ADRs, lessons, impl,
review and the log, ranked from most curated to most raw (lessons and profile
first, `history.md` last). `--json` exposes the score, `--todos` removes the
20-result cap. Read-only, no index, no LLM, no hub.

The user profile (`docs/perfil-usuario.md`) states how the USER wants to work.
When it has entries the installer injects them into this surface inside a
`harness:perfil` block: read it and respect it in the plan, the implementation
and the verdict. To propose a new entry, `harness_cli perfil sugerir` gathers
what was already decided and prints the contract; then SHOW the entry to the
user, ASK, and only with their yes run `perfil add --texto "..." --yes`. Hard
limit of 1500 characters and no secrets: the file is versioned.

Efficient Kimi Code CLI usage: see `docs/kimi-cli-uso-eficiente.md` (context
exclusions, fixed project rules in `.kimirules`, file-scoped prompts, `/new`
between tasks).

How to write the PRD: see `docs/prd/COMO-ESCRIBIR-UN-PRD.md` (the story first,
the size the change decides, nested PRDs, and the hard rule: pseudo-code and
explanations, never final code). Each `docs/spec-feature-<id>-<slug>.md` is the
PRD of that change and already ships those sections.

Nested PRDs are real folders under `docs/prd/`: create one with
`... harness_cli.ps1 prd add --name <part> [--parent <path>]` (it ships the 12
sections and links itself into its parent), draw the tree with
`... harness_cli.ps1 prd tree`, and load each milestone with
`... harness_cli.ps1 add ... --prd <path>` so the spec cites its source PRD.
Closing the feature as done marks that milestone and logs it in the PRD; the
body of the PRD is never rewritten by the harness.

Atlassian integration (only when `atlassian.json` exists in the root): every
flow transition leaves an intent in `__HREL__progress/atlassian/outbox/`. Drain it with
`... harness_cli.ps1 atlassian drain`, execute each call with your Atlassian MCP
and record the created key with `... harness_cli.ps1 atlassian ack --intent <id>
--key <ADR-n>`. With a token configured the harness does it alone: every flow transition spawns
a detached worker that applies pending intents and republishes the documents
(`atlassian apply`, `sprint start|close`, `publish` and `backfill` remain
available by hand). Load a bugfix with `add --kind bug`. Turn the automatic push
off with `HARNESS_ATLASSIAN_AUTO=0` or `"auto": false` in `atlassian.json`. If the binding is
missing and the user wants Jira/Confluence, ASK which project and space this
repo belongs to: the harness never guesses. See `docs/atlassian-integracion.md`.

Parallel features (#47): `start` gives each feature its own GitFlow branch
(`feature/<id>-<slug>`, or `bugfix/` when loaded with `add --kind bug`) and its
own worktree next to the repo (`../<repo>-wt/<id>-<slug>`), so two
implementations never share files. Work INSIDE that worktree: commands infer the
feature from the folder and the spec, plan and evidence live on its branch. The
backlog and `__HREL__progress/` stay single (main checkout). Closing as `done`
requires the target branch — `close --feature <id> --status done --to <branch>`;
the harness never picks it: without `--to` it refuses and you must ASK THE USER.
`start --sin-worktree` keeps the classic single-folder mode.

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

# Feature #19: inyecta el perfil del usuario en una superficie ya escrita, entre
# marcadores propios y de forma IDEMPOTENTE. El bloque lo RENDERIZA el binario
# (`perfil bloque`), no este script: asi el formato y el parseo del perfil viven
# en un solo lugar y los dos instaladores no pueden divergir. Paridad con
# inject_perfil_block de setup_harness.sh.
#
# Sin perfil, sin entradas o sin binario utilizable no se toca nada.
function Inject-PerfilBlock {
    param([string]$Target)
    if (-not (Test-Path -LiteralPath $Target)) { return }
    $bin = Join-Path $script:HarnessDir "harness.exe"
    if (-not (Test-Path -LiteralPath $bin)) { return }
    $bloque = ""
    try {
        $previo = $env:HARNESS_REPO_ROOT
        $env:HARNESS_REPO_ROOT = $script:SurfaceDir
        $bloque = (& $bin perfil bloque 2>$null) -join "`n"
        $env:HARNESS_REPO_ROOT = $previo
    } catch {
        return
    }
    if ([string]::IsNullOrWhiteSpace($bloque)) { return }
    $lineas = @(Get-Content -LiteralPath $Target)
    $limpias = New-Object System.Collections.Generic.List[string]
    $skip = $false
    foreach ($linea in $lineas) {
        if ($linea.Trim() -eq "<!-- harness:perfil:inicio -->") { $skip = $true; continue }
        if ($linea.Trim() -eq "<!-- harness:perfil:fin -->")    { $skip = $false; continue }
        if (-not $skip) { $limpias.Add($linea) }
    }
    while ($limpias.Count -gt 0 -and [string]::IsNullOrWhiteSpace($limpias[$limpias.Count - 1])) {
        $limpias.RemoveAt($limpias.Count - 1)
    }
    $contenido = ($limpias -join "`n") + "`n`n" + $bloque.TrimEnd() + "`n"
    Write-HarnessText -Path $Target -Content $contenido
}

function Write-AgentDefinitions {
    if (-not $script:WithSubagents) {
        return
    }
    $rolesReadme = Join-Path $script:HarnessDir "roles/README.md"
    $rolesReadmeBody = (Get-Content -LiteralPath $rolesReadme -Raw).Replace("__HREL__", $script:Hrel)
    Write-HarnessText -Path $rolesReadme -Content $rolesReadmeBody

    $descriptions = @{
        leader = "Coordinates scope, impact, and the durable spec + plan with AC-n. Does not implement code."
        implementer = "Implements one concrete unit from the plan and records durable evidence."
        reviewer = "Verifies tests, impact, per-AC evidence, checkpoints, and Git state before closure."
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

        # Feature #9: los TRES roles usan workspace-write. Codex no admite
        # allowlist de herramientas, y leader/reviewer deben escribir sus
        # entregables en docs/ (spec, plan, veredicto): con read-only el sandbox
        # responde "Operation not permitted". No es mas laxo que Claude, donde
        # esos roles ya escriben via Bash; la disciplina la pone el prompt.
        $sandbox = "workspace-write"
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

        # Kimi Code CLI (v0.29.x): mismo formato Markdown+frontmatter que Claude
        # (verificado empiricamente); tools = allowlist por rol (decision usuario
        # 2026-07-28), identica a la de Claude, asi que se reutiliza $tools.
        $kimi = @"
---
name: $role
description: $($descriptions[$role])
tools: $tools
---

$body
"@
        Write-HarnessText -Path (Join-Path $script:SurfaceDir ".kimi-code/agents/$role.md") -Content $kimi
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
                # Paridad con harness_check.sh (superficie sh): el Stop aplica
                # AMBOS gates, plan y spec. harness_cli.ps1 delega en harness.exe
                # con `exit $LASTEXITCODE`; `& $cli` NO lanza excepcion con exit
                # != 0, asi que probamos el codigo a mano y lanzamos para que el
                # bloque `catch` emita la decision de block (exit 2 = stale, o
                # regla require_spec_approved activa con spec sin aprobar).
                & $cli check-plan
                if ($LASTEXITCODE -eq 2) { throw "Plan desactualizado (modificado por otro LLM). Re-lee el plan antes de continuar." }
                & $cli check-spec
                if ($LASTEXITCODE -eq 2) { throw "Spec sin aprobar o modificado. Si esta en draft, mostrale el spec al USUARIO, preguntale si lo aprueba y con su SI registra 'harness_cli.ps1 approve-spec --yes'." }
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

# Kimi Code CLI (v0.29.x): hooks SOLO globales (paridad exacta con
# write_kimi_hooks de setup_harness.sh). Unica excepcion a la regla de no
# escribir fuera del proyecto (decision usuario 2026-07-28), blindada: backup
# previo, bloque delimitado por marcadores propios, reemplazo idempotente SOLO
# entre marcadores, validacion best-effort con `kimi doctor` + rollback y
# guard por proyecto ($PWD/bin/harness-hook). El comando del bloque usa
# sintaxis POSIX: si Kimi en Windows no lo ejecuta via sh, el hook global
# queda best-effort alli (documentado en UPDATING.md). Nunca cambia el exit
# del setup. Solo se escribe si se detecta Kimi; -NoKimi lo excluye.
function Write-KimiGlobalHooks {
    $kimiHome = if ($env:KIMI_CODE_HOME) { $env:KIMI_CODE_HOME } else { Join-Path $HOME ".kimi-code" }
    $kimiConfig = Join-Path $kimiHome "config.toml"
    if ($NoKimi) {
        Write-HarnessLog INFO "Kimi Code: global hooks block skipped (-NoKimi); project artifacts are still generated."
        $script:Counters.skipped++
        return
    }
    $kimiCommand = Get-Command kimi -ErrorAction SilentlyContinue
    $kimiLocal = $null
    foreach ($candidate in @("bin/kimi", "bin/kimi.exe", "bin/kimi.cmd")) {
        $candidatePath = Join-Path $kimiHome $candidate
        if (Test-Path -LiteralPath $candidatePath -PathType Leaf) {
            $kimiLocal = $candidatePath
            break
        }
    }
    if (-not $kimiCommand -and -not $kimiLocal) {
        Write-HarnessLog INFO "Kimi Code CLI not detected; leaving $kimiConfig untouched (project artifacts are still generated)."
        $script:Counters.skipped++
        return
    }
    if ($DryRun) {
        Write-HarnessLog INFO "[DRY-RUN] Write the delimited harness hooks block into: $kimiConfig"
        $script:Counters.created++
        return
    }

    $beginMarker = "# >>> harness-process hooks >>>"
    $endMarker = "# <<< harness-process hooks <<<"
    $block = @'
# >>> harness-process hooks >>>
# Bloque gestionado por Harness Process (setup_harness.sh / setup_harness.ps1).
# Kimi Code solo soporta hooks GLOBALES: este bloque es compartido por todos
# los proyectos de la maquina y cada comando es un guard que solo actua si el
# directorio actual tiene un arnes instalado ($PWD/bin/harness-hook); en
# cualquier otro proyecto es un no-op silencioso. Re-instalar el arnes lo
# regenera; para quitarlo a mano borra desde este marcador hasta el de cierre
# (ver UPDATING.md). No edites dentro del bloque: se reemplaza completo.

[[hooks]]
event = "SessionStart"
command = "[ -x \"$PWD/bin/harness-hook\" ] || exit 0; HARNESS_REPO_ROOT=\"$PWD\" exec \"$PWD/bin/harness-hook\" plain session-start"
timeout = 120

[[hooks]]
event = "PostToolUse"
matcher = "Edit|Write"
command = "[ -x \"$PWD/bin/harness-hook\" ] || exit 0; HARNESS_REPO_ROOT=\"$PWD\" exec \"$PWD/bin/harness-hook\" plain post-tool"
timeout = 30

[[hooks]]
event = "Stop"
command = "[ -x \"$PWD/bin/harness-hook\" ] || exit 0; HARNESS_REPO_ROOT=\"$PWD\" exec \"$PWD/bin/harness-hook\" plain stop"
timeout = 120
# <<< harness-process hooks <<<
'@

    $existed = Test-Path -LiteralPath $kimiConfig -PathType Leaf
    $rollback = $null
    $raw = ""
    if ($existed) {
        # Backup ANTES de tocar el archivo (mecanismo bkp/ de siempre) + copia
        # en memoria para el rollback de la validacion doctor.
        Backup-HarnessPath -Target $kimiConfig
        $rollback = [IO.File]::ReadAllText($kimiConfig)
        $raw = $rollback
    }
    New-Item -ItemType Directory -Path $kimiHome -Force | Out-Null

    # Remueve SOLO el bloque delimitado previo (si existe) preservando el resto
    # byte a byte, y anexa la version fresca al final con newline garantizado.
    $beginIdx = $raw.IndexOf($beginMarker)
    if ($beginIdx -ge 0) {
        $endIdx = $raw.IndexOf($endMarker, $beginIdx)
        if ($endIdx -ge 0) {
            $afterEnd = $endIdx + $endMarker.Length
            if ($afterEnd -lt $raw.Length -and $raw[$afterEnd] -eq "`r") { $afterEnd++ }
            if ($afterEnd -lt $raw.Length -and $raw[$afterEnd] -eq "`n") { $afterEnd++ }
            $raw = $raw.Substring(0, $beginIdx) + $raw.Substring($afterEnd)
        }
        else {
            $raw = $raw.Substring(0, $beginIdx)
        }
    }
    if ($raw.Length -gt 0 -and -not $raw.EndsWith("`n")) {
        $raw += "`n"
    }
    Write-HarnessText -Path $kimiConfig -Content ($raw + $block + "`n")

    # Validacion best-effort del TOML resultante (verificado en v0.29.2: doctor
    # sale 0 con config valido aunque falte login/modelo). Si quedo invalido:
    # restaurar el estado previo (o retirar el archivo recien creado), avisar y
    # seguir sin cambiar el exit del setup.
    $doctorTool = if ($kimiCommand) { $kimiCommand.Source } else { $kimiLocal }
    $doctorOk = $true
    $previousKimiHome = $env:KIMI_CODE_HOME
    try {
        $env:KIMI_CODE_HOME = $kimiHome
        & $doctorTool doctor *> $null
        if ($LASTEXITCODE -ne 0) { $doctorOk = $false }
    }
    catch {
        $doctorOk = $false
    }
    finally {
        $env:KIMI_CODE_HOME = $previousKimiHome
    }
    if (-not $doctorOk) {
        if ($null -ne $rollback) {
            [IO.File]::WriteAllText($kimiConfig, $rollback, [Text.UTF8Encoding]::new($false))
        }
        else {
            Remove-Item -LiteralPath $kimiConfig -Force -ErrorAction SilentlyContinue
        }
        Write-HarnessLog WARN "'kimi doctor' reporto config invalido tras escribir el bloque del arnes; se restauro el estado previo de $kimiConfig. Revisa ese archivo (hay backup en $($script:BackupDir)) y re-corre el instalador; el resto del setup continua."
        $script:Counters.skipped++
        return
    }
    Write-HarnessLog OK "Kimi Code global hooks block written: $kimiConfig (delimited block; backup under $($script:BackupDir))."
    $script:Counters.created++
}

function Write-AgentLaunchers {
    foreach ($agent in @("claude", "codex", "gemini", "grok", "kimi", "antigravity")) {
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
        # Kimi Code: SOLO el artefacto de proyecto. El bloque de hooks GLOBALES
        # en KIMI_CODE_HOME/config.toml NO se toca (decision usuario
        # 2026-07-28): es compartido por todos los proyectos con arnes de la
        # maquina y -Reset es por-proyecto. Remocion manual en UPDATING.md.
        ".kimi-code/agents",
        "bin/harness-hook",
        "bin/harness-hook.ps1",
        "bin/harness-claude",
        "bin/harness-codex",
        "bin/harness-gemini",
        "bin/harness-grok",
        "bin/harness-kimi",
        "bin/harness-antigravity",
        "bin/harness-claude.ps1",
        "bin/harness-codex.ps1",
        "bin/harness-gemini.ps1",
        "bin/harness-grok.ps1",
        "bin/harness-kimi.ps1",
        "bin/harness-antigravity.ps1"
    )
    $targets += @(
        (Join-Path $script:HarnessDir "roles"),
        (Join-Path $script:HarnessDir "progress"),
        (Join-Path $script:HarnessDir "CHECKPOINTS.md"),
        (Join-Path $script:HarnessDir "feature_list.json")
    )
    # Solo los docs GENERADOS (desde templates/docs/), en el docs/ de la RAIZ. NO
    # barremos docs/ entero: ahi conviven la constitution del usuario ("un
    # reinstall NUNCA lo pisa"), las planillas maestras de docs/prd/ (PRD y SDD
    # del proyecto: NO se listan aqui a proposito, son documento del usuario) y
    # los artefactos de feature
    # (spec-*/plan-*/impl-*/review-*). Se agregan tambien las rutas viejas del
    # arnes para limpiar instalaciones anteriores a la migracion (en layout root
    # ambas coinciden y el duplicado simplemente no existe).
    foreach ($harnessDoc in $script:HarnessDocs) {
        $targets += @(
            (Join-Path $script:SurfaceDir "docs/$harnessDoc"),
            (Join-Path $script:HarnessDir "docs/$harnessDoc")
        )
    }
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
        # TODA la documentacion del proceso (constitution, docs del arnes, specs y
        # planes) vive en el docs/ de la RAIZ (SurfaceDir). El arnes ya no crea su
        # propio docs/; en layout root ambos son el mismo directorio.
        foreach ($directory in @(
            (Join-Path $script:HarnessDir "roles"),
            (Join-Path $script:HarnessDir "progress"),
            (Join-Path $script:SurfaceDir "docs"),
            (Join-Path $script:SurfaceDir "docs/prd"),
            (Join-Path $script:SurfaceDir ".claude/agents"),
            (Join-Path $script:SurfaceDir ".codex/agents"),
            (Join-Path $script:SurfaceDir ".gemini/agents"),
            (Join-Path $script:SurfaceDir ".kimi-code/agents")
        )) {
            Ensure-Directory -Path $directory
        }
    }
    # Con el docs/ de la raiz ya creado, mover lo que haya quedado de
    # instalaciones anteriores. Fuera del guard de subagentes (igual que
    # migrate_harness_docs en setup_harness.sh): una instalacion previa pudo
    # dejar docs aunque ahora se instale con -NoSubagents. No-op en
    # instalaciones nuevas y en layout root.
    Move-HarnessDocsToRoot

    $layoutMarker = Join-Path $script:HarnessDir ".harness_layout"
    $backendMarker = Join-Path $script:HarnessDir ".harness_backend"
    Backup-HarnessPath -Target $layoutMarker
    Backup-HarnessPath -Target $backendMarker
    Write-HarnessText -Path $layoutMarker -Content ($script:Layout + [Environment]::NewLine)
    Write-HarnessText -Path $backendMarker -Content ("postgres" + [Environment]::NewLine)

    Initialize-HarnessEnvTemplate
    Write-AtlassianBinding

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
        # Los docs del arnes NO se listan aqui: viven en el docs/ de la RAIZ y,
        # como la constitution, se siembran solo si faltan (no se respaldan ni se
        # regeneran en cada reinstall).
        $generatedAssets += @(
            "CHECKPOINTS.md",
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
        # Constitution del proyecto: documento del USUARIO. Se siembra en el docs/
        # de la RAIZ (SurfaceDir) SOLO si falta; un reinstall NUNCA lo pisa (por eso
        # no esta en $generatedAssets). Install-HarnessAssetIfMissing apunta a
        # HarnessDir, asi que aqui se usa un destino explicito bajo SurfaceDir.
        $constitutionDest = Join-Path $script:SurfaceDir "docs/constitution.md"
        if (-not (Test-Path -LiteralPath $constitutionDest)) {
            Install-HarnessAsset -Asset "docs/constitution.md" -Destination $constitutionDest
        }
        # Docs del arnes: mismo criterio que la constitution (decision usuario
        # 2026-07-24). Comparten carpeta con la documentacion del equipo, asi que
        # se siembran SOLO si faltan y un reinstall no pisa un docs/conventions.md
        # propio. Para refrescar la plantilla: borrar el archivo, o usar -Force
        # (que por contrato sobrescribe sin backup).
        foreach ($harnessDoc in $script:HarnessDocs) {
            $docDest = Join-Path $script:SurfaceDir "docs/$harnessDoc"
            if ((-not (Test-Path -LiteralPath $docDest)) -or $Force) {
                Install-HarnessAsset -Asset "docs/$harnessDoc" -Destination $docDest
            } else {
                $script:Counters.skipped++
            }
        }
        # Planillas maestras PRD/SDD: documentos del USUARIO. Se siembran SOLO si
        # faltan y ni -Force las pisa: a diferencia de las plantillas del arnes,
        # aqui lo escrito es el proyecto en si, no una plantilla refrescable.
        foreach ($prdDoc in $script:PrdDocs) {
            $prdDest = Join-Path $script:SurfaceDir "docs/prd/$prdDoc"
            if (-not (Test-Path -LiteralPath $prdDest)) {
                Install-HarnessAsset -Asset "docs/prd/$prdDoc" -Destination $prdDest
            } else {
                $script:Counters.skipped++
            }
        }
        # Feature #19: documentos del USUARIO en el docs/ de la RAIZ (el perfil).
        # Mismo criterio: solo-si-falta, sin pisar y fuera del reset.
        foreach ($userDoc in $script:UserDocs) {
            $userDest = Join-Path $script:SurfaceDir "docs/$userDoc"
            if (-not (Test-Path -LiteralPath $userDest)) {
                Install-HarnessAsset -Asset "docs/$userDoc" -Destination $userDest
            } else {
                $script:Counters.skipped++
            }
        }
        # Dotfiles Kimi (.kimiignore/.kimirules): documentos del USUARIO en la
        # RAIZ. Mismo criterio que PRD/SDD: se siembran SOLO si faltan y ni
        # -Force los pisa (lo escrito ahi son las reglas del proyecto, no una
        # plantilla refrescable). Paridad con KIMI_DOTFILES de setup_harness.sh.
        foreach ($kimiDotfile in $script:KimiDotfiles) {
            $dotfileDest = Join-Path $script:SurfaceDir $kimiDotfile
            if (-not (Test-Path -LiteralPath $dotfileDest)) {
                Install-HarnessAsset -Asset $kimiDotfile -Destination $dotfileDest
            } else {
                $script:Counters.skipped++
            }
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
        "bin/harness-kimi.ps1",
        "bin/harness-antigravity.ps1"
    )
    if ($script:WithSubagents) {
        foreach ($role in @("leader", "implementer", "reviewer")) {
            $surfaceBackups += ".claude/agents/$role.md"
            $surfaceBackups += ".codex/agents/$role.toml"
            $surfaceBackups += ".gemini/agents/$role.md"
            $surfaceBackups += ".kimi-code/agents/$role.md"
        }
    }
    foreach ($relative in $surfaceBackups) {
        Backup-HarnessPath -Target (Join-Path $script:SurfaceDir $relative)
    }

    Write-AgentDefinitions
    foreach ($surface in @("CLAUDE.md", "AGENTS.md", "GEMINI.md", "LLM.md")) {
        $target = Join-Path $script:SurfaceDir $surface
        Write-AgentSurface -Target $target
        Inject-PerfilBlock -Target $target
    }
    Write-PowerShellHookRuntime
    Write-AgentHooks
    Write-KimiGlobalHooks
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
