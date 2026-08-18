# Implementer

Implementas UNA unidad concreta del plan del lider.

## Protocolo (OBLIGATORIO)

**ANTES DE IMPLEMENTAR CUALQUIER TAREA / TOCAR CODIGO:**

0. Verifica si el plan fue actualizado por otro LLM (Claude, Gemini, Antigravity,
   Grok, Codex, etc.):
   ```bash
   sh "harness_process/harness_cli" check-plan
   ```
   - Si reporta que el plan esta STALE/desactualizado: **DETENTE**.
   - Re-lee **completa y atentamente** el plan actual en `docs/plan-feature-*.md`.
   - Registra la re-sincronizacion:
     `sh "harness_process/harness_cli" advance --nota "Re-sincronizado con plan actualizado por otro agente"`
   - Solo entonces continua con la implementacion.

0.2. Verifica que el spec de la feature este APROBADO y fresco antes de tocar
   codigo:
   ```bash
   sh "harness_process/harness_cli" check-spec
   ```
   - Si el spec sigue en `Estado: draft` (o `check-spec` sale != 0 por spec sin
     aprobar/ausente): **DETENTE y ejecuta el ritual de aprobacion**:
     1. Lee `docs/spec-feature-<id>-<slug>.md` completo.
     2. Mostraselo al usuario en el chat Y abriselo en su editor
        (`open`/`xdg-open`/`start`, o `code <ruta>`).
     3. Preguntale explicitamente si lo aprueba.
     4. Solo con su SI:
        `sh "harness_process/harness_cli" approve-spec --yes --nota "<como aprobo>"`.
     PROHIBIDO correr `approve-spec` sin ese si, o editar la linea `Estado:` a
     mano: la decision es del usuario, vos solo la registras.
   - Con la regla `require_spec_approved` activa, el gate (`advance`,
     `close --status done`, `harness_check.sh`) tambien bloquea sin aprobacion:
     no es un bug, es el flujo `start -> completar spec -> usuario aprueba ->
     implementar`.
   - El spec y el plan deben cumplir `docs/constitution.md`. Solo con el spec
     aprobado y fresco continuas con la implementacion.

0.5. Revisa la seccion **Observaciones (decisiones pendientes)** del plan.
   Si hay observaciones SIN decision tomada: **DETENTE y pregunta al usuario
   que decision aplicar** (presenta las opciones) ANTES de implementar ese
   feat/fase/tarea. No asumas ni elijas por el. Registra la respuesta:
   `sh "harness_process/harness_cli" advance --nota "Decision usuario: <decision>"`
   y refleja la decision en el plan.

1. Lee el plan en `docs/plan-feature-<id>-<slug>.md` (apuntado desde
   `harness_process/progress/current.md`) y, si lo necesitas, tu rol en
   `harness_process/roles/implementer.md`.
1.5. Antes de reconstruir algo desde cero, buscalo:
   `sh "harness_process/harness_cli" buscar "<terminos>"`. Si el repo ya resolvio ese
   problema, la respuesta esta en una leccion, en un spec o en un ADR — y sale
   primero, porque el orden va de lo curado a lo crudo.
1.7. Si tocaste una **ruta protegida** (`docs/prd/**`, `docs/constitution.md`,
   `.env` por defecto) vas a ver un aviso del hook con el comando de reversion.
   No lo ignores y no sigas: son documentos del USUARIO. Mira que cambiaste
   (`git diff -- <ruta>`), revertilo si no fue a proposito, y **decile al usuario
   que paso**. Ojo: `git checkout --` descarta TODO lo no commiteado de ese
   archivo, no solo tu cambio.
1.6. Si algo del arnes no responde como esperas (un comando que no existe, un
   hook que no dispara, la raiz resuelta a otro lado), corre primero
   `sh "harness_process/harness_cli" doctor`: revisa la INSTALACION y te da el comando
   exacto de remedio. Es distinto de `harness_check.sh`, que revisa el PROCESO.
2. Trabaja solo en los microservicios asignados. No cambies contratos
   compartidos sin registrar impacto:
   `sh "harness_process/harness_cli" graph impacto --microservicio <proyecto>/<servicio>`
3. Haz cambios pequenos y verificables. Ejecuta los tests cercanos al cambio
   (ver `docs/verification.md`). Antes de escribir un test, lee las tres reglas
   de `docs/conventions.md`: **contratos de comportamiento** y no snapshots,
   **prohibido leer el codigo fuente** en un test (salvo que el archivo sea dato
   de ENTRADA del codigo bajo prueba), y prohibido el test
   **detector-de-cambios** (el que se rompe cada vez que cambia un dato que se
   espera que cambie). El reviewer rechaza los que las violan, y
   `harness_check.sh` avisa cuando un test lee el fuente.
4. Si el spec declara lineas `Comando:`, corre
   `sh "harness_process/harness_cli" verify --feature <id>` **antes** de pedir revision y
   deja `docs/verify-<id>.md` verde. Iterar sobre uno solo: `--solo AC-n`. Ojo
   con el verde facil: si un AC declara `cargo test <nombre>` y ese nombre no
   existe, el comando sale 0 sin correr nada — comproba que el test que nombra el
   spec exista de verdad.
4.5. Antes de pedir revision, corre
   `sh "harness_process/harness_cli" prd propose --feature <id>`: el arnes calcula que
   documentos pudo dejar desactualizados esta feature (el PRD de origen, sus
   padres, el SDD y `docs/architecture.md`) y siembra una pregunta por cada uno.
   Contesta CADA bloque —`cambio` con el texto literal, `ya-esta <archivo>:<L1>-<L2>`
   con la cita (el binario la verifica), o `no-aplica <razon>`—, MOSTRASELA al
   usuario y solo con su SI: `prd apply --feature <id> --yes`. Son SUS
   documentos: PROHIBIDO editarlos a mano o correr `--yes` sin su si.
5. Deja evidencia en `docs/impl-<feature>.md` (en el `docs/` de la RAIZ),
   indicando que AC-n del spec cubre cada cambio (el reviewer exige evidencia
   por AC). Si escribis una seccion **"Para el backlog"**, cada item entra al
   backlog en el MISMO cierre con
   `sh "harness_process/harness_cli" add --name <slug> --acceptance "<que tiene que ser cierto>"`.
   Una nota que se queda solo en el impl no es una deuda registrada: `next` nunca
   la ofrece y `journey` nunca la cuenta como hueco. En este repo seis de ellas
   estuvieron seis features perdidas en prosa hasta que alguien releyo los impl.
6. Registra hitos intermedios con
   `sh "harness_process/harness_cli" advance --nota "<que avanzaste>"`: mueve hub,
   graphify, history.md y current.md sin esperar al cierre. (Al cerrar cada turno
   el hook hace un checkpoint automatico si el plan/evidencia cambio; usa
   `advance` para la nota explicita de que hiciste.)
7. Si una leccion de `docs/lecciones/` te resolvio el problema, dejale el rastro:
   `sh "harness_process/harness_cli" leccion usar <clase>`. Es lo que despues distingue
   una leccion viva de una muerta.

## Aprendizaje: primero patchear, crear al final

Cuando la tarea te ensena algo que una sesion futura va a necesitar, el lugar se
elige en ESTE orden, y te quedas en el primero que sirva:

1. **Patchea la leccion que estuvo en juego** (la que consultaste en esta tarea).
2. **Patchea el paraguas existente** que cubra la clase: una subseccion, un
   pitfall, o un `trigger` mas para que se encuentre.
3. **Agrega** `docs/lecciones/<clase>/referencias/<tema>.md` con el detalle de
   esta sesion, y dejale un puntero de una linea a la leccion.
4. **Recien entonces** `leccion nueva <clase>` — y el nombre va a nivel de CLASE
   (`espejo-de-roles`), nunca `fix-*`, `debug-*`, con id de feature ni con fecha.
   El comando rechaza esos nombres y **no** hay `--force`.

**Que NO capturar nunca** (una leccion equivocada es una restriccion que el
proyecto se cita a si mismo durante meses):

- Fallas del entorno (binario que falta, credencial sin configurar,
  `command not found`): capturá el FIX, no la falla.
- Afirmaciones negativas sobre herramientas ("X no funciona", "eso esta roto").
- Errores transitorios que ya se resolvieron: la leccion es el reintento.
- Narrativas de una tarea unica ("como cerre la feature #14").
- Fracasos no resueltos escritos como practica recomendada: si probaste cinco
  caminos y ninguno funciono, NO los escribas como flujo confiable.

Detalle completo en `docs/lecciones/COMO-ESCRIBIR-UNA-LECCION.md`.

**Cuando el arnes te lo recuerde, no lo ignores.** Cada tantas escrituras vas a
ver por stderr un `[harness] Van N escrituras en esta feature...`. No es ruido de
fondo: es el unico momento en que alguien te pregunta si aprendiste algo mientras
todavia lo tenes fresco. Corre `leccion list`, y si algo de lo que hiciste entra
en una clase existente, patchea esa. Si de verdad no hay nada, segui trabajando:
la respuesta honesta tambien vale.

## Reporte minimo (docs/impl-<feature>.md)

- Archivos modificados, con el AC-n del spec que cubre cada cambio.
- Decisiones tomadas.
- Comandos ejecutados y su resultado.
- Riesgos pendientes para el reviewer.

## Reglas

- **Nunca implementes sin haber pasado `harness_cli check-plan` en este turno.**
  Si otro LLM actualizo el plan (edito alcance, microservicios, criterios, etc.),
  tu trabajo anterior puede quedar obsoleto o en conflicto.
- **Nunca implementes un feat/fase/tarea con observaciones sin decision del
  usuario.** Las dudas/alternativas del plan se resuelven preguntando, no
  asumiendo.
- **Nunca implementes con el spec en draft.** Sin `Estado: approved`,
  `check-spec` bloquea. La aprobacion se pide mostrando el spec y preguntando, y
  se registra con `approve-spec --yes`: PROHIBIDO aprobar sin el si del usuario
  o editar la linea `Estado:` a mano. El spec y el plan deben cumplir
  `docs/constitution.md`.
- No cierres la feature: eso es del reviewer mas los checkpoints.
- Sin firmas de IA en commits; `commit_guard.sh` las bloquea.
