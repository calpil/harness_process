# Implementacion - Feature #12: prd_story_method

Spec: docs/spec-feature-12-prd-story-method.md (Estado: approved,
2026-08-12T02:19:54Z)
Plan: docs/plan-feature-12-prd-story-method.md

Fuente del metodo: `how-i-spec.pdf` ("Escribe tu maldito PRD"), aportado por el
usuario. El metodo quedo instalado en las tres superficies donde el arnes crea
especificacion: la planilla maestra, una guia nueva y la plantilla de spec que
genera el binario.

## Evidencia por AC

### AC-1 — anatomia nueva del PRD maestro (U1)

`templates/docs/prd/PRD-master.md` reescrito (espejado en `docs/prd/`). Orden de
secciones verificado con `grep -n '^## '`:

```
32:## 1. Resumen (hoy -> despues)      94:## 6. Como funciona hoy -> como va a funcionar
40:## 2. La historia                  106:## 7. Los datos
64:## 3. Objetivos / No-objetivos     118:## 8. Pseudo-codigo (el acuerdo)
78:## 4. Usuarios y jobs-to-be-done   139:## 9. Restricciones y supuestos
84:## 5. Metricas de exito            146:## 10. Hitos -> features
                                      157:## 11. Riesgos
                                      163:## 12. Decisiones abiertas
```

- Encabezado con `Estado`, `Duenno`, `Ultima actualizacion` y `Alcance` ("que
  abarca y que NO toca"), mas punteros a la guia, al SDD y a la constitution
  (`PRD-master.md:1-9`).
- Seccion 2 con los bloques **ANTES** y **DESPUES** narrados (el ejemplo de Marta
  del PDF) y el recuadro asi-no / asi-si (`PRD-master.md:40-62`).
- Seccion 3 con IDs citables en dos tablas: `O1`/`O2` y `NO1`
  (`PRD-master.md:64-76`). La tabla de metricas gana la columna "Mide" que cita
  el `O-n` (`PRD-master.md:90-92`).
- Seccion 6 con el flujo dibujado dos veces en un bloque HOY / DESPUES
  (`PRD-master.md:98-104`).

### AC-2 — datos y pseudo-codigo a nivel producto + regla dura (U1)

Decision de Alan al aprobar el spec: el maestro tambien los lleva.

- `## 7. Los datos` (`PRD-master.md:106-116`): tabla disparador / interruptor /
  candado, en palabras, remitiendo el esquema fisico al SDD.
- `## 8. Pseudo-codigo (el acuerdo)` (`PRD-master.md:118-137`): esqueleto
  `CUANDO ... ¿guards? -> si no, no hacemos nada ... ENTONCES ...` mas la linea
  **Promesas**, y la frase que evita que el maestro se desactualice: cada feature
  refina el suyo y el detalle vinculante vive en su spec.
- Regla dura destacada arriba de todo (`PRD-master.md:23-29`): fija estructura en
  pseudo-codigo y explicaciones; **nunca** codigo final, implementacion exacta,
  pantallas terminadas ni configuracion.

### AC-3 — la cadena del arnes intacta (U1)

`## 10. Hitos -> features` conserva la tabla que alimenta el backlog y la linea
`sh harness_cli add --name <slug> --service <servicio> --acceptance "<criterio>"`
(`PRD-master.md:146-154`), ahora con columna "Objetivo que cumple" para citar el
`O-n`. El encabezado sigue enlazando `docs/prd/SDD-master.md` y
`docs/constitution.md` (`PRD-master.md:5-7`).

### AC-4 — la guia del metodo (U2)

`templates/docs/prd/COMO-ESCRIBIR-UN-PRD.md` (nuevo, espejado en `docs/prd/`):

| Pieza del AC | Seccion |
| --- | --- |
| (a) contiene / nunca contiene | `## 1. Que es un PRD` (tabla de dos columnas + la regla dura) |
| (b) la historia, asi-no / asi-si | `## 2. Todo empieza con una historia` |
| (c) tamano y anidamiento | `## 3. El tamano lo decide el cambio` (tabla 1 pag / 3-8 / 10+ / anidados + diagrama de PRDs anidados) |
| (d) anatomia con el ejemplo | `## 4. Anatomia: las partes de un PRD` (0 Encabezado, 1 Resumen, 2 La historia, 3 Objetivos/No-objetivos, 4 Hoy -> Despues, 5 Los datos, 6 Pseudo-codigo con CUANDO/guards/ENTONCES/Promesas) |
| (e) mapeo al arnes | `## 5. Como se aplica en este arnes` (tabla producto/tecnico/cambio + la cadena PRD -> backlog -> spec -> impl) y `## 6. Ahora te toca` |

### AC-5 — siembra en ambos instaladores (U3, U4)

- `setup_harness.sh:369-377`: `HARNESS_DOCS` gana `prd/COMO-ESCRIBIR-UN-PRD.md`
  (primer elemento con subdirectorio; el comentario lo documenta).
  `setup_harness.sh:1637`: alta en `required_assets`.
- `setup_harness.ps1:79-87` y `:441`: misma alta en `$script:HarnessDocs` y en
  los required assets.
- Los tres consumidores toleran el subdirectorio: `install_asset` hace
  `mkdir -p "$(dirname "$destination")"` (`setup_harness.sh:1657`), los reset
  targets arman la ruta completa (`:555-560`) y `migrate_harness_docs` tambien
  hace `mkdir -p` (`:1690`). Confirmado en fixtures reales por el smoke (AC-9).
- `PRD_DOCS` / `$script:PrdDocs` sin cambios: `PRD-master.md` y `SDD-master.md`
  siguen siendo del USUARIO (sembrado solo-si-falta, ni `--force` los pisa, y no
  entran en reset targets). El smoke lo re-verifica con sus sentinels
  (`tests/setup_smoke.sh:283-284` + `:310` para el reinstall, `:465-466` +
  `:505-507` para el `--reset`).

### AC-6 — la plantilla del spec (U5)

`rust/src/spec.rs` `spec_template()`: encabezado con `Estado: draft` en la linea
3, `Plan:`, `Constitution:` y la linea nueva
`Metodo: docs/prd/COMO-ESCRIBIR-UN-PRD.md (este spec es el PRD del cambio)`.
Secciones nuevas con su comentario guia: `## La historia (antes -> despues)`
(ANTES/DESPUES), `## Hoy -> Como va a funcionar` (bloque HOY/DESPUES),
`## Los datos que se tocan` (disparador / interruptor / candado) y
`## Pseudo-codigo (el acuerdo)` (CUANDO / guards / ENTONCES / Promesas).

Verificado end-to-end, no solo por unit test: binario recompilado
(`cargo build --release --locked`) y `harness start --feature 1` corrido en un
fixture temporal aislado; el spec generado trae las nueve secciones en el orden
aprobado y el bloque de pseudo-codigo completo.

### AC-7 — tests de Rust (U5)

- `spec_template_should_declare_draft_and_sections` ampliado: verifica
  `t.lines().nth(2) == "Estado: draft"` (contrato de `spec_state`: primeras diez
  lineas), la linea `Metodo:` y las cuatro secciones nuevas con sus marcadores
  (`ANTES:`, `DESPUES:`, `- disparador:`, `- interruptor:`, `- candado:`,
  `CUANDO <ocurre el disparador>`, `ENTONCES ...`, `Promesas:`).
- Test nuevo `spec_template_sections_should_keep_the_prd_order`: compara la lista
  completa de encabezados `## ` contra el orden aprobado (falla si alguien
  reordena o inserta una seccion).
- `cargo test --locked`: **51 passed** (unit) + **27 passed** (cli_basics), 0
  fallidos. Los tests de `approve_spec`, firmas y `spec_gate` siguen verdes.
- Specs ya existentes: `write_spec` solo escribe si el archivo no existe
  (`spec.rs:79-81`); `git diff --stat -- docs/spec-feature-*.md` no reporta
  cambios en los specs #1..#11.

### AC-8 — superficies multi-LLM (U3, U4)

- `setup_harness.sh:954-957`: bullet nuevo en la lista "Archivos principales" de
  `write_agent_surface` (CLAUDE/AGENTS/GEMINI/LLM), describiendo la guia (la
  historia primero, el tamano, PRDs anidados, nunca codigo final). Ademas el
  bullet del spec ahora lo nombra como "el PRD del cambio" con sus secciones
  (`:946-951`) y el del PRD maestro refleja la anatomia nueva (`:958-961`).
- `setup_harness.ps1:664-667`: parrafo equivalente en ingles en
  `Write-AgentSurface` (paridad razonable: esa superficie no tiene lista de
  archivos, mismo criterio que la feature #11).
- Sin tocar `write_basic_agent_surface` (variante `--no-subagents`) ni
  `.grok/GROK.md`.

### AC-9 — smoke tests (U6)

`tests/setup_smoke.sh`:
- `:156-157` guia sembrada en layout **root**.
- `:235-240` guia sembrada en la raiz **multi-repo** + secciones del metodo
  (`## 2. Todo empieza...`, `## 3. El tamano...`, `NUNCA CONTIENE`).
- `:243-246` planilla con la anatomia nueva: `## 2. La historia`,
  `## 8. Pseudo-codigo (el acuerdo)`, `## 10. Hitos -> features` (renumerado
  desde `## 7.`) y la linea `harness_cli add`.
- `:255-256` la superficie instalada enlaza `COMO-ESCRIBIR-UN-PRD`.
- `:490-491` el `--reset` limpia la guia (es plantilla del arnes) mientras los
  sentinels confirman que el PRD/SDD del usuario sobreviven (`:505-509`);
  `:520-521` el reinstall posterior la vuelve a sembrar.

`tests/setup_smoke.ps1` en paridad: `:132` y `:192` (siembra en root y limpieza
por reset, ambas listas de harness docs), `:144-158` (siembra + secciones de la
guia + secciones nuevas del PRD con la renumeracion), `:166` (superficie).

Resultado: `bash tests/setup_smoke.sh` -> **exit 0**, incluyendo
"[Ok] planillas maestras docs/prd/ (PRD + SDD): siembra, no-pisa y supervivencia
al reset". Sin `pwsh` en la maquina, la version ps1 se verifico estaticamente
(grep de paridad linea a linea), como en las features #1 y #4 a #11.

### AC-10 — documentacion (U7)

- `README.md:284-287` arbol de `docs/prd/` con la guia; `:294-327` explica el
  metodo, la distincion producto/cambio y el regimen de cada archivo (planillas
  del usuario vs guia refrescable, `:322-327`).
- `AGENTS.md:39-51` (dogfooding): el spec descrito como PRD del cambio, la guia
  como archivo principal y el PRD maestro con la anatomia nueva.
- `UPDATING.md:153-194` y `templates/UPDATING.md:193-234`: seccion de planillas
  ampliada con la guia, el PRD reescrito, el regimen de reset y el parrafo que
  baja el metodo al spec generado. Tambien la lista de archivos sembrados
  (`UPDATING.md:56-60`, `templates/UPDATING.md:96-100`).
- `docs/architecture.md:77-84` (paso 0 del flujo SDD) y `:162-171` (siembra: la
  guia como excepcion en `docs/prd/`, `:167-171`, con la ruta con subdirectorio).
- Espejos exactos: `diff templates/docs/prd/PRD-master.md docs/prd/PRD-master.md`
  y `diff templates/docs/prd/COMO-ESCRIBIR-UN-PRD.md docs/prd/COMO-ESCRIBIR-UN-PRD.md`
  sin salida.

### AC-11 — verificacion oficial (U8)

| Comando | Resultado |
| --- | --- |
| `bash harness_check.sh` | `[Ok] Harness Check limpio.` (incluye el gate de espejo de roles, intacto: `roles/*.md` no se tocaron) |
| `cargo test --locked` | 51 + 27 passed, 0 failed |
| `cargo clippy --all-targets --all-features --locked -- -D warnings` | sin warnings |
| `bash tests/setup_smoke.sh` | exit 0 |

## Notas

- El hub Postgres no es alcanzable desde esta maquina (`error connecting to
  server: connection timed out` en `start`/`advance`); el impacto se calculo por
  lectura directa del repo, como en features anteriores en la misma situacion.
- El binario local `harness` (no versionado) se recompilo para que este mismo
  repo genere specs con la plantilla nueva.
- No se corrio `setup_harness.sh` dentro del checkout fuente (footgun conocido):
  toda la verificacion de siembra pasa por los fixtures del smoke.
