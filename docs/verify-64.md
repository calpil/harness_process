# Verificacion de AC - Feature #64

Corrida: 2026-08-29T01:06:13Z
Raiz de ejecucion: /Users/alan/harness_process-wt/64-el-arnes-no-promete-enforcement-que-no-hace
Resultado: 12 verde(s), 0 en rojo, 0 manual(es).

| AC | Estado | Comando | Exit | ms |
| --- | --- | --- | --- | --- |
| AC-1 | verde | `cd rust && out=$(cargo test gate_review 2>&1) && printf %s "$out" \| grep -qE "[1-9][0-9]* passed" && ! printf %s "$out" \| grep -q "FAILED"` | 0 | 5142 |
| AC-2 | verde | `cd rust && cargo test gate_review_ignora_prosa 2>&1 \| grep -E "[1-9][0-9]* passed"` | 0 | 88 |
| AC-3 | verde | `cd rust && cargo test veredicto_exige_cobertura_de_ac 2>&1 \| grep -E "[1-9][0-9]* passed"` | 0 | 89 |
| AC-4 | verde | `cd rust && cargo test veredicto_estampa_y_habilita_el_cierre 2>&1 \| grep -E "[1-9][0-9]* passed"` | 0 | 928 |
| AC-5 | verde | `cd rust && cargo test gate_review_solo_approved 2>&1 \| grep -E "[1-9][0-9]* passed"` | 0 | 87 |
| AC-6 | verde | `cd rust && cargo test require_review_default_false 2>&1 \| grep -E "[1-9][0-9]* passed"` | 0 | 82 |
| AC-7 | verde | `! grep -qE "require_tests_to_close\|require_impact_check\|one_feature_at_a_time" templates/feature_list.json` | 0 | 4 |
| AC-8 | verde | `bash tests/setup_smoke.sh >/dev/null 2>&1` | 0 | 70936 |
| AC-9 | verde | `bash tests/parity_check.sh` | 0 | 659 |
| AC-10 | verde | `grep -n "2026-08-22" UPDATING.md` | 0 | 5 |
| AC-11 | verde | `! grep -rqi "una sola a la vez" roles/ templates/roles/ .claude/agents/ && for r in leader implementer reviewer README; do [ "$(cat roles/$r.md)" = "$(sed "s\|__HREL__\|harness_process/\|g" templates/roles/$r.md)" ] \|\| exit 1; done` | 0 | 25 |
| AC-13 | verde | `cd rust && cargo test la_cita_tiene_que_apuntar_a_algo_que_existe 2>&1 \| grep -E "[1-9][0-9]* passed"` | 0 | 301 |
