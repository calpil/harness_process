# Impl - Feature #61: el_merge_del_cierre_no_toca_tu_checkout

Spec: docs/spec-feature-61-el-merge-del-cierre-no-toca-tu-checkout.md
Origen: el cierre de la #60 fallo con este bug (2026-08-27), y el spec de la #60
lo dejo anotado como hermano.

## El diagnostico

`git.rs` prometia en su cabecera que "el merge corre en un worktree temporal (no
toca tu checkout)" y que "el cierre de una feature no puede exigirte tener el
escritorio ordenado". `merge_en` tenia una excepcion silenciosa:

```rust
if rama_actual(principal).as_deref() == Some(destino) {
    return merge_aqui(principal, destino, rama);   // <-- TU checkout
}
```

El comentario la justificaba con "git no permite dos worktrees sobre la misma
rama". Es cierto, pero **si permite un worktree en HEAD detached sobre su
commit**, que es todo lo que hace falta para mergear aislado. Medido:

```
$ git worktree add --detach /tmp/wt main    # con main checkouteada en el principal
Preparing worktree (detached HEAD 063b374)  # funciona
```

Y para sincronizar despues:

```
$ git reset --keep <merge>
  con B.md sucio (el merge no lo toca)  -> avanza y CONSERVA B.md
  con A.md sucio (el merge si lo toca)  -> aborta sin cambiar nada
```

Se evaluo y se DESCARTO la alternativa de avanzar la rama dejando el arbol
atras. Medido: tras `update-ref` con el arbol en el commit viejo,
`git status` muestra `MM A.md` y `git diff` muestra la REVERSION del merge; un
commit distraido desharia el trabajo recien integrado.

## Que cambio

| Archivo | Cambio |
| --- | --- |
| `rust/src/git.rs` | `merge_en` usa SIEMPRE `worktree add --detach`; nueva `avanzar_rama` (`reset --keep` / `update-ref` con guarda); nuevas `colisiones`, `sucios`, `archivos_del_merge`, `ruta_de_status`; cabecera sin la excepcion muda |
| `rust/src/commands/close.rs` | `integrar` consulta `colisiones` ANTES de anunciar, commitear o mergear; `mensaje_de_colision` (pura) |
| `README.md`, `UPDATING.md`, `templates/UPDATING.md`, `docs/architecture.md` | documentacion |

## Evidencia por AC

- **AC-1**: `cargo test merge_en_la_rama_abierta_no_usa_el_checkout_principal`.
  Cierra hacia `main` estando en `main`: el merge queda hecho, no hay
  `.git/MERGE_HEAD` en el principal, y el reflog de HEAD **no** contiene
  `merge bugfix/1-cobranza` — llego ahi por un `reset`, no por haber mergeado.
- **AC-2**: `cargo test cierre_con_cambios_sin_commitear_que_no_chocan`. Con
  `NOTAS.md` modificado sin commitear, el cierre integra y el archivo conserva
  el contenido local intacto.
- **AC-3**: `cargo test colision_se_detecta_antes_de_tocar_nada`. Con el mismo
  archivo sucio en el checkout y tocado por la feature: exit 2, y se verifica
  que `main` no se movio, que la rama NO tiene commit de cierre, que no hay
  `MERGE_HEAD`, y que los dos contenidos (el del usuario y el de la feature)
  quedaron intactos.
- **AC-4**: `cargo test mensaje_de_colision_nombra_archivos_y_remedio`. Funcion
  pura: nombra cada archivo, da las tres salidas, marca cual DESCARTA, dice que
  no paso nada y como retomar con la feature y la rama reales.
- **AC-5**: `cargo test merge_a_rama_no_checkouteada_sigue_funcionando`. Cierre
  a `develop` estando en `main`: `develop` recibe el merge y `main` no se toca.
- **AC-6**: `cargo test conflicto_real_no_deja_nada_a_medias`. Los dos lados
  commitean la misma linea: el cierre falla, `main` no se movio, no hay
  `MERGE_HEAD` y no queda ningun worktree `harness-merge-` suelto.
- **AC-7**: `cargo test colisiones_solo_consulta_y_no_muta` (+
  `colisiones_ignora_lo_sucio_que_el_merge_no_toca`,
  `colisiones_vacias_si_el_destino_no_esta_abierto`,
  `archivos_del_merge_incluye_lo_sin_commitear_del_worktree`,
  `ruta_de_status_lee_las_formas_de_porcelain`). Se llama con el arbol sucio y
  despues HEAD, `git status` y el contenido siguen identicos.
- **AC-8**: la cabecera de `rust/src/git.rs` documenta la regla sin excepcion y
  explica el caso irreductible con las dos alternativas descartadas y por que.

## Dos bugs que encontraron los tests

1. **El worktree temporal no era unico.** Se llamaba
   `harness-merge-<destino>-<pid>`: dos merges del mismo proceso (o dos tests en
   paralelo) se pisaban, y uno borraba el worktree del otro a mitad del merge.
   Solo se hizo visible al empezar a usar worktree SIEMPRE. Ahora es un
   `tempfile::TempDir` por invocacion. Rompio un test preexistente
   (`merge_should_integrate_and_keep_history`), que es como se descubrio.
2. **`git status --porcelain` parseado por posicion fija.** `git()` le hace
   `trim()` a la salida, asi que la PRIMERA linea pierde el espacio de la
   columna X: ` M A.md` llega como `M A.md` y cortar en la columna 3 devolvia
   `.md`. La colision no se detectaba nunca. Ahora se parte por el primer
   espacio despues de los codigos, que funciona con y sin ese espacio y respeta
   las rutas con espacios.

## Lo que NO se toco

Sin `git stash` automatico ni detras de un flag (decision USUARIO 2026-08-27).
El fallo del merge sigue saliendo con exit 1 como en la #47; el GATE de colision
sale 2, como el resto de los gates del cierre.

**Deuda anotada, no pagada:** `close` escribe `status: done` en el backlog ANTES
de integrar, asi que una integracion fallida deja la feature marcada `done` sin
estar integrada. Paso al cerrar la #60. Esta feature lo reduce (el gate corre
antes de commitear y mergear) pero no lo elimina: el cierre sigue sin ser
transaccional. Esta declarado en Fuera de alcance del spec.
