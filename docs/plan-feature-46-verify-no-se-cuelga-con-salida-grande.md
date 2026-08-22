# Plan - Feature #46: verify_no_se_cuelga_con_salida_grande

Estado: in_progress
Microservicios:
- harness

## Alcance

Una funcion (`ejecutar`) y su ayudante (`leer_salida`) en
`rust/src/verificacion.rs`. Nada mas del binario cambia, y ningun formato de
reporte cambia salvo la linea de recorte por tope.

Fuera: streaming en vivo, cambiar el timeout por default, paralelizar los AC.

## Impacto entre microservicios

`harness` solo. El hub no conoce este repo como microservicio (consultado el
2026-08-22: 4 proyectos, 33 servicios, ninguno es el arnes). El cambio afecta a
todo proyecto instalado en cuanto se re-corra el instalador, porque `verify` es
del binario.

## Consulta al grafo (graphify)

`contexto --feature 46` (feature #56) dio: mapa de 472 lineas que **cubre** el
tema en 20 secciones, grafo fresco (0 dias), 12 features relacionadas, 0 huecos,
~4.779 tokens. No hizo falta explorar el repo: la #56 se pago sola aca.

## Delegacion (implementer)

1. `lector()`: un hilo por pipe que lee hasta el EOF y retiene la COLA con tope
   `MAX_SALIDA_BYTES` (4 MB, OBS-1), sobre un `VecDeque` para que el tope no
   vuelva cuadratica la lectura — AC-1, AC-2, AC-7.
2. `lanzar_lectores()` / `juntar_lectores()`: los hilos arrancan ANTES de
   `wait_timeout` (esa es toda la feature) y se juntan despues, con stdout
   primero y stderr al final — AC-3, AC-4.
3. `ejecutar()`: usa los lectores; en el camino de error de `wait` tambien mata
   al hijo y junta, para no dejar hilos colgados — AC-5.
4. La linea de recorte por tope en la salida del reporte, diciendo cuanto quedo
   afuera y sobre que se midio — AC-7 (OBS-2).
5. Seis tests que ANTES se colgaban, cada uno midiendo su duracion — AC-1..AC-7.
6. Declarar `bash tests/setup_smoke.sh` como comando de AC-8: la feature se
   verifica a si misma con el caso que la disparo — AC-8.

## Criterios de cierre (reviewer)

- Los 9 AC con evidencia; los con `Comando:` corridos de verdad.
- Que el timeout siga cortando (AC-5): el arreglo no puede cambiar un cuelgue
  por otro.
- Que la leccion de la #44 siga viva (AC-4): el estado se mide sobre la salida
  completa, no sobre las 20 lineas del reporte.
- `cargo test`, `clippy`, `setup_smoke.sh` y `harness_check.sh` limpios.

## Riesgos

- **Hilos que no terminan**: si el hijo deja un descriptor abierto (un nieto que
  hereda el pipe), el `join` podria esperar. Mitigado porque al matar al hijo
  por timeout los pipes se cierran; anotado como reparo si aparece.
- **Memoria**: acotada por el tope de 4 MB por descriptor.

## Observaciones (decisiones pendientes)

- OBS-1 [DECIDIDA]: tope de 4 MB por comando.
- OBS-2 [DECIDIDA]: se retiene la COLA y el estado se mide sobre lo retenido,
  declarandolo en el reporte.

### Avance 2026-08-22T17:58:14Z
Feature #46 lista: lectores en hilos con tope de 4 MB y gracia de 2s, seis tests que antes se colgaban mas el del nieto, y el smoke declarado como AC-8 corriendo verde en 63s dentro de verify.

---
Cerrado: 2026-08-22T17:58:57Z - status=done - 
