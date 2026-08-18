# Verificacion de AC - Feature #23

Corrida: 2026-08-17T05:38:46Z
Resultado: 20 verde(s), 0 en rojo, 0 manual(es).

| AC | Estado | Comando | Exit | ms |
| --- | --- | --- | --- | --- |
| AC-1 | verde | `cd rust && cargo test verificacion::tests::parse` | 0 | 141 |
| AC-2 | verde | `cd rust && cargo test verify_should_do_nothing_without_declared_commands` | 0 | 691 |
| AC-3 | verde | `cd rust && cargo test verificacion::tests::manual` | 0 | 97 |
| AC-4 | verde | `cd rust && cargo test verify_should_print_each_command_before_running_it` | 0 | 698 |
| AC-5 | verde | `cd rust && cargo test verify_should_refuse_to_run_commands_from_a_draft_spec` | 0 | 701 |
| AC-6 | verde | `cd rust && cargo test verify_should_time_out_a_hung_command` | 0 | 1707 |
| AC-7 | verde | `grep -rn "verify" bin/harness-hook setup_harness.sh \| grep -v "^setup_harness.sh:.*#" \| grep -c "harness_cli.*verify" \|\| true` | 0 | 11 |
| AC-8 | verde | `cd rust && cargo test verify_should_write_a_report_per_ac` | 0 | 738 |
| AC-9 | verde | `cd rust && cargo test verify_should_include_output_of_failures` | 0 | 692 |
| AC-10 | verde | `cd rust && cargo test verify_json_should_expose_the_result_per_ac` | 0 | 681 |
| AC-11 | verde | `cd rust && cargo test verify_should_run_a_single_ac_on_demand` | 0 | 665 |
| AC-12 | verde | `cd rust && cargo test close_should_stay_identical_without_the_verify_rule` | 0 | 700 |
| AC-13 | verde | `cd rust && cargo test close_should_demand_a_verify_report` | 0 | 655 |
| AC-14 | verde | `cd rust && cargo test close_should_block_on_a_red_report` | 0 | 658 |
| AC-15 | verde | `cd rust && cargo test close_should_block_on_a_stale_report` | 0 | 677 |
| AC-16 | verde | `cd rust && cargo test close_should_never_execute_verify_commands` | 0 | 665 |
| AC-17 | verde | `cd rust && cargo test start_should_document_the_command_line_in_the_spec_template` | 0 | 653 |
| AC-18 | verde | `grep -q "require_verify_green" README.md UPDATING.md docs/architecture.md` | 0 | 6 |
| AC-19 | verde | `grep -q "verify" roles/leader.md roles/implementer.md roles/reviewer.md` | 0 | 5 |
| AC-20 | verde | `cd rust && cargo clippy --all-targets --all-features --locked -- -D warnings` | 0 | 165 |
