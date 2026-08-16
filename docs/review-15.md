# Veredicto del reviewer - Feature #15: atlassian_binding_and_outbox

Veredicto: **approved**
Fecha: 2026-08-16
Spec: `docs/spec-feature-15-atlassian-binding-and-outbox.md` (Estado: approved,
sello 2026-08-16T03:15:43Z, 25 AC)
Evidencia: `docs/impl-15.md`

## Verificacion oficial

| Comando | Resultado |
| --- | --- |
| `cargo test` | 117 unit + 34 integracion = **151 en verde** |
| `cargo clippy --all-targets -- -D warnings` | limpio (0 hallazgos) |
| `bash tests/setup_smoke.sh` | exit 0, con los bloques nuevos de Atlassian |
| `./harness_check.sh` | **limpio** (el `.graphify_stale` heredado de la #14 se resolvio en el refresh del `advance`) |

## Cobertura de los AC

25 de 25 con evidencia. 24 con verificacion **real** contra
`calpil.atlassian.net` por las dos rutas (agente con MCP y REST con token).

- AC-1..AC-5 (binding): smoke del instalador en tres fixtures (por flags, por
  config file, sin config) y tests de integracion del CLI.
- AC-6..AC-12 (outbox y ejecutor con agente): ciclo completo de la feature #1
  del fixture -> ADR-1 (Epic), ADR-2 (Story, Done), ADR-3/4/5 (subtasks AC-n),
  3 comentarios, 2 transiciones, `Intents pendientes: 0`.
- AC-15..AC-18 (ejecutor REST): feature #2 aplicada entera por `apply` ->
  ADR-6/7/8. Error accionable verificado en real (HTTP 400 con el mensaje de
  Jira, exit 1, intents preservados).
- AC-19..AC-21 (sprints): sprint #14 creado, activado, con ADR-6 adentro y
  cerrado reportando lo no terminado. Es la parte que el MCP oficial no puede
  hacer y que justifica la ruta con token.
- AC-22..AC-24 (Confluence): 4 paginas en el space SD, idempotencia por hash
  (segunda corrida sin cambios) y actualizacion a v2 al editar el documento,
  con enlaces cruzados pagina <-> issue en las dos direcciones.
- AC-25: las dos rutas, de punta a punta.

**AC-13 es el unico PARCIAL**: la paridad de `setup_harness.ps1` se verifico por
lectura y por asserts de contenido (en el smoke sh y en el ps1), pero no se
ejecuto porque no hay PowerShell en esta maquina. Es el mismo limite aceptado y
documentado en las features #1, #13 y #14.

## Constitution

- **Articulo 1**: tests nuevos junto al codigo tocado (unitarios en los 8
  modulos nuevos + 7 de integracion + 3 bloques en el smoke) y los cuatro
  comandos oficiales en verde.
- **Articulo 2**: spec `approved` ANTES de implementar, con el si explicito del
  usuario. Cuando el alcance cambio (OBS-5: "todo junto"), el spec se amplio de
  14 a 25 AC y se volvio a mostrar y a aprobar antes de seguir: no se
  implemento nada bajo una aprobacion vieja.
- **Articulo 3**: cada item D1..D10 del plan cita sus AC-n; `impl-15.md` y este
  veredicto se organizan por AC.
- **Articulo 4**: el token viaja solo por entorno o config ignorada por git;
  `status` dice presente/ausente y nunca el valor; el binding versionable solo
  lleva nombres de proyecto y space; HTTPS forzado (`https_only`), timeouts
  explicitos y exit codes estables (0/1/2). Ademas, el instalador ahora deja
  `.harness.env` en el `.gitignore` del proyecto **aunque el archivo ya
  existiera**.
- **Articulo 5**: las diez decisiones (OBS-1..OBS-10) estan registradas en el
  spec y en el plan con su fecha; ninguna se implemento con la observacion
  abierta. Las dos preguntas del usuario durante la implementacion
  (credenciales globales y siembra de `.harness.env`) se resolvieron en la
  misma feature y quedaron con tests.
- **Articulo 6**: la unica dependencia nueva (`ureq`) entra con
  `docs/adr/ADR-0001-cliente-http-ureq.md`; `base64` se implemento a mano para
  no sumar otra. `templates/` propagado (UPDATING.md, docs/ y el doc nuevo).

## Reparos / observaciones del reviewer

1. **AC-13 no ejecutado** (sin PowerShell). La logica esta cubierta por lectura
   y asserts; la primera corrida real en Windows deberia confirmar la siembra de
   `.harness.env` y la escritura de `atlassian.json`.
2. **Issues de prueba en el sitio real**: ADR-1..ADR-8, el sprint #14 y las 4
   paginas del space SD quedaron creados en `calpil.atlassian.net` con la
   autorizacion explicita del usuario. Estan listados en `impl-15.md` por si
   quiere borrarlos; el proyecto ADR estaba vacio, asi que no se mezclaron con
   trabajo real.
3. **Sincronizacion en un solo sentido** (fuera de alcance por decision): un
   issue movido a mano en el board no reescribe `feature_list.json`. Si el
   equipo empieza a trabajar desde Jira, va a hacer falta una feature propia
   para la vuelta.
4. **Conversion Markdown -> storage acotada** (OBS-10): titulos, listas, tablas,
   codigo, enlaces, negrita e inline code. Documentos con markdown mas exotico
   se van a ver mas planos en Confluence; cada pagina enlaza al archivo del repo
   como fuente de verdad.
5. **El titulo del epic sale del H1 del PRD**: en el fixture, como el PRD era la
   plantilla sin completar, el epic quedo como `PRD Master - <nombre del
   proyecto>`. En un proyecto con su PRD escrito toma el titulo real.
