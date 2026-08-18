# Analisis: el drift entre los documentos y el codigo

Fecha: 2026-08-17
Origen: revision pedida por Alan sobre el diagnostico "la causa raiz esta en
AGENTS.md:55-56". Este documento NO es un spec: es la evidencia verificada y las
decisiones que Alan tomo, para que la feature que salga de aca no vuelva a
discutirlas.

## 1. El diagnostico, verificado

El diagnostico decia: `close --status done` marca el hito y deja bitacora, pero
"el cuerpo del PRD no lo reescribe nadie: es del USUARIO", y no hay ningun gate
que contraste documentos contra codigo.

Es correcto. La regla no es un descuido: esta replicada en cuatro lugares.

| Donde | Que dice |
| --- | --- |
| `AGENTS.md:55-56` | "El cuerpo del PRD no lo reescribe nadie: es del USUARIO." |
| `rust/src/prd.rs:246` | "Es un documento del USUARIO desde el momento en que se crea: nadie lo vuelve a escribir." |
| `rust/src/prd.rs:586-588`, `commands/close.rs:79-81` | "NUNCA reescribe el cuerpo del documento." |
| `setup_harness.sh:425-431` (`PRD_DOCS`) | Se siembra una sola vez, no se respalda, no se regenera, no entra en `--reset`. |

Lo que `close --status done` toca del PRD son exactamente dos cosas
(`prd::echo_close`, `rust/src/prd.rs:589`):

1. la ultima celda de la fila de hito cuyo slug coincide con el nombre de la
   feature, que pasa a `done (fecha)`;
2. una linea de bitacora.

Es idempotente y best-effort: si el PRD falta o falla, `close` imprime `[i]` y
sigue. Las secciones 1-9 del PRD —la historia, los datos, el pseudo-codigo del
acuerdo, las restricciones— pueden decir cualquier cosa y el arnes nunca se
entera.

## 2. Que contrasta cada gate que existe hoy

| Gate | Que contrasta | Documento vs codigo |
| --- | --- | --- |
| `harness_check.sh:269,281` | nombre/ubicacion del PRD y `Padre:` vs arbol real | no |
| `harness_check.sh:298,327` | tabla de hitos vacia; `feature.prd` existe | no |
| `check-spec` / `check-plan` | firma: detecta ediciones de otro LLM, no divergencia con el codigo | no |
| espejo de roles | `.claude/agents/*` vs `roles/*` vs `templates/roles/*` | no |
| `verify` + `require_verify_green` (feature #23) | los `Comando:` que declaran los AC del spec | **si, y es el unico** |

La feature #23 (cerrada, con `require_verify_green: true` en este repo) es la
primera vez que el arnes contrasta un documento contra el codigo real. Pero
cubre **solo los AC del spec**: nunca el cuerpo del PRD, ni el SDD, ni
`docs/architecture.md`. Y un AC sin `Comando:` sale `manual`, que no es fallo.

## 3. Lo que el diagnostico no dice, y es peor

- **El SDD no lo toca nadie, nunca.** `docs/prd/SDD-master.md` se siembra vacio
  con el instalador (`PRD_DOCS`) y el unico codigo que lo lee es el publicador de
  Confluence (`rust/src/atlassian/confluence.rs`, `commands/atlassian.rs:942`).
  No hay comando que lo cree, lo actualice ni lo verifique: se esta publicando
  una plantilla vacia como si fuera el diseno tecnico del proyecto.
- **Ni los roles ni los checkpoints mencionan el PRD.** `grep -i "prd\|sdd\|architecture"`
  sobre `roles/leader.md`, `roles/implementer.md`, `roles/reviewer.md` y
  `CHECKPOINTS.md` da **cero** resultados. "Es del USUARIO" no significa solo que
  el arnes no lo escribe: significa que a nadie se le recuerda que hay que
  escribirlo.
- `docs/architecture.md` esta igual: plantilla del instalador, citada solo como
  fuente de `buscar` (`rust/src/buscar.rs:414`), sin gate ni dueno en el flujo.

## 4. Las decisiones de Alan (2026-08-17)

Instruccion: "siempre actualiza los prd y sdd en el flujo del arnes, agregado en
el instalador".

- **D-1. Mecanica: el agente PROPONE, el usuario APRUEBA.** Mismo ritual que
  `approve-spec`. Al cerrar, el arnes exige una propuesta de diff del cuerpo del
  PRD/SDD, se la muestra al usuario, y solo con su SI se escribe
  (`prd apply --feature <id> --yes`). El documento sigue siendo del usuario, pero
  deja de quedar mintiendo. DESCARTADAS: que el arnes lo reescriba solo (rompe la
  regla de las cuatro replicas del punto 1) y el gate de frescura a secas (no
  ayuda a escribir).
- **D-2. Alcance:** el PRD de origen de la feature, sus PRDs padres en el arbol,
  `docs/prd/SDD-master.md` y `docs/architecture.md`.
- **D-3. Distribucion:** entra por el instalador, no solo en este repo (roles,
  CHECKPOINTS y superficies bajo `templates/`, con su espejo en la raiz).
- **D-4. Orden:** primero se cierra la feature en vuelo; esto entra despues como
  feature nueva con su spec y su aprobacion.

## 5. La interaccion con la feature #26 (rutas_protegidas_deny)

La #26, en vuelo, protege `docs/prd/**` y `docs/constitution.md` de las
herramientas de escritura del agente (PreToolUse previene, PostToolUse detecta,
`harness_check.sh` es la red de seguridad; el binario del arnes queda exceptuado
porque `close` escribe el hito).

Las dos features son las dos mitades del mismo problema y **componen solo si la
propuesta se escribe fuera de la ruta protegida**:

```
agente  -> escribe docs/prd-diff-<id>.md      (fuera de docs/prd/**: permitido)
usuario -> lee el diff, dice que si
binario -> prd apply --feature <id> --yes     (escribe en docs/prd/**: el binario
                                               no es la herramienta del agente)
```

Es decir: el spec de la feature nueva tiene que declarar explicitamente que
`prd apply` es, como `close`, una escritura del binario sobre una ruta protegida,
y que la propuesta del agente jamas toca `docs/prd/**` directamente.
