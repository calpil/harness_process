//! `harness autocheck` (paridad: harness.py cmd_autocheck). Checkpoint
//! automatico de los hooks: silencioso, idempotente, best-effort absoluto.

use std::collections::BTreeSet;

use crate::features::{active_indices, feature_mut, load_features, save_features};
use crate::graphify;
use crate::memories::hub_register;
use crate::paths::HarnessPaths;
use crate::plan::update_plan_sig;
use crate::progress::{log, touch_autocheck_stamp};
use crate::pycompat::{mtime_f64, py_str};

pub fn run(paths: &HarnessPaths, no_graphify: bool) -> anyhow::Result<()> {
    if let Err(exc) = inner(paths, no_graphify) {
        // Best-effort absoluto: corre al cierre de cada turno; nunca abortes.
        println!("[autocheck] omitido: {exc}");
    }
    Ok(())
}

fn inner(paths: &HarnessPaths, no_graphify: bool) -> anyhow::Result<()> {
    let mut data = load_features(paths)?;
    let active = active_indices(&data);
    if active.len() != 1 {
        return Ok(());
    }
    let idx = active[0];
    let feature_id = {
        let feature = feature_mut(&mut data, idx)?;
        py_str(feature.get("id"))
    };
    let last = mtime_f64(&paths.autocheck_stamp).unwrap_or(0.0);
    // Vigila lo que el agente REALMENTE mantiene: el estado vivo (current.md)
    // y CUALQUIER doc del proyecto (docs/*.md).
    let mut watched: Vec<std::path::PathBuf> = Vec::new();
    if paths.current.exists() {
        watched.push(paths.current.clone());
    }
    if paths.plans.is_dir() {
        for entry in std::fs::read_dir(&paths.plans)? {
            let entry = entry?;
            let name = entry.file_name();
            if name.to_string_lossy().ends_with(".md") {
                watched.push(paths.plans.join(name));
            }
        }
    }
    let mut changed: BTreeSet<String> = BTreeSet::new();
    for p in &watched {
        if mtime_f64(p)? > last {
            if let Some(base) = p.file_name() {
                changed.insert(base.to_string_lossy().into_owned());
            }
        }
    }
    if changed.is_empty() {
        return Ok(());
    }
    let nota = format!(
        "auto: {}",
        changed.iter().cloned().collect::<Vec<_>>().join(", ")
    );
    log(paths, &format!("autocheck feature #{feature_id} {nota}"))?;
    hub_register("advance", "in_progress", &format!("feature-{feature_id}"), &nota);
    // Si el plan fue uno de los cambiados, refresca su firma para que futuros
    // implementers vean que fue actualizado (posiblemente por otro LLM).
    if changed
        .iter()
        .any(|c| c.contains("plan-feature") || c.ends_with(".md"))
    {
        let feature = feature_mut(&mut data, idx)?;
        update_plan_sig(paths, feature);
        save_features(paths, &data)?;
    }
    if !no_graphify {
        graphify::refresh_bg(&paths.repo_root);
    }
    touch_autocheck_stamp(paths);
    println!("[autocheck] avance auto en feature #{feature_id}: {nota}");
    Ok(())
}
