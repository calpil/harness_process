//! `harness start --feature <id>` (paridad: harness.py cmd_start).

use serde_json::{Value, json};

use crate::exit::Exit;
use crate::features::{
    feature_mut, feature_status, features_slice, find_feature_index, load_features, save_features,
};
use crate::memories::update_memories;
use crate::paths::HarnessPaths;
use crate::plan::{update_plan_sig, write_plan};
use crate::progress::{log, now_stamp, touch_autocheck_stamp};
use crate::pycompat::{py_str, relpath};

pub fn run(paths: &HarnessPaths, fid: &str) -> anyhow::Result<()> {
    let mut data = load_features(paths)?;
    let idx = find_feature_index(&data, fid)?;
    let other_active: Vec<&Value> = features_slice(&data)
        .iter()
        .filter(|f| feature_status(f) == Some("in_progress") && py_str(f.get("id")) != fid)
        .collect();
    if let Some(first) = other_active.first() {
        return Err(Exit::msg(format!(
            "Ya hay feature in_progress: #{} {}",
            py_str(first.get("id")),
            py_str(first.get("name"))
        ))
        .into());
    }
    {
        let feature = feature_mut(&mut data, idx)?;
        feature.insert("status".to_string(), json!("in_progress"));
        feature.insert("started_at".to_string(), json!(now_stamp()));
    }
    save_features(paths, &data)?;
    let (rel_plan, feature_id, feature_name, services, meta_name) = {
        let feature = feature_mut(&mut data, idx)?;
        let plan = write_plan(paths, feature)?;
        let rel_plan = relpath(&plan, &paths.repo_root)
            .unwrap_or_else(|| plan.clone())
            .to_string_lossy()
            .into_owned();
        // Capturar firma del plan para detectar ediciones por otros LLMs
        update_plan_sig(paths, feature);
        let services: Vec<String> = feature
            .get("microservicios")
            .and_then(Value::as_array)
            .map(|a| a.iter().map(|s| py_str(Some(s))).collect())
            .unwrap_or_default();
        // meta del hub: feature.get("name", "") (default "", no "None")
        let meta_name = feature
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        (
            rel_plan,
            py_str(feature.get("id")),
            py_str(feature.get("name")),
            services,
            meta_name,
        )
    };
    save_features(paths, &data)?;
    std::fs::create_dir_all(&paths.progress)?;
    let mut current = format!("# Feature #{feature_id}: {feature_name}\n\n");
    current.push_str("Estado: in_progress\n");
    current.push_str(&format!("Plan: {rel_plan}\n\n"));
    current.push_str("Microservicios:\n");
    for service in &services {
        current.push_str(&format!("- {service}\n"));
    }
    current.push_str("\nEvidencia:\n- \n");
    std::fs::write(&paths.current, current)?;
    log(paths, &format!("start feature #{feature_id} {feature_name}"))?;
    update_memories(
        "start",
        "in_progress",
        &format!("feature-{feature_id}"),
        &meta_name,
        false,
        &paths.repo_root,
    );
    touch_autocheck_stamp(paths); // linea base: el plan recien creado no dispara autocheck
    println!("Feature #{feature_id} iniciada. Plan: {rel_plan}");
    println!("  (firma del plan registrada para deteccion de actualizaciones por otros agentes)");
    Ok(())
}
