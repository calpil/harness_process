Aplicado: 2026-08-30T20:23:26Z por USUARIO (confirmacion explicita)

# Documentos al dia - Feature #66: el_stop_hook_no_entra_en_bucle

Contesta CADA bloque con uno de los tres veredictos y despues corre
`sh harness_cli prd apply --feature 66`:

- `Veredicto: cambio` + `Antes:` y `Despues:` (texto LITERAL del documento)
- `Veredicto: ya-esta <archivo>:<L1>-<L2>` (el binario verifica la cita)
- `Veredicto: no-aplica <razon>` (la razon no puede estar vacia)

## Documento: docs/prd/PRD-master.md

Que cuenta: que se construye y por que
Presente en: docs/prd/PRD-master.md:1 (spec `master`), docs/prd/PRD-master.md:1 (spec `proyecto`), docs/prd/PRD-master.md:101 (spec `evento`) y 189 más
Ausente en: -
Candidato despues:
- Cambio de la feature en: `commit_guard.sh`, `harness_check.sh`, `setup_harness.sh`, `templates/commit_guard.sh` y 4 ruta(s) más. Revisa si este documento debe reflejarlo.

Veredicto: no-aplica el cuerpo de este PRD sigue en plantilla sin completar (`# PRD Master - <nombre del proyecto>`) y es del USUARIO. Esta feature no cambia QUE se construye: cambia como se comporta el fin de turno cuando el check encuentra algo que el agente no puede resolver. Eso vive en el SDD (este mismo diff lo cambia) y en architecture.md. Su bitacora la deja el propio cierre.

## Documento: docs/prd/SDD-master.md

Que cuenta: como se construye, a nivel proyecto
Presente en: docs/prd/SDD-master.md:1 (spec `master`), docs/prd/SDD-master.md:10 (spec `ningun`), docs/prd/SDD-master.md:10 (spec `ninguna`) y 251 más
Ausente en: -
Candidato despues:
- Cambio de la feature en: `commit_guard.sh`, `harness_check.sh`, `setup_harness.sh`, `templates/commit_guard.sh` y 4 ruta(s) más. Revisa si este documento debe reflejarlo.

Veredicto: cambio
Antes:
- **Cuando un gate se saltea algo, lo dice.** Una linea `[i]` con el repo y la
  razon. Un guard que se calla en silencio es indistinguible de uno apagado.
Despues:
- **Cuando un gate se saltea algo, lo dice.** Una linea `[i]` con el repo y la
  razon. Un guard que se calla en silencio es indistinguible de uno apagado.
- **Y cuando bloquea, ofrece una salida que el que lee PUEDA tomar** (feature
  #66). El guard nombraba el repo y nada mas ("Cambios sin commitear en: docs"),
  con dos remedios: commitear —trabajo que puede ser de otra sesion, a ciegas— o
  apagar el guard para todo el repo. Ahora nombra los archivos no exentos y
  agrega la salida que faltaba: si no es tuyo, decilo y no lo commitees.

**Un gate del fin de turno no puede quedarse sin salida** (feature #66). La
diferencia entre un gate y una trampa es si existe una accion que lo satisfaga.
Cuando lo que falla no depende del agente —un repo hermano sucio de otra sesion,
un espejo de rol cuyo remedio es re-correr el instalador, un spec en draft que
EXIGE el si del usuario— cada intento de cerrar el turno lo volvia a disparar.
`harness_check.sh` bloquea la PRIMERA vuelta —la unica chance del agente de
arreglar lo que SI es suyo— y degrada la segunda: imprime TODO (mas, no menos),
dice que no lo puede resolver solo, y deja cerrar.

La señal de "segunda vuelta" llega por dos caminos, y el segundo existe porque el
primero es prestado: `HARNESS_STOP_HOOK_ACTIVE`, que sale del JSON del evento
pero lo manda el CLI (de Claude y Kimi hay evidencia; de Codex, Gemini y Grok no
hay ninguna), y `progress/.stop_streak`, el centinela propio que se da cuenta
solo cuando el MISMO conjunto de fallos se repite. La firma es del conjunto y no
de la cantidad, asi que un problema nuevo reinicia la racha y vuelve a bloquear.

**Una defensa que depende de que el otro se acuerde de avisar no es una
defensa.** Y el bug de origen fue de la misma familia: habia DOS escritores de
hooks y uno no se entero del contrato —cinco superficies pasaban por
`bin/harness-hook` y `.claude/settings.json` en POSIX no—, asi que
`tests/parity_check.sh` gana un modo que lo impide.

## Documento: docs/architecture.md

Que cuenta: el mapa de lo que YA existe
Presente en: docs/architecture.md:100 (spec `directo`), docs/architecture.md:101 (spec `lectura`), docs/architecture.md:102 (spec `lecciones`) y 560 más
Ausente en: -
Candidato despues:
- Cambio de la feature en: `commit_guard.sh`, `harness_check.sh`, `setup_harness.sh`, `templates/commit_guard.sh` y 4 ruta(s) más. Revisa si este documento debe reflejarlo.

Veredicto: cambio
Antes:
`harness_check.sh`; `autocheck` y `nudge` son best-effort y NUNCA bloquean
(tragan errores y re-firman en segundo plano).
Despues:
`harness_check.sh`; `autocheck` y `nudge` son best-effort y NUNCA bloquean
(tragan errores y re-firman en segundo plano).

`harness_check.sh` bloquea la PRIMERA vuelta y DEGRADA la segunda (feature #66):
imprime todos los problemas, agrega que no los puede resolver solo y sale 0. La
segunda vuelta se reconoce por `HARNESS_STOP_HOOK_ACTIVE` —que `bin/harness-hook`
saca del JSON del evento— o por el centinela `progress/.stop_streak`, que corta
cuando el mismo conjunto de fallos se repite aunque el CLI no mande nada. Correr
`bash harness_check.sh` a mano nunca degrada. Los seis eventos `Stop` entran por
`bin/harness-hook`, y `tests/parity_check.sh` (modo `cableado-hooks`) impide que
vuelva a existir un cableado que se lo saltee.

