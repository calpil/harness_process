# Plan - Feature #36: deudas_anotadas_del_arnes

Estado: in_progress
Microservicios:
- harness

## Alcance

Las seis deudas que el propio arnes se anoto en las secciones "Para el backlog"
de `impl-23` a `impl-26`, mas el hito #27 del PRD maestro. Ninguna es grande;
todas estaban escritas y ninguna estaba hecha.

Spec aprobado (15 AC, cada uno con su `Comando:`):
`docs/spec-feature-36-deudas-anotadas-del-arnes.md`.

## Peldano elegido: 1 (extender lo que ya existe)

Las seis son correcciones sobre caminos que ya existen: un exit code, un parser
de flag, el alcance de un grep, una poda, un chequeo mas fino y un ancho de
columna. **Cero comandos nuevos, cero flags nuevos, cero dependencias.**

**Peldano elegido: 1 (extender lo que ya existe) porque las seis son
correcciones dentro de funciones que ya estan escritas; ninguna necesita
superficie propia.**

Y una decision de forma que tambien es escalera: **una feature en vez de seis**.
Seis specs, seis planes, seis rituales de aprobacion y seis lecciones para
cambios de pocas lineas cada uno seria ceremonia, no proceso. La historia es
coherente: el arnes paga lo que se anoto.

## Impacto entre microservicios
<!-- sh harness_cli graph impacto --microservicio <proyecto>/<servicio> -->

`sh harness_cli graph impacto --microservicio harness_process/harness` -> hub sin
responder, como en las once features anteriores.

- `rust/src/spec.rs`: el exit code del gate (AC-1).
- `rust/src/commands/verify.rs`: `--solo` con varios AC (AC-3, AC-4).
- `harness_check.sh` (+ espejo) y `tests/conventions_check.sh`: el alcance a
  `rust/src/` (AC-5, AC-6).
- `rust/src/rutas.rs` y `rust/src/commands/rutas.rs`: la poda (AC-7, AC-8).
- `rust/src/doctor.rs`: a donde apunta cada hook (AC-9, AC-10).
- `rust/src/commands/leccion.rs`: el ancho dinamico (AC-11, AC-12).
- `tests/deudas_check.sh` (NUEVO) y el rol del implementer (AC-13, AC-14).

**Riesgo**: seis cambios chicos en seis lugares distintos es mas superficie de
error que un cambio grande en uno. Mitigado porque cada uno tiene su AC con su
comando propio y ninguno comparte test con otro: si uno rompe algo, se ve cual.

## Consulta al grafo (graphify)
<!-- graphify query "<pregunta de la task>" -->

`sh harness_cli buscar "para el backlog"`. Lo que decidio el plan:

- **La nota del backlog sobre los exit codes estaba mal.** Decia "1 / 1 / 2"; al
  medirlo, el gate de leccion **ya** sale 2 (`lecciones.rs:707`) y el unico
  distinto es el de spec (`spec.rs:325`, via `Exit::msg`). Es el segundo caso en
  dos features de una razon escrita sin verificar — el mismo hallazgo que la #30
  agrego a `probar-contra-datos-reales`.
- La violacion historica de la regla 2 de convenciones estaba en
  `src/verificacion.rs`, o sea **fuera** del alcance del chequeo. Se encontro a
  mano. Eso es lo que el AC-5 corrige.
- `progress/` ya es el lugar del estado local del arnes (`.last_autocheck`,
  `.nudge_lecciones`), asi que la poda de `.rutas_arnes` no mueve nada de sitio.

## Delegacion (implementer)

- **D1 (AC-1, AC-2)** — `spec.rs`: `spec_gate` pasa de `Exit::msg` (codigo 1) a
  `Exit { code: 2 }`. Test que fija los tres gates en el mismo codigo, y otro que
  confirma que el uso invalido sigue distinguiendose.
- **D2 (AC-3, AC-4)** — `verify.rs`: `--solo` parte por coma, normaliza cada
  entrada y falla nombrando **cual** no existe.
- **D3 (AC-5, AC-6)** — El chequeo de convenciones tambien recorre `rust/src/`.
  Modo `detecta-en-src` en `conventions_check.sh` con la prueba del rojo.
- **D4 (AC-7, AC-8)** — Poda del registro en cada consulta: una entrada cuya
  ruta ya no aparece en `git status` deja de eximir y se saca. La funcion que
  decide sigue siendo **pura**; la escritura vive en el comando.
- **D5 (AC-9, AC-10)** — `doctor`: el area de hooks lee el archivo de settings de
  cada backend instalado y verifica que apunte a `bin/harness-hook`.
- **D6 (AC-11, AC-12)** — `leccion list`: ancho = el nombre mas largo. Test de
  que orden, campos, `--json` y exit codes no cambian.
- **D7 (AC-13, AC-14)** — Cerrar #27 y #31-#35 citando esta feature, y el rol del
  implementer: una nota de "Para el backlog" entra al backlog en el mismo cierre.
- **D8 (AC-15)** — Verificacion oficial.

## Criterios de cierre (reviewer)

- Evidencia por AC-1..AC-15 en `docs/impl-36.md`; veredicto en `docs/review-36.md`.
- `sh harness_cli verify --feature 36` **verde**, con sus 15 comandos.
- **La prueba del rojo en las dos que la admiten**: sembrar un test que lea
  fuente dentro de `rust/src/` y confirmar que el chequeo lo reporta; sembrar un
  hook mal apuntado y confirmar que `doctor` lo reporta.
- **Cero regresiones de formato**: `leccion list` con el catalogo real, comparado
  a ojo contra la salida anterior, y `--json` byte a byte igual.
- **La poda no borra de mas**: con una ruta protegida todavia modificada, la
  entrada sobrevive.
- **#27 y #31-#35 cerradas**, sin quedar duplicadas con esta.
- `cargo test`, `cargo clippy --all-targets -- -D warnings`,
  `bash tests/setup_smoke.sh`, `bash harness_check.sh`: todo verde.

## Riesgos

- **Seis cambios chicos dispersos.** Mitigado por un AC y un test por deuda.
- **Cambiar un exit code puede romper un consumidor.** Por eso el AC-2 existe:
  `harness_check.sh` distingue rc=1 de rc=2 en varios gates y hay que confirmar
  que sigue haciendolo bien.
- **Un chequeo mas amplio puede volverse ruidoso.** El AC-6 exige que la suite
  real siga sin violaciones despues de ampliar el alcance.

## Observaciones (decisiones pendientes)
<!-- Una observacion por linea. Si hay observaciones SIN decision, el
     implementer DEBE preguntar al usuario que decision aplicar ANTES de
     implementar ese feat/fase/tarea, y registrar aqui la respuesta. -->

**Ninguna abierta.** Las dos del spec fueron decididas por Alan el 2026-08-18:

- OBS-1 **exit 2** para los tres gates; se mueve solo el de spec -> D1.
- OBS-2 la poda ocurre **en cada consulta** de violaciones -> D4.

## Skills aplicadas

- **`rust-testing`**: un test por deuda, ninguno compartido, para que un fallo
  diga cual de las seis rompio.
- **`rust-best-practices`**: peldano 1 en las seis; la poda mantiene la funcion
  de decision pura y deja la escritura en el comando.
- **`rust-patterns`**: `--solo` pasa de `Option<&str>` a una lista normalizada en
  un solo lugar, para que el resto del comando no sepa que ahora hay varios.

### Avance 2026-08-18
Plan de la #36 escrito: D1-D8 citando cada AC. Las seis deudas venian de las secciones "Para el backlog" de impl-23 a impl-26 y del hito #27. Al medir la primera se descubrio que la nota estaba mal (el gate de leccion ya salia 2), segundo caso en dos features de una razon escrita sin verificar.

### Avance 2026-08-18T03:09:55Z
Feature #36 implementada: las seis deudas pagadas (exit code del gate de spec unificado en 2, --solo con varios AC nombrando cual falta, conventions mirando rust/src, poda del registro de rutas, doctor verificando a donde apunta cada hook, y leccion list con ancho dinamico). Tres hallazgos: la nota del backlog sobre los exit codes estaba mal (el de leccion ya salia 2), ampliar el alcance de conventions destapo un bug de pipefail que mataba el chequeo en silencio desde la #24, y el gate de spec rechazo cerrar las seis entradas como done porque nunca tuvieron spec propio.

---
Cerrado: 2026-08-18T03:11:52Z - status=done - Las seis deudas que el arnes se anoto en sus propios impl, pagadas con un AC y un test cada una. Tres hallazgos al medir: la nota del backlog sobre exit codes estaba mal, ampliar el alcance de conventions destapo un bug de pipefail que mataba el chequeo en silencio desde la #24, y el gate de spec rechazo cerrar las seis entradas como done porque nunca tuvieron spec propio.
