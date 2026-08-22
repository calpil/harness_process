# Evidencia de implementacion - Feature #50: mensaje_de_cierre_dice_la_verdad

Spec: `docs/spec-feature-50-mensaje-de-cierre-dice-la-verdad.md` (approved, 7 AC)
Plan: `docs/plan-feature-50-mensaje-de-cierre-dice-la-verdad.md`

## Que se cambio

En `rust/src/commands/close.rs`, el camino del cierre que NO integra:

```
antes:  println!("  Rama {rama} conservada (...); su worktree tambien.");
        // afirmaba sobre las dos cosas sin mirar ninguna

despues: let hay_rama = git::rama_existe(&principal, &rama);
         let hay_worktree = worktree.is_dir();
         if let Some(linea) = mensaje_conservacion(&rama, status, hay_rama, hay_worktree) {
             println!("{linea}");
         }
```

`mensaje_conservacion()` es una funcion **pura**: recibe dos booleanos y el
nombre de la rama, y devuelve el texto o `None`. Toda la decision vive ahi, asi
que la tabla de casos se prueba sin repo, sin archivos y sin proceso.

## Evidencia por AC

| AC | Estado | Evidencia |
| --- | --- | --- |
| AC-1 rama + worktree | OK | Unit `mensaje_conservacion_should_only_claim_what_exists` (caso `(true, true)`): dice "conservados", nombra la rama y el estado. El test de integracion `close_blocked_should_keep_branch_and_worktree` verifica el texto en la salida real |
| AC-2 solo la rama | OK | Mismo unit, caso `(true, false)`: "conservada" + "worktree ya no esta" |
| AC-3 solo el worktree | OK | Mismo unit, caso `(false, true)`: "rama ... ya no esta" + "queda su worktree" |
| AC-4 nada -> silencio | OK | Mismo unit, caso `(false, false)`: devuelve `None`. Es el caso que motivo la feature |
| AC-5 sin rama en el backlog | OK | El `let Some(rama) = rama else { return Ok(()) }` previo no se toco: sin `branch`, el cierre no imprime nada, como siempre |
| AC-6 el resto del cierre intacto | OK | Solo cambio esa rama del `if`; los 170 tests de integracion (estado, archivado, gates, GitFlow) siguen verdes sin tocarlos |
| AC-7 comandos oficiales | OK | `cargo test`: 348 unit + 170 integracion = **518**; `clippy --all-targets -- -D warnings` limpio; `setup_smoke.sh` exit 0; `harness_check.sh` limpio |

## Un test que hizo su trabajo

`close_blocked_should_keep_branch_and_worktree` (de la feature #47) fallo al
cambiar el texto, porque esperaba el "conservada" viejo. Se actualizo al plural
("y su worktree conservados") con un comentario que explica por que. Es
exactamente lo que se le pide a un test de salida: avisar cuando el contrato con
el lector cambia.

## Nota sobre el origen

El caso lo encontro el uso real — borrar a mano la rama y el worktree de la #48
y cerrarla despues —, no la suite. Queda como segundo ejemplo del "OK que dice
de mas" en la leccion `probar-contra-datos-reales`.
