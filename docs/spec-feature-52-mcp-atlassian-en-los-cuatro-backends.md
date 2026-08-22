# Spec - Feature #52: mcp_atlassian_en_los_cuatro_backends

Estado: approved
Aprobado: 2026-08-22T14:10:49Z por USUARIO (confirmacion explicita) - Alan aprobo el spec de la feature #52 en el chat (13 AC): el instalador deja el MCP de Atlassian por proyecto en Claude, Kimi y Grok, imprime los dos comandos de Codex (que no admite alcance de proyecto), no hace el OAuth y no pisa lo existente
Plan: docs/plan-feature-52-mcp-atlassian-en-los-cuatro-backends.md
PRD: docs/prd/PRD-master.md
Constitution: docs/constitution.md
Metodo: docs/prd/COMO-ESCRIBIR-UN-PRD.md (este spec es el PRD del cambio)

## La historia (antes -> despues)

ANTES: el arnes sabe hablar con Atlassian desde la feature #15 y lo hace solo
desde la #16. Pero la mitad agente de esa integracion — `atlassian drain`, que
imprime un plan de llamadas MCP — asume algo que el arnes nunca instalo: que el
agente TIENE el MCP de Atlassian conectado. Alan lo comprobo a mano: Claude lo
tenia, y Codex, Kimi y Grok no. Conectarlos costo una tarde de arqueologia — un
formato distinto por CLI (`mcp.json`, `config.toml`, plugin curado), un flujo de
OAuth distinto, y un descubrimiento por backend: Grok no implementa OAuth sobre
HTTP y necesita el bridge `mcp-remote`; Codex necesita ADEMAS el plugin
`atlassian-rovo`, sin el cual el agente responde "necesito que instales el
plugin" aunque el servidor este conectado y autenticado.

Nada de eso quedo escrito en ningun lado. El proximo proyecto —o la proxima
maquina— empieza de cero.

DESPUES: instalar el arnes en un repo que tiene binding de Atlassian deja
tambien el MCP configurado para los agentes que trabajan en ESE repo. El
instalador escribe la configuracion de proyecto de cada backend que la admite,
con el formato que realmente usa, y para el que no la admite imprime el comando
exacto en vez de tocar la configuracion global del usuario a sus espaldas. La
autorizacion (OAuth) sigue siendo del usuario — el arnes no la puede hacer y no
la finge: dice, por CLI, que comando correr.

## Hoy -> Como va a funcionar

```
HOY                                    DESPUES
setup_harness.sh                       setup_harness.sh (con atlassian.json)
  |__ hooks y superficies por backend    |__ hooks y superficies por backend
  |__ (nada de MCP)                      |__ .mcp.json               (Claude, proyecto)
                                         |__ .kimi-code/mcp.json     (Kimi, proyecto)
                                         |__ .grok/config.toml       (Grok, proyecto, via mcp-remote)
                                         |__ Codex: NO toca ~/.codex — imprime los dos comandos
                                         |__ imprime como autorizar en cada CLI

`atlassian drain` asume el MCP        el MCP esta donde `drain` lo necesita
```

## Recorridos de usuario (priorizados)

- P1: Como Alan instalando el arnes en un proyecto nuevo con Jira, quiero que
  los agentes que use ahi ya tengan el MCP de Atlassian, para que
  `atlassian drain` sea ejecutable sin una tarde de configuracion.
- P1: Como Alan, quiero que el arnes NO toque la configuracion global de mis
  CLIs sin pedirmelo, para que instalar un arnes en un repo no cambie como se
  comportan mis herramientas en todos los demas.
- P1: Como Alan, quiero que lo que el arnes no puede hacer (el OAuth) me lo diga
  con el comando exacto, para no tener que buscarlo en la documentacion de cada
  CLI.
- P2: Como Alan con MCP ya configurado a mano, quiero que el instalador respete
  lo que tengo, para no perder mis ajustes.

## Criterios de aceptacion (Given/When/Then)

### Cuando corresponde

- AC-1: Given un repo SIN `atlassian.json`, When corro el instalador, Then no se
  escribe ninguna configuracion de MCP: sin binding no hay nada que conectar.
- AC-2: Given un repo CON binding de Atlassian, When corro el instalador, Then
  se escribe la configuracion de proyecto de los backends que la admiten y se
  informa que se hizo.
- AC-3: Given `--no-mcp-atlassian`, When corro el instalador con binding, Then
  no se escribe ninguna configuracion de MCP (valvula de escape).

### Por backend, con su formato real

- AC-4: Given un repo con binding, When corro el instalador, Then queda
  `.mcp.json` en la raiz del proyecto con el servidor `atlassian`
  (`https://mcp.atlassian.com/v1/mcp/authv2`), que es el scope de proyecto que
  Claude Code aprueba por repo.
- AC-5: Given lo mismo, Then queda `.kimi-code/mcp.json` con el mismo servidor
  en formato `mcpServers` (HTTP directo: Kimi resuelve el OAuth por su cuenta).
- AC-6: Given lo mismo, Then queda `.grok/config.toml` con el servidor en
  formato `[mcp_servers.atlassian]` **via `npx -y mcp-remote@latest <url>`**,
  porque el cliente HTTP de Grok no implementa el flujo OAuth de MCP
  (verificado: falla con `OAuth authorization required`, igual que con otros
  servidores OAuth).
- AC-7: Given que Codex NO admite MCP por proyecto, When corro el instalador,
  Then NO se toca `~/.codex/config.toml`: se imprimen los DOS comandos que hacen
  falta (`codex mcp add atlassian --url ...` y
  `codex plugin add atlassian-rovo@openai-curated`), explicando que el segundo
  es imprescindible aunque el servidor este conectado.

### Respetar lo que ya hay, y no fingir lo que no se puede

- AC-8: Given que el archivo de un backend ya existe con un servidor
  `atlassian`, When corro el instalador, Then no se pisa y se informa que se
  respeto lo existente.
- AC-9: Given que el archivo existe con OTROS servidores MCP, When corro el
  instalador, Then se agrega `atlassian` conservando los demas (no se
  reemplaza el archivo).
- AC-10: Given cualquier caso, When termina el instalador, Then imprime como
  autorizar en cada CLI (`/mcp` en Claude, `/mcp-config login atlassian` en
  Kimi, primer uso en Grok, `codex mcp add` en Codex) y deja claro que el arnes
  NO hace el OAuth.
- AC-11: Given el instalador de Windows, When corro `setup_harness.ps1`, Then
  escribe exactamente los mismos archivos con el mismo contenido (paridad
  verificada por assert).

### Verificacion

- AC-12: Given el repo del arnes, When corro `cargo test`,
  `cargo clippy -- -D warnings`, `bash tests/setup_smoke.sh` y
  `harness_check.sh`, Then los cuatro terminan limpios, con asserts del smoke
  para: sin binding no se escribe nada, con binding se escriben los tres
  archivos, no se pisa lo existente y se conservan otros servidores.
- AC-13: Given los archivos que escribe el instalador, When los uso de verdad en
  este repo, Then al menos un backend levanta el MCP desde la configuracion de
  PROYECTO (no la global) y responde una consulta de solo lectura.

## Los datos que se tocan

- disparador: `setup_harness.{sh,ps1}` cuando existe `atlassian.json` con
  proyecto Jira.
- interruptor: `--no-mcp-atlassian` (y la ausencia de binding, que ya apaga
  todo).
- candado: la presencia de un servidor `atlassian` en el archivo del backend.
- `.mcp.json` (raiz del proyecto, formato Claude Code).
- `.kimi-code/mcp.json` (formato `mcpServers`).
- `.grok/config.toml` (formato `[mcp_servers.<nombre>]`, stdio con mcp-remote).
- `~/.codex/config.toml`: **NO se toca**. Solo se imprimen los comandos.
- Ninguno lleva credenciales: la URL del MCP es publica y el OAuth lo hace cada
  CLI contra Atlassian.

## Pseudo-codigo (el acuerdo)

```
CUANDO el instalador termina de escribir las superficies

  ¿hay atlassian.json con proyecto?   -> si no, no hacemos nada
  ¿pidieron --no-mcp-atlassian?       -> si si, no hacemos nada

  para cada backend que admite configuracion POR PROYECTO:
     ¿ya tiene un servidor atlassian? -> si si, lo respetamos y lo decimos
     si no, lo agregamos conservando los demas servidores

  para Codex (que no admite proyecto):
     imprimimos los dos comandos, no tocamos su config global

  ENTONCES decimos como autorizar en cada CLI,
           con la restriccion de que el arnes NUNCA hace el OAuth
           ni escribe credenciales.
```

Promesas: sin binding no pasa nada · no se pisa lo que ya tenes · la
configuracion global de tus CLIs no se toca sin pedirtelo · el OAuth es tuyo y
se te dice como hacerlo.

## No funcionales

- SLOs: son tres archivos chicos; no agrega tiempo perceptible al instalador.
- Seguridad (Articulo 4): no se escriben credenciales en ningun archivo. La
  unica configuracion global posible (Codex) queda fuera: se imprime el comando
  para que la corra el usuario.
- Observabilidad: el instalador dice que escribio, que respeto y que falta
  autorizar, por backend.

## Fuera de alcance

- Hacer el OAuth: es interactivo y del usuario.
- Instalar `mcp-remote` o Node: se asume `npx` disponible; si falta, se avisa.
- Configurar MCPs que no sean el de Atlassian.
- Tocar `~/.codex/config.toml`, `~/.kimi-code/config.toml` o `~/.grok/config.toml`
  del usuario.

## Observaciones (decisiones pendientes)

- OBS-1 [DECISION DEL IMPLEMENTER, registrada]: alcance de PROYECTO donde el CLI
  lo admite (Claude, Kimi, Grok) y nada de configuracion global para Codex. El
  arnes ya tiene un precedente de escritura global — los hooks de Kimi — y quedo
  documentado como "la unica excepcion": no conviene sumar otra sin que el
  usuario la pida.
- OBS-2 [HALLAZGO VERIFICADO, 2026-08-22]: Grok necesita `mcp-remote` porque su
  cliente HTTP no completa el OAuth (`OAuth authorization required`, reproducido
  tambien con otro servidor OAuth del usuario). Con el bridge: handshake OK y 40
  tools.
- OBS-3 [HALLAZGO VERIFICADO, 2026-08-22]: Codex necesita el plugin
  `atlassian-rovo@openai-curated` ADEMAS del servidor MCP. Con el servidor solo,
  el agente responde "necesito que instales/conectes el plugin"; con el plugin
  solo, "no hay acceso visible". Los dos juntos: responde. Por eso AC-7 imprime
  los dos comandos.
- OBS-4 [DECIDIDA por el USUARIO, 2026-08-22]: los archivos de MCP del proyecto
  quedan VERSIONABLES: el instalador no los agrega al `.gitignore`. Que el MCP
  del proyecto sea parte del repo es lo que hace que el proximo que clone no
  repita la arqueologia, y no llevan credenciales (solo la URL publica del MCP;
  el OAuth es de cada usuario contra Atlassian).
