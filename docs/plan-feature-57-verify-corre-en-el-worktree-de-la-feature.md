# Plan - Feature #57: verify_corre_en_el_worktree_de_la_feature

Estado: in_progress
Microservicios:
- harness

## Alcance

Resolver una vez el contexto documental de la feature antes de leer su spec y
derivar de ese mismo `docs/` la raíz de ejecución de todos los comandos. El
reporte queda junto al spec y declara la raíz medida; la ausencia de worktree
mantiene el fallback actual con diagnóstico explícito.

## Impacto entre microservicios
<!-- sh harness_cli graph impacto --microservicio <proyecto>/<servicio> -->

- Impacto local (`ADR/harness`; Hub inaccesible por DNS): `commands::verify`,
  `HarnessPaths::para_feature`, `verificacion::{ejecutar,render_reporte}` y
  el reporte versionado `docs/verify-<id>.md` de cada worktree.

## Consulta al grafo (graphify)
<!-- graphify query "<pregunta de la task>" -->

- El mapa enlaza `verify` con el spec, el ejecutor shell y el reporte. El
  punto seguro para aislar los tres es el borde de `HarnessPaths` antes de
  filtrar los AC, no dentro de cada comando individual.

## Delegacion (implementer)

- U1 [AC-1, AC-2, AC-3]: resolver rutas y raíz desde el worktree registrado,
  pasarla a todos los comandos y escribir/mostrar el reporte en el mismo árbol.
- U2 [AC-4]: conservar la raíz documental efectiva sin worktree y diagnosticar
  el fallback, sin elegir CWD ajeno.
- U3 [AC-5, AC-6]: fixture desde principal con contenido discrepante y estados
  verde/rojo/timeout/vacío, más fallback local y conservación del formato.

## Criterios de cierre (reviewer)

- Cada AC ejecutable recibe una única raíz de feature; el principal no puede
  dar verde cuando el archivo solo existe en la rama.
- La ruta indicada en salida/reporte coincide con su destino, y los estados
  existentes siguen bloqueando sin cambios de semántica.

## Riesgos

- Usar `repo_root` para el comando mientras el reporte usa `plans` dividiría
  evidencia y código; la raíz se deriva del padre de `plans` una sola vez.

## Observaciones (decisiones pendientes)
<!-- Una observacion por linea. Si hay observaciones SIN decision, el
     implementer DEBE preguntar al usuario que decision aplicar ANTES de
     implementar ese feat/fase/tarea, y registrar aqui la respuesta. -->
- Sin decisiones pendientes: `--feature` y su worktree válido mandan sobre el
  CWD de quien invoca; sin worktree se mantiene la raíz documental efectiva.

### Avance 2026-08-25T11:15:00Z

Plan #57 completado: U1-U3 cubren AC-1..AC-6 con raíz única, diagnóstico y
fixtures de aislamiento de código/evidencia.

### Avance 2026-08-26T00:29:31Z
Plan completo: verify ejecuta y reporta desde un unico worktree por feature.
