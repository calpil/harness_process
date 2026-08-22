# Veredicto del reviewer - Feature #52: mcp_atlassian_en_los_cuatro_backends

Veredicto: **approved**
Fecha: 2026-08-22
Spec: `docs/spec-feature-52-mcp-atlassian-en-los-cuatro-backends.md` (approved, 13 AC)
Evidencia: `docs/impl-52.md`

Revision adversarial (feature #51): se intento refutar cada AC antes de darlo
por bueno, y abajo esta lo que NO se pudo probar.

## Verificacion oficial

| Comando | Resultado |
| --- | --- |
| `cargo test` | 355 unit + 173 integracion = **528 en verde** |
| `cargo clippy --all-targets -- -D warnings` | limpio |
| `bash tests/setup_smoke.sh` | exit 0, con el bloque nuevo |
| `./harness_check.sh` | limpio |

## Intentos de refutacion

| AC | Como se intento romper | Resultado |
| --- | --- | --- |
| AC-1 | Instalar sin binding y buscar los archivos | No se rompio: no se escribe nada |
| AC-8 | Poner un `atlassian` propio apuntando a otra URL y reinstalar | No se rompio: `mio.example` sigue intacto |
| AC-9 | Borrar `atlassian` del archivo, dejar otro servidor y reinstalar | No se rompio: quedan los dos |
| AC-13 | **Dudar del resultado**: `grok inspect` mostraba el servidor, pero podia venir de la config global del usuario | Se renombro el del proyecto: aparece el nombre inventado, asi que sale del archivo del instalador |
| AC-7 | Verificar que la config global de Codex no se toque | No se rompio: el instalador no la abre; imprime los comandos |

## Lo que NO se pudo probar

- **AC-11 (PowerShell)**: sin `pwsh` en esta maquina. Verificado por lectura y
  asserts de contenido, como en las features #1, #13, #14, #15, #16, #47 y #51.
- **AC-4 y AC-5 en uso real**: se verifico el contenido de `.mcp.json` y de
  `.kimi-code/mcp.json`, pero no se levanto Claude ni Kimi *desde ese sandbox*
  para confirmar que los toman. Con Grok si se hizo (AC-13). El formato de los
  dos salio de la documentacion oficial y del archivo que ya funciona en la
  maquina del usuario.
- **Que `mcp-remote` exista en la maquina destino**: si falta `npx`, el
  instalador avisa, pero no se probo el camino de un sistema sin Node.
- **Estabilidad de los formatos**: los tres se verificaron el 2026-08-22 contra
  los CLIs instalados. Si un CLI cambia su formato, este bloque queda viejo y
  nada lo detecta automaticamente.

## Hallazgo colateral (fuera del alcance de esta feature)

Verificando AC-12 aparecio un cuelgue real del arnes: `harness_check.sh:120`
invoca `commit_guard.sh` sin cerrar stdin, y el guard arranca con
`INPUT=$(cat)` (linea 3) porque en su uso normal recibe el JSON del hook por la
entrada. Si quien llama al check deja stdin abierto — una corrida en segundo
plano, CI, o un CLI que no cierra el pipe — `cat` se queda esperando para
siempre y el check nunca termina (medido: 18 minutos colgado hasta matarlo).
Con `</dev/null` corre limpio. No es de esta feature; queda anotado en el
backlog.

## Constitution

- **Articulo 1**: asserts nuevos en el smoke para los seis caminos; cuatro
  comandos en verde.
- **Articulo 2**: spec `approved` antes de implementar; OBS-4 (versionar los
  archivos) se pregunto y se registro ANTES de escribir D2/D3.
- **Articulo 3**: D1..D9 citan sus AC-n.
- **Articulo 4**: no se escriben credenciales (solo la URL publica) y NO se toca
  ninguna configuracion global del usuario: la unica que haria falta (Codex) se
  deja como comando impreso.
- **Articulo 5**: cuatro observaciones registradas, dos de ellas hallazgos
  verificados contra los CLIs reales.
- **Articulo 6**: sin dependencias nuevas; `templates/` propagado (guia y
  UPDATING).

## Reparos

1. **El instalador escribe archivos en la raiz del proyecto destino**
   (`.mcp.json`, `.grok/config.toml`). Es la decision del usuario (versionables),
   pero conviene saber que van a aparecer en el `git status` del repo.
2. **Codex queda a medias por diseño**: el arnes no puede dejarlo listo sin
   tocar config global. Si mañana Codex admite alcance de proyecto, este es el
   lugar para cerrarlo.
3. **Nada detecta si un CLI cambia su formato de MCP**: los asserts verifican lo
   que el arnes escribe, no que el CLI lo siga entendiendo.
