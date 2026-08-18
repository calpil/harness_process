# Veredicto de revision - Feature #30: paridad_ps1_verificable

Veredicto global: **aprobado con el limite central declarado**.

Spec: `docs/spec-feature-30-paridad-ps1-verificable.md` (11 AC)
Plan: `docs/plan-feature-30-paridad-ps1-verificable.md` (`Peldano elegido: 1`)
Evidencia: `docs/impl-30.md`
Reporte: `docs/verify-30.md` (11 verde, 0 rojo, 0 manual)

## Estado por AC

Los once, cubiertos. Lo que merece atencion:

| AC | Estado | Por que |
| --- | --- | --- |
| AC-2 | cubierto | La prueba del rojo siembra la opcion en **copias**, nunca en los archivos reales. Sin ese modo, "no reporto nada" seria indistinguible de "no sabe reportar" |
| AC-3 | cubierto | Las cinco asimetrias con razon, y las razones **verificadas contra el codigo** — dos estaban mal |
| AC-6 | cubierto | El limite esta en la doc con esas palabras, no insinuado |
| AC-8 | cubierto | El test verifica que el bloque NO toque `failures`, no solo que exista |

## Que cambia de verdad

La deuda no se cierra "declarandola resuelta": se cierra porque ahora hay algo
que **falla** cuando los instaladores divergen. Antes, la unica forma de
enterarse era que alguien instalara en Windows y se rompiera.

Y el chequeo ya encontro algo real sin ejecutar nada: **cinco asimetrias
existentes** que nadie habia decidido. Cuatro resultaron legitimas y una
(`--with-postgres`) resulto ser un no-op historico que nadie recordaba.

## Lo que verifique ademas de los AC

- **Cero falsos positivos** sobre los dos instaladores reales tal como estan hoy.
- **La prueba del rojo**: opcion sembrada -> reportada, nombrando en cual falta.
- **Las razones de las asimetrias, una por una contra el codigo.** Dos estaban
  mal y se corrigieron. Este es el chequeo que mas valor agrego de toda la
  revision.
- **No necesita PowerShell**: corrio entero en esta maquina, que no lo tiene.
- **El aviso no cambia el exit code**: `harness_check.sh` sigue saliendo por sus
  propios gates.

## Observaciones (no bloquean)

1. **El limite es grande y hay que repetirlo**: paridad **estructural**, no
   funcional. El `.ps1` sigue sin ejecutarse nunca en CI ni aca. La feature
   convierte "nadie lo verifica" en "se verifica lo que se puede sin Windows", y
   eso es una mejora real, no la solucion completa.
2. **La comparacion de smokes es por keyword.** Es lo mejor que se puede hacer
   con dos archivos escritos con estilos distintos, pero un `.ps1` que mencione
   "Reset" sin probarlo pasaria. Anotado en el backlog.
3. **El parseo depende del formato** del `case` y del `param()`. Si cambian, el
   chequeo falla ruidosamente en vez de callar — que es la direccion correcta del
   error.

## Riesgo que queda vivo

Que alguien lea "paridad verificada" y entienda "el instalador de Windows
funciona". Por eso el AC-6 existe y por eso la frase esta en la doc y en el
README: **no ejecuta el instalador de Windows**. Es disciplina de redaccion, y
esta dicha como tal.
