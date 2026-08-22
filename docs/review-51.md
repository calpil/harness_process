# Veredicto del reviewer - Feature #51: revision_adversarial_y_modelos_por_rol

Veredicto: **approved**
Fecha: 2026-08-22
Spec: `docs/spec-feature-51-revision-adversarial-y-modelos-por-rol.md` (approved, 18 AC)
Evidencia: `docs/impl-51.md`
Material: `harness revision --feature 51` (478 lineas, ~6654 tokens)

Este veredicto aplica la regla que la propia feature introduce: se intento
REFUTAR cada AC antes de darlo por bueno, y lo que sigue dice tambien que NO se
pudo probar.

## Verificacion oficial

| Comando | Resultado |
| --- | --- |
| `cargo test` | 355 unit + 173 integracion = **528 en verde** |
| `cargo clippy --all-targets -- -D warnings` | limpio |
| `bash tests/setup_smoke.sh` | exit 0 |
| `./harness_check.sh` | limpio |

## Intentos de refutacion

| AC | Como se intento romper | Resultado |
| --- | --- | --- |
| AC-1/AC-2 | Reinstalar dos veces y comparar; cambiar el modelo por variable de entorno | No se rompio: el smoke exige `cmp -s` sin diff y el override cambia el resultado |
| AC-4 | Editar el espejo a mano y reinstalar (era el bug original que motivo la feature) | El instalador lo regenera, pero ahora con los valores que el usuario pidio: `max` dejo de aparecer |
| AC-11/AC-13 | Correr `revision` sin `verify-<id>.md`, sin `impl-<id>.md` y sin rama propia | No se rompio: arma el paquete y nombra cada ausencia en `## Falta` |
| AC-12 | Diff de 200 lineas con presupuesto de 20 | No se rompio: recorta y declara `se muestran 20 de N` |
| AC-12c | Medir el paquete real de esta feature | 478 lineas / ~6654 tokens: tres ordenes de magnitud por debajo del problema |
| AC-11 (trabajo en curso) | **SI SE ROMPIO**: con el trabajo sin commitear, el paquete decia "archivos tocados: ninguno" | Corregido (compara el worktree contra la base) y cubierto por test |
| AC-11 (archivos nuevos) | **SI SE ROMPIO**: un archivo creado y no indexado era invisible | Corregido (se listan marcados `(nuevo, sin git add)`) y cubierto por test |
| Fuera de alcance | Correr la suite parado en un worktree | **SE ROMPIO OTRA COSA**: el foco de la feature #47 desviaba los docs de los sandboxes al worktree real. Corregido aca (se exige el mismo repo) |

## Lo que NO se pudo probar

- **AC-3 (PowerShell)**: no hay `pwsh` en esta maquina. La paridad se verifico
  por lectura y por asserts de contenido, no ejecutando el instalador. Es el
  mismo limite aceptado en las features #1, #13, #14, #15, #16 y #47.
- **AC-6 a AC-10 (el rol)**: son instrucciones para un agente. Se verifico que
  el texto diga lo que el AC exige y que llegue a los espejos, pero que un
  reviewer efectivamente refute no se puede probar con un test: se vera en las
  proximas revisiones.
- **AC-16 en su forma fuerte**: este veredicto usa el paquete y declara sus
  intentos, pero lo escribio el mismo agente que implemento. Un reviewer
  independiente seguiria siendo mas duro.
- **AC-12c en features grandes**: se midio sobre esta feature. Una con un diff
  enorme va a recortar mucho mas; el paquete lo dira, pero cuanto material util
  queda afuera en ese caso no se probo.

## Constitution

- **Articulo 1**: tests nuevos junto al codigo tocado (7 unit del modulo, 3 de
  integracion, un bloque nuevo en el smoke) y los cuatro comandos en verde.
- **Articulo 2**: spec `approved` antes de implementar; el dato de los 10M de
  tokens quedo registrado en el spec como disparador (OBS-5).
- **Articulo 3**: D1..D9 citan sus AC-n; evidencia y veredicto por AC.
- **Articulo 4**: `revision` es de solo lectura y no imprime el contenido de
  rutas protegidas: las nombra y avisa que fueron tocadas.
- **Articulo 5**: cuatro decisiones del usuario registradas (OBS-1..OBS-4).
- **Articulo 6**: sin dependencias nuevas; `roles/` y `templates/roles/`
  propagados modulo `__HREL__`; espejos `.claude/agents/*` regenerados.

## Reparos

1. **El `Peldano elegido` esta declarado** en el plan (comando nuevo), con la
   razon: la disciplina escrita en el rol es lo que ya fallo — 10M de tokens.
2. **Opus en el implementer encarece cada implementacion**. Es decision
   explicita del usuario; el ahorro esta del lado de la revision.
3. **El paquete no incluye el contenido de los archivos nuevos**, solo los
   nombra. Si el reviewer necesita ver uno, tiene que abrirlo — es deliberado
   para no inflar el paquete, pero conviene saberlo.
