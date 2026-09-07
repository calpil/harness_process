# Spec - Feature #74: add no protege contra la feature duplicada

Estado: approved
Aprobado: 2026-09-07T02:39:06Z por USUARIO (confirmacion explicita) - Alan: 'Aprobado'/'approved' a los siete AC; --clave; sin aviso por parecidos

Feature: `feature_list.json` id 74 · kind `feature` · reabierta el 2026-09-07
por decision del usuario ("implementa #74"), tras haberse cerrado `blocked` el
2026-09-06 por falta de evidencia.
Origen: idea 7 del catalogo de `docs/analisis-hermes-agent.md` (Hermes usa un
idempotency-key en `kanban create`).

## La historia (antes -> despues)

Lo medido sigue siendo lo medido, y va primero:

| Medicion (2026-09-07, 83 features reales) | Resultado |
| --- | --- |
| Nombres normalizados repetidos | 0 |
| Pares de nombres con solapamiento de palabras >= 50 % (Jaccard) | 0 |
| Features cargadas dos veces por reintento | 0 registradas |

Es decir: en este repo el duplicado no ocurrio. Lo que si ocurre, y no se ve en
el backlog porque no deja huella, es el ESCENARIO: dos sesiones en paralelo
que descubren el mismo problema (hoy hay siete sesiones de realestate
abiertas), un agente que re-corre `add` tras un comando interrumpido, o un
script que carga hitos de un PRD y se relanza. El arnes hoy no tiene ninguna
defensa: `add` solo valida `--kind`, `--prd` y `--depends-on`, y despues
escribe.

Despues: `add` compara el nombre NORMALIZADO con el backlog antes de escribir.
Si coincide con una feature ABIERTA, se niega (exit 2) nombrandola: la feature
ya existe, trabaja en esa. Si coincide con una CERRADA, avisa y sigue: un
nombre repetido sobre algo cerrado es una regresion, y eso es legitimo. Y para
scripts y reintentos, `--clave <k>`: con la misma clave, `add` devuelve la
feature existente sin crear otra (exit 0), que es exactamente el idempotency
key de Hermes.

## Hoy -> Como va a funcionar

- Hoy `add --name X` escribe siempre. Despues, ANTES de escribir (mismo lugar
  que las validaciones de `--kind`, `--prd` y `--depends-on`):
  1. `normalizar(X)`: minusculas, sin acentos, sin puntuacion, espacios
     colapsados, sin las palabras vacias mas comunes (`el`, `la`, `de`, `que`,
     `y`, `no`...). Es la misma idea que `contexto` usa para "cubre el tema".
  2. Si alguna feature con `status` en {`pending`, `in_progress`, `blocked`}
     tiene el mismo nombre normalizado: exit 2, sin escribir, con
     `Ya existe #N (<status>): "<nombre>". Trabaja en esa, o si es OTRA cosa,
     ponele un nombre que lo diga.` Sin flag de escape (decision del usuario
     en la #80, OBS-4: un escape se vuelve el default).
  3. Si coincide solo con features cerradas (`done`, `superseded`,
     `resuelto-aguas-arriba`): `[i] Mismo nombre que #N (done 2026-09-05). Si
     es una regresion, decilo en el spec y cita la #N.` y sigue.
- `add --clave <k>`: si ya hay una feature con `clave: k`, imprime `Feature #N
  ya existe (clave k).` y sale 0 SIN escribir, sin intent de Atlassian y sin
  linea en la bitacora. Si no, crea la feature con el campo `clave` guardado.
  La clave es opaca (lo que el script quiera: un slug, un id de Jira, un hash).
- Ninguna feature existente se migra ni se toca: `clave` es opcional, como
  `kind`, `prd` y `depends_on`.
- La similitud parcial (nombres parecidos pero no iguales) NO se implementa:
  medida en 0 pares, y un aviso por parecido sin evidencia es ruido (OBS-2).

## Recorridos de usuario (priorizados)

- P1 (dos sesiones, mismo problema): la sesion B corre `add --name "el
  instalador respalda sus scripts pero no el backlog"` cuando A ya la cargo
  como #78 pending -> exit 2, `Ya existe #78 (pending)` -> B trabaja en la #78.
- P2 (regresion): `add --name "el checkout compartido tiene capacidad uno"`
  cuando la #76 esta done -> `[i] Mismo nombre que #76 (done ...)` -> se crea
  la #84 -> el spec cita la #76.
- P3 (script que carga hitos): `add --name ... --clave prd/cobranza/mora/3`
  dos veces -> la segunda dice `Feature #N ya existe (clave ...)` y sale 0; el
  backlog tiene UNA feature.

## Criterios de aceptacion (Given/When/Then)

- AC-1: Given una feature abierta (`pending`, `in_progress` o `blocked`) con
  nombre N, When `add --name N'` con `normalizar(N') == normalizar(N)`
  (mayusculas, acentos, puntuacion y espacios distintos), Then exit 2, el
  backlog queda byte-identico, no hay linea en `history.md` ni intent, y el
  mensaje nombra `#id`, el status y el nombre original.
  Comando: `cd rust && cargo test --locked add_duplicado_abierto`
- AC-2: Given solo features CERRADAS con ese nombre normalizado, When `add`,
  Then se crea la feature nueva y stderr lleva `[i] Mismo nombre que #id
  (<status> <fecha>)`; stdout dice `Feature #M agregada.` como siempre.
  Comando: `cd rust && cargo test --locked add_mismo_nombre_cerrada`
- AC-3: Given `add --name X --clave K` ya corrido, When se corre de nuevo (con
  el mismo o con otro nombre) con `--clave K`, Then exit 0, stdout `Feature #N
  ya existe (clave K).`, backlog byte-identico, sin bitacora ni intent. Y la
  primera vez, la feature guarda `"clave": "K"`.
  Comando: `cd rust && cargo test --locked add_clave_idempotente`
- AC-4: Given `normalizar`, When se prueba, Then `"El Instalador, respalda
  el BACKLOG"` y `"instalador respalda backlog"` normalizan igual, y `"add no
  protege"` y `"close no protege"` normalizan distinto.
  Comando: `cd rust && cargo test --locked normalizar_nombre`
- AC-5: Given un nombre distinto y sin `--clave`, When `add`, Then el backlog
  resultante es byte-identico al que producia `add` antes de esta feature
  (ningun campo nuevo).
  Comando: `cd rust && cargo test --locked add_sin_clave_no_cambia_el_backlog`
- AC-6: Given `README.md` (seccion de `add`), `UPDATING.md` (dos copias) y
  `add --help`, When se leen, Then dicen la regla del duplicado abierto, el
  aviso sobre cerradas y `--clave`.
  Comando: `cmp UPDATING.md templates/UPDATING.md`
- AC-7: Given la suite, When corre, Then `cargo test --locked`, clippy
  `-D warnings` y `tests/parity_check.sh` verdes.
  Comando: `bash tests/parity_check.sh`

## Los datos que se tocan

- `feature_list.json`: campo opcional `clave` (string) por feature. Nada mas.
- `progress/history.md`: sin lineas nuevas para los rechazos ni para el
  `ya existe` (no paso nada).

## Pseudo-codigo (el acuerdo)

```
// rust/src/duplicados.rs (nuevo, puro, sin IO)
pub fn normalizar(nombre: &str) -> String
pub enum Coincidencia { Abierta { id, status, nombre }, Cerrada { id, status, fecha }, Ninguna }
pub fn buscar(features: &[Value], nombre: &str) -> Coincidencia   // abierta gana sobre cerrada
pub fn por_clave(features: &[Value], clave: &str) -> Option<id>

// add.rs, antes de load/escribir:
if let Some(k) = clave && let Some(id) = por_clave(..) { println!("Feature #{id} ya existe (clave {k})."); return Ok(()) }
match buscar(.., name) {
    Abierta{..} => Err(Exit{2, "Ya existe #N (status): ..."}),
    Cerrada{..} => eprintln!("[i] Mismo nombre que #N (status fecha) ..."),
    Ninguna => {}
}
// y `feature.insert("clave", k)` si vino
```

Estilo (skills `rust-patterns` / `rust-best-practices`, leidos): estados como
enum (`Coincidencia`), `&str` en parametros, sin `unwrap` fuera de tests,
iteradores sobre bucles; tests unitarios en `#[cfg(test)]` del modulo y de
integracion en `rust/tests/cli_basics.rs` con nombres que describen el
escenario (`rust-testing`). `rust-async-patterns` no aplica: `add` es
sincrono y local.

## No funcionales

- Costo: una pasada por el backlog (83 features hoy, miles en el peor caso),
  sin red. Igual que `dependencias::motivo_invalido`.
- Sin cambios para el instalador ni el ps1: `--clave` es del binario.

## Fuera de alcance

- Similitud parcial de nombres (OBS-2).
- Deduplicar features que ya existen: el backlog real tiene 0.

## Observaciones (decisiones pendientes)

- OBS-1 (DECIDIDA 2026-09-07: `--clave`): nombre del flag: `--clave` (el CLI habla castellano: `--sin-worktree`,
  `--leccion`, `--absorbida-por`). Alternativa: `--idempotency-key`, como Hermes.
- OBS-2 (DECIDIDA 2026-09-07: no): ¿aviso por nombres PARECIDOS (Jaccard >= 0.5)? Propuesta: no. Medido
  0 pares; sin evidencia, un umbral es un numero inventado (misma razon que la
  #28 con la confianza).
- OBS-3: la rama de integracion se pregunta antes de `close --status done --to`.
