# Verificacion de AC - Feature #44

Corrida: 2026-08-19T02:55:51Z
Resultado: 17 verde(s), 0 en rojo, 0 manual(es).

| AC | Estado | Comando | Exit | ms |
| --- | --- | --- | --- | --- |
| AC-1 | verde | `cd rust && cargo test casos_corridos_should_not_opine_about_non_libtest_output` | 0 | 171 |
| AC-2 | verde | `cd rust && cargo test casos_corridos_should_count_zero_on_the_real_empty_filter` | 0 | 84 |
| AC-3 | verde | `cd rust && cargo test casos_corridos_should_sum_across_test_binaries` | 0 | 80 |
| AC-4 | verde | `cd rust && cargo test casos_corridos_should_count_ignored_tests_as_no_evidence` | 0 | 81 |
| AC-5 | verde | `cd rust && cargo test vacio_should_block_without_pretending_to_be_red` | 0 | 80 |
| AC-6 | verde | `cd rust && cargo test ejecutar_should_mark_an_empty_test_run_as_vacio` | 0 | 86 |
| AC-7 | verde | `cd rust && cargo test ejecutar_should_keep_a_real_test_run_green` | 0 | 84 |
| AC-8 | verde | `cd rust && cargo test ejecutar_should_not_mark_a_non_test_command_as_vacio` | 0 | 90 |
| AC-9 | verde | `cd rust && cargo test render_should_count_empty_runs_apart_from_red` | 0 | 81 |
| AC-10 | verde | `cd rust && cargo test close_should_block_on_an_empty_verification` | 0 | 666 |
| AC-11 | verde | `cd rust && cargo test etiqueta_should_round_trip_for_every_estado` | 0 | 94 |
| AC-12 | verde | `bash tests/verify_vacio_check.sh` | 0 | 1005 |
| AC-13 | verde | `cd rust && cargo test consolidar_without_aplicar_should_not_touch_anything` | 0 | 1701 |
| AC-14 | verde | `test "$(grep -c "\| vacio \|" docs/verify-28.md)" -eq 0` | 0 | 8 |
| AC-15 | verde | `grep -q "Peldano elegido:" docs/plan-feature-44-verify-detecta-filtro-vacio.md` | 0 | 5 |
| AC-16 | verde | `grep -q "vacio" README.md UPDATING.md templates/UPDATING.md` | 0 | 6 |
| AC-17 | verde | `cd rust && cargo clippy --all-targets --all-features --locked -- -D warnings` | 0 | 162 |
