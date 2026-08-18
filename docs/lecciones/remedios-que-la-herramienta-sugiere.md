---
nombre: remedios-que-la-herramienta-sugiere
descripcion: Un comando que le sugeris al usuario lo va a correr sin mirar. Que sea cierto.
triggers: [remedio, sugerencia, comando, doctor, diagnostico, aviso, destructivo, git checkout, rm, restaurar]
relacionadas: [probar-contra-datos-reales, criterios-de-cierre-que-se-pueden-fallar, promesas-estructurales-vs-disciplina]
origen: [26]
usos: 0
ultimo_uso:
ultima_actualizacion: 2026-08-17
estado: activa
---

## Cuando aplica

Cuando tu herramienta detecta algo y le **sugiere al usuario un comando** para
resolverlo: un doctor con su remedio por linea, un aviso con "revertilo con...",
un lint con "corregilo asi", un chequeo que dice como volver atras.

Sintoma de que esta mal: el comando se escribio pensando en el caso ideal, y
nunca se corrio en el caso real.

## El error de fondo

**Un comando sugerido se corre sin mirar.** Ese es el punto entero: para eso lo
sugeriste. Lo que en la cabeza del que lo escribio era "revertí el cambio del
agente", en la maquina del que lo corre es lo que el comando de verdad hace.

Ejemplo real, y caro: la #26 avisaba de una ruta protegida tocada con

```
docs/prd/PRD-master.md    git checkout -- docs/prd/PRD-master.md
```

Se corrio tal cual y **borro los hitos y la bitacora de tres features** que
estaban marcados pero sin commitear. `git checkout` no revierte "el cambio del
agente": revierte el archivo entero a HEAD. En un repo con trabajo sin commitear
—o sea, casi siempre— eso es tirar todo.

## Procedimiento

1. **Escribi el comando para el estado REAL, no para el ideal.** El mismo
   problema tiene remedios distintos segun el estado: un archivo trackeado se
   revierte con `git checkout`, uno sin trackear **no** (ahi el comando no hace
   nada y el usuario cree que lo arreglo). Si el remedio depende del estado,
   calculalo, no lo asumas.
2. **Mirar antes de actuar.** Cuando el remedio es destructivo, el primer comando
   que ofreces es el que MUESTRA (`git diff`, `git status`, `cat`), y el
   destructivo va segundo. Dos comandos en orden, no uno.
3. **Deci que destruye, en mayusculas si hace falta.** `(DESCARTA todo lo no
   commiteado de ese archivo)`, `(BORRA el archivo)`. No es cortesia: es la
   informacion que decide si correrlo o no.
4. **Condiciona lo irreversible.** "si no fue tuyo:", "si no la pusiste vos:".
   Un comando destructivo presentado como orden se obedece; presentado como
   condicion, se piensa.
5. **Nunca lo ejecutes solo.** El remedio lo corre una persona. Si de verdad hace
   falta automatizarlo, va detras de un flag explicito (`--aplicar`) y con aviso
   previo. Ver [[promesas-estructurales-vs-disciplina]].
6. **Corré el remedio que sugerís, sobre el repo real, antes de cerrar.** Es el
   unico modo de descubrir que no hace lo que pensabas.

## Pitfalls

- **El remedio que no remedia.** `git checkout --` sobre una ruta sin trackear
  sale 0 y no cambia nada. Peor que un error: el usuario cree que quedo resuelto.
- **El remedio que hace de mas.** El que arriba borro tres features. Lo mismo
  aplica a `rm -rf`, `git reset --hard`, `--force`, `DROP`: si el alcance real es
  mas ancho que el problema, decilo o no lo sugieras.
- **Suponer que el repo esta limpio.** Casi nunca lo esta. Un remedio que asume
  "no hay nada sin commitear" es un remedio que un dia se lleva puesto el trabajo
  del dia.
- **Probarlo solo en fixtures.** En el sandbox el archivo tiene una linea y no
  hay historia; el caso que duele necesita trabajo previo sin commitear. Ver
  [[probar-contra-datos-reales]].
- **Confundir "el comando salio 0" con "el problema se resolvio".** Es la version
  del remedio del mismo error que [[criterios-de-cierre-que-se-pueden-fallar]]
  describe para las verificaciones.

## Verificacion

```bash
# 1. Corré el remedio que tu herramienta sugiere, sobre el repo real,
#    con trabajo sin commitear encima. ¿Hizo SOLO lo que decia?
git stash list; git status --porcelain | head

# 2. Probalo en los dos estados: trackeado y sin trackear.
#    ¿El comando cambia? Si no cambia, uno de los dos esta mal.

# 3. Leelo como lo lee el usuario: si dice "revertí X", ¿revierte X,
#    o revierte el archivo entero?
```

Regla practica: si no podes escribir en una linea **que se pierde** al correr el
comando que sugeris, todavia no lo entendes lo suficiente como para sugerirlo.
