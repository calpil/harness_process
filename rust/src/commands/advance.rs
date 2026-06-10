//! `harness advance --nota "..."` (paridad: harness.py cmd_advance).

use std::io::Write;

use crate::exit::Exit;
use crate::features::{active_feature_index, feature_mut, load_features, save_features};
use crate::memories::update_memories;
use crate::paths::HarnessPaths;
use crate::plan::{plan_path, update_plan_sig};
use crate::progress::{log, now_stamp, touch_autocheck_stamp};
use crate::pycompat::py_str;

pub fn run(
    paths: &HarnessPaths,
    fid: Option<&str>,
    nota: &str,
    no_graphify: bool,
) -> anyhow::Result<()> {
    let mut data = load_features(paths)?;
    let idx = active_feature_index(&data, fid)?;
    let (feature_id, plan) = {
        let feature = feature_mut(&mut data, idx)?;
        let status = feature.get("status").and_then(|v| v.as_str());
        if status != Some("in_progress") {
            return Err(Exit::msg(format!(
                "Feature #{} no esta in_progress (status={}); usa start.",
                py_str(feature.get("id")),
                py_str(feature.get("status"))
            ))
            .into());
        }
        (py_str(feature.get("id")), plan_path(paths, feature))
    };
    let stamp = now_stamp();
    // 1) Plan: deja rastro del hito en el cuerpo del plan (append, no pisa).
    if plan.exists() {
        let mut f = std::fs::OpenOptions::new().append(true).open(&plan)?;
        write!(f, "\n### Avance {stamp}\n{nota}\n")?;
    }
    // Actualizar firma del plan (por si el avance modifico el archivo o el
    // lider edito entre turnos)
    {
        let feature = feature_mut(&mut data, idx)?;
        update_plan_sig(paths, feature);
    }
    save_features(paths, &data)?;
    // 2) current.md: suma el avance a la evidencia (append, no reescribe).
    if paths.current.exists() {
        let mut f = std::fs::OpenOptions::new()
            .append(true)
            .open(&paths.current)?;
        writeln!(f, "- {stamp} {nota}")?;
    }
    // 3) history.md: una linea append-only.
    log(paths, &format!("advance feature #{feature_id} {nota}"))?;
    // 4) Memorias: hub (in_progress, con la nota) + graphify (best-effort).
    update_memories(
        "advance",
        "in_progress",
        &format!("feature-{feature_id}"),
        nota,
        !no_graphify,
        &paths.repo_root,
    );
    touch_autocheck_stamp(paths); // un advance manual tambien resetea la linea base
    let extra = if no_graphify { "" } else { " (hub + graphify)" };
    println!("Avance registrado en feature #{feature_id}{extra}.");
    Ok(())
}
