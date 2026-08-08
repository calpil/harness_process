# Review - Feature #10: layout_inferred_from_footprint

Spec: docs/spec-feature-10-layout-inferred-from-footprint.md (Estado: approved, sellado 2026-07-29T04:19:41Z)
Plan: docs/plan-feature-10-layout-inferred-from-footprint.md
Impl: docs/impl-10.md

## Veredicto global

**approved**, con una salvedad conocida (ningun hallazgo bloquea):

1. AC-11 (paridad Windows) queda **parcial**: revision estatica real, sin
   ejecucion, porque no hay `pwsh` ni `powershell` en esta maquina (verificado
   el 2026-07-30 con `which`, rc=1 para ambos). Misma limitacion aceptada en
   #1, #4, #5, #6, #7, #8 y #9.

Nota de proceso: la implementacion se commiteo en `c09e1dd` en la sesion del
2026-07-29 sin evidencia por AC. Esta revision NO acepto esa omision: la
evidencia (`docs/impl-10.md`) se reconstruyo el 2026-07-30 re-ejecutando TODO
sobre el working tree actual, y yo re-verifique cada punto contra el arbol
real antes de firmar. El commit existe desde ayer, pero ninguna afirmacion se
acepto sin re-ejecutarla.

## Aprobacion del spec

```
Estado: approved
Aprobado: 2026-07-29T04:19:41Z por USUARIO (confirmacion explicita) - Alan
aprobo el spec #10 en el chat (2026-07-29): inferencia del layout por huella
del padre cuando falta el marker, con aviso [i]
```

`sh harness_cli check-spec` rc=0 (`[OK] Spec aprobado y fresco`) y
`sh harness_cli check-plan` rc=0 (`[OK] Plan fresco para implementacion`),
re-corridos el 2026-07-30. Las dos decisiones de la U0 (inferencia, no
re-versionar; aviso `[i]` discreto) estan registradas como DECIDIDAS por el
usuario en spec y plan; nada de lo implementado las contradice (Articulo 5).

## Reproduccion antes/despues (re-ejecutada por mi, 2026-07-30)

Criterio de cierre del plan: sin esta prueba el veredicto no vale. Fixture
propia en `/tmp/f10_fixture/proyecto/` con huella real
(`docs/constitution.md` + `CLAUDE.md` + `AGENTS.md`) y `harness_process/`
dentro. El bloque de resolucion se extrajo VERBATIM del script en ambas
versiones: pre-fix de `git show c09e1dd^:harness_check.sh` y actual del
working tree:

```
A. SIN marker + padre CON huella:
   ANTES   REPO_ROOT=.../proyecto/harness_process     <- bug reproducido
   DESPUES [i] .harness_layout ausente: layout subdir inferido por la huella
           de instalacion del padre: REPO_ROOT=.../proyecto. Re-corre el
           instalador (setup_harness.sh / setup_harness.ps1) para regenerar
           el marker.
           REPO_ROOT=.../proyecto                     <- fix, rc=0
B. AC-3: marker=root + huella  -> .../proyecto/harness_process, SIN aviso
C. AC-4: sin marker, sin huella -> .../pelado/harness_process, SIN aviso
D. marker=subdir + huella      -> .../proyecto, SIN aviso (comportamiento
                                  previo intacto)
E. AC-6: HARNESS_REPO_ROOT=/tmp/custom_root -> /tmp/custom_root, SIN aviso
```

El bug se reproduce EXACTAMENTE como lo describe el spec (misma forma que la
reproduccion del 2026-07-29 embebida en el spec), y el fix lo corrige en los
cinco escenarios. La fixture se elimino tras la prueba.

## Criterios de cierre del plan, uno por uno

- **Evidencia POR AC (AC-1..AC-13)**: presente en `docs/impl-10.md`,
  reconstruida con corridas reales (ver nota de proceso arriba).
- **Bug ANTES / fix DESPUES**: arriba. Sin esto el veredicto no valia; esta.
- **AC-3 y AC-4 explicitos** (los que evitan que la inferencia se pase de
  lista): escenarios B y C. Ademas en Rust:
  `repo_root_should_not_infer_when_marker_says_root`,
  `repo_root_should_not_infer_when_marker_is_empty_or_unknown`,
  `repo_root_should_not_infer_without_parent_footprint`, e integracion
  `explicit_root_marker_should_never_infer_subdir`. La rama de inferencia
  esta gateada por `elif [ ! -f "$harness_marker" ]` (ausencia REAL del
  archivo), lo que hace estructuralmente imposible que un marker presente
  dispare la inferencia.
- **AC-7 (guardrail #7 sin regresion) verificado en ESTE checkout**:
  `bash harness_check.sh` imprime `[i] Checkout fuente del arnes detectado
  (.harness_layout=subdir sin huella de instalacion en el padre):
  REPO_ROOT=/Users/alan/harness_process` y sale rc=0. Escenario D confirma
  ademas que marker `subdir` + huella resuelve al padre sin aviso, como antes.
- **diff de los 4 scripts raiz vs templates/ = identicos**: verificado el
  2026-07-30, los cuatro (`harness_check.sh`, `harness_status.sh`, `init.sh`,
  `commit_guard.sh`) byte a byte (AC-9, Articulo 6).
- **Comandos oficiales de `docs/verification.md` en verde** (AC-13), corrida
  2026-07-30: `bash tests/setup_smoke.sh` rc=0;
  `(cd rust && cargo clippy --all-targets --all-features --locked --
  -D warnings)` rc=0; `(cd rust && cargo test --locked)` rc=0 con
  50 unit + 27 integracion (eran 44+22 antes de la feature, segun el commit).
- **Sin regresion multi-LLM**: el smoke del 2026-07-30 muestra intactas las
  lineas `[Ok]` previas (backend Kimi con gate de espejo y bloque global,
  docs del arnes en la raiz con migracion, plantillas PRD/SDD, Rust-only,
  layouts, reinstall, dry-run, reset) y la nueva `[Ok] marker ausente: layout
  subdir inferido ...` (AC-10).
- **AC-11 (estatica)**: `tests/setup_smoke.ps1` tiene el bloque "Feature #10"
  (linea 364) en paridad con el `.sh`: verifica que los CUATRO scripts
  sembrados traen la regla (aviso, `harness_parent_footprint`, el gate de
  ausencia, guardrail #7 intacto) y, con bash disponible, ejecuta fixtures
  `New-LostMarkerCase` (ausente -> infiere con aviso; `root` -> nunca
  infiere). Sin `pwsh` en la maquina: revision estatica registrada como tal.
  Los here-strings que arreglo la #7 no se tocaron (el diff del commit en
  `setup_harness.ps1` es NULO; la feature no modifico ese archivo, tal como
  manda el plan).
- **Commits Conventional sin trailers de IA**: `c09e1dd` es
  `fix(layout): ...`, cuerpo descriptivo, sin `Co-authored-by` ni
  `Generated with` ni marcas de IA.

## Chequeo de impacto

El radio declarado en el plan (4 scripts + 4 espejos + `paths.rs` +
`cli_basics.rs` + smoke sh/ps1 + UPDATING raiz/template + architecture +
README) coincide con el `git show --stat c09e1dd`: 18 archivos, ninguno fuera
de la lista. `setup_harness.sh`/`.ps1`, hooks, superficies y roles sin
cambios, como manda el plan. `rust/src/paths.rs::repo_root_from_marker`
sigue siendo el punto unico que consumen `HarnessPaths::from_root` y
`GraphEnv::resolve`, asi que binario y scripts no pueden divergir (Riesgo 4
del plan, cubierto ademas por AC-8/AC-10).

Estado del arbol al firmar: los unicos cambios sin commit (`setup_harness.sh`,
`setup_harness.ps1`, `templates/.kimiignore`, `templates/.kimirules`,
siembra de dotfiles Kimi) NO pertenecen a esta feature — son trabajo en
curso ajeno al spec #10; no se tocaron en esta revision.

## Hallazgos (ninguno bloquea)

1. La evidencia por AC no se escribio en la sesion que commiteo (se
   reconstruyo al dia siguiente). El proceso manda "implementer escribe
   evidencia por AC-n" como parte de la unidad; conviene no cerrar sesiones
   de implementacion sin el `docs/impl-<n>.md`. Impacto real acotado: el
   working tree era identico al commit y toda la evidencia se re-ejecuto.
2. El aviso `[i]` de inferencia se emite en CADA invocacion de cada script y
   del binario mientras el marker falte (riesgo ya aceptado en el plan). En
   este checkout NO se ve porque el marker local existe; en instalaciones
   sin marker sera visible en cada corrida hasta re-correr el instalador. Si
   molestara, bajarle el volumen es una decision nueva del usuario, no de
   esta feature.

## Veredicto

La feature cumple AC-1..AC-13 con evidencia re-ejecutada, respeta las
decisiones del usuario y la constitution, y deja verdes los tres comandos
oficiales. **Apta para `close --status done`** una vez corridos
`bash harness_check.sh` (rc=0, ya verificado) y el advance correspondiente.
