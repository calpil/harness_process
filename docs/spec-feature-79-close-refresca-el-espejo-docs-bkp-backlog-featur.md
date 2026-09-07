# Spec - Feature #79: close refresca el espejo docs/bkp-backlog/feature_list.json al cerrar una feature

Estado: approved
Aprobado: 2026-09-07T02:54:25Z por USUARIO (confirmacion explicita) - Alan: 'Aprobado' a los AC; OBS-2 solo en close; OBS-3 la bitacora se espeja en esta misma feature (AC-10)

Feature: `feature_list.json` id 79 · kind `feature`.
Origen: el 2026-09-06 la otra sesion versiono un espejo del backlog en
`docs/bkp-backlog/feature_list.json` (commit `eb6e8a9`) porque `feature_list.json`
esta gitignorado y no tenia punto de restauracion; el usuario decidio
mantenerlo y que `close` lo refresque solo. El 2026-09-06 23:14 el backlog de
ESTE repo se borro por error y se restauro desde ese espejo: 83 features,
identico al ultimo estado porque el espejo se habia refrescado a mano en el
ultimo cierre. La bitacora (`progress/history.md`), que no tiene espejo, se
perdio.

## La historia (antes -> despues)

Hoy el espejo se refresca a mano: `cp feature_list.json
docs/bkp-backlog/feature_list.json` y un commit, cada vez que alguien se
acuerda. Entre dos refrescos, el espejo esta viejo; el instalador (#78) lo
nombra al sembrar un backlog ausente y dice cuantas features tiene, pero no
puede saber cuantas le faltan.

Despues: cada `close` (con cualquier `--status`: `done`, `blocked`,
`superseded`, `resuelto-aguas-arriba`) deja el espejo de la RAIZ byte-identico
al backlog recien guardado, y lo dice en el mensaje de cierre, con la misma
advertencia que el sello: queda sin commitear. Un espejo que se refresca en
cada cierre esta, como mucho, un alta o un arranque atras del backlog vivo.

## Hoy -> Como va a funcionar

- Se espejan DOS archivos (decision del usuario, OBS-3): el backlog
  (`feature_list.json`) y la bitacora (`progress/history.md`), en
  `docs/bkp-backlog/feature_list.json` y `docs/bkp-backlog/history.md`. Son los
  dos datos del proyecto que el instalador no puede regenerar (#78) y la
  bitacora es la que se perdio hoy sin copia.
- `rules.espejo_backlog` (opcional) decide la politica:
  - ausente (**Auto**): se refresca SOLO si el directorio `docs/bkp-backlog/`
    ya existe en la raiz; el directorio es el opt-in del usuario, y adentro se
    crean o refrescan los dos archivos. Un proyecto que nunca creo el espejo
    no gana archivos versionables sin pedirlo (son documentos del usuario,
    como el PRD).
  - `true` (**Siempre**): se refresca y, si no existe, se crea (directorio
    incluido).
  - `false` (**Nunca**): no se toca, exista o no.
- El refresco ocurre en la FASE 3 de `close`, despues de `save_features` y de
  la linea de bitacora del cierre (asi la bitacora espejada incluye ese mismo
  cierre), y es una copia byte-identica escrita de forma atomica
  (`write_text_atomic`, como el propio backlog).
- La raiz es la del repo PRINCIPAL (`raiz_del_prd`), igual que el sello y la
  bitacora del PRD (#60, #71): con `docs/` como repo aparte, el espejo vive en
  ese repo y se commitea con el.
- Best-effort, no mudo (como `echo_to_prd`, #60): si la copia falla, el cierre
  NO falla; sale `[!]` por stderr con la ruta y el error, y el backlog queda
  cerrado igual. El espejo es un respaldo; un respaldo que impide cerrar es
  peor que ninguno.
- Mensaje de cierre: `Espejo del backlog refrescado: docs/bkp-backlog/feature_list.json,
  docs/bkp-backlog/history.md (sin commitear: vive en la raiz, no en la rama).`
  Cuando no aplica (Auto sin directorio, o Nunca) no dice nada: no paso nada.
- `add` y `start` NO refrescan (OBS-2): el cierre es el momento en que el
  estado cambia de verdad; un espejo a un alta de distancia del vivo es el
  precio de no escribir un archivo versionable en cada comando.

## Recorridos de usuario (priorizados)

- P1 (este repo, hoy): `close --feature 79 --status done --to main` ->
  `save_features` -> espejo byte-identico -> mensaje lo nombra -> el commit de
  bitacora y sello lo lleva.
- P2 (realestate, docs como repo aparte): el espejo esta en
  `<raiz>/docs/bkp-backlog/` -> `close` lo refresca ahi -> se commitea con el
  repo docs.
- P3 (proyecto nuevo, sin espejo): `close` no crea nada; el usuario que lo
  quiera pone `"espejo_backlog": true` en `rules` y el proximo cierre lo crea.
- P4 (backlog borrado por error, otra vez): el instalador (#78) nombra el
  espejo con sus N features; `cp` y listo. Lo que falte es lo posterior al
  ultimo cierre.

## Criterios de aceptacion (Given/When/Then)

- AC-1: Given el directorio `docs/bkp-backlog/` en la raiz (con un
  `feature_list.json` viejo adentro) y sin `rules.espejo_backlog`, When `close
  --status done`, Then el espejo queda byte-identico a `feature_list.json`
  despues del cierre (la feature figura `done` en los dos) y stdout dice
  `Espejo del backlog refrescado: docs/bkp-backlog/feature_list.json`.
  Comando: `cd rust && cargo test --locked espejo_refrescado_al_cerrar`
- AC-2: Given el mismo espejo, When `close --status blocked`, Then tambien se
  refresca (la feature figura `blocked` en el espejo).
  Comando: `cd rust && cargo test --locked espejo_refrescado_al_bloquear`
- AC-3: Given sin `docs/bkp-backlog/` y sin regla (Auto), When `close`, Then no
  se crea el directorio ni los archivos, y stdout no menciona el espejo.
  Comando: `cd rust && cargo test --locked espejo_auto_sin_archivo_no_crea`
- AC-4: Given `rules.espejo_backlog: true` y sin `docs/bkp-backlog/`, When
  `close`, Then se crea el directorio con los dos archivos, byte-identicos al
  backlog y a la bitacora, y stdout lo dice.
  Comando: `cd rust && cargo test --locked espejo_siempre_lo_crea`
- AC-5: Given `rules.espejo_backlog: false` y un `docs/bkp-backlog/` con
  contenido viejo, When `close`, Then los dos archivos quedan byte-identicos a
  lo viejo y stdout no los menciona.
  Comando: `cd rust && cargo test --locked espejo_nunca_no_toca`
- AC-6: Given la politica como enum (`Auto`, `Siempre`, `Nunca`) y su decision
  pura `decidir(politica, existe)`, When se prueban las seis combinaciones,
  Then Auto solo con existe, Siempre siempre, Nunca nunca.
  Comando: `cd rust && cargo test --locked espejo_politica`
- AC-7: Given que la escritura del espejo falla (el destino es un directorio
  ilegible o un archivo de solo lectura), When `close`, Then el cierre termina
  igual (exit 0, feature cerrada en el backlog) y stderr lleva `[!]` con la
  ruta y el error.
  Comando: `cd rust && cargo test --locked espejo_falla_sin_impedir_el_cierre`
- AC-8: Given `UPDATING.md` (dos copias), `README.md` y `docs/architecture.md`,
  When se leen, Then dicen la politica, el momento (FASE 3, tras guardar) y que
  el espejo queda sin commitear.
  Comando: `cmp UPDATING.md templates/UPDATING.md`
- AC-9: Given la suite, When corre, Then `cargo test --locked`, clippy
  `-D warnings` y `tests/parity_check.sh` verdes.
  Comando: `bash tests/parity_check.sh`
- AC-10: Given `docs/bkp-backlog/` (Auto) o la regla en `true`, When `close`,
  Then `docs/bkp-backlog/history.md` queda byte-identico a
  `progress/history.md` DESPUES de la linea `close feature #N` de ese mismo
  cierre (la linea esta en los dos).
  Comando: `cd rust && cargo test --locked espejo_bitacora`

## Los datos que se tocan

- `docs/bkp-backlog/feature_list.json` (RAIZ): copia byte-identica del backlog.
- `docs/bkp-backlog/history.md` (RAIZ): copia byte-identica de la bitacora.
- `feature_list.json` -> `rules.espejo_backlog` (opcional, bool).
- Nada en `progress/`: el refresco no deja linea en la bitacora (el mensaje
  de cierre ya lo dice, y la bitacora es de transiciones, no de copias).

## Pseudo-codigo (el acuerdo)

```
// rust/src/espejo.rs (nuevo)
pub const ESPEJO_REL: &str = "docs/bkp-backlog/feature_list.json";
pub enum Politica { Auto, Siempre, Nunca }
impl Politica { pub fn from_rules(data: &Value) -> Politica }      // ausente -> Auto
pub fn decidir(politica: Politica, existe_dir: bool) -> bool       // puro
pub fn refrescar(raiz: &Path, backlog: &Path, bitacora: &Path, politica: Politica) -> anyhow::Result<Vec<PathBuf>>
    // decide contra raiz/docs/bkp-backlog/, crea el dir si Siempre, copia cada fuente que exista
    // con write_text_atomic; devuelve las rutas escritas (vacio = no aplico)

// close.rs, FASE 3, despues de save_features y de log("close feature ..."):
match espejo::refrescar(&raiz_del_prd(paths), &paths.features, &paths.history, Politica::from_rules(&data)) {
    Ok(rutas) if !rutas.is_empty() => espejo_msg = Some(rutas relativas unidas por ", "),
    Ok(_) => {}
    Err(err) => eprintln!("[!] No se pudo refrescar el espejo del backlog ({ESPEJO_REL}): {err:#}. El cierre sigue."),
}
// ... y en el mensaje final: " Espejo del backlog refrescado: {rel} (sin commitear: vive en la raiz, no en la rama)."
```

Estilo (skills `rust-patterns`, `rust-best-practices`, `rust-testing`): la
politica es un enum (tres estados, sin bool ambiguo), `decidir` es pura y
tiene sus seis casos, `refrescar` recibe `&Path` y devuelve `Result` con
contexto, sin `unwrap` fuera de tests; un comportamiento por test, nombres que
se leen como una oracion. `rust-async-patterns` no aplica: `close` es sincrono.

## No funcionales

- Costo: leer y escribir un archivo de ~140 KB una vez por cierre. Nada de red.
- Atomico: nunca queda un espejo a medio escribir (mismo `write_text_atomic`
  que el backlog).

## Fuera de alcance

- Commitear el espejo: es un documento del usuario en la raiz, como el sello
  y la bitacora del PRD; lo commitea el usuario (o el mismo commit del cierre).
- Refrescar en `add`/`start` (OBS-2).
- Espejo de `progress/current*.md`: es estado vivo que el sello de cierre ya
  archiva en `docs/` (#71).

## Observaciones (decisiones pendientes)

- OBS-1: la rama de integracion se pregunta antes de `close --status done --to`.
- OBS-2 (DECIDIDA 2026-09-07: solo en close): ¿refrescar tambien en `add` y `start`? Propuesta: no. El cierre es el
  cambio de estado que importa; refrescar en cada comando deja un archivo
  versionable modificado a cada rato y ensucia `git status` sin que nadie
  vaya a commitearlo.
- OBS-3 (DECIDIDA 2026-09-07: incluirlo aca): `progress/history.md` se perdio
  hoy y no tiene espejo. El usuario decidio espejarla en esta misma feature
  (AC-10), con la misma politica y en el mismo directorio. La bitacora crece
  sin tope (470 lineas, ~60 KB antes de perderse): el costo es un archivo
  versionado que cambia en cada cierre, y se acepta a cambio de no volver a
  perderla.
