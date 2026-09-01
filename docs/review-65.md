# Review — feature #65: el arnes cierra lo resuelto aguas arriba
Revisado: approved · 2026-09-01T22:17:03Z · estampado por `harness revision --veredicto`

Revision adversarial. El mandato fue intentar ROMPER, no confirmar.

| AC | evidencia | veredicto |
| --- | --- | --- |
| AC-1 | rust/tests/cli_basics.rs:7150 | cubierto |
| AC-2 | rust/tests/cli_basics.rs:7181 | cubierto |
| AC-3 | rust/src/commands/close.rs:898 | cubierto |
| AC-4 | rust/tests/cli_basics.rs:7217 | cubierto |
| AC-5 | rust/tests/cli_basics.rs:7240 | cubierto |
| AC-6 | rust/tests/cli_basics.rs:7266 | cubierto |
| AC-7 | rust/src/prd.rs:1562 | cubierto |
| AC-8 | rust/src/atlassian/emit.rs:590 | cubierto, reescrito: media una copia |
| AC-9 | rust/src/commands/close.rs:927 | cubierto, reescrito: no podia fallar |
| AC-10 | rust/src/commands/close.rs:984 | cubierto |
| AC-11 | docs/impl-65.md:1 | cubierto (manual) |

## Lo que se rompio

Los dos hallazgos son sobre los TESTS, no sobre el estado nuevo, y los dos se
encontraron con la prueba del rojo — no leyendo el codigo.

**1. El AC-9 no podia fallar.** `todos_los_estados_tienen_su_rama` asertaba que
cada estado no era la cadena vacia y que `AGUAS_ARRIBA == "resuelto-aguas-arriba"`
—una constante igual a su propio literal—. El AC-9 dice "los cinco sitios que
comparan el literal del estado tienen su rama"; el test no comprobaba ninguno.
Se descubrio al mutar: borrar la rama de produccion de Atlassian lo dejaba VERDE.

**2. El AC-8 medía una copia.** `aguas_arriba_no_reabre_el_ticket` llamaba a un
`transicion_de` definido DENTRO de `mod tests`: una reimplementacion de la tabla
de produccion. Borrar la rama de produccion tampoco lo movia.

Es la misma trampa que el cross-check de `verificacion.rs` en la #67 —dos
instrumentos que se copian coinciden siempre y no miden nada— encontrada el mismo
dia en dos features distintas.

### El arreglo, y por que no es otra tautologia

Las decisiones salieron de los `match` inline y son produccion consultable:
`close::ESTADOS_DE_CIERRE` (rust/src/commands/close.rs:37),
`emit::efecto_de` (rust/src/atlassian/emit.rs:259),
`prd::cuenta_en_el_avance` (rust/src/prd.rs:892) y
`status::ESTADOS_CON_BUCKET` (rust/src/commands/status.rs:13).

El test recorre la tabla completa —cinco estados x cuatro decisiones— contra esas
cuatro funciones, mas un estado inventado que tiene que caer en el brazo por
defecto de todas. Verificado ROJO con cuatro mutaciones distintas: borrar la rama
de Atlassian, hacer que el PRD lo cuente, sacarle el bucket en `status`, y sacar
el estado del CLI. Un estado nuevo que no se agregue a la tabla NO COMPILA.

## Lo que aguanto

**El barrido de consumidores.** Hay 21 archivos en `rust/src/` que comparan el
`status` contra un literal, no los cinco que nombra el AC-9. Se miraron todos los
que quedan fuera de la tabla:

| consumidor | que hace con el estado nuevo |
| --- | --- |
| `commands/next.rs:10` | solo ofrece `pending`: no la ofrece |
| `features.rs:90` | activa = `in_progress`: no la toma |
| `commands/advance.rs:33` | exige `in_progress`: la rechaza |
| `journey.rs:260` | solo sigue `done`: la saltea, igual que `superseded` |
| `lecciones.rs:701` | gate solo si `done`: no le pide leccion |
| `documentos.rs:511` | gate solo si `done`: no le pide docs |
| `spec.rs:323` | gate solo si `done`: no le pide spec aprobado |
| `verificacion.rs:548` | gate solo si `done`: no le pide verify verde |
| `revision.rs:622` | gate solo si `done`: no le pide review |

**Ninguno cae en un brazo por defecto peligroso.** Todos gatean sobre `done`,
`pending` o `in_progress` y excluyen el estado nuevo por construccion, igual que
a `blocked` y `superseded`.

**La forma de la referencia.** Se probaron veinte formas contra
`forma_de_referencia_externa`. Rechaza correctamente: mayusculas (`P/Feature-1`),
ruta anidada (`a/feature-1/b`), proyecto vacio (`/feature-1`), doble barra,
espacio interno, digito unicode de ancho completo (`feature-１`), salto de linea
embebido, `..`, doble guion, notacion cientifica (`1e3`) y backticks en el id.
Acepta correctamente proyecto con unicode, y espacios al borde que se trimean.
Un id de 23 digitos se acepta y **no desborda**, porque nunca se parsea a numero.

## Lo que quedo abierto, con nombre

- **Es un escape mas barato que `superseded`.** `superseded` exige un
  `superseded_by` que se valida contra el backlog; `resuelto-aguas-arriba` exige
  un `resuelto_en` del que solo se comprueba la forma. Cerrar con una referencia
  inventada saltea los cinco gates de `done`. Es **deliberado y esta declarado**
  —el arnes no puede abrir el repo de aguas arriba, y `status` dice literal "sin
  verificar"— pero conviene tenerlo escrito: la defensa es que un humano lo lea,
  no el binario.
- **`feature-0` y `feature-007` se aceptan.** Formas que ningun backlog puede
  tener (los ids arrancan en 1) o que un humano no va a encontrar tal cual.
  Observacion, no defecto: la funcion comprueba forma, no existencia.
- **Un proyecto con caracteres significativos de markdown** entra tal cual al
  comentario de Atlassian, dentro de un span de codigo. No se probo el render en
  el tablero. Es autoinfligido —lo tipea quien cierra— y `atlassian::markdown` ya
  trata los delimitadores sin par como literales.
- **El AC-11 es MANUAL y el arnes no lo ve.** Esta escrito `- AC-11 (MANUAL):` y
  `ac_de` exige `- AC-<digitos>:`, asi que ni `verify` ni el gate del review lo
  cuentan. Pasa en los specs #64, #65, #66 y #67: el AC que pide explicitamente
  que lo mire una persona es el unico que el arnes no le exige a nadie. Es un
  defecto del arnes, no de esta feature.
