# Verificacion de AC - Feature #36

Corrida: 2026-08-18T03:10:16Z
Resultado: 15 verde(s), 0 en rojo, 0 manual(es).

| AC | Estado | Comando | Exit | ms |
| --- | --- | --- | --- | --- |
| AC-1 | verde | `cd rust && cargo test close_gates_should_share_one_exit_code` | 0 | 1245 |
| AC-2 | verde | `cd rust && cargo test close_should_keep_usage_errors_separate_from_gates` | 0 | 673 |
| AC-3 | verde | `cd rust && cargo test verify_solo_should_accept_several_acs` | 0 | 713 |
| AC-4 | verde | `cd rust && cargo test verify_solo_should_name_the_missing_ac` | 0 | 695 |
| AC-5 | verde | `bash tests/conventions_check.sh detecta-en-src` | 0 | 858 |
| AC-6 | verde | `bash tests/conventions_check.sh sin-violaciones` | 0 | 689 |
| AC-7 | verde | `cd rust && cargo test rutas_registro_should_drop_entries_that_are_no_longer_dirty` | 0 | 113 |
| AC-8 | verde | `cd rust && cargo test rutas_registro_should_keep_live_exemptions` | 0 | 98 |
| AC-9 | verde | `cd rust && cargo test doctor_should_detect_a_hook_pointing_to_another_path` | 0 | 99 |
| AC-10 | verde | `cd rust && cargo test doctor_should_stay_quiet_with_well_wired_hooks` | 0 | 103 |
| AC-11 | verde | `cd rust && cargo test leccion_list_should_size_the_column_to_the_longest_name` | 0 | 707 |
| AC-12 | verde | `cd rust && cargo test leccion_list_should_not_change_order_fields_or_json` | 0 | 664 |
| AC-13 | verde | `bash tests/deudas_check.sh backlog-cerrado` | 0 | 127 |
| AC-14 | verde | `grep -q "Para el backlog" roles/implementer.md` | 0 | 5 |
| AC-15 | verde | `cd rust && cargo clippy --all-targets --all-features --locked -- -D warnings` | 0 | 1240 |
