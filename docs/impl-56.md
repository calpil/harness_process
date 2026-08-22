# Evidencia de implementacion - Feature #56: paquete_de_contexto_para_implementar

Spec: `docs/spec-feature-56-paquete-de-contexto-para-implementar.md` (approved, 16 AC)
Plan: `docs/plan-feature-56-paquete-de-contexto-para-implementar.md`

## Que se construyo

`rust/src/contexto.rs` + `harness contexto [--feature <id> | --tema "<texto>"]`:
el gemelo del paquete de revision de la #51, del lado de implementar. Entrega el
material ya juntado y —lo que no existia— **avisa cuando no hay material**.

## La prueba que importa: el caso real

Contra el proyecto que disparo la feature, con el puntero ya arreglado:

```
$ harness contexto --tema "motor de reajuste"      # (en GolandProjects/realestate)

## Mapa
/Users/alan/WebstormProjects/realestate/docs/architecture.md (656 lineas)
(alcanzado siguiendo el puntero de /Users/alan/GolandProjects/realestate/docs/architecture.md)

## Cobertura del tema
EL MAPA NO CUBRE ESTE TEMA: '.../docs/architecture.md' no menciona ninguno de
estos terminos: motor, reajuste.
No es que no haya mapa: es que el tema no esta escrito ahi. [...]

## Grafo (graphify)
.../graphify-out/graph.json tiene 14 dias (vencido a los 7): lo que diga puede estar viejo.

## Historia (lo que ya se decidio)
[12 hits]
- (+8647 coincidencias mas; el resto con `harness buscar "motor de reajuste"`)

[paquete] 47 lineas, ~651 tokens estimados.
```

**651 tokens contra los 693.600 del mapeo**, y la primera linea util es
exactamente la que hacia falta.

Control sobre un tema que el mapa SI cubre, en el mismo proyecto:

```
$ harness contexto --tema "multi tenant RLS"
El mapa cubre el tema. Secciones que lo mencionan (39):
[recortado] se muestran 300 de 656 lineas del mapa. El resto esta en el archivo.
[paquete] 337 lineas, ~5288 tokens estimados.
```

## Evidencia por AC

| AC | Estado | Evidencia |
| --- | --- | --- |
| AC-1 el paquete, en orden, solo lectura | OK | `contexto_should_deliver_the_package_and_say_what_is_missing`: verifica las 7 secciones, `## Falta`, el tamaño y que no escriba archivos |
| AC-2 `--tema` sin feature | OK | Corrida real de arriba, y el mismo test de AC-6 |
| AC-3 sin feature ni tema | OK | `contexto_should_refuse_without_feature_or_topic`: exit **2** y el mensaje nombra `--feature` y `--tema` |
| AC-4 sigue el puntero | OK | Unit `contexto_puntero` + corrida real: llego a las 656 lineas y **dice** por que puntero |
| AC-5 puntero roto | OK | `contexto_puntero`: el hueco dice `NO existe` con la ruta. Es el bug real que tenia `realestate` |
| AC-6 el mapa no cubre | OK | `contexto_should_warn_when_the_map_does_not_cover_the_topic` + la corrida real (texto arriba) |
| AC-7 solo las secciones que mencionan | OK | Segunda mitad del mismo test (`el motor corre mensual` sin el aviso) + control `multi tenant RLS` |
| AC-8 grafo vencido a los 7 dias | OK | `grafo_vencido_a_los_siete_dias` + corrida real: `14 dias (vencido a los 7)` |
| AC-9 `--max-lineas` declara el recorte | OK | `contexto_presupuesto` (20 de 200) + real: `300 de 656 lineas` |
| AC-10 tamaño en lineas y tokens | OK | `[paquete] 337 lineas, ~5288 tokens estimados` |
| AC-11 tope K de hits | OK | Real: 12 hits + `(+8647 coincidencias mas)`. El ORDEN sigue siendo el de `buscar` (deuda #39) |
| AC-12 `start` resume siempre | OK | `start_should_always_print_the_context_summary`: exige `== Contexto ==`, `NO cubre` y `harness contexto` |
| AC-13 roles | OK | `roles/leader.md` (paso 2, `PEDI EL PAQUETE ANTES DE LEER NADA`) y `roles/implementer.md` (paso 0.5), los dos con que hacer cuando el mapa no cubre |
| AC-14 espejo templates | OK | `diff` modulo `__HREL__` limpio en los dos roles; `.claude/agents/{leader,implementer}.md` regenerados; asserts nuevos en el smoke |
| AC-15 sin hub, sin grafo, sin mapa | OK | `contexto_sin_nada`: 3 huecos, el paquete sale igual. El hub se consulta en un hilo con limite de 5s |
| AC-16 los cuatro comandos | OK | 362 unit + 177 integracion = **539**, clippy 0, smoke exit 0, check limpio. `verify --feature 56`: 5 verdes, 0 rojos, 11 manuales |

## Un hallazgo propio, antes de que lo encontrara el reviewer

Revisando el codigo recien escrito: un puntero **relativo** (`` `mapa-real.md` ``)
se estaba resolviendo contra el directorio de trabajo, no contra el del
documento — el mismo puntero habria apuntado a lugares distintos segun desde
donde se corriera el comando. Arreglado y con test propio
(`puntero_relativo_se_resuelve_contra_el_documento`).
