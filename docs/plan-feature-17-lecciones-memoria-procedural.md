# Plan - Feature #17: lecciones_memoria_procedural

Estado: in_progress
Microservicios:
- harness

## Alcance

Primer hito del PRD `docs/prd/aprendizaje/PRD-aprendizaje.md`: darle al arnes una
**memoria procedural** ordenada por clase de trabajo (`docs/lecciones/<clase>.md`)
en vez de por id de feature, con el comando `leccion` para manejarla, el gate
opcional del cierre que obliga a **declarar** que se aprendio, y las reglas de
captura portadas de Hermes a la guia y a los tres roles.

Fuera de este hito (cada uno es su propia feature): el nudge automatico (#18), el
perfil de usuario (#19), `buscar` (#20), las transiciones del curador (#21) y el
mapa `journey` (#22). Ninguna llamada a un modelo entra en esta feature.

Spec aprobado (20 AC): `docs/spec-feature-17-lecciones-memoria-procedural.md`.

## Impacto entre microservicios
<!-- sh harness_cli graph impacto --microservicio <proyecto>/<servicio> -->

Ejecutado: `sh harness_cli graph impacto --microservicio harness_process/harness`
-> **el hub PostgreSQL no responde en este entorno** (`error connecting to server:
connection timed out`), el mismo error que ya emitio `start --feature 17`. No es
un bloqueo de esta feature: el hub es best-effort en todo el binario y la propia
feature esta especificada para **funcionar sin hub** (AC-9).

Impacto por inspeccion directa, que en este repo es exacto porque hay **un solo
microservicio** (`harness`) y su superficie es conocida:

- `rust/` — modulo nuevo + un comando nuevo + un gate en `close`. No toca
  `atlassian/`, `graph/` ni `prd.rs`.
- `setup_harness.sh` / `setup_harness.ps1` — una entrada en la lista de docs
  generados; el resto (siembra, no-pisado, reset, migracion) sale gratis de esa
  lista, que ya tiene tres consumidores.
- `harness_check.sh` (+ su espejo en `templates/`) — un bloque de gate nuevo.
- `roles/*.md` (+ espejos) — reglas de captura.
- Docs de proceso: `architecture.md`, `README.md`, `UPDATING.md`, superficies.

**Riesgo cero para lo existente**: sin `docs/lecciones/` y sin la regla
`require_leccion`, todos los flujos de hoy quedan byte a byte iguales (AC-10,
AC-18).

## Consulta al grafo (graphify)
<!-- graphify query "<pregunta de la task>" -->

`graphify query "lecciones memoria procedural aprendizaje del arnes"` -> 143 nodos.
Lo util que devolvio y que este plan usa:

- La siembra de docs pasa por `install_asset()` + `write_file_notice()` +
  `track_action()` + `backup_file()` en `setup_harness.sh`, alimentados por la
  lista `HARNESS_DOCS` (y su gemela `$script:HarnessDocs` en el `.ps1`).
  **Consecuencia de diseno**: agregar una sola entrada a esa lista resuelve AC-1,
  AC-19 y la mitad de OBS-3 sin escribir codigo nuevo de instalador. Es el peldano
  de menor huella disponible.
- `PRD_DOCS` / `$script:PrdDocs` es la lista de "documentos del USUARIO" que **no**
  entra a los reset targets: confirma que basta con NO listar las lecciones para
  que sobrevivan al `--reset` (AC-19).
- `prd.rs` (1117 lineas) es el precedente mas cercano: un arbol de archivos
  markdown en `docs/` con validacion de nombre, plantilla y gate propio en
  `harness_check.sh`. Se imita su estructura, no se modifica.

## Delegacion (implementer)

- **D1 (AC-2, AC-14)** — Escribir `templates/docs/lecciones/COMO-ESCRIBIR-UNA-LECCION.md`:
  el formato exacto del frontmatter y las cuatro secciones del cuerpo (AC-2), el
  **orden de preferencia** (patchear la que estuvo en juego > paraguas existente >
  `referencias/<tema>.md` > crear clase nueva) y la lista completa de **que NO
  capturar** (los cinco items) (AC-14). Incluye la advertencia de que las lecciones
  son archivos versionados y no llevan secretos.
- **D2 (AC-1, AC-19)** — Agregar `lecciones/COMO-ESCRIBIR-UNA-LECCION.md` a
  `HARNESS_DOCS` en `setup_harness.sh` y a `$script:HarnessDocs` en
  `setup_harness.ps1`. Verificar que las tres consumidoras de la lista (siembra,
  reset targets, migracion) crean el subdirectorio, como ya hacen con `prd/`. Las
  lecciones en si NO se listan en ningun lado: esa ausencia es la que las hace
  sobrevivir al `--reset` (AC-19).
- **D3 (AC-2, AC-4)** — `rust/src/lecciones.rs`: modelo de la leccion (parseo y
  serializado del frontmatter, preservando el cuerpo tal cual) y
  `validar_nombre_de_clase()` con las cinco reglas de rechazo del AC-4 (contiene
  `feature` o `#`, empieza con `fix-`/`debug-`/`audit-`/`hotfix-`, contiene una
  fecha `YYYY-MM-DD`, o contiene un numero de tres o mas digitos). **Sin escape
  hatch** (OBS-1): la funcion no acepta un flag que la desactive.
- **D4 (AC-3, AC-5, AC-6, AC-7, AC-8, AC-9)** — `rust/src/commands/leccion.rs` +
  `LeccionCommand` en `cli.rs`: `nueva` (crea desde plantilla, rechaza nombre malo
  con exit 2 y sin escribir, rechaza duplicado empujando a patchear), `list`
  (ordenado por uso desc, `--json`), `show` (con sugerencias por similitud si no
  existe) y `usar` (incrementa `usos` y sella `ultimo_uso` sin tocar el cuerpo ni
  `ultima_actualizacion`). Ningun subcomando abre conexion al hub (AC-9).
- **D5 (AC-10, AC-11, AC-12, AC-13)** — Gate del cierre: `--leccion` y
  `--leccion-motivo` en `close` (`cli.rs`), lectura de la regla `require_leccion`
  desde `rules`, las tres ramas del gate, el campo **opcional** `leccion` en la
  entrada de la feature (OBS-5: solo se escribe cuando se declara, sin migrar las
  16 ya cerradas) y la clase en la linea de `progress/history.md`. Una clase
  inexistente **falla** (OBS-2).
- **D6 (AC-15)** — `roles/leader.md`, `roles/implementer.md`, `roles/reviewer.md`
  y sus espejos en `templates/roles/`: cada rol cita las reglas de D1 en lo que le
  toca (el lider decide la clase, el implementer patchea antes que crear, el
  reviewer verifica que la declaracion del cierre exista y sea honesta). Re-correr
  el instalador para regenerar los espejos de los cuatro backends y dejar el gate
  de espejos limpio.
- **D7 (AC-16, AC-17)** — `docs/architecture.md` (+ `templates/docs/`) documenta el
  limite de los **tres almacenes** (hub = eventos, lecciones = procedimiento,
  perfil = preferencias) y que las lecciones no agregan nada al hub. `README.md`,
  `UPDATING.md` y las superficies generadas (heredocs de ambos instaladores +
  `AGENTS.md` de la raiz) documentan el comando, el formato y la regla
  `require_leccion` con su default apagado.
- **D8 (AC-18)** — Bloque de gate en `harness_check.sh` y su espejo
  `templates/harness_check.sh`: frontmatter ilegible o `nombre` que no coincide con
  el archivo **BLOQUEA** nombrando el archivo (OBS-4); leccion sin `triggers` avisa
  `[i]`; sin `docs/lecciones/` el bloque entero se omite.
- **D9 (AC-20)** — Tests: unitarios en `lecciones.rs` (validacion de nombre,
  round-trip del frontmatter, telemetria de `usar`), de integracion en
  `rust/tests/cli_basics.rs` (los cuatro subcomandos y las tres ramas del gate),
  y en `tests/setup_smoke.sh` + `tests/setup_smoke.ps1` (siembra, idempotencia y
  supervivencia de una leccion al `--reset`). `cargo test` y
  `cargo clippy --all-targets -- -D warnings` verdes.
- **D10 (AC-9)** — Verificacion explicita de que la ruta `leccion` no toca el hub:
  test que corre los subcomandos con el hub inalcanzable y compara exit codes y
  salida contra la corrida normal.

## Criterios de cierre (reviewer)

- Evidencia por AC-1..AC-20 en `docs/impl-17.md`, cada una con su comando y su
  salida; veredicto por AC en `docs/review-17.md`.
- `bash harness_check.sh` limpio (incluye el gate nuevo de D8 y el gate de espejos
  de roles tras D6).
- `cargo test` y `cargo clippy --all-targets -- -D warnings` verdes;
  `bash tests/setup_smoke.sh` verde.
- Compatibilidad demostrada: con el repo sin `docs/lecciones/` y sin
  `require_leccion`, `close --status done` se comporta igual que antes (AC-10).
- Las reglas portadas (orden de preferencia + los cinco items de que NO capturar)
  aparecen textualmente en la guia y citadas en los tres roles (AC-14, AC-15).
- `templates/` y la raiz espejados (regla de mantenedor de UPDATING.md).
- El hito 1 del PRD `aprendizaje` queda marcado por el cierre.

## Riesgos

- **La validacion de nombre rechaza un nombre legitimo.** Mitigacion: las cinco
  reglas son explicitas y estan en el mensaje de error junto a dos ejemplos
  validos; el remedio es renombrar, que es exactamente lo que la regla busca.
  Decidido sin escape hatch (OBS-1).
- **El gate del cierre molesta antes de que existan lecciones.** Mitigacion:
  `require_leccion` nace apagada (decision del PRD) y `ninguna + motivo` siempre es
  salida valida.
- **El bloque nuevo de `harness_check.sh` frena cierres por formato.** Mitigacion:
  solo bloquea frontmatter ilegible o nombre que no coincide; todo lo demas avisa,
  y sin `docs/lecciones/` el bloque no corre.
- **Desincronizacion `templates/` <-> raiz.** Mitigacion: D6 y D8 tocan pares de
  archivos y el gate de espejos del propio `harness_check.sh` lo detecta.
- **El hub no responde en este entorno.** No bloquea: es best-effort en todo el
  binario y la feature esta especificada para funcionar sin el (AC-9, D10).

## Observaciones (decisiones pendientes)
<!-- Una observacion por linea. Si hay observaciones SIN decision, el
     implementer DEBE preguntar al usuario que decision aplicar ANTES de
     implementar ese feat/fase/tarea, y registrar aqui la respuesta. -->

**Ninguna abierta.** Las cinco observaciones del spec fueron decididas por Alan el
2026-08-16 en el mismo acto de aprobacion, y estan registradas en la seccion
"Observaciones" de `docs/spec-feature-17-lecciones-memoria-procedural.md` y en el
sello de aprobacion:

- OBS-1 sin `--force` en la validacion de nombre (hardline) -> D3.
- OBS-2 `close --leccion <clase>` con clase inexistente **falla** -> D5.
- OBS-3 la guia es plantilla del arnes (refrescable, en reset targets); las
  lecciones no -> D1, D2.
- OBS-4 `harness_check.sh` **bloquea** por frontmatter ilegible -> D8.
- OBS-5 el campo `leccion` es opcional y no migra lo ya cerrado -> D5.

Decisiones heredadas del PRD (Alan, 2026-08-16): archivos en `docs/` con el limite
de tres almacenes y funcionamiento sin hub (D7, D10), `require_leccion` apagada por
default (D5), perfil versionado (fuera de alcance aca, es la #19).

### Avance 2026-08-16T20:03:11Z
Plan de la #17 escrito por el lider: alcance, impacto (hub inalcanzable, documentado), consulta a graphify, D1-D10 citando cada AC-n, criterios de cierre y riesgos. Las 5 observaciones quedaron decididas por Alan en el acto de aprobacion del spec, asi que no hay ninguna abierta.

### Avance 2026-08-16T22:14:21Z
D1-D10 implementados: guia de lecciones (orden de preferencia + lista anti-veneno), entrada unica en HARNESS_DOCS de ambos instaladores, modulo lecciones.rs (validacion de nombre de clase sin escape hatch, frontmatter con round-trip, telemetria, scan y gate), comando leccion list|show|nueva|usar, gate opcional require_leccion en close, reglas en los tres roles + espejos, tres almacenes en architecture.md, README/UPDATING/superficies, bloque de integridad en harness_check.sh y 26 tests nuevos. El pase de reviewer encontro y corrigio el bug de CRLF en render(). Evidencia por AC en docs/impl-17.md, veredicto en docs/review-17.md.

---
Cerrado: 2026-08-16T22:14:34Z - status=done - Memoria procedural del arnes: docs/lecciones/<clase>.md ordenado por clase de trabajo, comando leccion list|show|nueva|usar, nombres de clase sin escape hatch, gate opcional require_leccion en el cierre, reglas de captura portadas a la guia y a los tres roles, y gate de integridad en harness_check.sh. 20 AC cubiertos (AC-20 parcial: smoke ps1 sin correr, sin PowerShell). Cero dependencias nuevas y funciona con el hub caido.

---
Cerrado: 2026-08-16T22:15:18Z - status=done - Memoria procedural del arnes: docs/lecciones/<clase>.md ordenado por clase de trabajo, comando leccion list|show|nueva|usar, nombres de clase sin escape hatch, gate opcional require_leccion en el cierre, reglas de captura portadas a la guia y a los tres roles, y gate de integridad en harness_check.sh. 20 AC cubiertos (AC-20 parcial: smoke ps1 sin correr, sin PowerShell). Cero dependencias nuevas y funciona con el hub caido.
