# Rutas protegidas

Los PRD y la constitution son documentos **del usuario**. Hasta la feature #26
eso era buena fe: el README lo decia y no habia un solo gate — cualquier agente
con permisos de escritura podia reescribir `docs/prd/PRD-master.md` en un `Edit`,
y con un backend en modo permisivo ni siquiera habia un prompt en el medio.

Ahora hay una lista y tres capas.

## La lista

Vive en `rules` de `feature_list.json`, al lado de las otras reglas del arnes:

```json
{
  "rules": {
    "rutas_protegidas": ["docs/prd/**", "docs/constitution.md", ".env"]
  }
}
```

Tres estados **distintos**:

| Configuracion | Que pasa |
| --- | --- |
| clave ausente | valen los tres defaults de arriba |
| lista propia | vale exactamente esa (los defaults **no** se suman) |
| lista vacia `[]` | proteccion **apagada**, porque el usuario lo pidio |

Los patrones son globs por segmento: `*` cubre un segmento, `**` cualquier
profundidad. `docs/prd/**` cubre `docs/prd/PRD-master.md` y
`docs/prd/aprendizaje/PRD-aprendizaje.md`.

```bash
sh harness_cli rutas                          # que esta protegido
sh harness_cli rutas --check <ruta>           # ¿esta protegida? (exit 2 si si)
sh harness_cli rutas --violaciones            # tocadas y sin commitear
```

## Las tres capas, y lo que cada una NO puede

| Capa | Cuando | Que puede | Que **no** puede |
| --- | --- | --- | --- |
| `PreToolUse` | antes de la escritura | **impedirla**, incluso en modo permisivo | existir en backends que no tienen el evento (hoy: solo Claude Code) |
| `PostToolUse` | despues de la escritura | avisar en el acto con el comando de reversion | **prevenir**: corre despues, el archivo ya se escribio |
| `harness_check.sh` | al cerrar el turno | bloquear con exit 2 | actuar en el momento del dano |

Esto importa y por eso va en una tabla: la capa de deteccion **no puede prevenir**.
Prometer bloqueo donde solo hay deteccion es peor que no prometer nada, porque
nadie revisa lo que cree cubierto.

## El arnes no se bloquea a si mismo

`close` escribe en `docs/prd/PRD-master.md` cada vez que marca un hito, y
`prd add` crea PRDs bajo `docs/prd/`. Las dos son rutas protegidas.

La proteccion es contra **las herramientas del agente**, no contra el binario:
cuando el arnes escribe una ruta protegida lo anota en `progress/.rutas_arnes`
junto con el mtime del archivo. La exencion vale **solo mientras nadie vuelva a
tocarlo**: si el agente lo edita despues, el mtime cambia y vuelve a ser
violacion.

## Adoptarla con trabajo ya en curso

Una instalacion que activa la proteccion en medio de una tarea tiene cambios
legitimos sin commitear que no hizo ningun agente. Para que el gate no arranque
en rojo por algo que nadie hizo mal:

```bash
sh harness_cli rutas --aceptar-estado-actual
```

Toma el estado actual como linea de base. Lo corre una persona, nunca un hook, y
a partir de ahi cualquier cambio **nuevo** sobre esas rutas se reporta.

## El remedio dice lo que destruye

Cuando una ruta protegida aparece tocada, el aviso trae dos comandos en orden:

```
docs/constitution.md
    mira que cambio: git diff -- docs/constitution.md | y si no fue tuyo:
    git checkout -- docs/constitution.md (DESCARTA todo lo no commiteado de ese archivo)
```

**Primero mirar, despues decidir**, y no es cortesia. Durante el desarrollo de
esta feature el aviso decia solo `git checkout -- <ruta>`; se corrio tal cual y
borro los hitos de tres features que estaban sin commitear. `git checkout` no
revierte "el cambio del agente": revierte el archivo entero a HEAD, con todo el
trabajo legitimo que hubiera encima.

Para una ruta **sin trackear** el comando es distinto (`rm -r`), porque
`git checkout` sobre algo que git no conoce no hace nada — un remedio que no
remedia es peor que ninguno.

## Apagarla

```json
{ "rules": { "rutas_protegidas": [] } }
```

O, para una sola corrida: `HARNESS_CHECK_MODE=warn bash harness_check.sh`.
