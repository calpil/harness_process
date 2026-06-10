//! `harness status` (paridad: harness.py cmd_status).

use serde_json::Value;

use crate::features::{active_indices, feature_status, features_slice, load_features};
use crate::paths::HarnessPaths;
use crate::plan::{get_plan_sig, is_plan_stale};
use crate::pycompat::py_str;

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
        println!(
            "  #{} [{}] {} ({services})",
            py_str(f.get("id")),
            py_str(f.get("status")),
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
    }
    Ok(())
}
