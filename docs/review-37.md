# Veredicto de revision - Feature #37: estado_superseded

Veredicto global: **aprobado**.

Spec: `docs/spec-feature-37-estado-superseded.md` (15 AC)
Plan: `docs/plan-feature-37-estado-superseded.md` (`Peldano elegido: 1`)
Evidencia: `docs/impl-37.md`
Reporte: `docs/verify-37.md` (15 verde, 0 rojo, 0 manual)

## Estado por AC

Los quince, cubiertos. Lo que merece mirarse:

| AC | Estado | Por que |
| --- | --- | --- |
| AC-5 | cubierto | Enciende `require_spec_approved` **y** `require_leccion` antes de cerrar. Es el escenario exacto que el 2026-08-18 obligo a usar `blocked` como sucedaneo |
| AC-8 | cubierto | **Dos** tests: que ignore las superseded Y que siga contando las `blocked`. El segundo es el que evita arreglar una cosa rompiendo otra |
| AC-10 | cubierto | Sobre el backlog REAL, no una fixture |
| AC-11 | cubierto | La migracion es explicita, y hay un test que lo fija |

## Lo que hace creible esta revision

**El numero cambio y se puede leer**: el PRD maestro paso de `23/36` a `23/30`.
Si no hubiera cambiado, la feature no habria hecho nada — y ese era el criterio
de cierre escrito en el plan.

Y el mapeo previo de los 14 consumidores del campo `status` **redujo la feature**:
tres de los cuatro ya trataban bien un valor nuevo, asi que el cambio de
comportamiento real fue **una linea**. Lo demas son tests de regresion que
convierten "no rompe nada" de afirmacion en contrato. Eso es lo que hay que
hacer antes de agregar un valor a un enum de facto, y esta vez se hizo.

## Lo que verifique ademas de los AC

- **El antes y el despues de `prd tree`**, sobre este repo.
- **Que la referencia se valide de verdad**: `--absorbida-por 99` sale 2, y una
  feature no puede absorberse a si misma.
- **Que una `blocked` de verdad no se toque**: sembrada y comprobada.
- **La suite entera**: 321 + 161 tests, sin una sola regresion, que es lo que
  mas riesgo tenia.

## Observaciones (no bloquean)

1. **El `status` sigue siendo `&str`, no un enum.** Un valor invalido escrito a
   mano en `feature_list.json` pasa desapercibido para todos los consumidores
   salvo clap. Convertirlo tocaria 14 lugares; queda anotado en el backlog en vez
   de hacerse de contrabando en esta feature.
2. **Las superseded ya no dejan rastro en `prd tree`.** Es lo decidido (OBS-1) y
   el costo esta declarado: `status` sigue mostrandolas, el arbol no.
3. **No hay estado para "descartada".** No hay caso real; agregarlo ahora seria
   disenar para un problema que nadie tiene.

## Lo que la revision adversarial encontro despues de los 15 AC verdes

Dos defectos reales que ningun AC cubria, y los dos dicen lo mismo:

1. **`superseded` movia el ticket de Jira a To Do.** El `match` de
   `emit::on_close` tenia `blocked`, `done` y `_`, y el `_` transiciona a
   `pending`. Dano cero aca (no hay binding), pero seis tickets movidos en
   cualquier instalacion con Jira.
2. **La migracion puso en rojo el AC-13 de la #36**, que aceptaba solo
   `done|blocked`. Arreglar el vocabulario rompio una verificacion cerrada dos
   features antes.

Lo que los une: **el mapeo previo de los 14 consumidores fue en Rust y por
igualdad**, y se salteo el unico `match` exhaustivo del repo y un test de shell.
El plan presumia de haber mapeado todo; mapeo lo que sabia buscar.

Los dos estan arreglados y fijados con tests. Y el segundo es la mejor evidencia
a favor de la observacion 1: si `status` fuera un enum, el defecto #1 habria
fallado en `cargo build`.

## Riesgo que queda vivo

Que `superseded` se use para tapar trabajo que no se hizo. Contra eso hay una
defensa verificable —la referencia se valida contra el backlog— y una que no lo
es: que la feature citada de verdad contenga ese trabajo. El arnes puede
comprobar que la feature exista; no puede comprobar que la haya absorbido. Esta
dicho asi en el rol del reviewer, como disciplina.
