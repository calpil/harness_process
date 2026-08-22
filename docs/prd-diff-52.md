Aplicado: 2026-08-22T14:46:08Z por USUARIO (confirmacion explicita)

# Documentos al dia - Feature #52: mcp_atlassian_en_los_cuatro_backends

Contesta CADA bloque con uno de los tres veredictos y despues corre
`sh harness_cli prd apply --feature 52`:

- `Veredicto: cambio` + `Antes:` y `Despues:` (texto LITERAL del documento)
- `Veredicto: ya-esta <archivo>:<L1>-<L2>` (el binario verifica la cita)
- `Veredicto: no-aplica <razon>` (la razon no puede estar vacia)

## Documento: docs/prd/PRD-master.md

Que cuenta: que se construye y por que
Presente en: -
Ausente en: docs/prd/PRD-master.md (no menciona 'mcp_atlassian_en_los_cuatro_backends')
Veredicto: cambio
Antes:
| 9 | Revisar en serio sin que cueste una fortuna | revision_adversarial_y_modelos_por_rol | <O1> | Un modelo por rol de Claude (implementer `claude-opus-5`, lider y reviewer `claude-fable-5`, los tres `xhigh`) definido en la tabla de roles de los dos instaladores y tuneable por variable; el reviewer intenta REFUTAR cada AC y verifica por su cuenta lo que la evidencia declara verde; y `revision --feature <id>` arma el paquete minimo (AC + estado de verify + evidencia + archivos + diff + rutas protegidas) acotado por presupuesto, que declara lo que recorta y reporta su propio tamaño | done (2026-08-22) |
Despues:
| 9 | Revisar en serio sin que cueste una fortuna | revision_adversarial_y_modelos_por_rol | <O1> | Un modelo por rol de Claude (implementer `claude-opus-5`, lider y reviewer `claude-fable-5`, los tres `xhigh`) definido en la tabla de roles de los dos instaladores y tuneable por variable; el reviewer intenta REFUTAR cada AC y verifica por su cuenta lo que la evidencia declara verde; y `revision --feature <id>` arma el paquete minimo (AC + estado de verify + evidencia + archivos + diff + rutas protegidas) acotado por presupuesto, que declara lo que recorta y reporta su propio tamaño | done (2026-08-22) |
| 10 | El MCP de Atlassian ya conectado en cada backend | mcp_atlassian_en_los_cuatro_backends | <O1> | Instalar el arnes en un repo con binding de Atlassian deja tambien el MCP por PROYECTO en los backends que lo admiten (`.mcp.json` de Claude, `.kimi-code/mcp.json` de Kimi y `.grok/config.toml` de Grok via `mcp-remote`, porque su cliente HTTP no completa el OAuth), y para Codex —que no admite alcance de proyecto— imprime los dos comandos (servidor + plugin `atlassian-rovo`, imprescindible) en vez de tocar su configuracion global; respeta lo que ya haya, no escribe credenciales y dice por CLI como autorizar | done (2026-08-22) |

## Documento: docs/prd/SDD-master.md

Que cuenta: como se construye, a nivel proyecto
Presente en: -
Ausente en: docs/prd/SDD-master.md (no menciona 'mcp_atlassian_en_los_cuatro_backends')
Veredicto: cambio
Antes:
**Que revisar no cueste una fortuna** (feature #51). Verificar lo implementado
Despues:
**El MCP se instala por proyecto; la autorizacion es del usuario** (feature #52).
`atlassian drain` imprime un plan de llamadas MCP desde la feature #15, pero el
arnes nunca instalaba el MCP que ese plan asume. Tres decisiones que valen para
cualquier integracion futura por MCP:

- **Alcance de proyecto, nunca global.** El instalador escribe la configuracion
  MCP DEL REPO (`.mcp.json`, `.kimi-code/mcp.json`, `.grok/config.toml`) y para
  el backend que no admite alcance de proyecto (Codex) imprime el comando en vez
  de tocar la configuracion global del usuario. Instalar un arnes en un repo no
  cambia como se comportan sus herramientas en los demas.
- **El arnes no hace el OAuth y no lo finge.** Dice, por CLI, que comando correr
  y deja claro que esa parte es del usuario.
- **Las rarezas de cada backend se reproducen y se escriben.** Grok necesita el
  bridge `mcp-remote`; Codex necesita el plugin `atlassian-rovo` ADEMAS del
  servidor. Las dos se verificaron contra los CLIs instalados y quedaron en el
  spec como hallazgos, no como deducciones.

**Que revisar no cueste una fortuna** (feature #51). Verificar lo implementado

## Documento: docs/architecture.md

Que cuenta: el mapa de lo que YA existe
Presente en: -
Ausente en: docs/architecture.md (no menciona 'mcp_atlassian_en_los_cuatro_backends')
Veredicto: cambio
Antes:
## Paquete de revision (feature #51)
Despues:
## MCP de Atlassian por proyecto (feature #52)

`write_mcp_atlassian()` en `setup_harness.sh` (`Write-McpAtlassian` en el ps1):
con `atlassian.json` presente y sin `--no-mcp-atlassian`, escribe la
configuracion MCP de PROYECTO de cada backend que la admite — `.mcp.json`
(Claude), `.kimi-code/mcp.json` (Kimi) y `.grok/config.toml` (Grok, via
`npx -y mcp-remote@latest`, porque su cliente HTTP no completa el flujo OAuth de
MCP). Codex no admite alcance de proyecto: NO se toca `~/.codex/config.toml`, se
imprimen `codex mcp add atlassian --url ...` y
`codex plugin add atlassian-rovo@openai-curated`, que hacen falta los dos.
Respeta un servidor `atlassian` ya declarado, conserva los demas servidores del
archivo y no escribe credenciales: la URL del MCP es publica y el OAuth lo hace
cada CLI contra Atlassian.

## Paquete de revision (feature #51)

