Aplicado: 2026-08-27T18:35:32Z por USUARIO (confirmacion explicita)

# Documentos al dia - Feature #61: el_merge_del_cierre_no_toca_tu_checkout

Contesta CADA bloque con uno de los tres veredictos y despues corre
`sh harness_cli prd apply --feature 61`:

- `Veredicto: cambio` + `Antes:` y `Despues:` (texto LITERAL del documento)
- `Veredicto: ya-esta <archivo>:<L1>-<L2>` (el binario verifica la cita)
- `Veredicto: no-aplica <razon>` (la razon no puede estar vacia)

## Documento: docs/prd/PRD-master.md

Que cuenta: que se construye y por que
Presente en: docs/prd/PRD-master.md:1 (spec `master`), docs/prd/PRD-master.md:1 (spec `nombre`), docs/prd/PRD-master.md:108 (spec `dispara`) y 185 más
Ausente en: -
Candidato despues:
- Cambio de la feature en: `README.md`, `UPDATING.md`, `docs/architecture.md`, `rust/src/commands/close.rs` y 3 ruta(s) más. Revisa si este documento debe reflejarlo.

Veredicto: no-aplica el cuerpo de este PRD sigue en plantilla sin completar y es del USUARIO. Esta feature no cambia que se construye ni por que: arregla el COMO de un mecanismo ya prometido. Su hito y su bitacora los escribe el propio cierre.

## Documento: docs/prd/SDD-master.md

Que cuenta: como se construye, a nivel proyecto
Presente en: docs/prd/SDD-master.md:1 (spec `master`), docs/prd/SDD-master.md:101 (spec `decision`), docs/prd/SDD-master.md:101 (spec `decisiones`) y 156 más
Ausente en: -
Candidato despues:
- Cambio de la feature en: `README.md`, `UPDATING.md`, `docs/architecture.md`, `rust/src/commands/close.rs` y 3 ruta(s) más. Revisa si este documento debe reflejarlo.

Veredicto: cambio
Antes:
- **El arnes nunca reescribe historia ni elige la rama destino.** Sin `--force`,
  sin rebase, sin squash y sin borrar ramas; el merge corre en un worktree
  temporal (no toca tu checkout) y `--to` lo decide el USUARIO.
Despues:
- **El arnes nunca reescribe historia ni elige la rama destino.** Sin `--force`,
  sin rebase, sin squash y sin borrar ramas; el merge corre en un worktree
  temporal (no toca tu checkout) y `--to` lo decide el USUARIO.
- **Y esa promesa no tiene excepciones** (feature #61). El worktree temporal se
  crea con `--detach`, asi que vale tambien cuando el destino es la rama que el
  usuario tiene abierta — el caso mas comun, y el que antes se colaba por una
  excepcion justificada en un limite de git que nadie volvio a comprobar. La
  rama destino se avanza despues con `reset --keep` (conserva lo que el usuario
  tenga sin commitear) o con `update-ref` con guarda de valor viejo. El unico
  caso irreductible —el merge cambia un archivo que el usuario tiene sucio— se
  DETECTA antes de commitear o mergear y detiene el cierre nombrando los
  archivos: el arnes no elige entre su merge y el trabajo ajeno.

## Documento: docs/architecture.md

Que cuenta: el mapa de lo que YA existe
Presente en: docs/architecture.md:101 (spec `consulta`), docs/architecture.md:104 (spec `ejecuta`), docs/architecture.md:106 (spec `restaurar`) y 325 más
Ausente en: -
Candidato despues:
- Cambio de la feature en: `README.md`, `UPDATING.md`, `docs/architecture.md`, `rust/src/commands/close.rs` y 3 ruta(s) más. Revisa si este documento debe reflejarlo.

Veredicto: ya-esta docs/architecture.md:288-297

