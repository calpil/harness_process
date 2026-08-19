# Verificacion de AC - Feature #28

Corrida: 2026-08-19T00:26:24Z
Resultado: 27 verde(s), 0 en rojo, 0 manual(es).

| AC | Estado | Comando | Exit | ms |
| --- | --- | --- | --- | --- |
| AC-1 | verde | `cd rust && cargo test consolidar_should_be_off_without_the_rule` | 0 | 678 |
| AC-2 | verde | `cd rust && cargo test consolidar_should_skip_cleanly_without_a_backend` | 0 | 101 |
| AC-3 | verde | `cd rust && cargo test consolidar_should_name_the_api_key_limitation` | 0 | 82 |
| AC-4 | verde | `cd rust && cargo test consolidar_override_should_win_over_detection` | 0 | 82 |
| AC-5 | verde | `cd rust && cargo test consolidar_should_detect_the_first_available_cli` | 0 | 80 |
| AC-6 | verde | `cd rust && cargo test consolidar_should_parse_the_output_of_both_backends` | 0 | 81 |
| AC-7 | verde | `cd rust && cargo test consolidar_should_never_send_the_lesson_body` | 0 | 81 |
| AC-8 | verde | `cd rust && cargo test consolidar_should_not_pass_the_prompt_through_a_shell` | 0 | 86 |
| AC-9 | verde | `cd rust && cargo test consolidar_should_drop_hallucinated_members` | 0 | 87 |
| AC-10 | verde | `cd rust && cargo test consolidar_should_respect_the_pin` | 0 | 79 |
| AC-11 | verde | `cd rust && cargo test consolidar_should_survive_a_garbage_answer` | 0 | 82 |
| AC-12 | verde | `cd rust && cargo test consolidar_without_aplicar_should_not_touch_anything` | 0 | 871 |
| AC-13 | verde | `cd rust && cargo test consolidar_aplicar_should_take_the_merge_from_argv` | 0 | 622 |
| AC-14 | verde | `cd rust && cargo test consolidar_aplicar_should_demand_a_motivo` | 0 | 673 |
| AC-15 | verde | `cd rust && cargo test consolidar_should_allow_an_existing_member_as_the_umbrella` | 0 | 642 |
| AC-16 | verde | `cd rust && cargo test consolidar_should_refuse_a_skeleton_umbrella` | 0 | 659 |
| AC-17 | verde | `cd rust && cargo test consolidar_should_demand_the_union_of_triggers` | 0 | 643 |
| AC-18 | verde | `cd rust && cargo test consolidar_should_demand_a_pointer_to_each_member` | 0 | 649 |
| AC-19 | verde | `cd rust && cargo test consolidar_should_archive_byte_for_byte_with_backup` | 0 | 664 |
| AC-20 | verde | `cd rust && cargo test consolidar_should_be_undoable_with_rollback` | 0 | 664 |
| AC-21 | verde | `cd rust && cargo test consolidar_report_should_list_each_merge_with_its_reason` | 0 | 645 |
| AC-22 | verde | `bash tests/consolidar_check.sh backend-real` | 0 | 6182 |
| AC-23 | verde | `bash tests/consolidar_check.sh catalogo-limpio` | 0 | 5666 |
| AC-24 | verde | `grep -q "Corrida contra el corpus real" docs/impl-28.md` | 0 | 5 |
| AC-25 | verde | `grep -q "Peldano elegido:" docs/plan-feature-28-consolidacion-de-lecciones-con-llm.md` | 0 | 5 |
| AC-26 | verde | `grep -q "lecciones consolidar" README.md UPDATING.md templates/UPDATING.md` | 0 | 5 |
| AC-27 | verde | `cd rust && cargo clippy --all-targets --all-features --locked -- -D warnings` | 0 | 271 |
