# Plan - Feature #56: paquete_de_contexto_para_implementar

Estado: in_progress
Microservicios:
- harness

## Alcance

Un comando `contexto` simetrico al `revision` de la #51, pero del lado de
implementar: entrega el material en vez de hacer que el agente lo busque, y
—esto es lo nuevo— **dice en voz alta cuando no hay material**. El disparador
esta medido: 693.6k tokens de mapeo sobre un tema que el mapa no cubria.

Fuera: escribir el mapa, arreglar `buscar` (#39), regenerar el grafo, levantar
el hub.

## Impacto entre microservicios

`harness` solo. El comando es de solo lectura: no toca estado, no escribe
archivos, no emite intents de Atlassian. El unico punto que cambia el
comportamiento de otro comando es el resumen que `start` imprime (AC-12).

Consultado en el hub el 2026-08-22: 4 proyectos, 33 microservicios, 208
dependencias. Este repo no figura como microservicio: no hay radio de impacto.

## Consulta al grafo (graphify)

`graphify-out/` de este repo no tiene `graph.json` (el arnes se indexa a si
mismo solo en instalaciones). Se declara como hueco, que es justo el
comportamiento que esta feature construye.

## Delegacion (implementer)

1. `rust/src/contexto.rs` — el modulo, espejo de `revision.rs` (AC-1, AC-2):
   `Paquete` con mapa, cobertura, impacto, grafo, historia, lecciones y
   relacionadas; `render_texto`, `render_json`, `tamano`, `resumen`.
2. Resolucion de punteros: `resolver_puntero()` detecta un `architecture.md` que
   solo apunta a otra ruta, sigue el destino (AC-4) y, si no existe, lo declara
   como hueco con la ruta que falta (AC-5). Test `contexto_puntero`.
3. Cobertura del tema: `cubre()` + `secciones_que_mencionan()` — la linea
   explicita de que el mapa NO cubre el tema (AC-6) y, si lo cubre, solo esas
   secciones (AC-7). Test `contexto_cobertura`.
4. Grafo: edad en dias contra el umbral de **7** decidido por el usuario (OBS-2,
   AC-8); sin `graph.json`, hueco con el comando que lo genera. `--con-grafo`
   invoca `graphify query` SOLO si el binario existe (OBS-1); por default nunca
   (AC-1).
5. Presupuesto: `recortar()` reusado del modulo de revision, `--max-lineas`
   (AC-9), tamaño en lineas y tokens (AC-10) y tope K de hits de `buscar`
   ordenados por curaduria (AC-11). Test `contexto_presupuesto`.
6. Hub con tiempo limite: la consulta de impacto corre con limite y su ausencia
   es un hueco, nunca un error (AC-15). Test `contexto_sin_nada`.
7. `rust/src/commands/contexto.rs` + `cli.rs`: `--feature`, `--tema`,
   `--max-lineas`, `--con-grafo`, `--json`; sin feature ni tema, exit 2 con las
   dos formas de invocarlo (AC-3).
8. `commands/start.rs`: imprime el resumen SIEMPRE (OBS-3, AC-12).
9. `roles/leader.md` y `roles/implementer.md`: el primer paso es pedir el
   paquete, y que hacer cuando avisa que el mapa no cubre (AC-13), con espejo en
   `templates/` (AC-14).

## Criterios de cierre (reviewer)

- Los 16 AC con evidencia; los cinco con `Comando:` corridos de verdad.
- `cargo test`, `cargo clippy --all-targets -- -D warnings`,
  `bash tests/setup_smoke.sh` y `bash harness_check.sh` limpios (AC-16).
- Revision adversarial de la #51: intentar ROMPER cada AC, no confirmarlo. En
  particular AC-6 (¿que pasa con un tema de una sola letra, o con acentos?) y
  AC-5 (¿que pasa si el puntero apunta a un directorio, o a si mismo?).
- El comando no escribe NADA: verificado con `git status` limpio despues de
  correrlo.

## Riesgos

- **El hub cuelga el comando**: mitigado con tiempo limite; sin respuesta, hueco.
- **Falsos "no cubre"**: un mapa que llama al tema de otra forma (sinonimos) va a
  dar "no cubre" aunque el tema este. Es un falso positivo caro. Mitigacion: el
  aviso dice los terminos que se buscaron, para que se vea POR QUE.
- **Que el paquete engorde hasta ser lo que queria evitar**: por eso el tamaño se
  reporta siempre y el presupuesto es un flag, no una constante escondida.

## Observaciones (decisiones pendientes)

- OBS-1 [DECIDIDA]: `graphify query` solo con `--con-grafo`, y solo si el binario
  existe.
- OBS-2 [DECIDIDA]: grafo vencido a los 7 dias.
- OBS-3 [DECIDIDA]: el resumen de `start` sale siempre.
- OBS-4 [DECIDIDA]: el puntero roto de `realestate` se arreglo aparte
  (2026-08-22); AC-5 se prueba con fixtures propias.

### Avance 2026-08-22T16:47:14Z
Plan #56 completo: modulo contexto.rs, comando, resumen en start, roles y espejos. Dos defectos propios encontrados en la revision adversarial (puntero relativo contra el cwd, falso 'no cubre' con tema sin terminos), arreglados con test.

---
Cerrado: 2026-08-22T16:50:32Z - status=done - 
