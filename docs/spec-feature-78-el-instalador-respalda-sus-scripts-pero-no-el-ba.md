# Spec - Feature #78: el instalador respalda sus scripts pero no el backlog, que es lo unico irrecuperable

Estado: approved
Aprobado: 2026-09-06T22:50:09Z por USUARIO (confirmacion explicita) - Alan: 'Aprobado' a los ocho AC, con OBS-1 = respaldar los datos igual (--force no lo saltea)
Plan: docs/plan-feature-78-el-instalador-respalda-sus-scripts-pero-no-el-ba.md
PRD: docs/prd/PRD-master.md
Constitution: docs/constitution.md
Metodo: docs/prd/COMO-ESCRIBIR-UN-PRD.md

## La historia (antes -> despues)

ANTES: el 2026-09-06 a las 19:12 el instalador corrio sobre realestate y dejo
`feature_list.json` en 131 bytes —la plantilla— y `progress/` reseteado. Se
perdieron 138 features, los estados vivos y la bitacora. `bkp/` tenia respaldo
de trece scripts de esa misma corrida y NINGUNO del backlog. La recuperacion la
hizo otra sesion a mano, desde un espejo del dia anterior mas los eventos del
dia; `history.md` no se pudo reconstruir.

DESPUES: ninguna corrida del instalador puede dejar el backlog sin respaldo, y
ninguna lo siembra en silencio cuando falta y hay una copia al lado.

## Lo que se midio (fixture, 2026-09-06)

Un backlog plantado con 2 features y 5 reglas, y cuatro corridas:

| Corrida | Backlog despues | Respaldo en bkp/ |
| --- | --- | --- |
| reinstalar | intacto | **ninguno** |
| `--reset` | **BORRADO** (archivo y `progress/`) | `feature_list.json.bak.<ts>` y `progress.bak.<ts>` |
| `--reset --force` | intacto (*) | ninguno |
| el archivo FALTA al reinstalar | **plantilla: 0 features, 2 reglas**, en silencio | ninguno |

(*) medido asi; `--force` cambia el camino y el reset no llega a borrar. No se
explica en este spec: se anota.

Realestate a las 19:12 quedo con 0 features, 2 reglas y `$PROJECT_NAME` sin
expandir, y `bkp/` sin respaldo del backlog. Eso **coincide con la cuarta
fila**, no con la segunda: el archivo ya faltaba cuando el instalador corrio,
y el instalador lo sembro con la plantilla sin decir una palabra — la unica
mencion es `feature_list.json` en medio de una linea `[OK]` que lista doce
cosas. Que lo borro antes no esta medido y este spec no lo adivina.

## Lo que esta mal, en tres piezas

1. **`--reset` borra el backlog y `progress/`** (`setup_harness.sh` linea 746,
   `setup_harness.ps1` linea 1848): estan en la lista de "superficies a
   regenerar". No son superficie: son los unicos datos del proyecto que el
   instalador no puede regenerar.
2. **Ninguna corrida normal respalda el backlog.** Se respaldan los scripts que
   el instalador va a sobreescribir; el backlog, que el instalador no
   sobreescribe pero que puede haber sido borrado por fuera, nunca.
3. **La siembra silenciosa.** Si el backlog falta, se copia la plantilla y sigue.
   Con 138 features en `docs/bkp-backlog/` al lado, no hubo aviso.

## Correccion de una idea previa (de la sesion que reporto el incidente)

La ficha proponia detectar la plantilla por `project == "$PROJECT_NAME"`. Medido:
el respaldo LEGITIMO de realestate tambien lo tiene sin expandir. No es un
discriminador. Este spec no olfatea la forma del archivo: respalda siempre y
avisa por AUSENCIA, no por contenido.

## Criterios de aceptacion (Given/When/Then)

- AC-1: Given un proyecto con backlog y `progress/`, When corre
  `setup_harness.sh --reset`, Then el backlog y `progress/` quedan
  byte-identicos: dejan de estar en `reset_targets`. Lo mismo en
  `setup_harness.ps1`.
  Comando: `bash tests/setup_smoke.sh`
- AC-2: Given cualquier corrida del instalador (normal, `--reset`, `--force`,
  `--dry-run` aparte), When empieza, Then ANTES de tocar nada respalda
  `feature_list.json`, `progress/current*.md` e `progress/history.md` en
  `bkp/` con timestamp, en un paso propio que `--force` NO saltea: `--force`
  significa "no respaldes lo que vas a regenerar", no "no respaldes los datos".
- AC-3: Given que `feature_list.json` falta, When el instalador va a sembrarlo,
  Then lo dice en `[WARN]` con su propia linea, nombra los respaldos que
  encuentra (`bkp/feature_list.json.bak.*` y `docs/bkp-backlog/feature_list.json`
  si existen, con cuantas features tiene cada uno) y el comando para restaurar.
  Siembra igual, para que la instalacion termine, pero nunca en silencio.
- AC-4: Given `progress/current.md` o `history.md` faltantes, When se siembran,
  Then el mismo aviso: una bitacora que nace vacia sobre un proyecto con
  respaldos no es una instalacion nueva, es una perdida.
- AC-5: Given el smoke del instalador, When corre, Then planta un backlog con
  features y reglas, corre reinstalar, `--reset` y `--reset --force`, y afirma
  que el backlog sigue byte-identico y que `bkp/` tiene su copia en cada
  corrida. Y un caso mas: borra el backlog, reinstala, y afirma que el aviso
  salio y nombro el respaldo. Cada uno tiene que fallar contra el instalador
  actual (prueba del rojo).
- AC-6: Given `setup_harness.ps1`, When se aplica lo mismo, Then el gate de
  paridad sigue verde y la asimetria, si queda alguna, esta declarada.
  Comando: `bash tests/parity_check.sh`
- AC-7: Given `UPDATING.md` y `--help`, When se leen, Then dicen que `--reset`
  no toca el backlog ni `progress/`, y que `--force` no evita el respaldo de
  datos.
- AC-8 (MANUAL): Given este mismo repo, cuyo backlog esta gitignorado y sin
  espejo, When se instale este cambio, Then queda registrado que la politica de
  espejo es una decision del usuario, pendiente, fuera de este spec.

## Los datos que se tocan

- `reset_targets` (sh) y `$targets` del reset (ps1): salen `feature_list.json` y
  `progress`.
- `bkp/`: gana `feature_list.json.bak.<ts>`, `progress/current*.md.bak.<ts>`,
  `progress/history.md.bak.<ts>` en cada corrida.
- Nada del formato del backlog cambia.

## Pseudo-codigo (el acuerdo)

```
AL EMPEZAR (toda corrida, incluso --force):
  respaldar backlog + progress/ si existen  -> bkp/*.bak.<ts>
RESET:
  borrar SOLO superficies generadas; el backlog y progress/ no estan en la lista
SEMBRAR:
  si falta el backlog: [WARN] + respaldos encontrados + comando de restauracion
  y recien entonces la plantilla
```

Promesas: ninguna corrida deja el backlog sin respaldo; `--reset` no lo borra;
una siembra sobre un backlog ausente nunca es silenciosa.

## No funcionales y verificacion

- Prueba del rojo obligatoria: los tests nuevos del smoke fallan contra el
  instalador actual (medido arriba: hoy `--reset` borra y la siembra calla).
- El respaldo es `cp -p`, mismo mecanismo que ya usan los scripts; no se
  inventa formato.
- `--dry-run` no respalda ni escribe, como hoy.

## Alcance de instalacion y fuera de alcance

Se corrige `harness_process`. Realestate ya esta recuperado por otra sesion; no
se rehace. Que borro el archivo antes de las 19:12 no esta medido y no se
adivina. La politica de espejo del backlog de este repo es decision del usuario.

## Observaciones (decisiones pendientes)

- OBS-1: `--force` hoy dice "sobrescribe archivos sin crear backup". Este spec lo
  reinterpreta como "sin backup de lo REGENERABLE". Si el usuario quiere que
  `--force` tambien saltee el respaldo de datos, es su decision; la propuesta es
  que no. DECIDIDA por el usuario el 2026-09-06: los datos se respaldan siempre.
- OBS-2: la rama de integracion se pregunta antes de `close --status done --to`.
