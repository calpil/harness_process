# Impl - Feature #79: close refresca el espejo del backlog y de la bitacora

Spec: docs/spec-feature-79-close-refresca-el-espejo-docs-bkp-backlog-featur.md
Plan: docs/plan-feature-79-close-refresca-el-espejo-docs-bkp-backlog-featur.md

## Lo que habia

El espejo `docs/bkp-backlog/feature_list.json` se refrescaba a mano al cerrar,
cuando alguien se acordaba. El 2026-09-06 el checkout se borro por error: el
backlog volvio de ese espejo (identico, porque se habia refrescado en el
ultimo cierre); la bitacora `progress/history.md` no tenia espejo y se perdio.

## El arreglo

Un modulo nuevo con la politica como enum y la decision pura, y una llamada
en la FASE 3 de `close` DESPUES de guardar el estado y de la linea de bitacora
de ese mismo cierre, para que la bitacora espejada lo incluya.

| AC | archivo:linea | evidencia |
| --- | --- | --- |
| AC-1 | rust/src/espejo.rs:101 · rust/src/commands/close.rs:345 · rust/tests/cli_basics.rs:8909 | `refrescar` copia BYTES con `write_bytes_atomic` (rust/src/features.rs:42) a `<raiz>/docs/bkp-backlog/`; `close` lo llama tras `save_features` y `log(...)`. Test: espejo viejo + cierre `done` -> espejo byte-identico al backlog con `"status": "done"` y stdout con la frase. |
| AC-2 | rust/tests/cli_basics.rs:8930 | El mismo camino con `--status blocked`: el espejo lleva `blocked`. |
| AC-3 | rust/src/espejo.rs:80 · rust/tests/cli_basics.rs:8944 | `decidir(Auto, false)` es `false`: sin directorio no se crea nada, stdout no menciona el espejo y stderr no lleva ningun `[!]` de refresco (ese ultimo aserto se agrego para que el test de integracion atrape al mutante "Auto siempre si"). |
| AC-4 | rust/src/espejo.rs:112 · rust/tests/cli_basics.rs:8956 | `Siempre` hace `create_dir_all` y escribe los dos archivos; el test compara bytes del backlog y de la bitacora con sus espejos y exige las dos rutas en stdout. |
| AC-5 | rust/src/espejo.rs:80 · rust/tests/cli_basics.rs:8977 | `Nunca` no toca: los dos archivos viejos quedan byte-identicos y stdout calla. |
| AC-6 | rust/src/espejo.rs:52 · :66 · :80 · :146-186 | `Politica` {Auto, Siempre, Nunca}, `from_rules` (ausente o no booleano -> Auto), `decidir` con sus seis combinaciones en cuatro unitarios. |
| AC-7 | rust/src/espejo.rs:42 · rust/src/commands/close.rs:362 · rust/tests/cli_basics.rs:8997 | `refrescar` intenta las DOS copias siempre y devuelve `Refresco {escritos, fallos}`; `close` emite un `[!] No se pudo refrescar el espejo <ruta>: <error>` por archivo fallido y nombra en stdout los que si escribio. Test: el destino del backlog es un directorio -> exit 0, feature `done`, `[!]` con `docs/bkp-backlog/feature_list.json`; la bitacora se espeja igual. Unitario rust/src/espejo.rs:189 cubre el fallo parcial con un byte no UTF-8 en la bitacora. |
| AC-8 | UPDATING.md:48 · README.md:1225 · docs/architecture.md:55 | Seccion en UPDATING (dos copias, `cmp`), parrafo en README (politica, MOMENTO y sin commitear), bullet del modulo en architecture (idem). |
| AC-9 | rust/src/espejo.rs:1 | Suite completa, clippy `-D warnings` limpio, paridad 10/10; `verify` 10 verdes (docs/verify-79.md), corrido antes y despues de los retoques de la revision. |
| AC-10 | rust/src/commands/close.rs:345 · rust/tests/cli_basics.rs:9011 | El refresco va despues de `log("close feature ...")`: la bitacora espejada contiene la linea de ese cierre y es byte-identica a la viva. |

## El rojo

Los siete tests de integracion se corrieron contra el binario de HEAD antes de
escribir el modulo: cinco cayeron por su aserto (stdout sin la frase, stderr
sin el `[!]`, espejo de bitacora ausente); dos pasaron —AC-3 y AC-5 son los
casos "no paso nada", y contra HEAD no pasa nada—. Los unitarios nacieron con
el modulo.

Mutaciones (reviewer):
- `Auto => true`: cae el unitario `decide_auto_only_when_the_directory_exists`.
  La primera version del test de AC-3 NO lo atrapaba: el mutante decide "si",
  no crea el directorio (eso es de `Siempre`), falla al escribir y lo avisa
  por stderr, y el test solo miraba stdout y el directorio. Se le agrego el
  aserto de stderr limpio y ahora tambien cae.
- `Siempre => existe_dir`: caen el unitario y el test de integracion de AC-4.
Las dos con `cmp` antes y despues, restauracion y `touch`; corridas con
`--no-fail-fast` para que los dos binarios de tests lleguen a correr.

## La revision adversarial (ultracode: tres lentes, solo lectura)

Tres agentes revisaron el worktree con lentes distintas —correctitud contra
el spec, casos borde, docs y consistencia— sin correr nada. Ningun hallazgo
bloqueante; los seis que trajeron se verificaron uno por uno:

1. **Fallo parcial** (dos lentes): `refrescar` cortaba en la primera copia
   fallida con `?` y perdia las rutas ya escritas; el `[!]` decia "el espejo
   del backlog" aunque el backlog si hubiera quedado fresco. Arreglado:
   `Refresco {escritos, fallos}`, las dos copias se intentan siempre, `close`
   avisa por archivo y nombra lo escrito (rust/src/espejo.rs:42, :101;
   close.rs:362). Unitario nuevo del caso.
2. **`read_to_string` no es byte a byte**: una bitacora con un byte no UTF-8
   habria fallado el espejo en cada cierre. Arreglado: `std::fs::read` +
   `write_bytes_atomic` (rust/src/features.rs:42; `write_text_atomic` delega).
3. **Separadores en Windows**: `raiz.join("docs/bkp-backlog")` mezcla `/` y
   `\` y los mensajes no coincidirian con lo documentado. Arreglado:
   `dir_del_espejo` arma la ruta por componente y `ruta_para_mensaje` imprime
   siempre con `/` (rust/src/espejo.rs:24, :30), con su unitario.
4. **README sin el momento del refresco** y **architecture sin "sin
   commitear"** (AC-8): agregados.
5. **El instalador (#78) etiqueta `history.md` como "(no es un backlog
   JSON)"** al nombrar los respaldos: pre-existente y fuera de este diff; queda
   como candidato a feature (contar lineas en vez de features para los
   candidatos que no son JSON).

## Estilo (skills cargados)

`rust-patterns` / `rust-best-practices`: la politica es un enum de tres
estados (un bool no puede decir "no lo toques"); `decidir` es pura y se prueba
sin filesystem; `refrescar` recibe `&Path`, devuelve `Result` con `Context` en
cada fallo, sin `unwrap`; el `Err` lo decide el que llama. `rust-testing`: un
comportamiento por test, nombres que se leen como oracion, `#[cfg(test)]` en
el modulo para lo puro e integracion en `cli_basics.rs` para el comando.
`rust-async-patterns` no aplica.

## Lo que NO hace

- No commitea el espejo: queda sin commitear, y el mensaje lo dice.
- No refresca en `add` ni en `start` (OBS-2).
- No espeja `progress/current*.md`: el sello de cierre ya archiva ese estado.
