# Arquitectura

Completa este archivo con:

- Microservicios y responsabilidades.
- Dependencias internas y externas.
- Servicios transversales.
- Riesgos conocidos.
- Flujos criticos.
- Flujo SDD: como `docs/constitution.md` (principios) y `docs/spec-feature-*.md`
  (criterios de aceptacion AC-n) guian el plan y la implementacion. Un AC puede
  declarar debajo `Comando: <shell>`; `sh harness_cli verify --feature <id>` los
  ejecuta y deja `docs/verify-<id>.md`. Es el unico comando que ejecuta shell:
  exige el spec aprobado y no lo llama ningun hook.
- Los tres almacenes de memoria, si usas el aprendizaje del arnes: el Memory Hub
  guarda **eventos**, `docs/lecciones/<clase>.md` guarda **procedimiento** (como
  se hace esta clase de tarea aca) y no se solapan. Las lecciones son archivos
  versionados y funcionan con el hub caido.
