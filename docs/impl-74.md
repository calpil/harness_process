# Impl - Feature #74: add no protege contra la feature duplicada

Spec: docs/spec-feature-74-add-no-protege-contra-la-feature-duplicada-y-no-.md
Plan: docs/plan-feature-74-add-no-protege-contra-la-feature-duplicada-y-no-.md

## Lo que habia (y lo que se midio)

`add` validaba `--kind`, `--prd` y `--depends-on` y despues escribia: ningun
control sobre el nombre. Medido antes de tocar nada, sobre las 83 features
reales: cero nombres normalizados repetidos y cero pares con solapamiento de
palabras >= 50 %. La feature se habia cerrado `blocked` por eso el 2026-09-06;
el usuario la reabrio el 2026-09-07 por el escenario que no deja huella (dos
sesiones, un reintento, un script), y el spec lo dice tal cual.

## El arreglo: un modulo puro y tres lineas de decision

| AC | archivo:linea | evidencia |
| --- | --- | --- |
| AC-1 | rust/src/duplicados.rs:79 · rust/src/commands/add.rs:70 · rust/tests/cli_basics.rs:8769 · :8792 | `buscar` devuelve `Coincidencia::Abierta` si alguna feature en `ABIERTAS` (:29, `blocked` incluida) tiene el mismo nombre normalizado; `add` responde exit 2 con `Ya existe #id (status): "nombre"` ANTES de `load`->`push`. Tests: nombre con mayusculas, acentos, puntuacion y espacios de mas -> rechazo, backlog y `history.md` byte-identicos; una feature `blocked` cuenta como abierta. |
| AC-2 | rust/src/duplicados.rs:79 · rust/src/commands/add.rs:70 · rust/tests/cli_basics.rs:8818 | Sin abiertas, `Coincidencia::Cerrada` con la fecha de `closed_at`; `add` avisa `[i] Mismo nombre que #id (done fecha)` por stderr y crea la feature. Test: sobre una `done`, se crea la #2 y stdout sigue diciendo `Feature #2 agregada.`. |
| AC-3 | rust/src/duplicados.rs:109 · rust/src/commands/add.rs:61 · :149 · rust/src/cli.rs:146 · rust/tests/cli_basics.rs:8845 | `--clave <k>` (opcional): `por_clave` encuentra la feature que la lleva y `add` imprime `Feature #N ya existe (clave k).`, exit 0, sin escribir, sin `log`, sin `emit::on_add`; si no existe, la feature guarda `"clave"`. Test: segunda corrida con OTRO nombre y la misma clave -> backlog e history identicos. |
| AC-4 | rust/src/duplicados.rs:36 · :126 · :131 | `normalizar`: minusculas, sin acentos (tabla fija), solo `[a-z0-9]`, sin las 21 palabras vacias de `VACIAS`, un espacio entre palabras. Tests unitarios: iguales con mayusculas/acentos/puntuacion; distintos por una palabra; vacio si solo hay palabras vacias. |
| AC-5 | rust/tests/cli_basics.rs:8869 | Sin `--clave` y sin duplicado, la feature tiene exactamente `acceptance, id, microservicios, name, status`: ningun campo nuevo. Este test ya pasaba contra HEAD (es la guarda de regresion) y sigue pasando. |
| AC-6 | README.md:1216 · UPDATING.md:48 · rust/src/cli.rs:146 | Parrafo en README tras el bloque de `add`; seccion en UPDATING (dos copias, `cmp`); doc del flag en `--help`. |
| AC-7 | rust/src/commands/add.rs:16 | Suite 483 + 273, clippy limpio (el octavo argumento de `add::run` disparo `too_many_arguments`: se agrupo lo opcional en `AltaOpts`, como `CierreOpts` en `close`), paridad 10/10. |

## El rojo

Los cinco tests de integracion se corrieron contra el binario de HEAD antes de
escribir el modulo: AC-1 (dos tests) cayo por `code=0` donde se esperaba 2;
AC-2 por stderr sin el `[i]`; AC-3 por `unexpected argument '--clave'` (el
flag ES la feature; es la unica caida por precondicion, y es la esperada);
AC-5 paso, como corresponde a una guarda de regresion.

Mutaciones (reviewer): quitar `to_lowercase()` de `normalizar` tumba
`normalizar_nombre_should_ignore_case_...`; sacar `blocked` de `ABIERTAS` tumba
`buscar_should_prefer_an_open_feature_...`. En las dos corridas cargo se
detuvo en el primer binario de tests que fallo (los unitarios), asi que los de
integracion no llegaron a correr con el mutante: la evidencia de las mutaciones
es la unitaria. `cmp` antes y despues, archivo restaurado y `touch`.

## Estilo (skills cargados: rust-patterns, rust-testing, rust-best-practices)

- Estados como enum (`Coincidencia`), sin `unwrap` fuera de tests, `&str` y
  `&[Value]` en parametros, iteradores (`filter`/`find`) en `buscar` y
  `por_clave`, modulo puro sin IO.
- Tests nombrados como una oracion que describe el escenario, un
  comportamiento por test (el caso `blocked` es un test aparte).
- `rust-async-patterns` no aplica: `add` es sincrono.

## Lo que NO hace

- No avisa por nombres parecidos (OBS-2, decision del usuario: medido 0 pares).
- No migra ni toca features existentes; `clave` solo aparece si vino.
- No cambia el instalador ni el ps1.
