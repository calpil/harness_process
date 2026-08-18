# Spec - Feature #30: paridad_ps1_verificable

Estado: approved
Aprobado: 2026-08-18T02:07:52Z por USUARIO (confirmacion explicita) - Alan aprobo en el chat tras el ritual. OBS-1: el chequeo avisa sin bloquear en harness_check. OBS-2: verification.md condiciona el smoke ps1 a tener Windows y nombra el chequeo de paridad como el sustituto que si corre siempre.
Plan: docs/plan-feature-30-paridad-ps1-verificable.md
PRD: docs/prd/PRD-master.md
Constitution: docs/constitution.md
Metodo: docs/prd/COMO-ESCRIBIR-UN-PRD.md (este spec es el PRD del cambio)

## La historia (antes -> despues)

ANTES: el repo **promete** paridad entre `setup_harness.sh` y `setup_harness.ps1`
—el README documenta los equivalentes `-Root`, `-NoGraphify`, `-DryRun`...— y
`docs/verification.md` manda correr `tests/setup_smoke.ps1` en Windows.

Esa promesa **no la verifica nadie desde hace once features**. Cada cierre
declaro el mismo limite: "esta maquina no tiene pwsh". Once veces seguidas. Y el
limite se acepto once veces porque la alternativa era instalar PowerShell.

El problema no es que el `.ps1` no se ejecute: es que **nadie se entera cuando
uno de los dos se adelanta al otro**. Hoy mismo, sin correr nada, hay cuatro
opciones que existen solo en el `.sh` (`--with-subagents`, `--install-graphify`,
`--install-antigravity`, `--with-postgres`) y una que existe solo en el `.ps1`
(`-CargoTargetDir`). Puede que todas esten bien; el punto es que **nadie lo
decidio**: se fueron desincronizando en silencio.

DESPUES: un chequeo estructural compara los dos instaladores **sin ejecutar
ninguno y sin PowerShell**: mismas opciones declaradas, mismas superficies
escritas. Las asimetrias legitimas se **declaran** en una lista con su razon; las
que no estan declaradas **fallan**. Asi, la proxima vez que alguien agregue una
opcion a un solo lado, se entera en el acto en vez de once features despues.

Y lo que el chequeo **no** cubre queda escrito: no ejecuta el instalador de
Windows. Un `.ps1` sintacticamente valido y estructuralmente paritario puede
igual fallar al correr. Prometer mas que eso seria repetir el error del hub de la
#25 (decir "alcanzable" cuando solo se midio TCP).

## Hoy -> Como va a funcionar

```
HOY                                     DESPUES
agrego --nueva-opcion al .sh            agrego --nueva-opcion al .sh
  -> el .ps1 queda atras                  -> el chequeo falla nombrando la opcion
  -> nadie se entera                      -> o la declaro como asimetria, con razon
  -> once features despues, sigue igual
```

## Recorridos de usuario (priorizados)

- P1: Como mantenedor, quiero enterarme **en el acto** de que agregue algo a un
  solo instalador, no cuando alguien lo instale en Windows.
- P1: Como Alan, quiero saber **cuales** son hoy las diferencias reales entre los
  dos, con su razon, en vez de suponer que estan iguales.
- P2: Como usuario de Windows, quiero que la promesa del README sea verificable o
  que diga exactamente hasta donde llega.

## Criterios de aceptacion (Given/When/Then)

<!-- Comportamiento con tests de shell; documentacion con greps. Ningun comando
     repetido entre dos AC. -->

### El chequeo de opciones

- AC-1: Given `setup_harness.sh` y `setup_harness.ps1`, When corre el chequeo,
  Then compara las opciones declaradas de los dos traduciendo `--kebab-case` a
  `-PascalCase`, y pasa cuando cada una tiene su equivalente o su declaracion.
  Comando: `bash tests/parity_check.sh opciones`
- AC-2: Given una opcion que existe en **un solo** instalador y **no** esta
  declarada como asimetria, Then el chequeo falla nombrandola y diciendo en cual
  de los dos falta.
  Comando: `bash tests/parity_check.sh detecta-opcion`
- AC-3: Given las asimetrias **legitimas** de hoy, Then estan declaradas en una
  lista con su razon escrita, y el chequeo pasa: `--with-subagents`,
  `--install-graphify`, `--install-antigravity` y `--with-postgres` (afirmativas
  de un default que ya esta encendido) y `-CargoTargetDir` (solo tiene sentido en
  Windows).
  Comando: `bash tests/parity_check.sh asimetrias-declaradas`

### El chequeo de superficies

- AC-4: Given los dos instaladores, When corre el chequeo, Then verifica que los
  dos escriban las mismas superficies (`CLAUDE.md`, `AGENTS.md`, `GEMINI.md`,
  `LLM.md`, hooks de cada backend) y falla si una aparece en uno solo.
  Comando: `bash tests/parity_check.sh superficies`
- AC-5: Given los dos smokes (`tests/setup_smoke.sh` y `tests/setup_smoke.ps1`),
  Then el chequeo compara que cubran los mismos bloques y falla si uno prueba
  algo que el otro no.
  Comando: `bash tests/parity_check.sh smokes`

### Lo que NO cubre, dicho de frente

- AC-6: Given `docs/verification.md` (+ espejo) y el README, Then dicen
  explicitamente que el chequeo **no ejecuta** el instalador de Windows y que un
  `.ps1` paritario puede fallar igual al correr.
  Comando: `grep -q "no ejecuta el instalador de Windows" docs/verification.md README.md`
- AC-7: Given la deuda de once features, Then queda **cerrada por trabajo o por
  decision**, no arrastrada: `docs/verification.md` deja de mandar a correr un
  smoke que nadie corre, y dice que hacer en su lugar.
  Comando: `bash tests/parity_check.sh promesa-acotada`

### Integracion y verificacion

- AC-8: Given `harness_check.sh`, Then corre el chequeo de paridad cuando existen
  los dos instaladores, y lo **avisa** sin bloquear: una asimetria no impide
  trabajar hoy, pero tiene que verse.
  Comando: `bash tests/parity_check.sh en-harness-check`
- AC-9: Given un repo sin `setup_harness.ps1`, Then el chequeo se omite sin
  ruido.
  Comando: `bash tests/parity_check.sh sin-ps1`
- AC-10: Given el plan, Then declara `Peldano elegido:` con su razon, como exige
  `docs/conventions.md`.
  Comando: `grep -q "Peldano elegido:" docs/plan-feature-30-paridad-ps1-verificable.md`
- AC-11: Given el repo fuente, When corre la verificacion oficial, Then
  `cargo test`, `cargo clippy --all-targets -- -D warnings`,
  `tests/setup_smoke.sh` y `harness_check.sh` siguen verdes.
  Comando: `cd rust && cargo clippy --all-targets --all-features --locked -- -D warnings`

## Los datos que se tocan

- disparador: no hay evento; el chequeo corre con `harness_check.sh` y a mano.
- interruptor: la ausencia de `setup_harness.ps1` apaga el chequeo entero.
- candado: no aplica (solo lectura).

## Pseudo-codigo (el acuerdo)

```
CUANDO corre el chequeo de paridad

  ¿existe setup_harness.ps1?  -> si no, no se dice nada

  opciones del .sh  -> traducidas a PascalCase
  opciones del .ps1 -> tal cual
  por cada una que este en uno solo:
      ¿esta declarada como asimetria, con razon?  -> se acepta
      si no                                       -> se reporta

  idem superficies y bloques de los smokes
```

## No funcionales

- **No requiere PowerShell**: es `grep`/`awk` sobre los dos archivos.
- Sin dependencias nuevas (Articulo 6).
- Corre en menos de un segundo: entra en `harness_check.sh` sin molestar.

## Fuera de alcance

- **Ejecutar** el instalador de Windows o el smoke `.ps1`. Requiere pwsh, y Alan
  decidio no instalarlo. Es el limite central y esta en el AC-6.
- Verificar que el `.ps1` sea sintacticamente valido (eso tambien exige pwsh).
- Comparar el COMPORTAMIENTO de los dos, solo la estructura declarada.

## Observaciones (decididas por Alan el 2026-08-18)

- OBS-1 **DECIDIDA: avisa, no bloquea.** Como el chequeo de convenciones de la
  #24. Una opcion de Windows desincronizada no impide trabajar hoy, y bloquear el
  cierre de todos por eso seria desproporcionado. El test `parity_check.sh` SI
  falla, asi que la asimetria queda cubierta por un AC ejecutable y no depende
  del aviso. -> AC-8.
- OBS-2 **DECIDIDA: la instruccion queda condicionada.** `docs/verification.md`
  pasa a decir "corrélo si tenes Windows a mano; si no, el chequeo de paridad es
  lo que hay", nombrando el sustituto que si corre siempre. Deja de ser una
  promesa que nadie cumple sin perder la referencia para quien pueda ejecutarla.
  -> AC-7.
