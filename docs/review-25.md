# Veredicto de revision - Feature #25: harness_doctor

Veredicto global: **aprobado con limites declarados**.

Spec: `docs/spec-feature-25-harness-doctor.md` (`Estado: approved`, 20 AC)
Plan: `docs/plan-feature-25-harness-doctor.md` (D1-D8, con `Peldano elegido:`)
Evidencia: `docs/impl-25.md`
Reporte: `docs/verify-25.md` (20 verde, 0 rojo, 0 manual)

## Estado por AC

Los veinte, cubiertos. La tabla completa esta en `impl-25.md`; aca solo lo que un
reviewer tiene que mirar con atencion:

| AC | Estado | Por que no alcanza con leer el test |
| --- | --- | --- |
| AC-2 | cubierto | El test siembra una falla real antes de exigir remedios. Sin eso, en un sandbox sano habria pasado sin revisar ni una falla — verde vacio |
| AC-13 | cubierto | Corrida real en este repo, **y cada linea verificada a mano** contra el estado del filesystem |
| AC-14 | cubierto | Asserta sobre el conjunto de areas, no sobre palabras. Dos versiones anteriores fallaron por grepear prosa |
| AC-15 | cubierto | Huella ruta+mtime+tamano de todo el arbol, antes y despues |
| AC-16 | cubierto | Cuatro modos de shell, incluido uno que verifica que la salida **no se buferee** |

## La escalera hizo trabajo real

Es lo primero que hay que decir, porque era la duda: la escalera de la #24 podia
volverse un tramite ("elijo el peldano que ya queria y escribo una razon").

No paso. La escalera **partio la feature en dos**. El diagnostico quedo en
peldano 3 con la razon escrita de por que el 1 y el 2 no alcanzaban — y el
argumento del peldano 2 es de verdad fuerte (`harness_check.sh --install`
funcionaria con el binario roto), lo que obligo a admitir el limite en vez de
esconderlo. Y la mitad que el peldano 3 no puede cubrir —diagnosticar un binario
ausente— se resolvio en peldano 1, extendiendo el lanzador.

Un diseno distinto del que habria salido sin la escalera. Eso es una convencion
funcionando.

## El hallazgo que hay que subrayar

`doctor` estuvo a punto de cerrar con un **OK falso**: reportaba el hub como
"alcanzable" porque el TCP conecta, mientras toda la sesion las operaciones
morian con `connection reset by peer`. Habria sido peor que no chequear nada: el
usuario lee "ok", descarta el hub y busca el problema en otro lado.

Se detecto porque el criterio de cierre exigia revisar la salida **linea por
linea contra la realidad**, no solo el exit code. La linea ahora dice
exactamente lo que se midio y nombra el sintoma que indica que el problema esta
mas adentro.

Es el mismo error de la #23 (`cargo test` saliendo 0 sin correr nada) con otra
cara: **el instrumento en verde no prueba que el instrumento mida**.

## Lo que verifique ademas de los AC

- **La prueba del rojo a mano** sobre el caso que ya rompio dos veces: binario
  con mtime viejo -> `[!!]`, nombra los scripts que lo superan, da el remedio,
  exit 2. Restaurado -> exit 0.
- **Cero falsos positivos en este repo**: `doctor` sale 0 en el checkout fuente,
  con `hooks` y `superficies` en `no_aplica`. Verificado contra `ls`: no hay
  `CLAUDE.md`, `.claude/settings.json` ni `bin/`, y eso es lo correcto aca.
- **El lanzador no rompio nada**: perdio el `exec`, asi que revise que un gate
  normal (exit 2 de `close`) pase intacto y sin aviso espurio, y que la salida no
  se buferee. Los dos son modos del test.
- **No solapa**: las siete areas de doctor no incluyen ninguna del proceso, y
  cada salida remite a la otra herramienta.

## Observaciones (no bloquean)

1. **El area de hooks no lee el contenido de los hooks**, solo verifica que
   `bin/harness-hook` exista y sea ejecutable. Un `settings.json` que apunte a
   otra ruta se le escapa. Anotado en el backlog.
2. **"Binario viejo" es una heuristica de mtime.** Un `touch` la engana. Aceptable
   porque el remedio es idempotente, pero conviene saberlo antes de confiar
   ciegamente en el `[ok]`.
3. **El lanzador ejecuta un proceso extra** (`harness help <sub>`) cuando un
   comando sale 2. Solo en ese caso, y descartando la salida. Costo bajo, pero es
   trabajo nuevo en un camino que antes era `exec` puro.
4. **`setup_smoke.ps1` sin correr, decima feature consecutiva.** Levantado en
   `review-23.md` y `review-24.md` sin decision. A esta altura la paridad ps1 es
   una promesa que el repo no verifica hace diez features: o entra al backlog con
   nombre propio, o conviene dejar de prometerla.

## Riesgo que queda vivo

El riesgo central de un reporte de problemas es el **falso positivo**, porque
hunde la herramienta entera. Contra eso hay tres defensas verificadas (el
`no_aplica` del checkout fuente, exigir solo las superficies de los backends
instalados, y la revision manual linea por linea) y una que no se puede
automatizar: que quien agregue un area nueva la corra contra este repo antes de
darla por buena. Esta dicho en el impl como disciplina, no disfrazado de
garantia.
