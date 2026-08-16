//! `harness add` (paridad: harness.py cmd_add).

use serde_json::{Map, Value, json};

use crate::features::{features_slice, load_features, save_features};
use crate::paths::HarnessPaths;
use crate::prd;
use crate::progress::log;
use crate::pycompat::py_str;

pub fn run(
    paths: &HarnessPaths,
    name: &str,
    services: &[String],
    acceptance: &[String],
    prd_ref: Option<&str>,
) -> anyhow::Result<()> {
    // El PRD se resuelve ANTES de tocar el backlog: una referencia mala no deja
    // una feature a medio cargar.
    let prd_slug = match prd_ref {
        Some(reference) => Some(prd::resolve(paths, reference)?),
        None => None,
    };
    let mut data = load_features(paths)?;
    // Python: int(id) para los ids cuyo str() es puramente digito.
    let max_id = features_slice(&data)
        .iter()
        .filter_map(|f| {
            let s = py_str(f.get("id"));
            if !s.is_empty() && s.chars().all(|c| c.is_ascii_digit()) {
                s.parse::<i64>().ok()
            } else {
                None
            }
        })
        .max()
        .unwrap_or(0);
    let fid = max_id + 1;
    let mut feature = Map::new();
    feature.insert("id".to_string(), json!(fid));
    feature.insert("name".to_string(), json!(name));
    feature.insert(
        "microservicios".to_string(),
        Value::Array(services.iter().map(|s| json!(s)).collect()),
    );
    feature.insert(
        "acceptance".to_string(),
        Value::Array(acceptance.iter().map(|s| json!(s)).collect()),
    );
    feature.insert("status".to_string(), json!("pending"));
    // Campo OPCIONAL: sin --prd la feature se guarda exactamente como siempre.
    if let Some(target) = &prd_slug {
        feature.insert("prd".to_string(), json!(target.reference()));
    }
    // data.setdefault("features", []).append(feature)
    let Some(obj) = data.as_object_mut() else {
        anyhow::bail!("feature_list.json: raiz no es un objeto");
    };
    obj.entry("features")
        .or_insert_with(|| Value::Array(Vec::new()));
    // Copia para los enganches de Atlassian: el original se muda al backlog.
    let snapshot = feature.clone();
    if let Some(arr) = obj.get_mut("features").and_then(Value::as_array_mut) {
        arr.push(Value::Object(feature));
    }
    save_features(paths, &data)?;
    log(paths, &format!("add feature #{fid} {name}"))?;
    // Feature #15: el PRD nace como epic y la feature como historia (AC-6).
    crate::atlassian::emit::on_add(paths, &snapshot);
    println!("Feature #{fid} agregada.");
    if let Some(target) = &prd_slug {
        println!("  PRD de origen: {}", prd::rel_path(&target.slug));
    }
    Ok(())
}
