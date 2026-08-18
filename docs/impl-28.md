# Evidencia de implementacion - Feature #28: consolidacion_de_lecciones_con_llm

Spec: `docs/spec-feature-28-consolidacion-de-lecciones-con-llm.md` (`Estado: approved`, 27 AC)
Plan: `docs/plan-feature-28-consolidacion-de-lecciones-con-llm.md` (D1-D8, `Peldano elegido: 3`)
PRD: `docs/prd/aprendizaje/PRD-aprendizaje.md` (ultimo hito)

## Corrida contra el corpus real

El criterio de entrada de la acceptance era poder verificarla de punta a punta
con un backend de verdad. Se cumplio: `claude -p` y `kimi -p` responden no
interactivos en esta maquina (`gemini` esta discontinuado con
`IneligibleTierError` y `codex` sin cuota hasta el 20).

```
$ sh harness_cli lecciones consolidar
Consultando a `claude` por 9 lecciones (nombre, descripcion y triggers; NUNCA el cuerpo)...

2 candidato(s) a consolidar:

  docs-generados-por-el-instalador + documentos-del-usuario-vs-plantillas (confianza 0.85)
      Ambas ensenan el mismo mecanismo: sumar o clasificar un doc es elegir la
      lista correcta (HARNESS_DOCS/USER_DOCS/PRD_DOCS) que decide siembra y
      reset, no escribir codigo.

  criterios-de-cierre-que-se-pueden-fallar + probar-contra-datos-reales (confianza 0.60)
      Las dos atacan el verde falso: una verificacion que no puede fallar o que
      solo corre contra fixtures conocidos tranquiliza sin medir nada real.

Esto SOLO informa: no se toco ningun archivo.
```

Y esto es lo interesante, porque **el modelo y la metrica lexica no coincidieron**:

| Par | Jaccard (triggers) | Modelo | Veredicto |
| --- | --- | --- | --- |
| docs-generados + documentos-del-usuario | **0.400** | 0.85 | fusionado |
| criterios-de-cierre + probar-contra-datos-reales | 0.048 | 0.60 | **no** fusionado |

El primero lo encontraron los dos. El segundo lo vio **solo el modelo**: sus
triggers casi no se cruzan, pero semanticamente son vecinas —y fue **mi propia
hipotesis previa**, que el analisis lexico habia descartado—. Se decidio no
fusionarlas: se citan mutuamente en `relacionadas` pero ensenan procedimientos
distintos (una, como escribir criterios de cierre; la otra, como calibrar contra
datos reales).

Es el argumento a favor de la decision OBS-3: **el orden de la confianza lleva
informacion aunque el umbral no se pueda calibrar**. 0.85 para el real, 0.60 para
el discutible.

## La fusion real, aplicada con el si de Alan

Se le mostro el paraguas completo antes de tocar nada. Con su aprobacion:

```
Paraguas:   documentos-del-usuario-vs-plantillas
Archivadas: docs-generados-por-el-instalador
Backup:     bkp/lecciones/consolidar/...
```

Verificado despues:

- **El cuerpo archivado es byte a byte identico** al original (solo cambio
  `estado: activa` -> `archivada` en el frontmatter).
- **La biblioteca paso de 9 a 8** lecciones.
- **`buscar "install_asset"`** —un trigger que solo existia en la archivada— ahora
  devuelve **el paraguas como primer resultado**. Ese es exactamente el efecto que
  el AC-17 fue disenado para producir: sin heredar los triggers, `buscar` habria
  devuelto la archivada con peso 30 en vez de la activa con 100.
- **Ningun pitfall se perdio.** Los originales sumaban 7 y el paraguas tiene 5,
  porque **dos pares eran el mismo pitfall escrito dos veces**: "tocar solo
  `setup_harness.sh`" == "olvidar el `.ps1`", y "listar contenido ganado en
  HARNESS_DOCS" == "meter un documento del usuario en HARNESS_DOCS" (las dos
  caras de la misma regla). Eso es exactamente el valor de fusionar.

## Archivos tocados

| Archivo | D | Que cambio |
| --- | --- | --- |
| `rust/src/consolidacion.rs` | D1-D4, D6 | NUEVO. Cadena de backend, prompt, parser, validacion y requisitos del paraguas; 20 tests |
| `rust/src/commands/leccion.rs` | D5, D7 | `consolidar` con la simetria de `curar` |
| `rust/src/cli.rs` | D5 | `LeccionesCommand::Consolidar` |
| `tests/consolidar_check.sh` | D8 | NUEVO. Cuatro modos, dos con backend REAL |
| `rust/tests/cli_basics.rs` | D7 | 9 tests de la mitad que muta, sin backend |
| README, UPDATING (+ espejo) | D8 | El comando y sus limites |

## Evidencia por AC

`sh harness_cli verify --feature 28`: los 27 comandos.

| AC | Evidencia |
| --- | --- |
| AC-1..AC-3 | apagada sin la regla; skip limpio; el skip nombra la limitacion de HTTP |
| AC-4..AC-6 | override gana; primer CLI de la tabla; **fixtures con la salida REAL de claude y de kimi** |
| AC-7 | `consolidar_should_never_send_the_lesson_body` inspecciona el prompt |
| AC-8 | contrato de comportamiento: un prompt con `$(...)` llega LITERAL |
| AC-9..AC-11 | alucinaciones descartadas y dichas; pin respetado; basura tolerada |
| AC-12..AC-14 | sin `--aplicar` cero escrituras; la fusion sale de argv; sin `--motivo` exit 2 |
| AC-15..AC-18 | el paraguas puede ser miembro; sin placeholders; union de triggers; punteros |
| AC-19..AC-21 | archivado byte a byte con backup; rollback; reporte con motivo |
| AC-22..AC-24 | `consolidar_check.sh` con backend real; catalogo limpio; esta seccion |
| AC-25..AC-27 | peldano; docs; 318 + 153 tests y clippy 0 |

## Tres defectos que encontre corriendo los tests, no escribiendolos

1. **El env var se filtraba entre tests.** `resolver_backend` leia
   `HARNESS_CONSOLIDAR_CMD` del entorno, que en Rust es **compartido entre los
   tests que corren en paralelo**: el test del override le rompia el suyo a otros
   dos. Se refactorizo para que el override llegue **por parametro** — la funcion
   quedo pura y los tests dejaron de depender del proceso.
2. **Mi assertion de inyeccion era tautologica.** El test comprobaba
   `!salida.contains("INYECTADO")` con un veneno que era literalmente
   `$(echo INYECTADO)`: el texto literal **contiene** esa palabra, asi que el
   test fallaba con el codigo bien. La assertion correcta es la **igualdad** con
   el literal.
3. **`sleep 30 <prompt>` no cuelga en macOS**, falla al instante con "invalid
   time interval", asi que el test del timeout medía otra cosa. Se reemplazo por
   un script propio que duerme sin importar los argumentos.

Los tres son de la misma familia que las cinco features anteriores: la suite
verde no prueba lo que uno cree hasta que se la mira de cerca.

## Limites declarados

- **El tramo de API key no esta implementado** (decision de Alan, OBS-1). El
  mensaje de skip lo dice con todas las letras. La cadena real es
  **override -> CLI -> skip**.
- **`gemini` y `codex` no se pudieron ejercitar**: el primero esta discontinuado
  para cuentas individuales, el segundo sin cuota. La agnosticidad se verifico
  con **dos** backends reales (claude y kimi), no con uno.
- **El parser toma el primer objeto de llaves balanceadas.** Si un backend
  imprimiera un `{...}` en su razonamiento antes del JSON, se quedaria con ese.
  Hay un test que fija ese limite en vez de disimularlo.
- **El modelo es no deterministico**: dos corridas pueden proponer distinto. Es
  aceptable porque la deteccion solo informa y lo que muta sale de argv.
- **El umbral no se puede calibrar** con 9 lecciones. Declarado, no inventado.

## Para el backlog

- **La deteccion no usa los `relacionadas`**, que son una senal barata y escrita
  a mano de que dos lecciones se tocan.
- **El paraguas se escribe a mano.** Un paso siguiente seria que el binario
  pre-arme el esqueleto con la union de triggers y los punteros ya puestos, para
  que la persona solo escriba la prosa.
- **`consolidar_check.sh` gasta cuota real** en cada corrida (dos llamadas al
  backend). Con muchas features eso se nota.
