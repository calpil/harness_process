# Veredicto de revision - Feature #28: consolidacion_de_lecciones_con_llm

Veredicto global: **aprobado con limites declarados**.

Spec: `docs/spec-feature-28-consolidacion-de-lecciones-con-llm.md` (27 AC)
Plan: `docs/plan-feature-28-consolidacion-de-lecciones-con-llm.md` (`Peldano elegido: 3`)
Evidencia: `docs/impl-28.md`
Reporte: `docs/verify-28.md`

## Estado por AC

Los veintisiete, cubiertos. Lo que merece mirarse con atencion:

| AC | Estado | Por que |
| --- | --- | --- |
| AC-6 | cubierto | Los fixtures son salidas **reales** de `claude` y de `kimi`, capturadas el mismo dia. Dos formas genuinamente distintas: JSON pelado contra JSON envuelto en vinnetas con banner y linea de sesion |
| AC-7 | cubierto | Inspecciona el prompt y verifica que NO contenga los encabezados del cuerpo. Es la frontera entera de la feature |
| AC-8 | cubierto | Contrato de comportamiento: el veneno llega literal. La primera version de este test era tautologica y fallaba con el codigo bien |
| AC-22 | cubierto | Habla con un backend de verdad. Sin esto, la acceptance decia explicitamente que la feature no cierra |
| AC-24 | cubierto | La corrida real esta documentada con lo que el modelo propuso Y lo que se decidio no fusionar |

## Lo que hace creible esta revision

**La feature se ejercito sobre la biblioteca real y la cambio.** La biblioteca de
Alan paso de 9 a 8 lecciones. Eso se puede leer, no es una metafora:

- El cuerpo archivado es **byte a byte identico**.
- `buscar "install_asset"` —un trigger que solo vivia en la archivada— devuelve
  ahora el paraguas como **primer resultado**. Sin el AC-17 (heredar los
  triggers), habria devuelto la archivada con peso 30 en vez de la activa con
  100. El AC no era decorativo.
- Los pitfalls pasaron de 7 a 5 **sin perder ninguno**: dos pares eran el mismo
  pitfall escrito dos veces. Ese es, literalmente, el valor de la feature.

## El hallazgo que vale la pena discutir

**El modelo y la metrica lexica no coincidieron, y los dos tenian parte de
razon.**

| Par | Jaccard | Modelo |
| --- | --- | --- |
| docs-generados + documentos-del-usuario | 0.400 | 0.85 |
| criterios-de-cierre + probar-contra-datos-reales | **0.048** | **0.60** |

El segundo par lo vio **solo el modelo**: sus triggers casi no se cruzan pero son
vecinas semanticas. Se decidio **no** fusionarlas —ensenan procedimientos
distintos— y esa decision la tomo una persona leyendo, que es exactamente el
reparto de trabajo que la feature propone.

Y es el mejor argumento a favor de la decision de Alan sobre la confianza: **el
orden lleva informacion aunque el umbral no se pueda calibrar**. Un umbral en
0.5 habria dejado pasar el par discutible como si fuera tan cierto como el otro.

## Lo que verifique ademas de los AC

- **Que el modelo no vea el cuerpo**, leyendo el prompt que se arma.
- **Que la deteccion no escriba**: `find | shasum` antes y despues, identicos, y
  sin `bkp/`.
- **Que el skip sea limpio** con `PATH=/nonexistent`: exit 0, sin rastro, y con
  el motivo dicho.
- **Que el rollback funcione**: la leccion archivada vuelve al catalogo activo.
- **Que dos backends reales parseen igual**, con sus salidas capturadas.

## Observaciones (no bloquean)

1. **El tramo de API key no existe** (decision de Alan). El skip lo nombra. Es la
   unica parte de la acceptance que no se implemento, y esta dicho en el spec, en
   el codigo, en el README y aca.
2. **Solo dos de los cuatro backends se pudieron ejercitar.** `gemini` esta
   discontinuado para individuos y `codex` sin cuota. Dos es suficiente para
   probar agnosticidad, pero no es lo mismo que cuatro.
3. **El parser toma el primer objeto balanceado.** Un backend que imprimiera
   `{...}` en su razonamiento antes del JSON lo confundiria. Hay un test que fija
   ese limite.
4. **`consolidar_check.sh` gasta cuota real** en cada corrida. Con la suite
   corriendo seguido, eso se nota.

## Riesgo que queda vivo

Que alguien fusione dos lecciones que ensenan cosas distintas porque el modelo
dijo 0.60 y sonaba convincente. Contra eso hay defensas verificables (el paraguas
tiene que heredar los triggers y citar a cada miembro, el motivo es obligatorio,
nada se borra y todo se deshace) y una que no lo es: que la persona lea las dos
lecciones antes de decir que si. En esta feature esa persona leyo y **rechazo**
uno de los dos candidatos — que es la mejor evidencia de que el reparto funciona,
pero no una garantia de que siempre vaya a pasar.
