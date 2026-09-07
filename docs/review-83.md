# Review - Feature #83: el Stop hook bloquea por graphify-out/.graphify_stale
Revisado: approved · 2026-09-07T01:48:15Z · estampado por `harness revision --veredicto`

Revisor: la misma sesion que implemento. Metodo: correr el test nuevo contra el
`harness_check.sh` de HEAD y leer por que cayo; despues con el fix; y los dos
checks que rodean al script (stop hook, paridad).

## Cobertura por AC

| AC | archivo:linea | veredicto |
| --- | --- | --- |
| AC-1 | harness_check.sh:142 · tests/graphify_stale_check.sh:34 | CUBIERTO. Rojo contra HEAD por el `[!]`; con el fix, rc 0 con el marcador. El test primero prueba que el fixture pasa limpio sin marcador, asi que el rc mide solo al marcador. |
| AC-2 | harness_check.sh:143 · tests/graphify_stale_check.sh:35 | CUBIERTO. El aviso nombra post-commit, `/graphify --update` y "No bloquea"; el test exige los tres. |
| AC-3 | templates/harness_check.sh:142 | CUBIERTO. `cmp` limpio. Es el espejo que en la #80 se olvido una vez; aca se hizo en el mismo paso. |
| AC-4 | tests/setup_smoke.sh:1713 | CUBIERTO. Enganchado al smoke; `verify` corre el smoke entero (docs/verify-83.md). |
| AC-5 | UPDATING.md:48 | CUBIERTO. `cmp` limpio entre las dos copias. |

## Lo que el review tiene que decir

El check bloqueaba por un archivo que el propio arnes crea en cada cierre y
que solo el propio arnes puede limpiar, con un debounce de 30 minutos. Es la
tercera vez que un enriquecimiento best-effort se cuela como gate (nudge en la
#18, avisos en la #80, este marcador): la regla es la misma, lo que no impide
trabajar se avisa. La severidad de cada `[!]` del check merece una pasada
entera con esa pregunta, pero excede este bug.

El primer `verify` dio dos rojos por una precondicion (sin binario en el
worktree), la misma forma que la #78 documento. Se arreglo en los dos tests de
fixture con un fallback al binario del checkout principal; el segundo `verify`
es el que vale.

## Riesgo declarado

- Nadie va a ver el `[i]` si no lee stderr del Stop hook: el grafo puede quedar
  sin enriquecer mas tiempo que antes. Es a proposito: el costo de un rebuild
  con LLM lo decide el usuario, no el Stop.

## Veredicto

Cinco AC con cobertura. El test cae contra HEAD por el `[!]` y pasa con el fix;
stop hook y paridad verdes; el smoke lo corre `verify`.
