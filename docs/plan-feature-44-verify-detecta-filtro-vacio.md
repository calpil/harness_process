# Plan - Feature #44: verify_detecta_filtro_vacio

Estado: in_progress
Microservicios:
- harness

## Alcance

`verify` mira la SALIDA de un comando que sale 0, no solo su exit code, y marca
`vacio` al AC cuyo comando reconocidamente no corrio ningun caso. Mas la deuda
que eso destapa: el test que le faltaba al AC-12 de la #28.

## Peldano elegido: 1 (extender lo que ya existe)

| Peldano | Se descarta porque |
| --- | --- |
| 1. extender | **elegido** |
| 2. flag | Un AC que no midio nada no es una preferencia del usuario: es un reporte equivocado. Un flag para apagarlo seria un flag para volver a mentir. Y el escape honesto ya existe: no declarar `Comando:` deja el AC en `manual`. |
| 3. comando nuevo | Nadie va a correr un `harness verify --auditar` aparte; el valor esta en que pase solo, en la corrida que ya se hace. |
| 4. superficie nueva | — |
| 5. dependencia | Parsear `test result:` son cuatro lineas de `split`. Una dependencia de parsing de libtest costaria mas que el problema. |

`Peldano elegido: 1 porque el modulo que ejecuta y clasifica los comandos ya
existe (rust/src/verificacion.rs), ya tiene el enum de estados y ya escribe el
reporte: lo unico que falta es que no descarte la salida del camino feliz y una
variante mas en un enum que el compilador ya obliga a cubrir.`

## Impacto entre microservicios

Solo `harness`. El cambio es interno a `verificacion.rs` salvo un consumidor:
`commands/close.rs` lee el reporte via `rojos_del_reporte`, que hoy compara
contra cadenas sueltas y pasa a derivar del enum.

## Delegacion (implementer)

1. **El detector puro** (AC-1..AC-4). `casos_corridos(salida) -> Option<usize>`
   en `verificacion.rs`: suma los `N passed` de cada linea `test result:`.
   `None` si no hay ninguna: no opinar es parte del contrato.
2. **El estado** (AC-5). `Estado::Vacio` con `etiqueta`/`simbolo`/`bloquea`.
   Los tres `match` son exhaustivos: el compilador marca los tres.
3. **La clasificacion** (AC-6..AC-8). En `ejecutar()`, el brazo
   `Some(s) if s.success()` deja de devolver `String::new()`: lee la salida,
   se la pasa al detector y decide `Verde` o `Vacio`.
4. **El reporte** (AC-9). `render_reporte` cuenta los `vacio` aparte del
   resumen y los incluye en la seccion de salidas.
5. **El lector** (AC-10, AC-11). `Estado::desde_etiqueta(&str) -> Option<Estado>`
   y `rojos_del_reporte` construido sobre el, no sobre `== "rojo"`.
6. **El sandbox** (AC-12). `tests/verify_vacio_check.sh`: spec con un AC que
   declara un `cargo test` inexistente, `verify` -> `vacio`, `close` -> exit 2.
7. **La deuda** (AC-13, AC-14). Escribir de verdad
   `consolidar_without_aplicar_should_not_touch_anything` con backend falso, y
   regenerar `docs/verify-28.md`.
8. **Docs** (AC-16): README, UPDATING, espejo de templates.

## Criterios de cierre (reviewer)

- Los 17 AC del spec en verde en `docs/verify-44.md`, corridos con el binario
  de esta feature.
- Ningun AC que hoy este verde en otra feature pasa a `vacio` sin que se lo
  declare y se lo pague. La auditoria dio un solo caso (#28) y el AC-13 lo paga.
- `cargo clippy -D warnings` limpio y la suite completa verde.
- `bash tests/parity_check.sh` y `bash tests/harness_check.sh` limpios.

## Riesgos

- **Falso positivo que ponga en rojo trabajo sano.** Mitigado por el contrato de
  `None`: si la salida no tiene `test result:`, no se opina. El riesgo real
  seria un AC cuyo unico test valido este `ignored`; se decidio que eso TAMBIEN
  es falta de evidencia (AC-4), y no hay ninguno asi en el repo.
- **La salida de comandos exitosos pasa a guardarse en el reporte.** Ya se
  guardaba la de los fallidos, que es la que trae entorno; el recorte de lineas
  es el mismo.
- **Que el detector mire el TEXTO DEL COMANDO en vez de la salida.** Seria mas
  facil (`comando.contains("cargo test")`) y estaria mal: un `cargo test` dentro
  de un script de shell quedaria afuera. La forma de la salida es el dato.

## Observaciones (decisiones pendientes)

- OBS-1 (DECIDIDA por Alan, 2026-08-18): esta feature va antes que la paraguas
  #38-#43, porque 13 de los 20 AC de la paraguas son `cargo test <nombre>`.

### Avance 2026-08-19T00:32:50Z
Feature #44 implementada: verify mira la salida ademas del exit code y marca vacio al AC que salio 0 sin ejecutar ningun caso. Contrafactico medido de verdad: con el test renombrado, el AC-12 de la #28 pasa de verde a vacio (175 ms) y el cierre lo nombra; con el test escrito, vuelve a verde midiendo 871 ms contra los 79 ms que tardaba en no correr nada. rojos_del_reporte dejo de comparar cadenas y deriva del enum, que es la forma estructural del defecto de la #37. 17/17 AC verdes.

---
Cerrado: 2026-08-19T00:42:37Z - status=done - verify mira la salida ademas del exit code y marca vacio al AC que salio 0 sin ejecutar ningun caso. Cerro el falso verde del AC-12 de la #28, medido de los dos lados: 175 ms en vacio con el test renombrado, 871 ms en verde con el test escrito, contra los 79 ms que tardaba en no correr nada. rojos_del_reporte dejo de comparar cadenas y deriva del enum.
