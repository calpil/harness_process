# Verificacion de AC - Feature #29

Corrida: 2026-08-18T13:15:23Z
Resultado: 23 verde(s), 0 en rojo, 0 manual(es).

| AC | Estado | Comando | Exit | ms |
| --- | --- | --- | --- | --- |
| AC-1 | verde | `cd rust && cargo test documentos_alcance_should_include_the_prd_chain_sdd_and_architecture` | 0 | 150 |
| AC-2 | verde | `cd rust && cargo test documentos_alcance_should_walk_nested_prds_without_repeating` | 0 | 104 |
| AC-3 | verde | `cd rust && cargo test documentos_alcance_should_skip_missing_documents` | 0 | 86 |
| AC-4 | verde | `cd rust && cargo test prd_propose_should_seed_one_block_per_document` | 0 | 675 |
| AC-5 | verde | `cd rust && cargo test prd_propose_should_not_clobber_existing_verdicts` | 0 | 664 |
| AC-6 | verde | `cd rust && cargo test prd_propose_should_precompute_presence_signals` | 0 | 646 |
| AC-7 | verde | `cd rust && cargo test prd_apply_should_reject_a_tampered_block_list` | 0 | 657 |
| AC-8 | verde | `cd rust && cargo test prd_apply_should_replace_the_literal_anchor_not_the_section` | 0 | 98 |
| AC-9 | verde | `cd rust && cargo test prd_apply_should_refuse_a_citation_that_does_not_hold` | 0 | 663 |
| AC-10 | verde | `cd rust && cargo test prd_apply_should_accept_no_aplica_with_a_reason` | 0 | 93 |
| AC-11 | verde | `cd rust && cargo test prd_apply_should_name_the_unresolved_block` | 0 | 85 |
| AC-12 | verde | `cd rust && cargo test prd_apply_without_yes_should_show_and_refuse_to_write` | 0 | 639 |
| AC-13 | verde | `cd rust && cargo test prd_apply_with_yes_should_write_seal_and_log` | 0 | 654 |
| AC-14 | verde | `cd rust && cargo test prd_apply_should_be_idempotent_by_content` | 0 | 95 |
| AC-15 | verde | `cd rust && cargo test prd_diff_should_live_outside_the_protected_path` | 0 | 649 |
| AC-16 | verde | `cd rust && cargo test prd_apply_should_register_its_own_writes` | 0 | 654 |
| AC-17 | verde | `cd rust && cargo test close_should_demand_the_docs_proposal_when_the_rule_is_on` | 0 | 1191 |
| AC-18 | verde | `cd rust && cargo test docs_gate_should_not_depend_on_verify_report_freshness` | 0 | 105 |
| AC-19 | verde | `cd rust && cargo test no_spec_command_should_invoke_prd_apply_yes` | 0 | 94 |
| AC-20 | verde | `grep -q "prd propose" CHECKPOINTS.md roles/implementer.md templates/CHECKPOINTS.md` | 0 | 5 |
| AC-21 | verde | `grep -q "prd apply" README.md UPDATING.md templates/UPDATING.md` | 0 | 4 |
| AC-22 | verde | `grep -q "Peldano elegido:" docs/plan-feature-29-prd-y-sdd-siempre-al-dia.md` | 0 | 3 |
| AC-23 | verde | `cd rust && cargo clippy --all-targets --all-features --locked -- -D warnings` | 0 | 145 |
