//! `harness next` (paridad: harness.py cmd_next).

use crate::features::{feature_status, features_slice, load_features};
use crate::paths::HarnessPaths;
use crate::pycompat::py_json_pretty;

pub fn run(paths: &HarnessPaths) -> anyhow::Result<()> {
    let data = load_features(paths)?;
    for f in features_slice(&data) {
        if feature_status(f) == Some("pending") {
            println!("{}", py_json_pretty(f)?);
            return Ok(());
        }
    }
    println!("No hay features pending.");
    Ok(())
}
