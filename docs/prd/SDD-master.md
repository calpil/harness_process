# SDD Master - Harness Process

Estado: en uso
Ultima actualizacion: 2026-08-18
Producto: docs/prd/PRD-master.md
Constitution: docs/constitution.md

> Documento del USUARIO: el instalador lo siembra una sola vez y nunca lo pisa.
> Es el "como" maestro del proyecto: la arquitectura objetivo y las decisiones
> tecnicas que ninguna feature individual deberia re-litigar.
>
> Relacion con los otros documentos:
> - `docs/prd/PRD-master.md`: que se construye y por que.
> - Este archivo: como se construye, a nivel proyecto.
> - `docs/architecture.md`: el mapa de lo que YA existe (se actualiza solo).
> - `docs/spec-feature-*.md`: el detalle por feature, con sus AC-n.

## 1. Arquitectura objetivo

<Diagrama en texto o descripcion de los componentes y como se hablan. Marca que
existe hoy y que hay que construir.>

```
<componente A> --(<protocolo>)--> <componente B>
```

| Componente | Responsabilidad | Estado | Owner |
| --- | --- | --- | --- |
| <servicio> | <que hace, en una linea> | <existe / a construir> | <equipo> |

## 2. Stack y dependencias

- Lenguajes / runtimes: <...>
- Almacenamiento: <base de datos, colas, cache>
- Servicios externos: <APIs de terceros, con su modo de falla>

> Regla sugerida (ajusta en la constitution): ninguna dependencia nueva de
> runtime entra sin una decision registrada abajo.

## 3. Contratos entre componentes

<Interfaces que cruzan un limite de equipo o de servicio: endpoints, eventos,
esquemas. Un cambio aqui impacta a otros; se registra impacto antes de mergear.>

| Contrato | Productor | Consumidores | Versionado |
| --- | --- | --- | --- |
| <endpoint/evento> | <servicio> | <servicios> | <como se versiona> |

## 4. Decisiones tecnicas

**Un gate solo verifica lo que puede ejecutar** (feature #46). El comando que
mejor prueba una feature suele ser el mas verboso, y era justo el que no se
podia declarar: `verify` leia los pipes DESPUES de esperar al proceso, asi que
cualquier comando que pasara los ~64 KB del buffer trababa a los dos. Tres
decisiones:

- **Leer mientras corre, no despues.** Un hilo por descriptor, lanzado antes de
  esperar. Es la unica forma de que el productor no se bloquee.
- **Ningun camino puede esperar sin limite.** El timeout corta al proceso y una
  gracia corta corta a los lectores: si un nieto heredo el pipe, se reporta lo
  leido y se sigue. Cambiar un cuelgue por otro es no haber arreglado nada.
- **Lo que se recorta se declara.** Tope de 4 MB reteniendo la cola —donde estan
  los resumenes que deciden el estado— y una linea en el reporte diciendo
  cuanto quedo afuera y sobre que se midio.

**El arnes no se bloquea a si mismo, tambien en el guard** (feature #58).
`docs/rutas-protegidas.md` ya declaraba la regla —"la proteccion es contra las
herramientas del agente, no contra el binario"— pero el commit guard no la
aplicaba, y en un proyecto donde `docs/` es su propio repo eso bloqueaba el
turno en cada documento que el arnes escribia. Dos decisiones:

- **La exencion es por ARTEFACTO y por UBICACION, nunca por carpeta.** Un
  `docs/runbook.md` sigue bloqueando, y un `impl-notas.md` dentro de un
  microservicio tampoco se exime: el nombre solo no alcanza. Un gate que se
  relaja de mas es peor que uno estricto, porque nadie revisa lo que cree
  cubierto.
- **Cuando un gate se saltea algo, lo dice.** Una linea `[i]` con el repo y la
  razon. Un guard que se calla en silencio es indistinguible de uno apagado.

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
llego a costar 10 millones de tokens, casi todos gastados explorando el repo.
Dos decisiones que valen para cualquier feature futura que involucre a un
agente revisando:

- **El material se entrega, no se busca.** `revision --feature <id>` arma el
  paquete (AC + estado de verify + evidencia + archivos + diff + rutas
  protegidas) acotado por presupuesto, declara lo que recorta y reporta su
  propio tamaño antes de que alguien lo lea.
- **Un modelo por rol, en la tabla de roles de cada instalador.** El que escribe
  codigo piensa con Opus; el que planifica y el que revisa, con Fable; los tres
  en `xhigh`. `.claude/agents/*.md` es artefacto generado: editarlo a mano no
  sobrevive a la instalacion.

**Aislamiento entre features** (feature #47). Dos implementaciones simultaneas
no comparten archivos: cada feature vive en su rama GitFlow y en su worktree
hermano del repo. Tres decisiones que valen para cualquier feature futura que
toque el flujo:

- **El estado del arnes es unico.** `feature_list.json` y `progress/` se
  resuelven contra el repo PRINCIPAL (`git rev-parse --git-common-dir`) aunque
  el binario se invoque desde un worktree: el backlog no se bifurca nunca.
- **Los docs se resuelven DESDE la feature, no desde el directorio actual.**
  `HarnessPaths::para_feature()` apunta `docs/` al worktree de esa feature, para
  que su spec, su plan y su evidencia viajen con el merge de su rama.
- **El arnes nunca reescribe historia ni elige la rama destino.** Sin `--force`,
  sin rebase, sin squash y sin borrar ramas; el merge corre en un worktree
  temporal (no toca tu checkout) y `--to` lo decide el USUARIO.

**Como habla el arnes con un LLM** (feature #28, la unica parte que usa modelo).
La cadena es `HARNESS_CONSOLIDAR_CMD` -> primer CLI de una tabla corta
(`claude -p`, `kimi -p`) -> **skip limpio**. Apagada por default y de forma
estructural: sin `rules.consolidar_backend` no se resuelve backend ni se mira el
entorno. Tres decisiones que valen para cualquier feature futura con modelo:

- **Al modelo se le manda lo minimo.** Ve nombre, descripcion y triggers; nunca
  el cuerpo de una leccion. Lo peor que puede hacer es equivocarse, no filtrar.
- **El prompt viaja como item de argv, jamas por `sh -c`.** Por eso NO se reusa
  `verificacion::ejecutar`, que si corre con shell.
- **El modelo propone; lo que muta sale de argv.** La mitad que escribe se
  verifica sin backend y de forma determinista.

El tramo HTTP con API key **no esta implementado** y el mensaje de skip lo dice.

<Una fila por decision estructural. Si una decision es grande o polemica, mueve
el detalle a su propio ADR y enlazalo aqui.>

| # | Decision | Alternativas descartadas | Por que | Fecha |
| --- | --- | --- | --- | --- |
| D1 | <lo que se decidio> | <opcion B, opcion C> | <razon> | <YYYY-MM-DD> |

## 5. Datos

**Los estados de una feature** (`feature_list.json`). Son cinco y significan
cosas distintas: `pending` (sin empezar, `next` la ofrece), `in_progress` (una
sola a la vez), `done` (hecha, con spec aprobado y su evidencia), `blocked`
(trabada por algo externo) y `superseded` (el trabajo se hizo en OTRA feature,
que se nombra en `superseded_by` y se valida al cerrar). Solo `done` pasa por los
cuatro gates de cierre; `superseded` no cuenta ni en el numerador ni en el
denominador de `prd tree`, porque no es trabajo hecho ni pendiente.

- Entidades principales y su dueno: <...>
- Migraciones: <como se aplican y como se revierten>
- Retencion y datos sensibles: <que se guarda, cuanto tiempo, con que proteccion>

## 6. No funcionales

- SLOs: <latencia, disponibilidad, throughput objetivo>
- Seguridad: <autenticacion, autorizacion, manejo de secretos>
- Observabilidad: <logs, metricas, trazas; que se alerta y a quien>
- Costos: <limites o presupuesto que condicionan el diseno>

## 7. Estrategia de verificacion

<Como se prueba el sistema, mas alla de los tests de cada feature. Los comandos
concretos viven en `docs/verification.md`.>

- **Tests automaticos**: unitarios en `rust/src/**` (modulos `mod tests`, sobre
  todo para las funciones PURAS: parsear, planificar, diagnosticar, decidir),
  integracion en `rust/tests/cli_basics.rs` (el binario de verdad contra un
  sandbox `tempfile`), y chequeos de shell en `tests/*.sh` para lo que vive
  fuera de Rust (los dos instaladores, los espejos, el corpus real del repo).
- **Los AC se ejecutan**: cada AC-n de un spec puede declarar `Comando:`, y
  `harness_cli verify --feature <id>` los corre y escribe `docs/verify-<id>.md`.
  Con `require_verify_green`, `close --status done` LEE ese reporte —nunca
  ejecuta— y no deja cerrar con alguno bloqueando.
- **Un AC que no midio nada no cuenta como verificado**: `cargo test <nombre>`
  con un filtro que no matchea sale 0, y eso ya produjo un falso verde real. Por
  eso `verify` mira la salida ademas del exit code y marca `vacio` al AC que
  reconocidamente no ejecuto ningun caso. Sobre salidas que no son de libtest no
  opina: el estado no cambia.
- **Entornos**: solo local. El arnes no se despliega; se instala en el repo del
  proyecto con `setup_harness.sh` / `.ps1`, y `tests/parity_check.sh` verifica
  que los dos hagan lo mismo.
- **Criterio de "listo"**: los AC del spec en verde en su reporte, la suite
  completa y `cargo clippy -D warnings` limpios, los chequeos de `tests/` en
  verde, y `harness_check.sh` sin problemas.

## 8. Riesgos tecnicos

| Riesgo | Probabilidad | Impacto | Mitigacion |
| --- | --- | --- | --- |
| <riesgo> | <alta/media/baja> | <alto/medio/bajo> | <plan> |

## 9. Decisiones abiertas

<Igual que en el PRD: sin decision registrada, se pregunta al USUARIO antes de
implementar lo que dependa de ella.>

- <pregunta> — DECIDIDO (<usuario>, <fecha>): <respuesta>
- <pregunta> — ABIERTA
