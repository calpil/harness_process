Aplicado: 2026-08-22T12:11:07Z por USUARIO (confirmacion explicita)

# Documentos al dia - Feature #49: architecture_en_el_worktree_de_la_feature

Contesta CADA bloque con uno de los tres veredictos y despues corre
`sh harness_cli prd apply --feature 49`:

- `Veredicto: cambio` + `Antes:` y `Despues:` (texto LITERAL del documento)
- `Veredicto: ya-esta <archivo>:<L1>-<L2>` (el binario verifica la cita)
- `Veredicto: no-aplica <razon>` (la razon no puede estar vacia)

## Documento: docs/prd/PRD-master.md

Que cuenta: que se construye y por que
Presente en: -
Ausente en: docs/prd/PRD-master.md (no menciona 'architecture_en_el_worktree_de_la_feature')
Veredicto: no-aplica es un defecto de implementacion de la feature #47, no una capacidad nueva del producto: el hito 8 de la tabla ya promete que los documentos del alcance viajan con la rama de la feature, y esto lo hace cierto para el tercero

## Documento: docs/prd/SDD-master.md

Que cuenta: como se construye, a nivel proyecto
Presente en: -
Ausente en: docs/prd/SDD-master.md (no menciona 'architecture_en_el_worktree_de_la_feature')
Veredicto: ya-esta docs/prd/SDD-master.md:59-61

## Documento: docs/architecture.md

Que cuenta: el mapa de lo que YA existe
Presente en: -
Ausente en: docs/architecture.md (no menciona 'architecture_en_el_worktree_de_la_feature')
Veredicto: cambio
Antes:
- Estado: `feature_list.json` y `progress/` son unicos y del repo principal;
  el estado vivo es `progress/current-<id>.md` por feature y `current.md` pasa a
  ser el indice de lo abierto, con `.last_autocheck-<id>` por feature.
Despues:
- Estado: `feature_list.json` y `progress/` son unicos y del repo principal;
  el estado vivo es `progress/current-<id>.md` por feature y `current.md` pasa a
  ser el indice de lo abierto, con `.last_autocheck-<id>` por feature.
- Documentos: los tres del alcance del cierre (el PRD de origen y sus padres, el
  SDD y `architecture.md`) se resuelven contra el `docs/` de la feature, asi que
  `prd apply` los escribe dentro de su worktree y el merge se los lleva.

