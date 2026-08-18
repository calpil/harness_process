# Verificacion de AC - Feature #26

Corrida: 2026-08-18T01:12:52Z
Resultado: 21 verde(s), 0 en rojo, 0 manual(es).

| AC | Estado | Comando | Exit | ms |
| --- | --- | --- | --- | --- |
| AC-1 | verde | `cd rust && cargo test deny_should_protect_the_three_defaults` | 0 | 242 |
| AC-2 | verde | `cd rust && cargo test deny_should_match_globs_at_any_depth` | 0 | 99 |
| AC-3 | verde | `cd rust && cargo test deny_should_normalize_absolute_and_relative_paths` | 0 | 93 |
| AC-4 | verde | `cd rust && cargo test deny_should_not_guess_beyond_the_list` | 0 | 96 |
| AC-5 | verde | `bash tests/deny_check.sh previene` | 0 | 560 |
| AC-6 | verde | `bash tests/deny_check.sh detecta` | 0 | 629 |
| AC-7 | verde | `bash tests/deny_check.sh red-de-seguridad` | 0 | 656 |
| AC-8 | verde | `grep -q "no puede prevenir" docs/rutas-protegidas.md` | 0 | 4 |
| AC-9 | verde | `cd rust && cargo test close_should_still_write_the_prd_milestone_when_protected` | 0 | 730 |
| AC-10 | verde | `bash tests/deny_check.sh no-se-autobloquea` | 0 | 674 |
| AC-11 | verde | `cd rust && cargo test deny_should_read_user_defined_paths` | 0 | 94 |
| AC-12 | verde | `cd rust && cargo test deny_should_fall_back_to_defaults_when_unconfigured` | 0 | 87 |
| AC-13 | verde | `cd rust && cargo test deny_should_be_disablable_with_an_empty_list` | 0 | 87 |
| AC-14 | verde | `bash tests/deny_check.sh compatible` | 0 | 605 |
| AC-15 | verde | `bash tests/deny_check.sh sin-costo` | 0 | 597 |
| AC-16 | verde | `cd rust && cargo test doctor_should_report_protected_paths_status` | 0 | 98 |
| AC-17 | verde | `diff -q docs/rutas-protegidas.md templates/docs/rutas-protegidas.md` | 0 | 5 |
| AC-18 | verde | `grep -q "rutas protegidas" README.md UPDATING.md templates/UPDATING.md` | 0 | 5 |
| AC-19 | verde | `grep -q "ruta protegida" roles/implementer.md roles/reviewer.md` | 0 | 4 |
| AC-20 | verde | `grep -q "Peldano elegido:" docs/plan-feature-26-rutas-protegidas-deny.md` | 0 | 4 |
| AC-21 | verde | `cd rust && cargo clippy --all-targets --all-features --locked -- -D warnings` | 0 | 160 |
