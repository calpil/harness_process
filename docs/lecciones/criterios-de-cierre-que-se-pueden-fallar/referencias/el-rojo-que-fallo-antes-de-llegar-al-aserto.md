# El rojo que fallo antes de llegar al aserto (feature #78)

Referencia de la leccion `criterios-de-cierre-que-se-pueden-fallar`: el caso concreto que sostiene una de sus reglas. Movida aca el 2026-09-06 (feature #80) para que la leccion de clase quede dentro del tope de lineas; el texto es el original, sin reescribir.

La #78 arreglaba un instalador que respalda trece scripts y ningun backlog. El
test nuevo planta un backlog, corre el instalador y afirma que `bkp/` tiene la
copia. Se corrio contra el instalador de HEAD para ver el rojo, y salio rojo. Dos
veces enganoso:

**1. Fallo en una precondicion, no en el aserto.** El mensaje decia
`falta el binario prebuilt`. Con el fix puesto, el mismo mensaje. Un test que se
cae antes de medir es rojo en el color y verde en la informacion: no distingue el
codigo roto del arreglado. La prueba del rojo exige leer **por que** fallo, no
que fallo; el rojo que sirve es el que nombra el aserto que uno escribio.

**2. El "VERDE" era una etiqueta mia, no una medicion.** El script que corria
rojo-y-verde imprimia `=== VERDE: check contra el fix ===` como titulo de
seccion y terminaba en `exit 0` porque los modos no propagaban su fallo. Debajo
del titulo habia cuatro `[!]`. Yo grepee los titulos. Es el 127-vs-124 de mas
arriba otra vez: una señal de exito que no sale de la medicion sino de quien la
anuncia. Regla: el veredicto lo imprime el aserto, nunca la seccion que lo
envuelve; y un `exit 0` con `[!]` adentro es un instrumento roto.

**3. Lo que el test si hizo bien: afirmar el efecto, no la llamada.** El fix
tenia un bug que ningun test de "se llamo a backup_datos" habria visto: el
instalador corre con `IFS=$'\n\t'`, asi que `for dato in $LISTA` con una lista
separada por espacios itera **una** palabra —la lista entera— y no encuentra
ningun archivo. La funcion corrio, respaldo cero, sin error. El test cayo porque
afirma que `bkp/feature_list.json.bak.*` existe y es byte-identico al backlog
plantado. Un criterio que mira el mundo despues del comando atrapa lo que un
criterio que mira el comando no puede.
