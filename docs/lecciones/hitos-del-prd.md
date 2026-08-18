---
nombre: hitos-del-prd
descripcion: La celda del slug se compara literal: sin backticks no se marca.
triggers: [PRD, hitos, echo_close, prd tree, close --status done, bitacora, slug]
relacionadas: [docs-generados-por-el-instalador]
origen: [17]
usos: 0
ultimo_uso:
ultima_actualizacion: 2026-08-16
estado: activa
---

## Cuando aplica

Cuando escribis o completas la tabla `## 10. Hitos -> features` de un PRD
(maestro o anidado) y esperas que `close --status done` marque el hito solo.

Sintoma de que esta mal: al cerrar, el arnes dice
`PRD actualizado (bitacora)` en vez de `PRD actualizado (hito marcado done +
bitacora)`, y la columna `Estado` de la fila sigue en `pendiente` aunque la
feature quedo cerrada. `prd tree` sigue contando el hito como no cumplido.

## Procedimiento

1. La tercera columna de la fila (`Slug de feature`) lleva el slug **pelado**,
   exactamente igual al campo `name` de la feature en `feature_list.json`:

   ```
   | 1 | Lecciones por clase | lecciones_memoria_procedural | O1 | ... | pendiente |
   ```

2. **No** lo pongas entre backticks, ni lo adornes: `echo_close` compara la celda
   contra `feature["name"]` **literal**. Un `` `slug` `` no matchea con `slug`.
3. La ultima columna es la que el arnes reescribe a `done (YYYY-MM-DD)`. Dejala
   en `pendiente`.
4. Verifica el nombre exacto antes de escribir la fila:

   ```bash
   sh harness_cli status | grep '#<id>'
   ```

## Pitfalls

- **Backticks en la celda del slug.** El caso que origino esta leccion: el hito
  quedo en `pendiente` y solo se escribio la bitacora. El cierre no falla ni
  avisa — la vuelta al PRD es best-effort por diseno (un PRD ausente o mal
  formado NUNCA puede impedir cerrar una feature), asi que el unico indicio es
  el texto `(bitacora)` sin `hito marcado done`.
- **Cambiar el nombre de la feature despues de escribir la fila.** El match es
  por nombre, no por id: si renombras la feature en el backlog, la fila deja de
  matchear.
- **Re-cerrar para arreglarlo.** Volver a correr `close --status done` SI marca
  el hito una vez corregida la fila (la bitacora no se duplica: detecta la
  entrada por `- #<id> <nombre> -> done`), pero deja una segunda linea `close` en
  `progress/history.md` y otro `Cerrado:` al pie del plan. Es recuperable, no
  gratis: mejor revisar la fila antes del primer cierre.

## Verificacion

```bash
# Al cerrar, el mensaje tiene que decir "hito marcado done + bitacora"
sh harness_cli close --feature <id> --status done --note "..."

# Y el arbol tiene que contarlo
sh harness_cli prd tree     # -> "features: N/M done" sube
grep -o "| <slug> |.*|" docs/prd/<parte>/PRD-<cadena>.md   # -> Estado: done (fecha)
```
