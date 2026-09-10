# Review - Feature #85: Copilot CLI como backend de primera clase: superficie .github/copilot-instructions.md, hooks en .github/copilot.json, agentes en .github/agents, fila en doctor y backend LLM en la tabla de CLIs
Revisado: approved · 2026-09-10T01:41:36Z · estampado por `harness revision --veredicto`

Revisor: la misma sesion que implemento, mas una revision adversarial de solo
lectura (ultracode: tres lentes —spec, casos borde en archivos del usuario,
docs— y un refutador; sin cargo, sin binario, sin red). Metodo propio: tres
tests de integracion y un check de shell contra HEAD, unitarios, tres
mutantes, suite completa, clippy `-D warnings`, paridad 11/11, el smoke
entero y `verify`, antes y despues de aplicar los hallazgos.

## Cobertura por AC

| AC | archivo:linea | veredicto |
| --- | --- | --- |
| AC-1 | rust/src/copilot.rs:62 · rust/tests/cli_basics.rs:9659 | CUBIERTO. Mutante "todo es del arnes" muerto por unitario e integracion; tras la revision, mezcla en el lugar (orden de claves) y prefijo por variable de entorno. |
| AC-2 | rust/src/copilot.rs:225 · rust/tests/cli_basics.rs:9737 | CUBIERTO. Round trip byte a byte del texto ajeno; dos bloques convergen; un marcador sin pareja no se toca. |
| AC-3 | rust/src/copilot.rs:408 · rust/tests/cli_basics.rs:9776 | CUBIERTO con una precision al spec: el JSON ajeno vuelve igual en CONTENIDO y orden (re-serializado con sangria de dos, BOM conservado), el `.md` byte a byte; un `hooks` vacio o no-objeto del usuario no se toca. |
| AC-4 | setup_harness.sh:1599 · setup_harness.ps1:1464 · tests/copilot_hook_check.sh:68 | CUBIERTO en bash (cuatro casos del runtime); el runtime PowerShell replica la logica y manda lo legible a stderr, pero no se ejecuta en esta maquina. Mutante "nunca bloquea" muerto. |
| AC-5 | setup_harness.sh:1921 · setup_harness.ps1:1784 · tests/copilot_hook_check.sh:96 | CUBIERTO. Deteccion, `--copilot`, `--no-copilot`, respaldo (tambien en el reset), `harness.exe` en Git Bash; paridad 11/11; el smoke sh corre el check y el smoke ps1 exige los archivos con un `copilot.cmd` falso. Mutante "instalar sin detectar" muerto. |
| AC-6 | rust/src/doctor.rs:288 · :724 | CUBIERTO. Tras la revision mira `hooks.agentStop` y no el texto entero. |
| AC-7 | rust/src/consolidacion.rs:41 · :729 · :744 | CUBIERTO. La pista de autenticacion sale en el skip y en el error del backend. |
| AC-8 | docs/impl-85.md:1 | PENDIENTE DE MEDICION: no hay sesion de GitHub en la maquina. Se deja el procedimiento y que decidir con el resultado (OBS-4). |
| AC-9 | README.md:1145 · UPDATING.md:857 · docs/architecture.md:161 · roles/README.md:73 · AGENTS.md:125 | CUBIERTO. Las dos copias de UPDATING y de roles/README identicas; el comando del AC pasa. |

## Lo que el review tiene que decir

1. **La revision adversarial encontro lo que los tests del feliz no ven.**
   Catorce hallazgos distintos, trece corregidos con test; el mas serio, un
   archivo del usuario que no fuera UTF-8 se reemplazaba en silencio. La
   causa comun: tratar "no pude leer" como "esta vacio". La regla que queda:
   con un archivo ajeno, ante cualquier duda no se toca y se avisa.
2. **Windows sin poder ejecutarlo.** Dos hallazgos del runtime y del
   instalador PowerShell (stdout con texto delante del JSON, comillas por argv
   en 5.1, `2>&1` bajo Stop) se corrigieron por lectura y con el patron que el
   propio repo usa en su smoke; no se corrieron aca. Van declarados.
3. **Lo no medido esta dicho, no disimulado.** `agentStop` como Stop es la
   lectura mas probable de la documentacion; sin login no se probo. Ahora lo
   dicen el spec, el README, UPDATING, roles/README, el SDD y el AC-8.

## Riesgo declarado

- Si `agentStop` disparara por cada tool call, el guard bloquearia a mitad
  del trabajo: el AC-8 lo mide y, si es asi, pasa a informativo en
  `sessionEnd` (decision OBS-4 / D14).
- El runtime y el instalador PowerShell no se ejecutaron en esta maquina.
- Los agentes de `.github/agents/` quedan fuera: Copilot aplica los roles como
  fases hasta verificar el formato.

## Veredicto

Nueve AC: ocho cubiertos con test, comando o docs, y el manual pendiente de
una sesion autenticada, declarado. Tres tests de integracion y un check rojos
contra HEAD, catorce unitarios, tres mutantes muertos, trece hallazgos de la
revision adversarial corregidos y uno registrado, suite 537 + 295 verde,
clippy y paridad limpios, smoke entero verde.
