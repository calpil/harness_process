# Spec - Feature #83: el Stop hook bloquea por graphify-out/.graphify_stale, un marcador de enriquecimiento best-effort

Estado: approved
Aprobado: 2026-09-07T01:43:47Z por USUARIO (confirmacion explicita) - Alan: 'Aprobado' a los cinco AC (el marcador pasa a [i], sin bloquear)

Feature: `feature_list.json` id 83 · kind `bug`.
Origen: captura del usuario (2026-09-06 22:40), Stop hook de realestate:
`[!] graphify-out/.graphify_stale existe; corre /graphify --update cuando aplique.`
contado como fallo junto a cambios sin commitear ajenos.

## La historia (antes -> despues)

`graphify-out/.graphify_stale` lo deja el arnes, no el usuario:

- el hook `post-commit` (init.sh:162) lo crea cada vez que un commit toca
  `README`, `AGENTS` o cualquier `.md` —o sea, en CADA cierre, que commitea
  spec, impl y review—, y solo lo borra si logra el rebuild semantico
  (init.sh:194), que esta *debounced* a 30 minutos y salta del todo si no hay
  backend LLM;
- `harness autocheck` (rust/src/graphify.rs:56-75) lo crea cuando
  `graphify update` falla o se pasa del timeout, y lo borra cuando el grafo
  vuelve a estar fresco.

Es decir: el marcador dice "el enriquecimiento del grafo esta pendiente", un
estado normal entre commits, y el arnes mismo lo limpia cuando puede. Pero
`harness_check.sh:139-142` lo cuenta como FALLO (`sumar_fallo`), y como el Stop
hook corre el check, el agente queda bloqueado hasta que alguien corra
`/graphify --update` —un rebuild con LLM, caro— o pasen los 30 minutos y el
hook lo limpie solo. Un enriquecimiento best-effort que bloquea el Stop es la
misma clase de error que la #18 evito para el nudge y la #80 para los avisos:
lo que no puede impedir trabajar se avisa, no se bloquea.

Despues: el marcador se AVISA con `[i]`, el mensaje dice quien lo va a limpiar
y como forzarlo, y el check no falla por el.

## Hoy -> Como va a funcionar

- Hoy: `[!] ... existe; corre /graphify --update cuando aplique.` + `sumar_fallo`.
- Despues: `[i] graphify-out/.graphify_stale: el grafo tiene un enriquecimiento
  pendiente. Lo limpia solo el hook post-commit en el proximo rebuild semantico
  (cada 30 min con backend LLM) o harness autocheck cuando graphify update
  vuelve a andar; para forzarlo: /graphify --update. No bloquea.` Sin
  `sumar_fallo`.
- `templates/harness_check.sh` identico (el instalador copia desde ahi).
- El resto del check no cambia: los cambios sin commitear ajenos que salieron
  en la misma captura son del guard y son correctos (archivos que no son
  artefactos del arnes en el repo `docs`; la salida 2 del guard aplica).

## Recorridos de usuario (priorizados)

- P1 (agente en realestate, tras un cierre): el hook dejo el marcador; el
  agente termina su turno -> Stop hook -> `[i]` del marcador, exit 0 si no hay
  otro problema -> el agente para sin pelear con el grafo.
- P2 (usuario que quiere el grafo al dia): lee el `[i]`, corre `/graphify
  --update` cuando le convenga; el marcador desaparece.

## Criterios de aceptacion (Given/When/Then)

- AC-1: Given un proyecto instalado con `graphify-out/.graphify_stale` presente
  y ningun otro problema, When corre `bash harness_check.sh`, Then sale `[i]`
  nombrando el marcador (no `[!]`), y el exit code es 0.
  Comando: `bash tests/graphify_stale_check.sh`
- AC-2: Given el mismo escenario, When se lee el aviso, Then dice quien lo
  limpia solo (hook post-commit en el proximo rebuild semantico; `autocheck`
  cuando `graphify update` vuelve a andar), como forzarlo (`/graphify
  --update`) y que no bloquea.
  Comando: `bash tests/graphify_stale_check.sh`
- AC-3: Given `harness_check.sh` y `templates/harness_check.sh`, When se
  comparan, Then son identicos.
  Comando: `cmp harness_check.sh templates/harness_check.sh`
- AC-4: Given el smoke del instalador, When corre, Then incluye
  `tests/graphify_stale_check.sh`; y el test cae contra el check de HEAD
  (prueba del rojo: exit 2 con `[!]`).
  Comando: `bash tests/setup_smoke.sh`
- AC-5: Given `UPDATING.md` (las dos copias), When se lee, Then dice que el
  marcador ya no bloquea y por que.
  Comando: `cmp UPDATING.md templates/UPDATING.md`

## Los datos que se tocan

- `harness_check.sh` y `templates/harness_check.sh`: el bloque del marcador.
- `tests/graphify_stale_check.sh` (nuevo) y su enganche en `tests/setup_smoke.sh`.
- `UPDATING.md` y `templates/UPDATING.md`: una seccion corta.
- No se toca `init.sh` (el hook sigue poniendo y sacando el marcador igual) ni
  `graphify.rs`.

## Pseudo-codigo (el acuerdo)

```
# harness_check.sh
if [ -f "$REPO_ROOT/graphify-out/.graphify_stale" ]; then
    echo "[i] graphify-out/.graphify_stale: ... No bloquea." >&2
    # sin sumar_fallo: es enriquecimiento best-effort, no un problema del proceso
fi
```

## No funcionales

- Ningun otro `[!]` del check cambia de severidad.
- El test corre en un fixture instalado por el instalador, como
  `tests/leccion_tope_check.sh` (#80), y no depende de `graphify` en el PATH.

## Fuera de alcance

- Cambiar cuando el hook crea el marcador o el debounce del rebuild.
- Los cambios sin commitear del repo `docs` de realestate: son trabajo de una
  sesion (scripts de publicacion a Confluence) y el guard hace bien en
  nombrarlos.

## Observaciones (decisiones pendientes)

- OBS-1: la rama de integracion se pregunta antes de `close --status done --to`.
- OBS-2: ¿un marcador con mas de N dias deberia volver a ser `[!]`? Propuesta:
  no. La edad del grafo (7 dias) ya la reporta `harness contexto`, y un
  bloqueo del Stop nunca es la herramienta para "el grafo esta viejo".
