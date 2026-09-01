Aplicado: 2026-08-31T02:42:33Z por USUARIO (confirmacion explicita)

# Documentos al dia - Feature #65: el_arnes_cierra_lo_resuelto_aguas_arriba

Contesta CADA bloque con uno de los tres veredictos y despues corre
`sh harness_cli prd apply --feature 65`:

- `Veredicto: cambio` + `Antes:` y `Despues:` (texto LITERAL del documento)
- `Veredicto: ya-esta <archivo>:<L1>-<L2>` (el binario verifica la cita)
- `Veredicto: no-aplica <razon>` (la razon no puede estar vacia)

## Documento: docs/prd/PRD-master.md

Que cuenta: que se construye y por que
Presente en: docs/prd/PRD-master.md:1 (spec `master`), docs/prd/PRD-master.md:1 (spec `nombre`), docs/prd/PRD-master.md:1 (spec `proyecto`) y 163 más
Ausente en: -
Candidato despues:
- Cambio de la feature en: `rust/src/atlassian/emit.rs`, `rust/src/cli.rs`, `rust/src/commands/close.rs`, `rust/src/commands/status.rs` y 2 ruta(s) más. Revisa si este documento debe reflejarlo.

Veredicto: no-aplica La tabla de hitos (seccion 10) llega hasta el #13 y desde el #14 los cierres van a la Bitacora, que la escribe el propio `close` — la #37, que agrego el estado analogo `superseded`, esta ahi y no en la tabla. Lo que si cambia va al SDD (los estados y la decision de que un estado nuevo decida en cada consumidor) y a architecture.md (el mapa decia cinco estados y decia que agregar uno era barato; las dos cosas quedaron falsas).

## Documento: docs/prd/SDD-master.md

Que cuenta: como se construye, a nivel proyecto
Presente en: docs/prd/SDD-master.md:1 (spec `master`), docs/prd/SDD-master.md:1 (spec `process`), docs/prd/SDD-master.md:10 (spec `ningun`) y 235 más
Ausente en: -
Candidato despues:
- Cambio de la feature en: `rust/src/atlassian/emit.rs`, `rust/src/cli.rs`, `rust/src/commands/close.rs`, `rust/src/commands/status.rs` y 2 ruta(s) más. Revisa si este documento debe reflejarlo.

Veredicto: cambio

Antes:
**Los estados de una feature** (`feature_list.json`). Son cinco y significan
cosas distintas: `pending` (sin empezar, `next` la ofrece), `in_progress`
(pueden convivir varias, cada una en su worktree, desde la #47), `done` (hecha,
con spec aprobado y su evidencia), `blocked` (trabada por algo externo) y
`superseded` (el trabajo se hizo en OTRA feature, que se nombra en
`superseded_by` y se valida al cerrar). Solo `done` pasa por los cinco gates de
cierre; `superseded` no cuenta ni en el numerador ni en el denominador de
`prd tree`, porque no es trabajo hecho ni pendiente.

Despues:
**Los estados de una feature** (`feature_list.json`). Son seis y significan
cosas distintas: `pending` (sin empezar, `next` la ofrece), `in_progress`
(pueden convivir varias, cada una en su worktree, desde la #47), `done` (hecha,
con spec aprobado y su evidencia), `blocked` (trabada por algo externo),
`superseded` (el trabajo se hizo en OTRA feature, que se nombra en
`superseded_by` y se valida al cerrar) y `resuelto-aguas-arriba` (el trabajo se
hizo en OTRO PROYECTO, que se nombra en `resuelto_en` con la forma
`<proyecto>/feature-<id>`, feature #65). Solo `done` pasa por los cinco gates de
cierre; `superseded` y `resuelto-aguas-arriba` no cuentan ni en el numerador ni
en el denominador de `prd tree`, porque no son trabajo hecho ni pendiente.

De la referencia externa se comprueba **la forma y nada mas**, y `status` lo dice
literal ("sin verificar"): la feature de aguas arriba vive en un repo que el
arnes no puede abrir, y validar su existencia seria prometer enforcement que no
se hace (feature #64). Por la misma razon el cierre no transiciona el ticket de
Atlassian: mandarlo a `done` afirmaria que este proyecto lo entrego, y dejarlo
caer en el brazo por defecto lo REABRIRIA.

**Un estado nuevo tiene que decidir en cada consumidor** (feature #65). El campo
es un `&str` comparado por igualdad en varios lugares, asi que un estado que no
declara su rama en alguno cae en el brazo por defecto de ese consumidor — que en
Atlassian significa reabrir el ticket. Las decisiones dejan de vivir en `match`
inline y son produccion consultable (`close::ESTADOS_DE_CIERRE`,
`emit::efecto_de`, `prd::cuenta_en_el_avance`, `status::ESTADOS_CON_BUCKET`), y
un test recorre la tabla completa —estados x consumidores— contra ellas. Un
estado que no se agregue a la tabla no compila; uno que se agregue sin decidir
que hace cada consumidor, falla.

## Documento: docs/architecture.md

Que cuenta: el mapa de lo que YA existe
Presente en: docs/architecture.md:1 (spec `process`), docs/architecture.md:101 (spec `modelo`), docs/architecture.md:102 (spec `leccion`) y 418 más
Ausente en: -
Candidato despues:
- Cambio de la feature en: `rust/src/atlassian/emit.rs`, `rust/src/cli.rs`, `rust/src/commands/close.rs`, `rust/src/commands/status.rs` y 2 ruta(s) más. Revisa si este documento debe reflejarlo.

Veredicto: cambio

Antes:
`pending` / `in_progress` / `done` / `blocked` / `superseded`. El campo es un
`&str` y **no** un enum: catorce lugares lo comparan por igualdad contra un valor
concreto, lo que hace barato agregar uno nuevo (la #37 agrego `superseded` con un
cambio real de una linea) y a la vez significa que un valor invalido escrito a
mano solo lo detecta clap. `superseded` exige `superseded_by`, que se valida
contra el backlog al cerrar.

Despues:
`pending` / `in_progress` / `done` / `blocked` / `superseded` /
`resuelto-aguas-arriba`. El campo es un `&str` y **no** un enum: varios lugares
lo comparan por igualdad contra un valor concreto, lo que a la vez significa que
un valor invalido escrito a mano solo lo detecta clap.

Agregar un estado NO es barato, y esta seccion decia lo contrario. La #37 lo
midio como "un cambio real de una linea"; la #65 encontro el costo verdadero: un
estado que no declara su rama en algun consumidor cae en el brazo por defecto de
ese consumidor, y en Atlassian el brazo por defecto REABRE el ticket. Peor, los
tests que deberian detectarlo no lo detectaban — uno asertaba que una constante
era igual a su propio literal y otro llamaba a una copia de la tabla definida
dentro de `mod tests`.

Por eso las decisiones por estado son ahora produccion consultable, una por
consumidor: `close::ESTADOS_DE_CIERRE` (que acepta el CLI),
`emit::efecto_de` (que le hace al ticket), `prd::cuenta_en_el_avance` (si suma al
avance) y `status::ESTADOS_CON_BUCKET` (si tiene bucket propio en la cabecera, o
cae en `otros=N`). El test `todos_los_estados_tienen_su_rama` recorre la tabla
completa contra las cuatro. `superseded` exige `superseded_by` y
`resuelto-aguas-arriba` exige `resuelto_en`; el primero se valida contra el
backlog al cerrar, del segundo se comprueba solo la forma.

