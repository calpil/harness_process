---
nombre: docs-generados-por-el-instalador
descripcion: Sumar un doc al arnes es una linea en HARNESS_DOCS, no codigo nuevo.
triggers: [HARNESS_DOCS, HarnessDocs, siembra, reset, plantilla, templates/docs, install_asset]
relacionadas: []
origen: [17]
usos: 1
ultimo_uso: 2026-08-17
ultima_actualizacion: 2026-08-16
estado: archivada
---

## Cuando aplica

Cuando una feature necesita que el arnes **siembre un documento nuevo** en el
`docs/` de la RAIZ del proyecto (una guia, una plantilla, un metodo), y hay que
decidir si ese archivo se refresca al reinstalar o si es del usuario.

Sintoma de que estas por hacerlo mal: te encontras escribiendo una funcion nueva
en `setup_harness.sh` para copiar un archivo, o agregando una ruta a mano en la
lista de reset targets.

## Procedimiento

1. Escribi la plantilla en `templates/docs/<ruta>.md`. La ruta puede llevar
   subdirectorio (`prd/...`, `lecciones/...`): los consumidores de la lista crean
   el directorio destino.
2. Agrega esa misma ruta relativa a **una sola lista**, en los dos instaladores:
   - `HARNESS_DOCS` en `setup_harness.sh`
   - `$script:HarnessDocs` en `setup_harness.ps1`
3. No escribas nada mas. Esa lista ya tiene tres consumidores y de ahi salen
   gratis: la **siembra** (solo si falta, nunca pisa), los **reset targets** y la
   **migracion** de instalaciones viejas que tenian el doc en otra ubicacion.

Para decidir en que lista va:

| Si el documento... | Va en | Consecuencia |
| --- | --- | --- |
| es plantilla del arnes (se refresca al reinstalar) | `HARNESS_DOCS` | entra a los reset targets |
| es del USUARIO (se siembra una vez y nunca se pisa) | `PRD_DOCS` / `$script:PrdDocs` | NO entra a los reset targets |
| es contenido ganado del proyecto (specs, planes, lecciones) | **ninguna lista** | sobrevive a `--reset` por omision |

## Pitfalls

- **Listar contenido ganado en `HARNESS_DOCS` lo borra con `--reset`.** La regla
  es al reves de lo que parece: lo que NO esta en ninguna lista es lo que
  sobrevive. Antes de agregar una ruta, preguntate si un `--reset` puede borrarla
  sin que duela.
- **Tocar solo `setup_harness.sh`.** Las dos listas son gemelas y el `.ps1` no
  hereda nada: si agregas la ruta en una sola, Windows queda sin el documento y
  el smoke de PowerShell no lo detecta salvo que lo assertes explicitamente.
- **Olvidarse del espejo `templates/` <-> raiz** para todo lo que ademas se copia
  (scripts como `harness_check.sh`). El gate de espejos del propio
  `harness_check.sh` lo detecta, pero recien cuando alguien lo corre.

## Verificacion

```bash
# La ruta esta en las dos listas
grep -n "<ruta>.md" setup_harness.sh setup_harness.ps1

# Siembra e idempotencia (no pisa lo existente) + supervivencia al reset
bash tests/setup_smoke.sh
```
