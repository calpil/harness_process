//! feature_list.json: lectura/escritura intercambiable con harness.py.
//! serde_json con `preserve_order` mantiene el orden de claves; la escritura
//! es atomica (tmp + rename) como `os.replace`.

use std::io::Write;
use std::path::Path;

use anyhow::Context;
use serde_json::{Map, Value};

use crate::exit::Exit;
use crate::paths::HarnessPaths;
use crate::pycompat::{py_json_pretty, py_str};

pub fn load_features(paths: &HarnessPaths) -> anyhow::Result<Value> {
    if !paths.features.exists() {
        let mut map = Map::new();
        let project = paths
            .root
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        map.insert("project".to_string(), Value::String(project));
        map.insert("features".to_string(), Value::Array(Vec::new()));
        return Ok(Value::Object(map));
    }
    let text = std::fs::read_to_string(&paths.features)
        .with_context(|| format!("no se pudo leer {}", paths.features.display()))?;
    let value = serde_json::from_str(&text)
        .with_context(|| format!("JSON invalido en {}", paths.features.display()))?;
    Ok(value)
}

pub fn save_features(paths: &HarnessPaths, data: &Value) -> anyhow::Result<()> {
    write_text_atomic(&paths.features, &py_json_pretty(data)?)
}

/// Escritura atomica (tmp en el mismo directorio + rename). En Windows el
/// persist puede fallar transitorio por locks de AV/indexer: reintenta 3x.
pub fn write_text_atomic(path: &Path, text: &str) -> anyhow::Result<()> {
    let dir = path.parent().filter(|p| !p.as_os_str().is_empty());
    let dir = dir.unwrap_or_else(|| Path::new("."));
    let mut tmp = tempfile::NamedTempFile::new_in(dir)
        .with_context(|| format!("no se pudo crear tmp en {}", dir.display()))?;
    tmp.write_all(text.as_bytes())?;
    let mut tmp = tmp;
    let mut last_err = None;
    for _ in 0..3 {
        match tmp.persist(path) {
            Ok(_) => {
                // open() de Python crea 0644 (umask tipica); NamedTempFile crea 0600.
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o644));
                }
                return Ok(());
            }
            Err(err) => {
                tmp = err.file;
                last_err = Some(err.error);
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
        }
    }
    Err(anyhow::anyhow!(
        "no se pudo reemplazar {}: {}",
        path.display(),
        last_err.map(|e| e.to_string()).unwrap_or_default()
    ))
}

/// `data.get("features", [])` de solo lectura.
pub fn features_slice(data: &Value) -> &[Value] {
    data.get("features")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[])
}

pub fn feature_status(feature: &Value) -> Option<&str> {
    feature.get("status").and_then(Value::as_str)
}

/// Indices de features con status == "in_progress".
pub fn active_indices(data: &Value) -> Vec<usize> {
    features_slice(data)
        .iter()
        .enumerate()
        .filter(|(_, f)| feature_status(f) == Some("in_progress"))
        .map(|(i, _)| i)
        .collect()
}

/// `find_feature`: compara `str(feature["id"]) == str(fid)`.
pub fn find_feature_index(data: &Value, fid: &str) -> Result<usize, Exit> {
    for (i, feature) in features_slice(data).iter().enumerate() {
        if py_str(feature.get("id")) == fid {
            return Ok(i);
        }
    }
    Err(Exit::msg(format!("Feature no encontrada: {fid}")))
}

/// `active_feature`: la feature de --feature, o la unica in_progress.
pub fn active_feature_index(data: &Value, fid: Option<&str>) -> Result<usize, Exit> {
    if let Some(fid) = fid {
        return find_feature_index(data, fid);
    }
    let active = active_indices(data);
    match active.as_slice() {
        [] => Err(Exit::msg(
            "No hay feature in_progress. Inicia una: harness.py start --feature <id>",
        )),
        [single] => Ok(*single),
        many => {
            let ids = many
                .iter()
                .map(|&i| format!("#{}", py_str(features_slice(data)[i].get("id"))))
                .collect::<Vec<_>>()
                .join(", ");
            Err(Exit::msg(format!(
                "Varias features in_progress ({ids}); especifica --feature <id>."
            )))
        }
    }
}

/// Acceso mutable a una feature por indice (el array debe existir).
pub fn feature_mut(data: &mut Value, index: usize) -> anyhow::Result<&mut Map<String, Value>> {
    data.get_mut("features")
        .and_then(Value::as_array_mut)
        .and_then(|arr| arr.get_mut(index))
        .and_then(Value::as_object_mut)
        .context("feature_list.json: feature inaccesible")
}

pub fn feature_at(data: &Value, index: usize) -> &Value {
    &features_slice(data)[index]
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;
    use serde_json::json;

    #[test]
    fn find_feature_index_should_compare_ids_as_python_str() {
        let data = json!({"features": [{"id": 1}, {"id": "2"}]});
        assert_eq!(find_feature_index(&data, "1").unwrap(), 0);
        assert_eq!(find_feature_index(&data, "2").unwrap(), 1);
        let err = find_feature_index(&data, "9").unwrap_err();
        assert_eq!(err.code, 1);
        assert_eq!(err.message.as_deref(), Some("Feature no encontrada: 9"));
    }

    #[test]
    fn active_feature_index_should_fail_without_in_progress() {
        let data = json!({"features": [{"id": 1, "status": "pending"}]});
        let err = active_feature_index(&data, None).unwrap_err();
        assert!(err.message.unwrap().starts_with("No hay feature in_progress"));
    }

    #[test]
    fn active_feature_index_should_fail_with_multiple_in_progress() {
        let data = json!({"features": [
            {"id": 1, "status": "in_progress"},
            {"id": 2, "status": "in_progress"}
        ]});
        let err = active_feature_index(&data, None).unwrap_err();
        assert_eq!(
            err.message.unwrap(),
            "Varias features in_progress (#1, #2); especifica --feature <id>."
        );
    }

    #[test]
    fn save_features_should_preserve_unknown_fields_and_key_order() {
        let dir = tempfile::tempdir().unwrap();
        let paths = HarnessPaths::from_root(dir.path().to_path_buf());
        let data = json!({
            "project": "demo",
            "rules": {"one_feature_at_a_time": true},
            "features": [{"id": 1, "name": "x", "extra": [1, 2]}]
        });
        save_features(&paths, &data).unwrap();
        let text = std::fs::read_to_string(&paths.features).unwrap();
        let reread: Value = serde_json::from_str(&text).unwrap();
        assert_eq!(reread, data);
        let keys: Vec<&String> = reread.as_object().unwrap().keys().collect();
        assert_eq!(keys, ["project", "rules", "features"]);
    }
}
