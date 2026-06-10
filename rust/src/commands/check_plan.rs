//! `harness check-plan` (paridad: harness.py cmd_check_plan).
//! Exit codes: 0 = fresco / no aplica; 1 = sin feature in_progress;
//! 2 = plan genuinamente stale (gate para harness_check.sh y hooks).

use crate::exit::Exit;
use crate::features::{active_feature_index, feature_at, load_features};
use crate::paths::HarnessPaths;
use crate::plan::{is_plan_stale, plan_staleness_message};

pub fn run(paths: &HarnessPaths, feature: Option<&str>) -> anyhow::Result<()> {
    let data = load_features(paths)?;
    let idx = active_feature_index(&data, feature)?;
    let Some(feature) = feature_at(&data, idx).as_object() else {
        anyhow::bail!("feature_list.json: feature invalida");
    };
    let stale = is_plan_stale(paths, feature);
    println!("{}", plan_staleness_message(paths, feature));
    if stale {
        // Codigo de error para que harness_check.sh y hooks lo usen como gate.
        return Err(Exit::code(2).into());
    }
    println!("[OK] Plan fresco para implementacion.");
    Ok(())
}
