Aplicado: 2026-08-22T17:07:33Z por USUARIO (confirmacion explicita)

# Documentos al dia - Feature #58: el_guard_no_bloquea_por_lo_que_escribe_el_arnes

Contesta CADA bloque con uno de los tres veredictos y despues corre
`sh harness_cli prd apply --feature 58`:

- `Veredicto: cambio` + `Antes:` y `Despues:` (texto LITERAL del documento)
- `Veredicto: ya-esta <archivo>:<L1>-<L2>` (el binario verifica la cita)
- `Veredicto: no-aplica <razon>` (la razon no puede estar vacia)

## Documento: docs/prd/PRD-master.md

Que cuenta: que se construye y por que
Presente en: -
Ausente en: docs/prd/PRD-master.md (no menciona 'el_guard_no_bloquea_por_lo_que_escribe_el_arnes')
Veredicto: cambio
Antes:
| 11 | Empezar con el material en la mano, no explorando | paquete_de_contexto_para_implementar | <O1> | `contexto --feature <id>` (o `--tema`) entrega el mapa —siguiendo el puntero si `architecture.md` apunta a otro archivo—, si ese mapa CUBRE el tema, el impacto del hub con limite, la edad del grafo (vencido a los 7 dias), la historia acotada, las lecciones que aplican y las features del mismo servicio; declara su tamaño y sus huecos, y el resumen sale solo en cada `start`. Disparador: un mapeo de 4 agentes y 693.6k tokens sobre un tema que el mapa no mencionaba | done (2026-08-22) |
Despues:
| 11 | Empezar con el material en la mano, no explorando | paquete_de_contexto_para_implementar | <O1> | `contexto --feature <id>` (o `--tema`) entrega el mapa —siguiendo el puntero si `architecture.md` apunta a otro archivo—, si ese mapa CUBRE el tema, el impacto del hub con limite, la edad del grafo (vencido a los 7 dias), la historia acotada, las lecciones que aplican y las features del mismo servicio; declara su tamaño y sus huecos, y el resumen sale solo en cada `start`. Disparador: un mapeo de 4 agentes y 693.6k tokens sobre un tema que el mapa no mencionaba | done (2026-08-22) |
| 12 | El arnes no se bloquea a si mismo | el_guard_no_bloquea_por_lo_que_escribe_el_arnes | <O1> | El commit guard deja de contar como sucios los documentos que escribio el propio arnes (specs, planes, impl, review, verify, estados, prd-diff, `docs/prd/**`, `docs/lecciones/**`, architecture y perfil), exigiendo nombre Y ubicacion bajo `docs/`; sigue bloqueando por codigo y por cualquier documento ajeno, y dice en una linea `[i]` cada vez que se saltea un repo. Disparador: en un proyecto donde `docs/` es su propio repo, cada start/advance/prd apply terminaba el turno pidiendo un commit por microservicio de archivos que el `close` iba a commitear | done (2026-08-22) |

## Documento: docs/prd/SDD-master.md

Que cuenta: como se construye, a nivel proyecto
Presente en: -
Ausente en: docs/prd/SDD-master.md (no menciona 'el_guard_no_bloquea_por_lo_que_escribe_el_arnes')
Veredicto: cambio
Antes:
**El material se entrega y el vacio se dice** (feature #56). La #51 dejo de
Despues:
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

## Documento: docs/architecture.md

Que cuenta: el mapa de lo que YA existe
Presente en: -
Ausente en: docs/architecture.md (no menciona 'el_guard_no_bloquea_por_lo_que_escribe_el_arnes')
Veredicto: cambio
Antes:
## Paquete de contexto (feature #56)
Despues:
## El commit guard y los artefactos del arnes (feature #58)

`commit_guard.sh` (y su plantilla) trae `es_artefacto_del_arnes()` y
`solo_artefactos_del_arnes()`: un repo hermano cuyos unicos cambios sin
commitear son documentos del arnes no cuenta como sucio, y se anuncia con una
linea `[i]`. Un artefacto se reconoce por NOMBRE (`spec-feature-*.md`,
`plan-feature-*.md`, `impl-*.md`, `review-*.md`, `verify-*.md`,
`estado-feature-*.md`, `prd-diff-*.md`, `prd/*`, `lecciones/*`,
`architecture.md`, `perfil-usuario.md`) **y** por UBICACION (la ruta empieza con
`docs/`, o el repo sucio es el propio `docs/`). Alcanza un archivo ajeno para
que el repo vuelva a bloquear. `HARNESS_COMMIT_GUARD_MODE=warn|off` sigue igual.

## Paquete de contexto (feature #56)

