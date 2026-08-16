# ADR-0001: cliente HTTP `ureq` para el ejecutor REST de Atlassian

Estado: aceptado
Fecha: 2026-08-15
Decide: el USUARIO (Alan), registrado en el spec de la feature #15 (OBS-9)
Feature: #15 `atlassian_binding_and_outbox`

## Contexto

El Articulo 6 de `docs/constitution.md` prohibe sumar dependencias de runtime a
`rust/Cargo.toml` sin un ADR que las justifique. La feature #15 necesita hablar
HTTPS con tres APIs de Atlassian:

- Jira platform v3 (crear issues, transicionar, comentar),
- Jira Agile 1.0 (boards y sprints, que el MCP oficial no expone),
- Confluence v2 (crear y actualizar paginas).

El binario es sincrono (el hub usa el crate `postgres`, tambien sincrono) y ya
trae `rustls` + `rustls-native-certs` por el TLS del hub, asi que no hace falta
sumar una pila TLS nueva.

## Opciones consideradas

1. **`ureq` con rustls** (elegida). Cliente HTTP sincrono, sin runtime async,
   que reusa la pila TLS que el binario ya tiene. API chica: `Agent` con
   timeouts explicitos, `send_json` / `read_json`, y `http_status_as_error(false)`
   para poder leer el cuerpo del error de Atlassian en vez de perderlo.
2. **`curl` como proceso externo**. Sin dependencia nueva, pero ata el arnes a
   un binario del sistema (con diferencias reales entre macOS, Linux y Windows),
   obliga a armar argumentos y parsear salida a mano, y hace mas fragil el
   manejo de errores y de secretos (el token pasaria por linea de comandos o por
   archivos temporales).
3. **`reqwest`**. Mas completo, pero arrastra un runtime async (`tokio`) que el
   binario no usa para nada mas: mucho peso y complejidad para lo que se
   necesita.

## Decision

Se suma `ureq` (features: `json`) como unica dependencia de runtime nueva.

`base64` NO se suma como dependencia: el header Basic necesita 20 lineas de
codificacion y se implementan en `src/atlassian/http.rs`, con vectores de prueba
del RFC 4648 en sus tests.

## Consecuencias

- El ejecutor REST (`atlassian apply`, `sprint`, `publish`) funciona sin agente
  en el medio y sin depender de binarios del sistema.
- Un cliente tipado permite timeouts (`timeout_global`), forzar HTTPS
  (`https_only(true)`) y mapear los 4xx/5xx a errores accionables con el mensaje
  real de Atlassian, como pide el Articulo 4.
- El binario crece: `ureq` y su arbol (`ureq-proto`, `url`, `webpki-roots`)
  comparten `rustls` con el hub, asi que el costo marginal es acotado.
- Si en el futuro hace falta HTTP en otra parte del arnes, ya hay un cliente
  disponible y no hay que volver a decidir esto.
