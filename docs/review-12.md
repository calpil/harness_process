# Review - Feature #12: prd_story_method

Spec: docs/spec-feature-12-prd-story-method.md (approved 2026-08-12T02:19:54Z)
Plan: docs/plan-feature-12-prd-story-method.md
Implementacion: docs/impl-12.md

**Veredicto: APROBADO para cierre.** Los 11 AC tienen evidencia verificable, la
verificacion oficial esta en verde y la constitution se cumple.

## Verificacion re-ejecutada en esta revision

| Comando | Resultado |
| --- | --- |
| `bash harness_check.sh` | `[Ok] Harness Check limpio.` — plan fresco, spec `approved` fresco, gate de espejo de roles sin novedad |
| `cargo test --locked` | 51 passed (unit) + 27 passed (cli_basics), 0 failed |
| `cargo clippy --all-targets --all-features --locked -- -D warnings` | sin warnings |
| `bash tests/setup_smoke.sh` | exit 0, con `[Ok] planillas maestras docs/prd/ (PRD + SDD): siembra, no-pisa y supervivencia al reset` |
| `diff templates/docs/prd/* docs/prd/*` | PRD-master, COMO-ESCRIBIR-UN-PRD y SDD-master identicos |
| `git diff --stat -- roles/ templates/roles/ .claude/` | sin salida: los roles no se tocaron, el gate de espejo no puede quedar stale |

## Cobertura por AC

| AC | Estado | Como se verifico |
| --- | --- | --- |
| AC-1 | cubierto | `grep -n '^## '` sobre `docs/prd/PRD-master.md` devuelve las 12 secciones en el orden exigido; seccion 2 con bloques ANTES/DESPUES narrados y seccion 3 con `O1`/`O2`/`NO1` en tablas |
| AC-2 | cubierto | Secciones 7 (disparador/interruptor/candado) y 8 (CUANDO/guards/ENTONCES/Promesas) presentes a nivel producto, con la clausula que delega el detalle vinculante al spec; regla dura destacada en `PRD-master.md:23-29` |
| AC-3 | cubierto | La tabla de hitos y la linea `sh harness_cli add ...` sobreviven (`PRD-master.md:146-154`); el smoke lo re-verifica en fixture (`tests/setup_smoke.sh:245-246`) |
| AC-4 | cubierto | Las cinco piezas del metodo estan en `COMO-ESCRIBIR-UN-PRD.md` (§1 contiene/nunca contiene, §2 historia asi-no/asi-si, §3 tabla de tamano + anidamiento, §4 anatomia con ejemplo, §5 mapeo al arnes) |
| AC-5 | **cubierto y re-ejecutado** | El smoke instala en fixtures reales: la guia aparece en `docs/prd/` en layout root y subdir, el `--reset` la limpia (es plantilla del arnes) y el reinstall posterior la resiembra, mientras los sentinels prueban que `PRD-master.md` del usuario sobrevive a reinstall y a reset. La ruta con subdirectorio funciona en los tres consumidores |
| AC-6 | **cubierto y re-ejecutado** | No solo unit test: binario recompilado + `harness start --feature 1` en fixture aislado; el spec generado trae encabezado con `Metodo:`, las nueve secciones en orden y el pseudo-codigo completo |
| AC-7 | cubierto | Dos tests: el ampliado (marcadores de cada seccion + `Estado:` en la linea 3) y el nuevo `spec_template_sections_should_keep_the_prd_order` (orden exacto de encabezados). `git diff` no toca los specs #1..#11 |
| AC-8 | cubierto | `setup_harness.sh:954-957` (lista "Archivos principales") y `setup_harness.ps1:664-667` (parrafo en ingles); el smoke verifica el enlace en el `AGENTS.md` **instalado** (`tests/setup_smoke.sh:255-256`). `write_basic_agent_surface` y `.grok/GROK.md` intactos |
| AC-9 | cubierto | Asserts nuevos en ambos smoke; el de bash corrio en esta revision con exit 0 |
| AC-10 | cubierto | README, AGENTS, UPDATING (raiz + template) y architecture actualizados; espejos `templates/` == raiz verificados con `diff` |
| AC-11 | cubierto | Tabla de arriba |

## Constitution

- **Art. 1 (calidad y tests primero):** cumplido — tests cercanos al cambio en
  las dos capas (unit de Rust para la plantilla, smoke de instalador para la
  siembra) y los cuatro comandos oficiales en verde.
- **Art. 2 (spec aprobado antes de implementar):** cumplido — el spec se mostro
  en el chat, se abrio en el editor, Alan respondio y recien ahi se ejecuto
  `approve-spec --yes`. El sello registra el ajuste que pidio.
- **Art. 3 (trazabilidad AC-n):** cumplido — cada unidad del plan (U1..U8) cita
  sus AC y `docs/impl-12.md` esta organizado por AC con rutas y lineas.
- **Art. 4 (seguridad y observabilidad):** cumplido — sin secretos; el ejemplo
  del PRD es ficticio; sin cambios en exit codes ni en el logging del instalador
  (la siembra reusa `install_asset` / `write_file_notice`).
- **Art. 5 (las decisiones del usuario mandan):** cumplido — los dos forks de
  diseno se preguntaron ANTES de implementar; el fork de "datos y pseudo-codigo"
  se resolvio en contra de la propuesta original del agente y se implemento como
  decidio Alan, con la decision registrada en spec y plan.
- **Art. 6 (reglas puente):** cumplido — sin dependencias nuevas en
  `rust/Cargo.toml`; `templates/` y raiz espejados; el cambio es
  backend-agnostico (la guia y las secciones del spec valen igual para Claude,
  Codex, Gemini y Kimi).

## Observaciones (no bloqueantes)

1. `--force` sobre la guia se hereda por construccion del bucle de
   `HARNESS_DOCS` (`if [ ! -f ... ] || [ "$FORCE" -eq 1 ]`), no tiene assert
   dedicado en el smoke. El comportamiento es el mismo que el de
   `conventions.md` y `verification.md`, ya cubiertos; no amerita bloquear.
2. `tests/setup_smoke.ps1` se verifico estaticamente (no hay `pwsh` en esta
   maquina), igual que en las features #1 y #4 a #11.
3. El hub Postgres no responde desde esta maquina; el impacto se calculo por
   lectura directa del repo. No afecta el resultado: la feature toca un unico
   microservicio (`harness`).
4. Fuera de alcance por decision del spec: `roles/*.md` no instruyen todavia al
   leader a narrar la historia. La plantilla del spec ya lo impone en el
   artefacto, que es donde importa; si mas adelante se quiere reforzar en el
   prompt de los roles, es una feature aparte (obliga a regenerar los espejos de
   los cuatro backends).

## Estado Git

Working tree con los cambios de la feature sin commitear (12 archivos
modificados + 4 nuevos: spec, plan, impl y la guia en `templates/` y `docs/`).
Falta el commit Conventional **sin trailers de IA** (`commit_guard.sh`).
