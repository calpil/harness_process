# Verificacion de AC - Feature #25

Corrida: 2026-08-17T19:16:30Z
Resultado: 20 verde(s), 0 en rojo, 0 manual(es).

| AC | Estado | Comando | Exit | ms |
| --- | --- | --- | --- | --- |
| AC-1 | verde | `cd rust && cargo test doctor_should_report_every_area_on_a_healthy_install` | 0 | 787 |
| AC-2 | verde | `cd rust && cargo test doctor_should_print_an_exact_remedy_for_every_problem` | 0 | 624 |
| AC-3 | verde | `cd rust && cargo test doctor_should_separate_failures_from_warnings` | 0 | 644 |
| AC-4 | verde | `cd rust && cargo test doctor_json_should_expose_area_state_and_remedy` | 0 | 630 |
| AC-5 | verde | `cd rust && cargo test doctor_should_detect_a_binary_older_than_the_scripts` | 0 | 651 |
| AC-6 | verde | `cd rust && cargo test doctor_should_detect_a_hook_pointing_nowhere` | 0 | 631 |
| AC-7 | verde | `cd rust && cargo test doctor_should_only_demand_surfaces_the_backend_uses` | 0 | 630 |
| AC-8 | verde | `cd rust && cargo test doctor_should_explain_which_root_it_resolved_and_why` | 0 | 637 |
| AC-9 | verde | `cd rust && cargo test doctor_should_treat_an_unreachable_hub_as_a_warning` | 0 | 660 |
| AC-10 | verde | `cd rust && cargo test doctor_should_split_required_and_optional_tools` | 0 | 647 |
| AC-11 | verde | `cd rust && cargo test doctor_should_report_graphify_as_optional` | 0 | 640 |
| AC-12 | verde | `cd rust && cargo test doctor_should_not_demand_surfaces_in_a_source_checkout` | 0 | 635 |
| AC-13 | verde | `sh harness_cli doctor` | 0 | 399 |
| AC-14 | verde | `cd rust && cargo test doctor_should_not_duplicate_the_process_checks` | 0 | 655 |
| AC-15 | verde | `cd rust && cargo test doctor_should_not_write_anything` | 0 | 643 |
| AC-16 | verde | `bash tests/doctor_launcher_check.sh` | 0 | 2939 |
| AC-17 | verde | `grep -q "harness_cli doctor" README.md UPDATING.md templates/UPDATING.md` | 0 | 9 |
| AC-18 | verde | `grep -q "doctor" roles/implementer.md setup_harness.sh setup_harness.ps1` | 0 | 8 |
| AC-19 | verde | `grep -q "Peldano elegido:" docs/plan-feature-25-harness-doctor.md` | 0 | 5 |
| AC-20 | verde | `cd rust && cargo clippy --all-targets --all-features --locked -- -D warnings` | 0 | 156 |
