# Spec - Feature #69: una linea AC ilegible no desaparece en silencio

Estado: approved
Aprobado: 2026-09-01T23:22:19Z por USUARIO (confirmacion explicita)
Plan: docs/plan-feature-69-una-linea-ac-ilegible-no-desaparece-en-silencio.md
PRD: docs/prd/PRD-master.md
Constitution: docs/constitution.md
Metodo: docs/prd/COMO-ESCRIBIR-UN-PRD.md (este spec es el PRD del cambio)

## La historia (antes -> despues)

Alan escribe un spec y se le va un dedo: `- AC-7 Given algo, When...`, sin los dos
puntos. El arnes lee el spec, no reconoce esa linea como AC, y **la descarta sin
decir una palabra**. `verify` corre seis criterios y dice "6 en total". El gate del
review pide filas para seis. El cierre pasa.

El AC-7 no existe. Nadie lo decidio: se perdio por un caracter.

La #68 arreglo la mitad peor de esto —antes el `Comando:` de esa linea se le
adjudicaba al AC de arriba, y `verify` imprimia un verde contra el criterio
equivocado— pero dejo la otra mitad abierta y la declaro: la linea se sigue
tirando en silencio. Este repo no quiere cosas silenciosas.

Despues: el arnes la nombra. `verify` la lista con su texto, y el gate del review
se niega a estampar un veredicto sobre un spec que tiene una linea que dice ser un
AC y no se puede leer.

## Hoy -> Como va a funcionar

Hoy `parsear` recuerda que vio una linea ilegible —para no colgarle el `Comando:`
al AC anterior— pero esa informacion **muere adentro de la funcion**. Nadie la
puede preguntar.

Despues hay una funcion hermana, pura como `parsear`, que devuelve esas lineas.
Dos consumidores la usan:

- `verify` imprime un `[!]` por cada una, con el texto de la linea, antes de
  correr nada. Es donde el autor esta mirando cuando el error todavia es barato.
- El gate del review se niega, por el mismo camino que ya usa cuando el spec no
  declara ningun AC (`revision.rs:705`). Un review no puede cubrir un criterio que
  el arnes no leyo.

Medido sobre los 55 specs del repo: **cero lineas ilegibles**. El arreglo no
cambia nada de lo que hay; se pone en medio del proximo typo.

## Recorridos de usuario (priorizados)

- P1: Como autor de un spec, quiero que un typo en el encabezado de un AC me lo
  diga el arnes y no me lo coma, para no descubrir en el review que un criterio
  nunca existio.
- P2: Como reviewer, quiero que el gate no me deje estampar approved sobre un spec
  con un AC ilegible: mi fila no puede cubrir lo que el arnes no leyo.

## Criterios de aceptacion (Given/When/Then)

- AC-1: Given un spec con `- AC-7 Given algo` (sin los dos puntos), When se corre
  `verify`, Then imprime un `[!]` que NOMBRA la linea con su texto, y el resto de
  los AC se verifica igual.
  Comando: `cd rust && out=$(cargo test verify_nombra_la_linea_ilegible 2>&1) && printf %s "$out" | grep -qE "[1-9][0-9]* passed" && ! printf %s "$out" | grep -q "FAILED"`

- AC-2: Given ese mismo spec, When se corre `revision --veredicto`, Then se niega
  nombrando la linea. Un review no puede cubrir un criterio que el arnes no leyo.
  Comando: `cd rust && out=$(cargo test el_gate_se_niega_con_un_ac_ilegible 2>&1) && printf %s "$out" | grep -qE "[1-9][0-9]* passed" && ! printf %s "$out" | grep -q "FAILED"`

- AC-3: Given las formas que SI son AC (`- AC-1:`, `- AC-4b:`, `- AC-11 (MANUAL):`)
  y la prosa que empieza distinto (`- ACR-1:`, `- Alcance: ...`), When se buscan
  lineas ilegibles, Then no aparece ninguna. Avisar de mas es peor que no avisar:
  un aviso que salta siempre se deja de leer.
  Comando: `cd rust && out=$(cargo test no_hay_falsos_ilegibles 2>&1) && printf %s "$out" | grep -qE "[1-9][0-9]* passed" && ! printf %s "$out" | grep -q "FAILED"`

- AC-4: Given los 55 specs reales del repo, When se buscan lineas ilegibles, Then
  hay **cero**. Medido antes de escribir esto: el arreglo no cambia nada de lo que
  existe, se pone en medio del proximo typo.
  Comando: `cd rust && out=$(cargo test el_corpus_real_no_tiene_ilegibles 2>&1) && printf %s "$out" | grep -qE "[1-9][0-9]* passed" && ! printf %s "$out" | grep -q "FAILED"`

- AC-5: Given una linea ilegible DENTRO de un bloque de codigo, When se busca,
  Then no cuenta: es documentacion. Sale gratis porque se usa el parser unico de
  la #67, y hay que fijarlo para que no se pierda.
  Comando: `cd rust && out=$(cargo test el_bloque_de_codigo_no_dispara_el_aviso 2>&1) && printf %s "$out" | grep -qE "[1-9][0-9]* passed" && ! printf %s "$out" | grep -q "FAILED"`

## Los datos que se tocan

- disparador: `verify` y el gate del review.
- lee: `docs/spec-feature-<id>.md`.
- escribe: nada nuevo; `verify` gana lineas de aviso en stdout.
- borra: nada.

## Pseudo-codigo (el acuerdo)

```
lineas_ac_ilegibles(spec):
    para cada linea FUERA de bloque de codigo:
        si empieza con "- AC-" y ac_de(linea) es None:
            juntarla
```

Es la misma condicion que `parsear` ya evalua para no migrar el comando. Se
extrae para que se pueda preguntar desde afuera, en vez de morir adentro.

## No funcionales

- Pura: no toca disco ni ejecuta nada, igual que `parsear`.
- Sin dependencias nuevas.

## Fuera de alcance

- **Adivinar que quiso escribir el autor.** El arnes nombra la linea; corregirla
  es de la persona.
- **Bloquear `verify`.** Avisa y sigue verificando el resto: cortar ahi le quitaria
  al autor el resultado de los AC que si estan bien.
- **Las lineas ilegibles de un review.** El review no declara AC; los cita.

## Observaciones (decisiones pendientes)

- OBS-1: el gate del review se niega, pero `close --status done` **no** tiene un
  gate propio para esto: llega por el del review, que ya es obligatorio con
  `require_review`. Si esa regla estuviera apagada, un spec con AC ilegible
  cerraria igual. Se propone no agregar un sexto gate y apoyarse en el del review;
  el usuario decide.
