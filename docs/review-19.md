# Veredicto del reviewer - Feature #19: perfil_de_usuario

Spec: `docs/spec-feature-19-perfil-de-usuario.md` (`Estado: approved`, sello
`2026-08-16T23:32:38Z por USUARIO (confirmacion explicita)`, 20 AC)
Plan: `docs/plan-feature-19-perfil-de-usuario.md` (D1-D10)
Evidencia: `docs/impl-19.md`
PRD de origen: `docs/prd/aprendizaje/PRD-aprendizaje.md` (hito 3)

## Veredicto global: `approved`

Los 20 AC cubiertos con evidencia ejecutada. La unica brecha es la ya conocida y
decidida: `tests/setup_smoke.ps1` no se ejecuta en esta maquina.

## Trazabilidad de la aprobacion (Articulo 2)

- Sello de `approve-spec` con quien/cuando y las cinco decisiones OBS-1..OBS-5.
- Linea `approve-spec feature #19` en `progress/history.md`.
- `check-spec` y `check-plan` limpios.
- El spec corrigio de oficio un error del backlog: hablaba de "las cinco
  superficies (… GROK.md …)" y `GROK.md` de la raiz **no existe** (el instalador
  genera cuatro y archiva cualquier `GROK.md` viejo). Quedo como OBS-1 decidida,
  no como un cambio silencioso.

## Estado por AC

| AC | Estado | Evidencia verificada |
| --- | --- | --- |
| AC-1 | cubierto | El perfil entra en `USER_DOCS` (documentos del USUARIO), no en `HARNESS_DOCS`. Smoke: sembrado, vacio, no-pisado al reinstalar y vivo tras `--reset` |
| AC-2 | cubierto | El encabezado no cuenta para el limite; round-trip preserva la prosa del usuario |
| AC-3 | cubierto | Integracion: la segunda entrada de 900 chars sale con exit 2 y **no** se escribe; el mensaje lista las entradas actuales |
| AC-4 | cubierto | `[N% - X/1500 chars]` + entradas numeradas; con perfil vacio explica como empezar |
| AC-5 | cubierto | `usados_con` cuenta el reemplazo (nuevo en lugar de viejo) y `replace` tambien falla por limite |
| AC-6 | cubierto | Los **tres** comandos salen con exit 2 sin `--yes` y el archivo no se crea |
| AC-7 | cubierto | Duplicado exacto: no-op con mensaje, exit 0 |
| AC-8 | cubierto | `Coincidencia` enum con los tres casos; los dos de error verificados end to end |
| AC-9 | cubierto | Las tres lineas (`perfil add/replace/remove`) en `history.md` |
| AC-10 | cubierto | Cinco familias de credencial + Unicode invisible con su codepoint; y el contrapeso: tres entradas reales que **no** dan falso positivo. Rechaza **antes** de escribir |
| AC-11 | cubierto | Prueba real: 1 bloque tras 1 inyeccion, 1 tras 3. Smoke lo verifica en las cuatro superficies tras reinstalar |
| AC-12 | cubierto | Con perfil vacio, ninguna superficie tiene bloque (smoke `sh` y `ps1`) |
| AC-13 | cubierto | Toda escritura avisa que las superficies se refrescan al reinstalar; el comando no toca ninguna |
| AC-14 | cubierto | Corrida real: 160 registros de las **tres** fuentes, agrupados, marcando lo ya citado, sin escribir nada |
| AC-15 | cubierto | El contrato sale al final con el Bien/Mal, la regla de la repeticion y el ritual del `--yes` |
| AC-16 | cubierto | Sin material: `Sin material todavia` y exit 0 |
| AC-17 | cubierto | Ningun camino importa `graph`; el hub esta caido en este entorno y todo corrio |
| AC-18 | cubierto | Prueba real: perfil de 1600 chars => `[GATE] ... 1600/1500` y `Check fallo con 1 problema(s)` |
| AC-19 | cubierto | README, UPDATING (+ espejo), architecture (los tres almacenes, perfil ya no "pendiente"), ambas superficies y los dos roles |
| AC-20 | cubierto | `cargo test` 176+64, clippy limpio, `setup_smoke.sh` exit 0, `harness_check.sh` limpio |

## Constitution

| Articulo | Verificacion |
| --- | --- |
| 1 - Calidad y tests | 176 unit + 64 integracion (28 nuevos), clippy `-D warnings` limpio, smoke exit 0 |
| 2 - Spec aprobado | Sello + history + gates verdes |
| 3 - Trazabilidad AC-n | Cada D cita sus AC; `impl-19.md` por AC; este veredicto lista AC-1..AC-20 |
| 4 - Seguridad y observabilidad | **Es el corazon de esta feature**: el escaneo bloquea credenciales, claves privadas y Unicode invisible antes de escribir un archivo que se versiona *y* se inyecta en cada prompt. Exit codes 0/2 estables; toda escritura auditada en `history.md` |
| 5 - Decisiones del usuario | Las 5 OBS decididas antes de implementar; y la feature entera existe para que las decisiones del usuario dejen de perderse |
| 6 - Reglas puente | **Sin dependencias nuevas** (`git diff` sobre `Cargo.toml`/`Cargo.lock` vacio); `templates/` y raiz espejados; backend-agnostico (el arnes junta y verifica, el agente propone, el usuario decide: ningun modelo se invoca) |

## Checkpoints

- [x] Feature activa refleja el estado real.
- [x] `check-plan` y `check-spec` limpios.
- [x] Sin observaciones pendientes (5 decididas).
- [x] Plan al dia con lo implementado.
- [~] **Impacto**: `graph impacto` intentado; el hub sigue sin responder.
      Derivado por inspeccion; coherente con el AC-17.
- [x] `graphify query` consultado; decidio dos cosas del diseno: la inyeccion
      como paso posterior a `write_agent_surface`, y reusar el patron de bloque
      entre marcadores de `write_kimi_hooks`.
- [x] Tests ejecutados y verdes.
- [ ] `validate_ui.sh`: no aplica.
- [x] Evidencia y veredicto por AC.
- [x] `harness_check.sh` limpio.
- [x] **Aprendizaje declarado** (ver abajo).

## Lo que el reviewer encontro y se corrigio antes de cerrar

**El gate podia fallar en falso tras un `git pull`.** `harness_check.sh` llama a
`perfil check`, un subcomando que solo existe desde esta feature. Quien actualiza
el repo sin re-correr el instalador se queda con el script nuevo y el binario
viejo: el binario responde `unrecognized subcommand` y el check habria contado un
problema **del perfil** que no existe. Ahora esa salida se detecta y se reporta
como `[i]` nombrando el remedio (re-correr el instalador), sin sumar failure. La
logica de las tres ramas (binario viejo / perfil ok / perfil pasado de limite) se
verifico aislada: solo la tercera suma failure.

Es exactamente el tipo de robustez ante instalacion parcial que ya motivo las
features #7 y #10, aplicada esta vez de forma preventiva.

## Riesgos que quedan abiertos

1. **`setup_smoke.ps1` sin ejecutar.** Las aserciones estan escritas en paridad
   (siembra vacia, encabezado, ausencia de bloque) pero no corrieron. Es la misma
   brecha de la #17 y la #18, y Alan decidio el 2026-08-16 dejarla declarada.
2. **`sugerir` devuelve 160 registros en este repo.** No trunca a proposito (un
   corte silencioso seria peor), y el numero baja solo a medida que las entradas
   citan sus features. En un repo con anios de historia igual seria inmanejable:
   un `--desde <feature>` es candidato a feature propia, no a parche aca.
3. **Falsos positivos del escaneo.** Acotados por el test que verifica que las
   entradas reales de este repo pasan, pero una frase con "token:" seria
   rechazada. El mensaje dice cual patron disparo, asi que el remedio es
   reescribir.
4. **El perfil de este repo quedo VACIO.** La feature entrega la maquinaria; las
   entradas son decision de Alan y ningun agente puede escribirlas por el. El
   valor real de la #19 recien se ve cuando el perfil tenga sus primeras
   entradas — y ese paso, por diseno, no lo puede cerrar el reviewer.

## Nota sobre la declaracion de cierre

La feature deja `docs/lecciones/documentos-del-usuario-vs-plantillas.md`: la
distincion entre los tres tipos de archivo que el instalador maneja (plantilla
refrescable / documento del usuario / contenido ganado) y en que lista va cada
uno. Es la primera vez que una leccion existente
(`docs-generados-por-el-instalador`) se **usa** para decidir un diseno, y esta
lo extiende con el caso que ella no cubria: un documento del usuario que **no**
vive bajo `docs/prd/`.
