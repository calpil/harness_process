Aplicado: 2026-08-22T16:48:11Z por USUARIO (confirmacion explicita)

# Documentos al dia - Feature #56: paquete_de_contexto_para_implementar

Contesta CADA bloque con uno de los tres veredictos y despues corre
`sh harness_cli prd apply --feature 56`:

- `Veredicto: cambio` + `Antes:` y `Despues:` (texto LITERAL del documento)
- `Veredicto: ya-esta <archivo>:<L1>-<L2>` (el binario verifica la cita)
- `Veredicto: no-aplica <razon>` (la razon no puede estar vacia)

## Documento: docs/prd/PRD-master.md

Que cuenta: que se construye y por que
Presente en: -
Ausente en: docs/prd/PRD-master.md (no menciona 'paquete_de_contexto_para_implementar')
Veredicto: cambio
Antes:
| 10 | El MCP de Atlassian ya conectado en cada backend | mcp_atlassian_en_los_cuatro_backends | <O1> | Instalar el arnes en un repo con binding de Atlassian deja tambien el MCP por PROYECTO en los backends que lo admiten (`.mcp.json` de Claude, `.kimi-code/mcp.json` de Kimi y `.grok/config.toml` de Grok via `mcp-remote`, porque su cliente HTTP no completa el OAuth), y para Codex —que no admite alcance de proyecto— imprime los dos comandos (servidor + plugin `atlassian-rovo`, imprescindible) en vez de tocar su configuracion global; respeta lo que ya haya, no escribe credenciales y dice por CLI como autorizar | done (2026-08-22) |
Despues:
| 10 | El MCP de Atlassian ya conectado en cada backend | mcp_atlassian_en_los_cuatro_backends | <O1> | Instalar el arnes en un repo con binding de Atlassian deja tambien el MCP por PROYECTO en los backends que lo admiten (`.mcp.json` de Claude, `.kimi-code/mcp.json` de Kimi y `.grok/config.toml` de Grok via `mcp-remote`, porque su cliente HTTP no completa el OAuth), y para Codex —que no admite alcance de proyecto— imprime los dos comandos (servidor + plugin `atlassian-rovo`, imprescindible) en vez de tocar su configuracion global; respeta lo que ya haya, no escribe credenciales y dice por CLI como autorizar | done (2026-08-22) |
| 11 | Empezar con el material en la mano, no explorando | paquete_de_contexto_para_implementar | <O1> | `contexto --feature <id>` (o `--tema`) entrega el mapa —siguiendo el puntero si `architecture.md` apunta a otro archivo—, si ese mapa CUBRE el tema, el impacto del hub con limite, la edad del grafo (vencido a los 7 dias), la historia acotada, las lecciones que aplican y las features del mismo servicio; declara su tamaño y sus huecos, y el resumen sale solo en cada `start`. Disparador: un mapeo de 4 agentes y 693.6k tokens sobre un tema que el mapa no mencionaba | done (2026-08-22) |

## Documento: docs/prd/SDD-master.md

Que cuenta: como se construye, a nivel proyecto
Presente en: -
Ausente en: docs/prd/SDD-master.md (no menciona 'paquete_de_contexto_para_implementar')
Veredicto: cambio
Antes:
**El MCP se instala por proyecto; la autorizacion es del usuario** (feature #52).
Despues:
**El material se entrega y el vacio se dice** (feature #56). La #51 dejo de
hacer que el REVISOR explorara; esta hace lo mismo con el que IMPLEMENTA, y
agrega la parte que faltaba: avisar cuando no hay nada que entregar. Tres
decisiones que valen para cualquier feature futura que le de contexto a un
agente:

- **Los punteros se siguen y se verifican.** Un `architecture.md` que apunta a
  otro archivo se resuelve contra el directorio del documento, y si el destino
  no existe eso es un HUECO con la ruta que falta — un diagnostico distinto de
  "no hay mapa". Un puntero roto se lee como "aca no hay nada escrito" y manda a
  explorar el repo entero.
- **El vacio se declara, no se disimula.** Si el mapa no menciona el tema, el
  paquete lo dice con esas palabras y con los terminos que busco, para que un
  falso positivo se pueda diagnosticar de un vistazo. Y si la consulta no tiene
  terminos utiles, el aviso apunta a la consulta, no al mapa.
- **El aviso no depende de que alguien lo pida.** `start` imprime el resumen
  siempre, porque el caso donde mas importa —el paquete vacio— es justo el que
  nadie pediria (`promesas-estructurales-vs-disciplina`).

**El MCP se instala por proyecto; la autorizacion es del usuario** (feature #52).

## Documento: docs/architecture.md

Que cuenta: el mapa de lo que YA existe
Presente en: -
Ausente en: docs/architecture.md (no menciona 'paquete_de_contexto_para_implementar')
Veredicto: cambio
Antes:
## MCP de Atlassian por proyecto (feature #52)
Despues:
## Paquete de contexto (feature #56)

`rust/src/contexto.rs` + `harness contexto [--feature <id> | --tema "<texto>"]
[--max-lineas N] [--con-grafo] [--json]`: el gemelo de `revision`, del lado de
implementar. Junta el mapa de `docs/architecture.md` —resolviendo el puntero si
lo hay, contra el directorio del documento— y decide si **cubre** el tema
(terminos sin acentos, sin palabras vacias, minimo tres letras); el impacto del
hub consultado en un hilo con limite de 5s; la edad del grafo de
`graphify-out/graph.json` (vencido a los 7 dias); la historia de `buscar`
acotada a 12 hits; las lecciones cuyos triggers pegan con el tema; y las
features `done` del mismo servicio. Es de SOLO LECTURA, declara lo que recorta,
reporta su tamaño en lineas y tokens estimados, y lista cada hueco con el
comando que lo consigue.

`graphify query` NO se invoca por default (cuesta): solo con `--con-grafo` y si
el binario esta. `commands/start.rs` imprime el resumen del paquete en cada
`start`, incluido —sobre todo— cuando esta vacio.

## MCP de Atlassian por proyecto (feature #52)

