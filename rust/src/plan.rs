//! Planes por feature y firmas anti-conflicto multi-LLM (paridad con
//! harness.py lineas 72-195). Los mensajes son VERBATIM: los agentes y los
//! scripts de hook los leen tal cual.

use std::path::{Path, PathBuf};

use serde_json::{Map, Number, Value};
use sha2::{Digest, Sha256};

use crate::paths::HarnessPaths;
use crate::pycompat::{mtime_f64, py_str, relpath};

pub fn slugify(text: &str) -> String {
    static RE: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
        #[allow(clippy::unwrap_used)] // patron constante validado por los tests
        regex::Regex::new(r"[^a-z0-9]+").unwrap()
    });
    let lower = text.to_lowercase();
    let s = RE.replace_all(&lower, "-");
    let s = s.trim_matches('-');
    let s = if s.is_empty() { "feature" } else { s };
    s.chars().take(48).collect()
}

pub fn plan_path(paths: &HarnessPaths, feature: &Map<String, Value>) -> PathBuf {
    let id = py_str(feature.get("id"));
    let name = feature
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or_default();
    paths
        .plans
        .join(format!("plan-feature-{id}-{}.md", slugify(name)))
}

/// Firma ligera del plan: mtime + sha256[:16] del contenido. Orden de claves
/// identico al dict de Python: path, mtime, size, hash.
pub fn plan_signature(path: &Path, repo_root: &Path) -> Option<Map<String, Value>> {
    if !path.exists() {
        return None;
    }
    let mtime = mtime_f64(path).ok()?;
    let content = std::fs::read(path).ok()?;
    let digest = hex::encode(Sha256::digest(&content));
    let hash = digest.get(..16)?.to_string();
    let rel = relpath(path, repo_root)?;
    let mut sig = Map::new();
    sig.insert(
        "path".to_string(),
        Value::String(rel.to_string_lossy().into_owned()),
    );
    sig.insert("mtime".to_string(), Value::Number(Number::from_f64(mtime)?));
    sig.insert("size".to_string(), Value::Number(Number::from(content.len())));
    sig.insert("hash".to_string(), Value::String(hash));
    Some(sig)
}

/// `feature.get("last_plan_sig")` si es dict.
pub fn get_plan_sig(feature: &Map<String, Value>) -> Option<&Map<String, Value>> {
    feature.get("last_plan_sig").and_then(Value::as_object)
}

/// Calcula y persiste la firma actual del plan dentro de la feature.
pub fn update_plan_sig(paths: &HarnessPaths, feature: &mut Map<String, Value>) {
    let path = plan_path(paths, feature);
    if let Some(sig) = plan_signature(&path, &paths.repo_root) {
        feature.insert("last_plan_sig".to_string(), Value::Object(sig));
    }
}

fn sig_mtime(sig: &Map<String, Value>) -> f64 {
    sig.get("mtime").and_then(Value::as_f64).unwrap_or(0.0)
}

pub fn is_plan_stale(paths: &HarnessPaths, feature: &Map<String, Value>) -> bool {
    let current = plan_signature(&plan_path(paths, feature), &paths.repo_root);
    let (Some(current), Some(last)) = (current, get_plan_sig(feature)) else {
        return false;
    };
    current.get("hash") != last.get("hash")
        || (sig_mtime(&current) - sig_mtime(last)).abs() > 1.0
}

pub fn plan_staleness_message(paths: &HarnessPaths, feature: &Map<String, Value>) -> String {
    let path = plan_path(paths, feature);
    let current = plan_signature(&path, &paths.repo_root);
    let last = get_plan_sig(feature);
    let Some(current) = current else {
        return format!("[!] No se pudo leer el plan actual: {}", path.display());
    };
    if last.is_none() {
        return "[!] Plan sin firma previa. Ejecuta harness.py check-plan despues de start/advance."
            .to_string();
    }
    if is_plan_stale(paths, feature) {
        let last = last.unwrap_or(&current);
        return format!(
            "[!] PLAN ACTUALIZADO POR OTRO LLM (Claude/Gemini/Antigravity/Grok/Codex/etc.)\n    Plan en disco: {} (mtime={:.0}, hash={})\n    Ultima firma conocida: mtime={:.0}, hash={}\n    Accion requerida: Re-lee COMPLETAMENTE el plan actualizado en docs/.\n    Luego confirma con: python3 harness.py check-plan  (debe salir limpio)\n    Registra la re-sincronizacion: python3 harness.py advance --nota \"Re-sincronizado con plan actualizado por otro agente\"",
            py_str(current.get("path")),
            sig_mtime(&current),
            py_str(current.get("hash")),
            sig_mtime(last),
            py_str(last.get("hash")),
        );
    }
    "Plan fresco (sin cambios desde la ultima firma registrada).".to_string()
}

pub fn plan_template(feature: &Map<String, Value>) -> String {
    let services: Vec<String> = feature
        .get("microservicios")
        .and_then(Value::as_array)
        .filter(|a| !a.is_empty())
        .map(|a| a.iter().map(|s| py_str(Some(s))).collect())
        .unwrap_or_else(|| vec!["(sin servicios)".to_string()]);
    let mut lines: Vec<String> = vec![
        format!(
            "# Plan - Feature #{}: {}",
            py_str(feature.get("id")),
            py_str(feature.get("name"))
        ),
        String::new(),
        "Estado: in_progress".to_string(),
        "Microservicios:".to_string(),
    ];
    lines.extend(services.iter().map(|s| format!("- {s}")));
    for tail in [
        "",
        "## Alcance",
        "",
        "## Impacto entre microservicios",
        "<!-- sh harness_cli graph impacto --microservicio <proyecto>/<servicio> -->",
        "",
        "## Consulta al grafo (graphify)",
        "<!-- graphify query \"<pregunta de la task>\" -->",
        "",
        "## Delegacion (implementer)",
        "- ",
        "",
        "## Criterios de cierre (reviewer)",
        "- ",
        "",
        "## Riesgos",
        "- ",
        "",
        "## Observaciones (decisiones pendientes)",
        "<!-- Una observacion por linea. Si hay observaciones SIN decision, el",
        "     implementer DEBE preguntar al usuario que decision aplicar ANTES de",
        "     implementar ese feat/fase/tarea, y registrar aqui la respuesta. -->",
        "- ",
        "",
    ] {
        lines.push(tail.to_string());
    }
    lines.join("\n")
}

/// Crea el plan en docs/ de la raiz si no existe (no pisa el del lider).
pub fn write_plan(
    paths: &HarnessPaths,
    feature: &Map<String, Value>,
) -> anyhow::Result<PathBuf> {
    std::fs::create_dir_all(&paths.plans)?;
    let path = plan_path(paths, feature);
    if !path.exists() {
        std::fs::write(&path, plan_template(feature))?;
    }
    Ok(path)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;
    use serde_json::json;

    fn feature(id: i64, name: &str) -> Map<String, Value> {
        let mut f = Map::new();
        f.insert("id".to_string(), json!(id));
        f.insert("name".to_string(), json!(name));
        f
    }

    fn paths_in(dir: &Path) -> HarnessPaths {
        let harness = dir.join("hp");
        std::fs::create_dir_all(&harness).unwrap();
        std::fs::write(harness.join(".harness_layout"), "subdir").unwrap();
        HarnessPaths::from_root(harness)
    }

    #[test]
    fn slugify_should_match_python_examples() {
        assert_eq!(slugify("Pago con QR (v2)"), "pago-con-qr-v2");
        assert_eq!(slugify("---"), "feature");
        assert_eq!(slugify(""), "feature");
        let long = "x".repeat(60);
        assert_eq!(slugify(&long).len(), 48);
    }

    #[test]
    fn plan_signature_should_have_python_key_order() {
        let dir = tempfile::tempdir().unwrap();
        let plan = dir.path().join("plan.md");
        std::fs::write(&plan, "contenido").unwrap();
        let sig = plan_signature(&plan, dir.path()).unwrap();
        let keys: Vec<&String> = sig.keys().collect();
        assert_eq!(keys, ["path", "mtime", "size", "hash"]);
        assert_eq!(sig.get("size"), Some(&json!(9)));
        assert_eq!(sig.get("path"), Some(&json!("plan.md")));
        assert_eq!(
            sig.get("hash").and_then(Value::as_str).map(str::len),
            Some(16)
        );
    }

    #[test]
    fn is_plan_stale_should_tolerate_one_second_mtime_drift() {
        let dir = tempfile::tempdir().unwrap();
        let paths = paths_in(dir.path());
        let mut f = feature(1, "demo");
        std::fs::create_dir_all(&paths.plans).unwrap();
        std::fs::write(plan_path(&paths, &f), "plan").unwrap();
        update_plan_sig(&paths, &mut f);
        assert!(!is_plan_stale(&paths, &f));
        // misma huella pero mtime corrido < 1s no es stale
        if let Some(Value::Object(sig)) = f.get_mut("last_plan_sig") {
            let m = sig.get("mtime").and_then(Value::as_f64).unwrap();
            sig.insert("mtime".into(), json!(m + 0.5));
        }
        assert!(!is_plan_stale(&paths, &f));
        // contenido distinto -> stale
        std::fs::write(plan_path(&paths, &f), "plan editado por otro LLM").unwrap();
        assert!(is_plan_stale(&paths, &f));
    }

    #[test]
    fn plan_template_should_end_like_python_join() {
        let f = feature(2, "Pago");
        let t = plan_template(&f);
        assert!(t.starts_with("# Plan - Feature #2: Pago\n\nEstado: in_progress\nMicroservicios:\n- (sin servicios)\n"));
        assert!(t.contains("## Riesgos\n- \n"));
        assert!(t.contains("## Observaciones (decisiones pendientes)"));
        assert!(t.ends_with("- \n"));
    }
}
