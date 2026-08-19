//! `harness status` (paridad: harness.py cmd_status).

use serde_json::Value;

use crate::features::{active_indices, feature_status, features_slice, load_features};
use crate::paths::HarnessPaths;
use crate::plan::{get_plan_sig, is_plan_stale};
use crate::pycompat::py_str;
use crate::spec::{is_spec_stale, spec_state};

pub fn run(paths: &HarnessPaths) -> anyhow::Result<()> {
    let data = load_features(paths)?;
    let features = features_slice(&data);
    let count = |status: &str| {
        features
            .iter()
            .filter(|f| feature_status(f) == Some(status))
            .count()
    };
    println!(
        "Backlog: {} feature(s) | active={} pending={} blocked={} done={}",
        features.len(),
        count("in_progress"),
        count("pending"),
        count("blocked"),
        count("done")
    );
    for f in features {
        let services = f
            .get("microservicios")
            .and_then(Value::as_array)
            .map(|a| {
                a.iter()
                    .map(|s| s.as_str().unwrap_or_default())
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .unwrap_or_default();
        let services = if services.is_empty() {
            "sin servicios".to_string()
        } else {
            services
        };
        // `superseded` se lee con quien la absorbio: sin eso, el estado no
        // dice nada mas que `blocked` (feature #37).
        let estado = match f.get("superseded_by").and_then(Value::as_str) {
            Some(por) if py_str(f.get("status")) == "superseded" => {
                format!("superseded por #{por}")
            }
            _ => py_str(f.get("status")),
        };
        println!(
            "  #{} [{estado}] {} ({services})",
            py_str(f.get("id")),
            py_str(f.get("name"))
        );
    }
    if paths.current.exists() {
        let content = std::fs::read_to_string(&paths.current)?;
        let content = content.trim();
        if !content.is_empty() {
            println!("\nprogress/current.md:");
            println!("{content}");
        }
    }

    // Reporte de frescura de planes (importante para multi-LLM)
    for idx in active_indices(&data) {
        let Some(f) = features[idx].as_object() else {
            continue;
        };
        if get_plan_sig(f).is_some() {
            if is_plan_stale(paths, f) {
                println!(
                    "  [!] #{} PLAN STALE - actualizado por otro agente/LLM. Ejecuta: harness.py check-plan",
                    py_str(f.get("id"))
                );
            } else {
                println!("  [plan] #{} fresco", py_str(f.get("id")));
            }
        }
        // Estado del spec SDD (draft/approved/ausente) + frescura multi-LLM.
        let spec_fresh = if is_spec_stale(paths, f) { "STALE" } else { "fresco" };
        println!(
            "  [spec] #{} {} ({spec_fresh})",
            py_str(f.get("id")),
            spec_state(paths, f).label()
        );
    }
    Ok(())
}
