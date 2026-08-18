---
nombre: documentos-del-usuario-vs-plantillas
descripcion: Tres tipos de archivo, tres listas: la que elijas decide si sobrevive.
triggers: [USER_DOCS, PRD_DOCS, HARNESS_DOCS, HarnessDocs, reset, documento del usuario, plantilla, siembra, templates/docs, install_asset]
relacionadas: [docs-generados-por-el-instalador]
origen: [19, 17]
usos: 0
ultimo_uso:
ultima_actualizacion: 2026-08-18
estado: activa
---

## Cuando aplica

Cuando una feature necesita que el arnes **siembre un documento nuevo** en el
`docs/` de la RAIZ del proyecto (una guia, una plantilla, un metodo), y hay que
decidir **quien es su duenno** y si se refresca al reinstalar.

Son dos preguntas y se contestan juntas: **en que lista va** (la decision, que no
se puede deshacer sin romperle el trabajo a alguien) y **como se agrega** (el
mecanismo, que es una linea y no codigo nuevo).

Sintomas de que lo estas haciendo mal:

- Te encontras escribiendo una funcion nueva en `setup_harness.sh` para copiar un
  archivo, o agregando una ruta a mano a los reset targets.
- Alguien corre `--reset` y pierde algo que habia escrito.
- Alguien reinstala y el arnes le pisa un archivo propio.

## Procedimiento

### 1. La decision: ¿en que lista va?

Hacete UNA pregunta: **si esto se borra, quien pierde trabajo?**

| Si se borra... | Es | Va en | Reset |
| --- | --- | --- | --- |
| no pierde nadie, se regenera igual | **plantilla del arnes** | `HARNESS_DOCS` | lo limpia y lo refresca |
| pierde el USUARIO lo que escribio | **documento del usuario** | `PRD_DOCS` (bajo `docs/prd/`) o `USER_DOCS` (en `docs/` a secas) | **no lo toca** |
| pierde el PROYECTO conocimiento ganado | **contenido ganado** | **ninguna lista** | **no lo toca** |

Un caso mixto y frecuente: **la guia es plantilla, el contenido no**. La guia de
lecciones (`COMO-ESCRIBIR-UNA-LECCION.md`) va en `HARNESS_DOCS` y se refresca;
las lecciones en si no van en ninguna lista y sobreviven. Lo mismo con el PRD: el
metodo (`COMO-ESCRIBIR-UN-PRD.md`) es plantilla, el `PRD-master.md` es del
usuario.

### 2. El mecanismo: es una linea, no codigo

1. Escribi la plantilla en `templates/docs/<ruta>.md`. La ruta puede llevar
   subdirectorio (`prd/...`, `lecciones/...`): los consumidores de la lista crean
   el directorio destino.
2. Agrega esa misma ruta relativa a **una sola lista**, en los **dos**
   instaladores. Las tres listas son gemelas:
   `HARNESS_DOCS` / `$script:HarnessDocs`, `PRD_DOCS` / `$script:PrdDocs`,
   `USER_DOCS` / `$script:UserDocs`.
3. No escribas nada mas. Esa lista ya tiene tres consumidores y de ahi salen
   gratis: la **siembra** (solo si falta, nunca pisa), los **reset targets** y la
   **migracion** de instalaciones viejas que tenian el doc en otra ubicacion.

## Pitfalls

- **Meter en `HARNESS_DOCS` algo que no se regenera solo.** Es el error caro, y
  tiene dos caras: un documento del USUARIO listado ahi se lo lleva el primer
  `--reset`, y contenido GANADO del proyecto (specs, planes, lecciones) tambien.
  La regla es al reves de lo que parece: **lo que NO esta en ninguna lista es lo
  que sobrevive**. Ante la duda no lo listes; agregarlo despues siempre se puede.
- **Tocar solo `setup_harness.sh`.** Las listas son gemelas y el `.ps1` **no
  hereda nada**: si agregas la ruta en una sola, Windows queda sin el documento.
  Desde la feature #30, `bash tests/parity_check.sh` compara los dos
  instaladores y falla si uno se adelanta.
- **Asumir que "documento del usuario" implica `docs/prd/`.** `PRD_DOCS` siembra
  bajo `docs/prd/`; para un documento del usuario que vive en `docs/` a secas
  (como `perfil-usuario.md`) hace falta `USER_DOCS`, con su propio bucle.
- **Que la plantilla del instalador y la del binario diverjan.** Si el binario
  tambien puede crear el archivo (por ejemplo cuando falta en una instalacion
  vieja), hay DOS fuentes del mismo encabezado. Un test que compare las dos lo
  atrapa; sin el, un repo instalado y uno migrado terminan distintos.
- **Olvidarse del espejo `templates/` <-> raiz** para todo lo que ademas se copia
  (scripts como `harness_check.sh`). El gate de espejos del propio
  `harness_check.sh` lo detecta, pero recien cuando alguien lo corre.

## Verificacion

```bash
# 1. En que lista quedo (tiene que aparecer en las dos gemelas y en ninguna otra)
grep -n "<archivo>" setup_harness.sh setup_harness.ps1

# 2. Que los dos instaladores no se hayan desincronizado
bash tests/parity_check.sh

# 3. Y lo que de verdad importa: siembra, idempotencia y supervivencia al reset
bash tests/setup_smoke.sh    # los sentinels de reset cubren cada categoria
```

Regla practica: antes de agregar una ruta a una lista, preguntate si un `--reset`
puede borrarla sin que duela. Si la respuesta es no, esa ruta no va en
`HARNESS_DOCS`.

---

Esta leccion absorbio a [[docs-generados-por-el-instalador]] (feature #17), que
contaba el mismo mecanismo desde el otro lado. Quedo archivada en
`docs/lecciones/archivo/` y sigue siendo consultable con `buscar`.
