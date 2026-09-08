# Verificacion de AC - Feature #84

Corrida: 2026-09-08T23:58:44Z
Raiz de ejecucion: /Users/alan/harness_process-wt/84-leccion-partir-parte-una-leccion-sobre-el-tope-a
Resultado: 9 verde(s), 0 en rojo, 1 manual(es).

| AC | Estado | Comando | Exit | ms |
| --- | --- | --- | --- | --- |
| AC-1 | verde | `cd rust && cargo test --locked partir_informa` | 0 | 1663 |
| AC-2 | verde | `cd rust && cargo test --locked secciones_por_feature` | 0 | 181 |
| AC-3 | verde | `cd rust && cargo test --locked partir_aplica` | 0 | 1292 |
| AC-4 | verde | `cd rust && cargo test --locked partir_seccion` | 0 | 1025 |
| AC-5 | verde | `cd rust && cargo test --locked partir_sigue_sobre_el_tope` | 0 | 826 |
| AC-6 | verde | `cd rust && cargo test --locked partir_dos_veces` | 0 | 815 |
| AC-7 | verde | `bash tests/leccion_tope_check.sh` | 0 | 4494 |
| AC-8 | verde | `cd rust && cargo test --locked contrato_de_particion` | 0 | 866 |
| AC-9 | manual | `(verificacion manual)` | - | 0 |
| AC-10 | verde | `bash tests/parity_check.sh && cmp UPDATING.md templates/UPDATING.md && cmp docs/lecciones/COMO-ESCRIBIR-UNA-LECCION.md templates/docs/lecciones/COMO-ESCRIBIR-UNA-LECCION.md && grep -q "leccion partir" README.md docs/architecture.md setup_harness.sh setup_harness.ps1` | 0 | 622 |

---

Los AC marcados `manual` no declaran comando: los verifica el
reviewer, como siempre. No cuentan como fallo.
