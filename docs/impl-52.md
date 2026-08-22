# Evidencia de implementacion - Feature #52: mcp_atlassian_en_los_cuatro_backends

Spec: `docs/spec-feature-52-mcp-atlassian-en-los-cuatro-backends.md` (approved, 13 AC)
Plan: `docs/plan-feature-52-mcp-atlassian-en-los-cuatro-backends.md`

## Que se construyo

`write_mcp_atlassian()` en `setup_harness.sh` (+ `Write-McpAtlassian` en el
ps1): cuando el repo tiene `atlassian.json`, el instalador escribe la
configuracion de MCP **de proyecto** de cada backend que la admite, respeta lo
que ya haya y explica lo que el arnes no puede hacer.

## Evidencia por AC

| AC | Estado | Evidencia |
| --- | --- | --- |
| AC-1 sin binding, nada | OK | Smoke (`mcp-atlassian`): tras instalar sin `atlassian.json`, no existen `.mcp.json` ni `.grok/config.toml`. Probado ademas a mano en `/tmp/mcp52-test` |
| AC-2 con binding, se escribe | OK | Smoke: los tres archivos existen tras la instalacion. Salida real: `MCP Atlassian: .mcp.json / .kimi-code/mcp.json / .grok/config.toml` |
| AC-3 `--no-mcp-atlassian` | OK | Smoke (`mcp-atlassian-off`): con binding y el flag, no se escribe `.mcp.json` |
| AC-4 Claude | OK | `.mcp.json` con `mcpServers.atlassian.url = https://mcp.atlassian.com/v1/mcp/authv2` (assert de URL en el smoke) |
| AC-5 Kimi | OK | `.kimi-code/mcp.json` con el mismo formato `mcpServers` |
| AC-6 Grok via mcp-remote | OK | `.grok/config.toml` con `command = "npx"` y `mcp-remote@latest` (assert en el smoke), con el porque escrito en el propio archivo |
| AC-7 Codex no se toca | OK | Smoke: el log trae `codex mcp add atlassian` y `codex plugin add atlassian-rovo`; `~/.codex/config.toml` no se modifica (el instalador nunca lo abre) |
| AC-8 no pisa lo existente | OK | Smoke: con un `atlassian` propio (`mio.example`), el archivo queda intacto y el log dice `ya lo declara (respetado)` |
| AC-9 conserva otros servidores | OK | Smoke: se deja solo `otro`, se reinstala y quedan `['atlassian', 'otro']`. Reproducido tambien a mano |
| AC-10 como autorizar | OK | Smoke: el log trae `falta AUTORIZAR` y la linea por CLI (Claude `/mcp`, Kimi `/mcp-config login`, Grok primer uso, Codex `codex mcp add`) |
| AC-11 paridad ps1 | PARCIAL (documentado) | `Write-McpAtlassian` con los mismos archivos, `mcp-remote@latest` y el plugin de Codex; asserts en el smoke. **No ejecutado**: no hay PowerShell en esta maquina |
| AC-12 comandos oficiales | OK | `cargo test`, `clippy --all-targets -- -D warnings`, `setup_smoke.sh` y `harness_check.sh` (desde el checkout principal, que es el que tiene el binario compilado). Numeros en `docs/review-52.md` |
| AC-13 uso real desde la config de PROYECTO | OK | Ver abajo |

## AC-13: la prueba inequivoca

`grok inspect` desde el sandbox mostraba `atlassian (stdio)`, pero eso no probaba
nada: la config GLOBAL del usuario tambien tiene un servidor con ese nombre. Se
renombro el del proyecto y se volvio a mirar:

```
MCP Servers (4)
  atlassian (stdio)                        config
  atlassian-solo-de-este-proyecto (stdio)  config

Config Sources
  User:    /Users/alan/.grok/config.toml
  Project: /private/tmp/mcp52-test/.grok/config.toml
```

El nombre inventado solo existe en el archivo que escribio el instalador, asi
que el servidor sale de la configuracion de PROYECTO y no de la del usuario.

## De donde salieron las dos rarezas

No se dedujeron: se reprodujeron con los CLIs instalados el 2026-08-22.

- **Grok**: con URL directa, `grok mcp doctor` responde
  `handshake failed (... Auth error: OAuth authorization required ...)`, y falla
  igual con otro servidor OAuth del usuario — o sea, es del cliente, no de
  Atlassian. Con `mcp-remote`: `handshake OK (protocol 2025-11-25)` y `40 tools`.
- **Codex**: con el servidor MCP solo, el agente responde *"Necesito que
  instales/conectes el plugin Atlassian Rovo"*. Con el plugin solo (se quito el
  servidor para comprobarlo), responde *"No hay acceso visible a
  calpil.atlassian.net"*. Con los dos: `mcp: atlassian/getVisibleJiraProjects
  (completed)` y devuelve `ADR` y `SCRUM`.
