Aplicado: 2026-08-29T00:42:24Z por USUARIO (confirmacion explicita)

# Documentos al dia - Feature #64: el_arnes_no_promete_enforcement_que_no_hace

Contesta CADA bloque con uno de los tres veredictos y despues corre
`sh harness_cli prd apply --feature 64`:

- `Veredicto: cambio` + `Antes:` y `Despues:` (texto LITERAL del documento)
- `Veredicto: ya-esta <archivo>:<L1>-<L2>` (el binario verifica la cita)
- `Veredicto: no-aplica <razon>` (la razon no puede estar vacia)

## Documento: docs/prd/PRD-master.md

Que cuenta: que se construye y por que
Presente en: docs/prd/PRD-master.md:1 (spec `master`), docs/prd/PRD-master.md:1 (spec `proyecto`), docs/prd/PRD-master.md:108 (spec `interruptor`) y 246 más
Ausente en: -
Candidato despues:
- Cambio de la feature en: `.claude/agents/leader.md`, `.claude/agents/reviewer.md`, `AGENTS.md`, `CHECKPOINTS.md` y 18 ruta(s) más. Revisa si este documento debe reflejarlo.

Veredicto: no-aplica el cuerpo de este PRD sigue en plantilla sin completar (`# PRD Master - <nombre del proyecto>`, `Duenno: <quien responde por este documento>`) y es del USUARIO: el instalador lo siembra una vez y nunca lo pisa. Esta feature no cambia QUE se construye ni por que; cambia como se cierra una feature (un gate mas) y saca tres reglas que no hacian nada. Eso vive en el SDD (este mismo diff lo cambia) y en architecture.md. Su bitacora la deja el propio cierre.

## Documento: docs/prd/SDD-master.md

Que cuenta: como se construye, a nivel proyecto
Presente en: docs/prd/SDD-master.md:1 (spec `master`), docs/prd/SDD-master.md:10 (spec `ningun`), docs/prd/SDD-master.md:10 (spec `ninguna`) y 265 más
Ausente en: -
Candidato despues:
- Cambio de la feature en: `.claude/agents/leader.md`, `.claude/agents/reviewer.md`, `AGENTS.md`, `CHECKPOINTS.md` y 18 ruta(s) más. Revisa si este documento debe reflejarlo.

Veredicto: cambio
Antes:
propiedades lo sostienen, ninguna es disciplina: el sello lo escribe SOLO el
binario, y estampar exige una fila por cada AC-n **del spec** citando
`archivo:linea`, que es lo que un review de cinco segundos no puede fabricar.
Despues:
Y conviene ser exacto sobre cuanto aguanta cada barrera, porque la primera
version de este texto prometia de mas y el reviewer lo desmintio con un `printf`
de cuatro lineas:

- **El sello** lo escribe solo el binario, pero es texto: un agente decidido lo
  tipea. **Filtra el descuido, no la mala fe.**
- **La cobertura por AC** es la que aguanta: una fila por cada AC-n del spec,
  cada una citando `archivo:linea` **que resuelve** (el archivo existe y tiene
  esa linea), verificada al estampar Y de nuevo en el cierre. Eso sube el costo
  de fabricar un review falso de cinco segundos a leer el codigo. No lo vuelve
  imposible: lo que el arnes NO comprueba es que la cita sea PERTINENTE al AC.

El corolario general, que vale mas que el mecanismo: **una barrera se documenta
por lo que filtra, no por lo que uno quisiera que filtrara.** Un gate descrito de
mas es un gate en el que se confia de mas.


## Documento: docs/architecture.md

Que cuenta: el mapa de lo que YA existe
Presente en: docs/architecture.md:101 (spec `lectura`), docs/architecture.md:102 (spec `leccion`), docs/architecture.md:102 (spec `lecciones`) y 806 más
Ausente en: -
Candidato despues:
- Cambio de la feature en: `.claude/agents/leader.md`, `.claude/agents/reviewer.md`, `AGENTS.md`, `CHECKPOINTS.md` y 18 ruta(s) más. Revisa si este documento debe reflejarlo.

Veredicto: ya-esta docs/architecture.md:182-186
