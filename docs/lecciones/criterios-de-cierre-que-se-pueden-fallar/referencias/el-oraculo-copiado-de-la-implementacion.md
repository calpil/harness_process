# El oraculo copiado de la implementacion (feature #73)

Referencia de la leccion `criterios-de-cierre-que-se-pueden-fallar`: el caso concreto que sostiene una de sus reglas. Movida aca el 2026-09-06 (feature #80) para que la leccion de clase quede dentro del tope de lineas; el texto es el original, sin reescribir.

El caso mas caro no es el test que falta: es el que esta, pasa, y describe el
bug como si fuera la intencion. Dos formas, las dos encontradas juntas.

**1. El oraculo que imita al codigo.** Un test corria sobre los 310+ AC reales
del repo con este comentario: *"INVARIANTE: el parser no inventa ni pierde
comandos"*. Comparaba lo que el parser reportaba contra lo que el spec declara,
y su contador de declaraciones decia:

```rust
} else if t.starts_with("Comando:") && ac_abierto {
    n += 1;
    ac_abierto = false; // solo el primero cuenta, como en `parsear`
}
```

Ese comentario —**"como en `parsear`"**— es el defecto entero. El oraculo no
media al parser: lo copiaba. La igualdad se cumplia siempre, sobre cualquier
corpus, hicieran lo que hicieran los dos lados. Mientras tanto el AC-8 de una
feature declaraba cuatro verificaciones, se corria una, y el reporte decia
"1 verde, 0 en rojo".

**2. El test que nombra el bug como si fuera el contrato.**
`parse_should_keep_only_the_first_command_of_an_ac`. No estaba mal escrito:
describia con precision lo que el codigo hacia. Lo que faltaba era la otra
pregunta. Este es mas peligroso que el primero porque parece deliberado — el
nombre suena a decision de diseño, y nadie discute una decision documentada.

**Como se detectan:**

- Buscar en los tests las palabras que delatan la copia: `como en <funcion>`,
  `igual que <modulo>`, `replica de`. Un oraculo tiene que salir del CONTRATO
  —lo que el spec o el usuario esperan— no de la implementacion.
- Ante un test cuyo nombre afirma una limitacion (`only_the_first`, `ignores_`,
  `keeps_just_`), preguntarse **por que** esa limitacion, y si alguien la
  decidio o simplemente ocurrio.
- La prueba definitiva es la mutacion: si rompes la produccion a proposito y el
  test sigue verde, el test no estaba mirando ahi. Con el bug reintroducido, el
  oraculo arreglado lo delato por nombre y archivo:
  `spec-feature-72-...md: el parser reporto 1 comando(s) y el spec declara 4`.

Regla corta: **un oraculo que se deriva de la implementacion no verifica,
acompaña**. Y un test verde no es evidencia de nada hasta que lo viste fallar.
