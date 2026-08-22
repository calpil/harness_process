# Veredicto del reviewer - Feature #56: paquete_de_contexto_para_implementar

Veredicto: **approved**
Fecha: 2026-08-22
Spec: `docs/spec-feature-56-paquete-de-contexto-para-implementar.md` (approved, 16 AC)
Evidencia: `docs/impl-56.md`

Revision adversarial (feature #51): el objetivo fue REFUTAR cada AC. Abajo esta
lo que se rompio de verdad y lo que no se pudo probar.

## Verificacion oficial

| Comando | Resultado |
| --- | --- |
| `cargo test` | 362 unit + 177 integracion = **539 en verde** |
| `cargo clippy --all-targets -- -D warnings` | 0 hallazgos |
| `bash tests/setup_smoke.sh` | exit 0 |
| `bash harness_check.sh` | limpio |
| `harness verify --feature 56` | **5 verdes, 0 en rojo, 11 manuales** (ver el hallazgo de abajo) |

## Lo que se rompio intentando romperlo

**Dos defectos reales encontrados y arreglados durante la revision**, los dos de
la misma familia: el comando afirmando algo que no habia verificado.

1. **Puntero relativo resuelto contra el CWD.** Un `architecture.md` que apunta
   a `` `mapa-real.md` `` se resolvia contra el directorio de trabajo: el mismo
   puntero habria dado mapas distintos segun desde donde se corriera el comando.
   Ahora se resuelve contra el directorio del documento.
   Test: `puntero_relativo_se_resuelve_contra_el_documento`.
2. **Falso "el mapa no cubre" con un tema sin terminos.** `--tema "de la"` (o
   `--tema "x"`) imprimia
   `EL MAPA NO CUBRE ESTE TEMA: ... no menciona ninguno de estos terminos: .`
   — acusando al mapa por una consulta que no preguntaba nada. Es exactamente el
   falso aviso que esta feature existe para evitar. Ahora el aviso apunta a la
   consulta y dice como arreglarla.
   Test: `tema_sin_terminos_no_acusa_al_mapa`.

3. **`verify` corre los comandos en el checkout principal, no en el worktree**
   (hallazgo del propio arnes, no mio). La primera corrida dio *4 AC sin casos*:
   `cargo test contexto_puntero` salia 0 habiendo ejecutado CERO tests, porque
   corria en `/Users/alan/harness_process/rust`, donde el codigo de esta feature
   todavia no existe — el reporte, en cambio, se escribia en el worktree. Con
   `HARNESS_REPO_ROOT` apuntando al worktree, los mismos cinco comandos dieron
   verde. Es de la misma familia que la #49 y la #54 y quedo como **feature
   #57**; no se arreglo aca porque el spec de esta feature no lo cubre.
   Importa que quede escrito: un `verify` que corre en el lugar equivocado
   devuelve verdes que no significan nada, que es el peor resultado posible de
   un gate.

## Intentos que NO rompieron nada

| Intento | Resultado |
| --- | --- |
| Puntero a una ruta inexistente | Hueco con la ruta real que se busco, no "no hay mapa" |
| Documento largo que nombra un `.md` | No se confunde con un puntero (tope de 20 lineas) |
| Tema con acentos (`migración` vs `migracion`) | Encuentra igual: `sin_acentos` |
| Repo sin mapa, sin grafo, sin hub y sin historia | El paquete sale con 3 huecos, sin panico y sin error |
| Hub inalcanzable | Limite de 5s en un hilo aparte; se declara como hueco |
| `--max-lineas` chico sobre un mapa grande | Recorta y lo declara (`300 de 656 lineas`) |

## Lo que NO se pudo probar

- **El orden de la historia.** El tope de 12 hits funciona (AC-11), pero los que
  entran son los primeros que devuelve `buscar`, y en la corrida real fueron
  encabezados de seccion irrelevantes de 8.659 coincidencias. La relevancia es
  la feature **#39**, no esta: el paquete acota el volumen, no lo ordena mejor.
  Queda dicho para que nadie lea el tope como "los 12 mas relevantes".
- **Sinonimos.** Un mapa que llame al tema de otra forma va a dar "no cubre"
  aunque el tema este. Mitigado a medias: el aviso lista los terminos buscados,
  asi que el falso positivo se diagnostica de un vistazo, pero no se evita.
- **PowerShell**: esta feature no toca los instaladores, asi que no hay paridad
  que ejecutar.
- **`--con-grafo` con `graphify` real**: se probo la rama sin binario (imprime el
  aviso); no se ejecuto una consulta real al grafo, que cuesta y es de otro
  proceso.

## Constitution

- **Articulo 1**: 11 tests nuevos (7 unit en `contexto.rs` + 4 de integracion),
  dos de ellos nacidos de los defectos de arriba.
- **Articulo 2**: spec `approved` antes de implementar, con las cuatro
  observaciones decididas por el usuario ANTES de escribir codigo.
- **Articulo 3**: la Delegacion del plan cita AC-n; esta tabla cierra el circulo.
- **Articulo 4**: solo lectura, sin secretos; los huecos traen el comando que los
  resuelve, que es lo que hace accionable un error.
- **Articulo 5**: OBS-1..OBS-4 preguntadas y registradas.
- **Articulo 6**: sin dependencias nuevas; `templates/roles/` y
  `.claude/agents/` sincronizados con `roles/`.

## Reparos

1. **El paquete puede engordar.** En un mapa grande y un tema muy presente
   (`multi tenant RLS`) el paquete fue de 337 lineas / ~5.288 tokens. Sigue
   siendo dos ordenes de magnitud menos que explorar, pero el default de 300
   lineas conviene revisarlo si aparece un mapa de miles.
2. **`start` ahora hace mas trabajo**: arma el paquete completo para imprimir un
   resumen de seis lineas. En este repo es instantaneo; en uno con un `docs/`
   enorme, `buscar` es lo primero que se va a notar.
3. **El hub se consulta una vez por servicio de la feature**, no por servicio
   afectado. Si una feature toca varios, el impacto que se ve es el del suyo.
