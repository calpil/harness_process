# Evidencia de implementacion - Feature #58: el_guard_no_bloquea_por_lo_que_escribe_el_arnes

Spec: `docs/spec-feature-58-el-guard-no-bloquea-por-lo-que-escribe-el-arnes.md` (approved, 10 AC)
Plan: `docs/plan-feature-58-el-guard-no-bloquea-por-lo-que-escribe-el-arnes.md`

## Que se construyo

Dos funciones en `commit_guard.sh` (y su plantilla): `es_artefacto_del_arnes()`
—la lista de patrones, aceptando la ruta con o sin prefijo `docs/`— y
`solo_artefactos_del_arnes()`, que devuelve 0 solo si hubo cambios y **todos**
son artefactos. El bucle que arma `DIRTY` consulta esa segunda y, cuando aplica,
imprime una linea `[i]` en vez de sumar el repo a la lista.

La exencion es **por artefacto, no por carpeta** (OBS-1): alcanza UN archivo
ajeno para que el repo vuelva a contar como sucio.

## La prueba que importa: el proyecto real

A/B sobre `GolandProjects/realestate` con un artefacto del arnes sin commitear
en `docs/`, sin tocar su instalacion:

```
ANTES (guard instalado hoy)
Cambios sin commitear en: docs ms-brokerage-service ms-client-service [...]

DESPUES (guard de esta feature)
[i] docs: solo artefactos del arnes sin commitear (los commitea 'close'); no cuenta como sucio.
Cambios sin commitear en: ms-brokerage-service ms-client-service [...]
```

`docs` sale de la lista; los ocho microservicios con codigo sin commitear siguen
bloqueando, que es para lo que el guard existe.

## Evidencia por AC

| AC | Estado | Evidencia |
| --- | --- | --- |
| AC-1 solo artefactos no bloquea | OK | Smoke `Guard #58` (caso 1) y fixture a mano: `rc=0` con `spec-feature-*`, `impl-*` y `prd/PRD-master.md` sucios. Tambien con un artefacto MODIFICADO, no solo sin trackear |
| AC-2 lo dice | OK | `[i] docs: solo artefactos del arnes sin commitear (los commitea 'close'); no cuenta como sucio.` Asserteado en el smoke |
| AC-3 el codigo sigue bloqueando | OK | Smoke: con `ms-auth/main.go` sucio, `rc=2` y el mensaje nombra `ms-auth` |
| AC-4 mixto bloquea | OK | Smoke: con codigo en otro repo, `docs` se sigue eximiendo y `ms-auth` bloquea; en el MISMO repo lo cubre el caso de AC-5 |
| AC-5 doc ajeno bloquea | OK | Smoke: `docs/runbook.md` sin commitear -> bloquea. La exencion es por archivo |
| AC-6 viaja por el instalador | OK | El cambio esta en `templates/commit_guard.sh`; el smoke prueba la copia **instalada** (`$SUBDIR_HARNESS/commit_guard.sh`), no la fuente |
| AC-7 paridad ps1 | OK | El `.ps1` no tiene logica de guard: copia el mismo archivo generado. No hay nada que duplicar (verificado: `commit_guard` no aparece con logica propia en el ps1) |
| AC-8 los cuatro comandos | OK | 362 unit + 177 integracion = **539**, clippy 0, smoke exit 0, check limpio |
| AC-9 casos en el smoke | OK | Bloque `Feature #58` con seis casos: solo-artefactos, la linea `[i]`, doc ajeno, codigo, artefacto modificado y el `impl-*.md` dentro de un microservicio. Ademas el smoke se aisla del entorno (`unset HARNESS_REPO_ROOT`). No tiene `Comando:`: dentro de `verify` el smoke se cuelga por la feature #46 (pipes leidos despues de esperar al proceso). Corrido a mano: **exit 0** |
| AC-10 el proyecto real | OK | A/B de arriba, sobre `GolandProjects/realestate` |

## Nota sobre el caso que lo disparo

El error que reporto Alan traia ademas dos lineas informativas
(`PRD-master.md no declara hitos todavia`) que NO son el fallo: son correctas —
esa tabla esta vacia en ese proyecto— y quedaron fuera de alcance.
