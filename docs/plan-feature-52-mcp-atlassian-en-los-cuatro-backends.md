# Plan - Feature #52: mcp_atlassian_en_los_cuatro_backends

Estado: in_progress
Microservicios:
- harness

## Alcance

Que instalar el arnes en un repo con binding de Atlassian deje tambien el MCP
configurado para los agentes de ESE repo. Entra: `.mcp.json` (Claude),
`.kimi-code/mcp.json` (Kimi) y `.grok/config.toml` con `mcp-remote` (Grok), el
aviso con los dos comandos de Codex (que no admite alcance de proyecto), el
flag `--no-mcp-atlassian`, y las instrucciones de autorizacion por CLI. No
entra: hacer el OAuth, tocar configuraciones globales, ni otros MCPs.

## Peldano elegido

Se queda en el peldano de "el instalador escribe archivos de configuracion",
que es lo que ya hace con hooks y superficies: no hay comando nuevo, ni
dependencia, ni superficie nueva. Lo unico que baja de peldano seria tocar la
config global de Codex, y por eso NO se hace: se imprime el comando.

## Impacto entre microservicios

Un solo microservicio: `harness`. Todo cuelga de la existencia de
`atlassian.json`, asi que un repo sin binding no cambia en nada (AC-1).

## Consulta al grafo (graphify)

No hace falta: el cambio vive en `setup_harness.sh` / `setup_harness.ps1`, junto
a `write_atlassian_binding`, que ya sabe si hay binding.

## Delegacion (implementer)

- D1 [AC-1, AC-2, AC-3]: gate del bloque (binding presente + flag de escape) y
  reporte de lo que se hizo.
- D2 [AC-4, AC-5]: `.mcp.json` y `.kimi-code/mcp.json` (JSON, formato
  `mcpServers`).
- D3 [AC-6]: `.grok/config.toml` con `[mcp_servers.atlassian]` via
  `npx -y mcp-remote@latest`, con el porque anotado en el propio archivo.
- D4 [AC-7, AC-10]: aviso de Codex (los dos comandos) y las instrucciones de
  autorizacion por CLI.
- D5 [AC-8, AC-9]: no pisar un `atlassian` existente y conservar otros
  servidores al agregar.
- D6 [AC-11]: paridad en `setup_harness.ps1`.
- D7 [AC-12]: asserts en `tests/setup_smoke.sh` para los cuatro casos.
- D8 [AC-13]: verificacion real: levantar el MCP desde la config de PROYECTO.
- D9: documentar en `docs/atlassian-integracion.md` (+ `templates/`) y UPDATING.

## Criterios de cierre (reviewer)

- Evidencia por AC-n en `docs/impl-52.md`.
- Los cuatro comandos oficiales limpios.
- AC-13 probado de verdad, no por lectura: un backend usando la config de
  proyecto.

## Riesgos

- R1: escribir archivos en la raiz del proyecto destino (`.mcp.json`) puede
  sorprender. Mitigacion: solo con binding, con flag de escape, sin pisar lo
  existente y anunciando cada archivo escrito.
- R2: `mcp-remote` depende de `npx`. Mitigacion: si falta, se avisa en vez de
  escribir una config que no va a levantar.
- R3: los formatos de MCP de cada CLI pueden cambiar. Mitigacion: los tres se
  verificaron contra la doc oficial y contra el CLI instalado (2026-08-22), y
  quedan citados en el spec.

## Observaciones (decisiones pendientes)

- OBS-1 a OBS-3: ver el spec (alcance de proyecto, Grok con mcp-remote, Codex
  necesita ademas el plugin).
- OBS-4 [DECIDIDA 2026-08-22]: versionables (no van al `.gitignore`).

### Avance 2026-08-22T14:44:17Z
Re-sincronizado con el spec #52 tras anotar OBS-2/OBS-3 (hallazgos verificados de Grok y Codex) y OBS-4 (decision del usuario: archivos MCP versionables). Los 13 AC no cambiaron.

---
Cerrado: 2026-08-22T14:49:31Z - status=done - 
