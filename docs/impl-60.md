# Impl - Feature #60: la_vuelta_al_prd_no_se_pierde_ni_miente

Spec: docs/spec-feature-60-la-vuelta-al-prd-no-se-pierde-ni-miente.md
Bugs de origen: #91 (el cierre pierde el hito y la bitacora sin avisar) y #92
(la bitacora enlaza rutas de worktree borrado e impl-<n>.md inexistente).

## El diagnostico, con la evidencia que lo prueba

No fue una hipotesis: quedo medido sobre la historia real de este repo.

- `git merge-tree --write-tree 8175209 0db6a5a` **reproduce el conflicto** en
  `docs/prd/PRD-master.md`. La base (`a50877c`) termina en `#46`; `main` habia
  agregado `#57/#38/#39` y la rama `#55`. Los dos lados apendean al final de la
  misma seccion.
- El commit de cierre `0db6a5a` SI escribio su linea (`docs/prd/PRD-master.md |
  1 +`). El merge `66b75a2` la descarto: el diff contra el segundo padre muestra
  `- #55 ...` y el diff combinado (`--cc`) del PRD queda **vacio**, o sea que el
  resultado es identico al lado de `main`.
- Barriendo los 18 merges de cierre del repo: **7 perdieron su linea**
  (#40, #41, #42, #43, #53, #54, #55) — exactamente las 7 que hubo que
  transcribir a mano en `cf62b24 docs(prd): preserva cierres 40-55`.
- `grep -c '· spec: \.\./' docs/prd/PRD-master.md` daba **18**.

La causa es una sola: `close.rs` resolvia el PRD con `paths.para_feature(f)`
(el `docs/` del worktree) y escribia ANTES de integrar.

## Que cambio

| Archivo | Cambio |
| --- | --- |
| `rust/src/prd.rs` | `echo_close` partida en `decidir_vuelta` (pura, valida punteros) + `aplicar_vuelta` (la unica que escribe). Nuevos: `Candidato`, `Descarte`, `PlanDeVuelta`, `escapa_de_la_raiz`, `bitacora_entries`, `hito_marcado`, `file_en_raiz`, `scan_dir` |
| `rust/src/spec.rs` | `spec_rel_raiz`: la ruta canonica post-merge, construida, no derivada del worktree con `relpath` |
| `rust/src/commands/close.rs` | conserva `raiz` antes de sombrear `paths`; `echo_to_prd` corre DESPUES de `integrar` y resuelve contra el checkout principal; `candidatos_de_bitacora`; `aviso_pendiente` a stderr |
| `rust/src/commands/prd.rs` | `doctor(paths, reparar)`: audita y repara |
| `rust/src/cli.rs` | subcomando `prd doctor [--reparar]` |
| `harness_check.sh` + `templates/harness_check.sh` | gate informativo (`[i]`, no suma fallo) |
| `tests/prd_doctor_check.sh` | modos `check` y `repo` |
| `README.md`, `UPDATING.md`, `templates/UPDATING.md`, `docs/architecture.md` | documentacion |

## Evidencia por AC

- **AC-1** (escribe en la raiz, no en el worktree): `cargo test
  close_should_write_the_prd_echo_in_the_root_not_the_worktree`. Cierra una
  feature con worktree real y verifica las dos mitades: la bitacora y el hito
  quedan en el PRD de la raiz, y `git show feature/1-cobranza:docs/prd/PRD-master.md`
  **no** contiene `-> done`. El log de cierre ya no viaja en la rama, que es lo
  que hacia posible el conflicto.
- **AC-2** (sin integracion no hay hito): `cargo test
  close_should_not_touch_the_prd_when_integration_fails`. `close --status done`
  sin `--to` sale 2 y el PRD sigue con `| pendiente |` y sin bitacora.
- **AC-3** (la regresion de las 7 perdidas): `cargo test
  dos_cierres_en_paralelo_conservan_las_dos_bitacoras`. Dos features abiertas a
  la vez, cada una en su worktree, cerrando una despues de la otra: quedan **las
  dos** lineas y **los dos** hitos marcados. Con el codigo anterior este test
  falla.
- **AC-4** (puntero relativo a la raiz): `cargo test
  punteros_de_bitacora_son_relativos_a_la_raiz`. Usa la forma exacta que tenian
  los 18 rotos y verifica que no entra a la linea; cubre tambien absoluta,
  `C:/` y `docs/../../fuera.md`.
- **AC-5** (omite en vez de mentir): `cargo test
  bitacora_omite_el_puntero_que_no_resuelve`. El `impl-<n>.md` ausente no se
  escribe y queda como descarte con motivo; sin ningun puntero valido la entrada
  se escribe igual.
- **AC-6** (informa y no escribe): `cargo test prd_doctor_reporta_y_no_escribe`.
  Sale 2, lista los hallazgos y el documento queda **byte a byte igual**.
  Tambien `bash tests/prd_doctor_check.sh check`.
- **AC-7** (`--reparar`): `cargo test
  prd_doctor_reparar_arregla_punteros_y_bitacoras_faltantes`. Reescribe el
  puntero al worktree, quita el `impl` que no existe en ningun lado, agrega la
  bitacora faltante con la fecha de `closed_at` (no la de hoy), marca los dos
  hitos, deja el cuerpo intacto y una segunda corrida no encuentra nada.
- **AC-8** (avisa y no cambia el cierre): `cargo test
  aviso_de_vuelta_al_prd_fallida_no_cambia_el_cierre`. Sin PRD en el repo, el
  cierre sale **0**, el stdout dice "Feature #1 cerrada como done" y el stderr
  trae el aviso y el comando exacto que lo repara.
- **AC-9** (`harness_check.sh` lo reporta): `bash tests/prd_doctor_check.sh
  check`. Verifica que el aviso aparece y que sale como `[i]` y no como `[!]`,
  que es lo que distingue informar de bloquear.
  DESVIACION: el spec declaraba `bash harness_check.sh --solo prd`, y
  `harness_check.sh` no tiene `--solo`. Implementarlo habria sido agregar una
  feature que nadie pidio, asi que el AC se verifica con el test de arriba, que
  comprueba lo mismo que el AC exige.
- **AC-10** (idempotencia): `cargo test vuelta_al_prd_es_idempotente`. La
  segunda aplicacion no hace nada y la fecha del PRIMER cierre es la que queda.
- **AC-11** (pura vs. escribe): `cargo test decidir_vuelta_es_pura_y_no_escribe`.
  El test le da un directorio vacio como testigo y comprueba que sigue vacio.
  La garantia de fondo no es el test: es que `decidir_vuelta` recibe
  `Candidato { existe }` ya resuelto y **no tiene con que** mirar el disco.
- **AC-12** (los 18 de este repo): `bash tests/prd_doctor_check.sh repo`.
  `sh harness_cli prd doctor --reparar` sobre el repo real reparo los 18
  punteros y agrego 13 bitacoras de features cerradas antes de que la vuelta al
  PRD existiera (#1..#13, con la fecha de su `closed_at`). `prd doctor` sale
  limpio y `grep -c '· spec: \.\./'` da **0**.

## Lo que NO se toco

El spec, el plan y la evidencia siguen viviendo en el worktree de la feature
(doctrina #47/#49/#54). Lo unico que salio del branch es el LOG de cierre, que
es de todas las features y de ninguna rama. El AC-5 de la #54 sigue valiendo
para el CUERPO del PRD que `prd propose/apply` modifica.

Tampoco se toco el algoritmo de merge, el orden de integracion ni la exigencia
de `--to`, y el cierre no commitea el PRD de la raiz: queda modificado sin
commitear, como el resto de los documentos del arnes en el checkout principal.
