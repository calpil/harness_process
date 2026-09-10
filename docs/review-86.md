# Review - Feature #86: Copilot medido en vivo: los hooks van por el .claude/settings.json que el arnes ya genera (sin .github/copilot.json), el Stop de los dos runtimes emite decision:block, y doctor revisa la confianza de la carpeta
Revisado: approved · 2026-09-10T02:31:32Z · estampado por `harness revision --veredicto`

Revisor: la misma sesion que implemento. Metodo: el check del runtime rojo
contra HEAD por su aserto, tres mutantes, unitarios de `doctor`, suite completa,
clippy `-D warnings`, paridad 11/11, el smoke entero, y el AC manual corrido con
Copilot CLI 1.0.83 logueado sobre un fixture instalado con el arnes real.

## Cobertura por AC

| AC | archivo:linea | veredicto |
| --- | --- | --- |
| AC-1 | setup_harness.sh:1577 · tests/hook_runtime_check.sh:44 | CUBIERTO. Cuatro estados del Stop; tres mutantes muertos. |
| AC-2 | setup_harness.sh:2714 · setup_harness.ps1:1609 · tests/parity_check.sh:235 | CUBIERTO. La paridad ahora exige el modo y el evento del Stop de Claude en el ps1 en vez de un literal que satisfacia un comando de Gemini. |
| AC-3 | setup_harness.ps1:1457 | CUBIERTO POR LECTURA: el runtime PowerShell no se ejecuta en esta maquina; el smoke ps1 verifica el cableado. |
| AC-4 | rust/src/doctor.rs:186 · :894 · :910 | CUBIERTO. Falla con remedio (confiar / reinstalar), ok con las dos, no aplica sin `copilot` o en el checkout fuente. |
| AC-5 | setup_harness.sh:1 | CUBIERTO. El comando del AC pasa; `harness copilot` es desconocido. |
| AC-6 | rust/src/consolidacion.rs:41 | CUBIERTO. Sin cambios. |
| AC-7 | docs/impl-86.md:1 | CUBIERTO Y MEDIDO: Copilot recibio el `reason` del Stop del arnes, volvio con `stop_hook_active: True` y termino. |
| AC-8 | README.md:1145 · UPDATING.md:857 · roles/README.md:73 · AGENTS.md:125 | CUBIERTO. Copias identicas; el comando del AC pasa (incluido el `grep` negativo del archivo inexistente, que tambien excluye el nombre del modo de la #85: la nota de migracion lo describe sin nombrarlo). |

## Lo que el review tiene que decir

1. **La #85 se cerro sobre una hipotesis y la #86 la corrige con datos.** Se
   dijo en la #85 que no se habia medido; la medicion, en cuanto hubo sesion,
   contradijo el archivo, el protocolo y la unidad del timeout. El costo fue
   una feature entera. La regla que queda en la leccion: con la herramienta en
   la maquina, se pide la sesion antes de escribir formato.
2. **Claude Code cambia de forma, no de fondo.** El Stop pasa de exit 2 +
   stderr a JSON `decision: block` con exit 0; el modelo ve el mismo detalle,
   ahora en `reason`. Es el protocolo que Claude documenta y el unico que
   Copilot entiende. Se midio con el runtime real (AC-7); con Claude Code no se
   corrio un fixture: es la misma sesion que escribe esto y no usa el
   instalador.
3. **La paridad tenia un verde falso.** El chequeo del Stop en el ps1
   greppeaba `harness-hook.ps1" plain stop`, que existia por un comando de
   Gemini, no por el settings de Claude. Ahora exige `Get-HookCommand -Mode
   "claude-json" -Event "stop"`.

## Riesgo declarado

- Claude Code con el nuevo JSON: documentado y consistente con `codex-json`,
  pero no se corrio una sesion de Claude Code contra un fixture. Si un dia el
  bloqueo no llega, el primer lugar es `bin/harness-hook claude-json stop`.
- El runtime PowerShell no se ejecuto aca.
- Quien instalo la #85 con `copilot` en el PATH tiene archivos muertos en
  `.github/`: UPDATING dice como limpiarlos.

## Veredicto

Ocho AC: siete cubiertos con test, comando o lectura declarada, y el manual
medido con el CLI real. Un check rojo contra HEAD por su aserto, tres unitarios
nuevos, tres mutantes muertos, suite 528 + 292 verde, clippy y paridad limpios,
smoke entero verde.
