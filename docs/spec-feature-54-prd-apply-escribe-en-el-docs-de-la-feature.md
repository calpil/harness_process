# Spec - Feature #54: prd_apply_escribe_en_el_docs_de_la_feature

Estado: approved
Aprobado: 2026-08-24T12:12:28Z por USUARIO (confirmacion explicita)
Plan: docs/plan-feature-54-prd-apply-escribe-en-el-docs-de-la-feature.md
PRD: docs/prd/PRD-master.md
Constitution: docs/constitution.md
Metodo: docs/prd/COMO-ESCRIBIR-UN-PRD.md (este spec es el PRD del cambio)

Alcance: resolver todos los documentos de `prd propose/apply` contra el worktree de la feature seleccionada.

## La historia (antes -> despues)
<!-- El corazon del spec: contala en palabras, sin tecnicismos, con una
     persona con nombre y un momento concreto. Si la historia no convence,
     el resto no importa. -->
ANTES: Diego está parado en el checkout principal y prepara documentos para una feature activa. El diff queda en su worktree, pero `prd propose/apply` lee o escribe el `docs/` del CWD y el cierre no puede llevar esos cambios con la rama.
DESPUES: Diego puede operar desde el principal o desde el worktree y siempre ve y modifica los documentos que pertenecen a la feature; el merge los incluye sin copias manuales.

## Hoy -> Como va a funcionar
<!-- El flujo, dibujado dos veces: dibujar el HOY obliga a reusar lo que ya
     existe en vez de inventar arquitectura nueva. -->
```
HOY                      DESPUES
principal + --feature -> docs CWD  principal + --feature -> docs del worktree
worktree -> docs feature                         |__ propuesta y destinos coherentes
                              |__ <componente> -> <componente>
```

## Recorridos de usuario (priorizados)
<!-- P1 imprescindible, P2 importante. Cada recorrido independientemente testeable. -->
- P1: Como agente que coordina desde `main`, quiero preparar y aplicar documentos para una feature aislada, para que su rama contenga todos sus artefactos.
- P2: Como revisor, quiero que lectura, propuesta y escritura apunten al mismo árbol, para no revisar un diff dividido.

## Criterios de aceptacion (Given/When/Then)
<!-- La Delegacion del plan cita estos IDs (AC-1, AC-2, ...); el reviewer exige evidencia por AC.
     OPCIONAL: debajo de un AC podes declarar COMO se prueba, y
     `sh harness_cli verify --feature <id>` lo ejecuta y deja
     docs/verify-<id>.md. Un AC sin comando lo verifica el reviewer,
     como siempre: no declarar comando NO es un fallo. -->
- AC-1: Given una feature con `worktree` registrado, When `prd propose --feature <id>` corre desde el checkout principal, Then lee el PRD, SDD y architecture del `docs/` de ese worktree y crea allí `prd-diff-<id>.md`.
- AC-2: Given una propuesta contestada en el worktree, When `prd apply --feature <id> --yes` corre desde el principal tras confirmación del usuario, Then escribe exclusivamente los documentos de ese worktree.
- AC-3: Given documentos con contenido distinto en principal y worktree, When propone o aplica, Then usa de punta a punta la versión de la feature y deja intacto el principal.
- AC-4: Given una feature sin worktree registrado, When se ejecutan los comandos, Then conserva el comportamiento actual contra su raíz efectiva y comunica la degradación sin elegir otro árbol.
- AC-5: Given los documentos escritos en el worktree, When la feature se cierre y se integre normalmente, Then viajan en el merge sin pasos de copia especiales.
- AC-6: Given fixtures de worktree, When corre la suite, Then prueba propose, apply confirmado, aislamiento del principal y el fallback sin depender del repositorio real.

## Los datos que se tocan
<!-- El plano de los datos: que dispara el flujo, que interruptor lo apaga y
     que candado evita que pase dos veces. Entidades y campos en palabras. -->
- disparador: `prd propose/apply --feature <id>`.
- selector: entrada `worktree` de la feature; la misma resolución usada por los documentos del ciclo de vida.
- destinos: PRD, SDD, architecture y `docs/prd-diff-<id>.md` dentro de un solo `docs/` de feature.
- candado: un selector ausente degrada al comportamiento existente, no al CWD de manera silenciosa cuando hay worktree declarado.

## Pseudo-codigo (el acuerdo)
<!-- La receta en palabras: que lo dispara, que lo frena y que promete.
     SIN CODIGO FINAL: el spec fija la estructura, no la implementacion. -->
```
CUANDO se ejecuta `prd propose` o `prd apply` para una feature

  resolver las rutas con `para_feature(feature)` una vez
  leer alcance, propuesta y destinos desde esa misma raíz
  ¿hay worktree válido? -> si no, usar la raíz efectiva existente

  ENTONCES producir o aplicar documentos en un solo árbol,
           sin copiar cambios al checkout principal.
```
Promesas: lectura y escritura coherentes · principal intacto · cierre integrable.

## No funcionales
- SLOs: resolver rutas no agrega red ni escaneos globales.
- Seguridad: no permite rutas externas al worktree validado de la feature.
- Observabilidad: mensajes y reportes muestran la ruta relativa al árbol que realmente se usó.

## Fuera de alcance
- Cambiar la semántica de aprobación explícita de `prd apply` o el algoritmo de merge.

## Observaciones (decisiones pendientes)
<!-- Mismo protocolo que el plan: si hay observaciones SIN decision, el
     implementer DEBE preguntar al usuario ANTES de implementar. -->
- Sin decisiones pendientes: `--feature` manda sobre el CWD cuando identifica un worktree válido.
