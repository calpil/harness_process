---
nombre: documentos-del-usuario-vs-plantillas
descripcion: Tres tipos de archivo, tres listas: la que elijas decide si sobrevive.
triggers: [USER_DOCS, PRD_DOCS, HARNESS_DOCS, reset, documento del usuario, plantilla, siembra]
relacionadas: [docs-generados-por-el-instalador]
origen: [19]
usos: 0
ultimo_uso:
ultima_actualizacion: 2026-08-16
estado: activa
---

## Cuando aplica

Cuando agregas un archivo nuevo que el arnes siembra en el proyecto destino y
tenes que decidir **quien es su duenno**. La leccion
[[docs-generados-por-el-instalador]] explica el mecanismo (una linea en una
lista); esta explica **cual** lista, que es la decision que no se puede
deshacer despues sin romperle el trabajo a alguien.

Sintoma de que lo pensaste mal: alguien corre `--reset` y pierde algo que habia
escrito, o reinstala y el arnes le pisa un archivo propio.

## Procedimiento

Hacete UNA pregunta: **si esto se borra, quien pierde trabajo?**

| Si se borra... | Es | Va en | Reset |
| --- | --- | --- | --- |
| no pierde nadie, se regenera igual | **plantilla del arnes** | `HARNESS_DOCS` | lo limpia y lo refresca |
| pierde el USUARIO lo que escribio | **documento del usuario** | `PRD_DOCS` (bajo `docs/prd/`) o `USER_DOCS` (en `docs/` a secas) | **no lo toca** |
| pierde el PROYECTO conocimiento ganado | **contenido ganado** | **ninguna lista** | **no lo toca** |

Las tres listas existen en los dos instaladores y son gemelas: `HARNESS_DOCS` /
`$script:HarnessDocs`, `PRD_DOCS` / `$script:PrdDocs`, `USER_DOCS` /
`$script:UserDocs`.

Un caso mixto y frecuente: **la guia es plantilla, el contenido no**. La guia de
lecciones (`COMO-ESCRIBIR-UNA-LECCION.md`) va en `HARNESS_DOCS` y se refresca; las
lecciones en si no van en ninguna lista y sobreviven. Lo mismo con el PRD: el
metodo (`COMO-ESCRIBIR-UN-PRD.md`) es plantilla, el `PRD-master.md` es del
usuario.

## Pitfalls

- **Meter un documento del usuario en `HARNESS_DOCS`.** Es el error caro: el
  primer `--reset` se lleva lo que escribio. Ante la duda, NO lo listes: lo que
  no esta en ninguna lista sobrevive, y siempre podes agregarlo despues.
- **Asumir que "documento del usuario" implica `docs/prd/`.** `PRD_DOCS` siembra
  bajo `docs/prd/`; para un documento del usuario que vive en `docs/` a secas
  (como `perfil-usuario.md`) hace falta `USER_DOCS`, con su propio bucle.
- **Que la plantilla del instalador y la del binario diverjan.** Si el binario
  tambien puede crear el archivo (por ejemplo, cuando falta en una instalacion
  vieja), hay DOS fuentes del mismo encabezado. Un test que compare las dos lo
  atrapa; sin el, un repo instalado y uno migrado terminan distintos.
- **Olvidar el `.ps1`.** Las listas son gemelas y el `.ps1` no hereda nada.

## Verificacion

```bash
# En que lista quedo (tiene que aparecer en las dos gemelas y en ninguna otra)
grep -n "<archivo>" setup_harness.sh setup_harness.ps1

# Y lo que de verdad importa: que sobreviva a un reset
bash tests/setup_smoke.sh    # los sentinels de reset cubren cada categoria
```
