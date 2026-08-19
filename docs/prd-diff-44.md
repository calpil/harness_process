Aplicado: 2026-08-19T00:38:08Z por USUARIO (confirmacion explicita)

# Documentos al dia - Feature #44: verify_detecta_filtro_vacio

Contesta CADA bloque con uno de los tres veredictos y despues corre
`sh harness_cli prd apply --feature 44`:

- `Veredicto: cambio` + `Antes:` y `Despues:` (texto LITERAL del documento)
- `Veredicto: ya-esta <archivo>:<L1>-<L2>` (el binario verifica la cita)
- `Veredicto: no-aplica <razon>` (la razon no puede estar vacia)

## Documento: docs/prd/PRD-master.md

Que cuenta: que se construye y por que
Presente en: -
Ausente en: docs/prd/PRD-master.md (no menciona 'verify_detecta_filtro_vacio')
Veredicto: cambio
Antes:
| 6 | El PRD, el SDD y architecture.md dejan de poder quedar mintiendo | prd_y_sdd_siempre_al_dia | <O1> | Al cerrar, el arnes calcula el alcance (PRD de origen + padres + SDD + architecture.md), siembra una pregunta por documento en `docs/prd-diff-<id>.md`, y solo con el SI del usuario `prd apply --yes` lo escribe; `require_docs_al_dia` lo exige al cerrar | done (2026-08-18) |
Despues:
| 6 | El PRD, el SDD y architecture.md dejan de poder quedar mintiendo | prd_y_sdd_siempre_al_dia | <O1> | Al cerrar, el arnes calcula el alcance (PRD de origen + padres + SDD + architecture.md), siembra una pregunta por documento en `docs/prd-diff-<id>.md`, y solo con el SI del usuario `prd apply --yes` lo escribe; `require_docs_al_dia` lo exige al cerrar | done (2026-08-18) |
| 7 | Un AC que no ejecuto ningun caso deja de contar como verificado | verify_detecta_filtro_vacio | <O1> | `verify` mira la SALIDA ademas del exit code: si reconoce el formato de libtest y la suma de `passed` es cero, el AC queda en `vacio`, se cuenta aparte en el resumen y bloquea el cierre igual que un rojo; sobre salidas que no son de tests el estado no cambia | done (2026-08-19) |


## Documento: docs/prd/SDD-master.md

Que cuenta: como se construye, a nivel proyecto
Presente en: -
Ausente en: docs/prd/SDD-master.md (no menciona 'verify_detecta_filtro_vacio')
Veredicto: cambio
Antes:
- Tests automaticos: <unitarios, integracion, e2e; que cubre cada nivel>
- Entornos: <local, staging, produccion>
- Criterio de "listo para produccion": <...>
Despues:
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


## Documento: docs/architecture.md

Que cuenta: el mapa de lo que YA existe
Presente en: -
Ausente en: docs/architecture.md (no menciona 'verify_detecta_filtro_vacio')
Veredicto: cambio
Antes:
clasifica en el enum `Estado` (`Verde` / `Rojo` / `Timeout` / `Manual`); un AC
  sin comando es `Manual` y **nunca** bloquea.
Despues:
clasifica en el enum `Estado` (`Verde` / `Rojo` / `Timeout` / `Manual` /
  `Vacio`); un AC sin comando es `Manual` y **nunca** bloquea. `Vacio` (feature
  #44) es el AC que salio 0 **sin ejecutar ningun caso**: lo decide
  `casos_corridos()`, otra funcion pura, que suma los `N passed` de las lineas
  `test result:` y devuelve `None` —"no opino"— cuando la salida no tiene esa
  forma. Ese `None` es lo que evita que el detector opine sobre un `grep` o un
  `bash`. `rojos_del_reporte()` deriva del enum via `Estado::desde_etiqueta()` en
  vez de comparar contra cadenas sueltas, para que un estado nuevo no se filtre
  por el cierre — que es como la #37 se llevo puesto el emisor de Jira.


