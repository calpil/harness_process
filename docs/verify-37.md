# Verificacion de AC - Feature #37

Corrida: 2026-08-18T23:59:19Z
Resultado: 15 verde(s), 0 en rojo, 0 manual(es).

| AC | Estado | Comando | Exit | ms |
| --- | --- | --- | --- | --- |
| AC-1 | verde | `cd rust && cargo test close_should_accept_the_superseded_status` | 0 | 982 |
| AC-2 | verde | `cd rust && cargo test superseded_should_demand_the_absorbing_feature` | 0 | 782 |
| AC-3 | verde | `cd rust && cargo test superseded_should_refuse_an_unknown_absorber` | 0 | 809 |
| AC-4 | verde | `cd rust && cargo test superseded_should_record_the_absorbing_feature` | 0 | 706 |
| AC-5 | verde | `cd rust && cargo test superseded_should_not_trigger_the_done_gates` | 0 | 707 |
| AC-6 | verde | `cd rust && cargo test next_should_not_offer_a_superseded_feature` | 0 | 661 |
| AC-7 | verde | `cd rust && cargo test status_should_show_who_absorbed_a_superseded_feature` | 0 | 673 |
| AC-8 | verde | `cd rust && cargo test prd_tree_should_ignore_superseded_features` | 0 | 104 |
| AC-9 | verde | `cd rust && cargo test journey_should_not_flag_a_superseded_feature` | 0 | 94 |
| AC-10 | verde | `bash tests/superseded_check.sh migradas` | 0 | 112 |
| AC-11 | verde | `cd rust && cargo test blocked_features_should_stay_blocked` | 0 | 654 |
| AC-12 | verde | `grep -q "superseded" README.md UPDATING.md templates/UPDATING.md` | 0 | 7 |
| AC-13 | verde | `grep -q "absorbida-por" roles/reviewer.md templates/roles/reviewer.md` | 0 | 5 |
| AC-14 | verde | `grep -q "Peldano elegido:" docs/plan-feature-37-estado-superseded.md` | 0 | 4 |
| AC-15 | verde | `cd rust && cargo clippy --all-targets --all-features --locked -- -D warnings` | 0 | 160 |
