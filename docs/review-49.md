# Veredicto del reviewer - Feature #49: architecture_en_el_worktree_de_la_feature

Veredicto: **approved**
Fecha: 2026-08-22
Spec: `docs/spec-feature-49-architecture-en-el-worktree-de-la-feature.md` (approved, 6 AC)
Evidencia: `docs/impl-49.md`

## Verificacion oficial

| Comando | Resultado |
| --- | --- |
| `cargo test` | en verde (17 del modulo `documentos`, dos nuevos) |
| `cargo clippy --all-targets -- -D warnings` | limpio |
| `bash tests/setup_smoke.sh` | exit 0 |
| `./harness_check.sh` | limpio |

## Cobertura de los AC

6 de 6. El cambio es de una linea y los dos tests nuevos cubren los dos lados:
con worktree la ruta sale del `docs/` de la feature, y sin worktree vuelve a la
raiz sin cambiar nada.

Lo que hace bueno al test de AC-4 es el fixture: siembra DOS `architecture.md`
(uno en el docs/ de la feature y otro en la raiz). La version con el bug
encontraba el de la raiz y pasaba `is_file()` sin quejarse — por eso la deuda
sobrevivio a 515 tests. Un test que solo sembrara uno no la habria detectado.

## Constitution

- **Articulo 1**: dos tests nuevos junto al codigo tocado y los cuatro comandos
  oficiales en verde.
- **Articulo 2**: spec `approved` antes de implementar.
- **Articulo 3**: D1..D3 citan sus AC-n; la evidencia se organiza por AC.
- **Articulo 4**: sin cambios de seguridad; no toca permisos ni escribe fuera
  del arbol.
- **Articulo 5**: sin decisiones abiertas; la unica observacion (OBS-1) explica
  el origen de la deuda.
- **Articulo 6**: sin dependencias nuevas; `expect()` evitado con `let-else`
  para respetar el lint del repo.

## Reparos / observaciones del reviewer

1. **La deuda venia de la feature #47 y la encontro su verificacion de cierre,
   no un test.** Es el mismo patron que la leccion `probar-contra-datos-reales`
   ya registra: los tests corrian desde una sola ubicacion. El AC-4 existe para
   que la proxima vez la encuentre la suite.
2. **Sin auditoria pendiente**: se verifico por grep que `architecture.md` era
   el UNICO lugar de produccion que armaba una ruta de docs contra `repo_root`;
   los otros dos usos son un test y el fallback correcto de `paths.rs`.
