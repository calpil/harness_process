//! `harness check-plan` (paridad: harness.py cmd_check_plan + vigilancia SDD).
//! Exit codes: 0 = fresco / no aplica; 1 = sin feature in_progress;
//! 2 = plan O spec genuinamente stale (gate para harness_check.sh y hooks);
//! el stdout distingue cual de los dos esta desactualizado.

use crate::exit::Exit;
use crate::features::{active_feature_index_con_foco, feature_at, load_features};
use crate::paths::HarnessPaths;
use crate::plan::{is_plan_stale, plan_staleness_message};
use crate::spec::{is_spec_stale, spec_staleness_message, spec_state};

pub fn run(paths: &HarnessPaths, feature: Option<&str>) -> anyhow::Result<()> {
    let data = load_features(paths)?;
    let idx = active_feature_index_con_foco(paths, &data, feature)?;
    // Feature #47: los docs (spec, plan, evidencia) viven en el worktree de la
    // feature, no en el directorio desde el que se corre el comando.
    let paths = &match feature_at(&data, idx).as_object() {
        Some(f) => paths.para_feature(f),
        None => paths.para_feature(&serde_json::Map::new()),
    };
    let Some(feature) = feature_at(&data, idx).as_object() else {
        anyhow::bail!("feature_list.json: feature invalida");
    };
    let plan_stale = is_plan_stale(paths, feature);
    println!("{}", plan_staleness_message(paths, feature));
    // Vigilancia del spec SDD: misma mecanica de firma que el plan.
    let spec_stale = is_spec_stale(paths, feature);
    println!("{}", spec_staleness_message(paths, feature));
    println!("[spec] Estado: {}", spec_state(paths, feature).label());
    if plan_stale || spec_stale {
        // Codigo de error para que harness_check.sh y hooks lo usen como gate.
        return Err(Exit::code(2).into());
    }
    println!("[OK] Plan fresco para implementacion.");
    Ok(())
}
