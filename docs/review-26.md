# Veredicto de revision - Feature #26: rutas_protegidas_deny

Veredicto global: **aprobado con limites declarados**.

Spec: `docs/spec-feature-26-rutas-protegidas-deny.md` (`Estado: approved`, 21 AC)
Plan: `docs/plan-feature-26-rutas-protegidas-deny.md` (D1-D8, `Peldano elegido: 1`)
Evidencia: `docs/impl-26.md`
Reporte: `docs/verify-26.md` (21 verde, 0 rojo, 0 manual)

## Estado por AC

Los veintiuno, cubiertos. La tabla completa esta en `impl-26.md`. Lo que un
reviewer tiene que mirar con atencion:

| AC | Estado | Por que no alcanza con leer el test |
| --- | --- | --- |
| AC-5 | cubierto **con limite** | Verifica el JSON del hook y el cableado, no una denegacion real de Claude Code. El limite esta declarado en el spec, en el test y en la doc |
| AC-9 | cubierto | `close` marca el hito en una ruta protegida y ademas registra su escritura. Sin esto el arnes se trabaria en cada cierre |
| AC-12 | cubierto | Distingue tres estados de configuracion, incluido "tipo equivocado -> defaults". Confundir "ausente" con "vacia" dejaria un proyecto desprotegido creyendo lo contrario |
| AC-14 | cubierto | La compatibilidad no es solo "no rompe": es que una instalacion con trabajo en curso pueda adoptarlo sin arrancar en rojo |

## El incidente, y por que cambia el veredicto de "aprobado" a "aprobado con una leccion cara"

Durante el desarrollo, el remedio que la propia herramienta imprimia
(`git checkout -- <ruta>`) **se corrio y borro los hitos y la bitacora de las
features #23, #24 y #25**, que estaban marcados pero sin commitear.

Tres cosas que hay que decir sobre eso:

1. **Se reconstruyo y se verifico** (`prd tree`: 5 hitos, PRD anidado, bitacora
   completa). No quedo perdida.
2. **La causa era de diseno, no un accidente**: el remedio prometia revertir "el
   cambio" y en realidad revierte el archivo entero a HEAD. En un repo sin
   commitear, eso es tirar todo. Un remedio destructivo que no dice que destruye
   es peor que ningun remedio, porque invita a correrlo.
3. **Esta arreglado y encodeado en un test** que exige el `git diff` antes del
   `git checkout` y la palabra `DESCARTA`. Y esta escrito arriba de todo en el
   impl, no en una nota al pie.

Merece subrayarse porque es la contracara exacta de las dos features anteriores:
la #25 encontro un `[ok]` que decia de mas, la #24 un test que se rompia solo, y
esta un **remedio que hacia de mas**. Tres formas del mismo error — el
instrumento que dice una cosa y hace otra.

## Lo que verifique ademas de los AC

- **La prueba del rojo sobre este repo**: toque `docs/constitution.md`,
  `harness_check.sh` lo reporto con el remedio y salio 2; al restaurar, limpio.
- **El arnes no se bloquea a si mismo**, con dos mecanismos verificados: el
  registro de escrituras propias y la caducidad de la exencion por mtime.
- **Cero falsos positivos** tras aceptar la linea de base: `harness_check.sh`
  limpio en este repo con trabajo en curso.
- **El remedio para rutas sin trackear no ofrece `git checkout`**, que no habria
  hecho nada.
- **El bloque de `harness_check.sh` no confunde stderr con hallazgos** (defecto
  propio, encontrado corriendolo y arreglado).
- **La proteccion se puede apagar** y con la lista vacia no reporta ni bloquea.

## Observaciones (no bloquean)

1. **La capa de prevencion existe solo para Claude Code**, y no se probo contra
   Claude Code de verdad. Es el limite mas grande de la feature. Mitigado
   estructuralmente: las capas 2 y 3 no dependen de ella.
2. **La ruta del tool call se extrae con `sed`, no con un parser JSON.** Ante un
   input que no matchea, el hook deja pasar. Es la eleccion correcta —nunca
   trabar el turno por no entender— pero significa que un formato distinto
   desactiva la prevencion en silencio.
3. **`progress/.rutas_arnes` crece sin poda.** Hoy dos lineas; conviene limpiarlo
   cuando el archivo se commitea. Anotado en el backlog.
4. **`setup_smoke.ps1` sin correr, undecima feature consecutiva.** Levantado en
   tres revisiones anteriores sin decision. Ya no es una observacion de feature:
   es una promesa del repo que nadie verifica hace once features.

## Riesgo que queda vivo

Esta es la primera feature que puede **impedirle trabajar al agente**. Las
mitigaciones estan y se probaron (tres defaults acotados, lista vacia como
interruptor, `HARNESS_CHECK_MODE=warn`, el arnes exento, linea de base para
adoptarla). Lo que no se puede automatizar es el criterio de quien amplie la
lista: proteger `docs/**` en vez de `docs/prd/**` trabaria el flujo entero. Esta
dicho en la doc como advertencia, no disfrazado de garantia.
