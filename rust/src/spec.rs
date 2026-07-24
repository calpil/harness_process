//! Specs por feature (Spec-Driven Development) y gate de aprobacion.
//! Espejo de plan.rs: la firma anti-conflicto multi-LLM REUSA
//! `plan::plan_signature` (mismo dict path/mtime/size/hash) sobre la clave
//! `last_spec_sig`. Solo el USUARIO aprueba un spec (draft -> approved);
//! los agentes tienen PROHIBIDO auto-aprobarlo.

use std::path::PathBuf;

use serde_json::{Map, Value};

use crate::exit::Exit;
use crate::paths::HarnessPaths;
use crate::plan::{plan_signature, sig_mtime, slugify};
use crate::pycompat::{py_str, relpath};

pub fn spec_path(paths: &HarnessPaths, feature: &Map<String, Value>) -> PathBuf {
    let id = py_str(feature.get("id"));
    let name = feature
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or_default();
    paths
        .plans
        .join(format!("spec-feature-{id}-{}.md", slugify(name)))
}

pub fn spec_template(feature: &Map<String, Value>) -> String {
    let id = py_str(feature.get("id"));
    let name = feature
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let mut lines: Vec<String> = vec![
        format!("# Spec - Feature #{id}: {}", py_str(feature.get("name"))),
        String::new(),
        "Estado: draft".to_string(),
        format!("Plan: docs/plan-feature-{id}-{}.md", slugify(name)),
        "Constitution: docs/constitution.md".to_string(),
    ];
    for tail in [
        "",
        "## Recorridos de usuario (priorizados)",
        "<!-- P1 imprescindible, P2 importante. Cada recorrido independientemente testeable. -->",
        "- P1: Como <rol>, quiero <accion>, para <resultado>.",
        "",
        "## Criterios de aceptacion (Given/When/Then)",
        "<!-- La Delegacion del plan cita estos IDs (AC-1, AC-2, ...); el reviewer exige evidencia por AC. -->",
        "- AC-1: Given <contexto>, When <accion>, Then <resultado observable>.",
        "",
        "## No funcionales",
        "- SLOs:",
        "- Seguridad:",
        "- Observabilidad:",
        "",
        "## Fuera de alcance",
        "-",
        "",
        "## Observaciones (decisiones pendientes)",
        "<!-- Mismo protocolo que el plan: si hay observaciones SIN decision, el",
        "     implementer DEBE preguntar al usuario ANTES de implementar. -->",
        "-",
        "",
    ] {
        lines.push(tail.to_string());
    }
    lines.join("\n")
}

/// Crea el spec (draft) en docs/ de la raiz si no existe (no pisa el editado
/// por el lider o aprobado por el usuario).
pub fn write_spec(
    paths: &HarnessPaths,
    feature: &Map<String, Value>,
) -> anyhow::Result<PathBuf> {
    std::fs::create_dir_all(&paths.plans)?;
    let path = spec_path(paths, feature);
    if !path.exists() {
        std::fs::write(&path, spec_template(feature))?;
    }
    Ok(path)
}

/// `feature.get("last_spec_sig")` si es dict.
pub fn get_spec_sig(feature: &Map<String, Value>) -> Option<&Map<String, Value>> {
    feature.get("last_spec_sig").and_then(Value::as_object)
}

/// Calcula y persiste la firma actual del spec dentro de la feature
/// (reusa plan_signature: mismo hashing y orden de claves).
pub fn update_spec_sig(paths: &HarnessPaths, feature: &mut Map<String, Value>) {
    let path = spec_path(paths, feature);
    if let Some(sig) = plan_signature(&path, &paths.repo_root) {
        feature.insert("last_spec_sig".to_string(), Value::Object(sig));
    }
}

/// Mismas tolerancias que is_plan_stale: hash distinto o drift de mtime > 1s.
/// Falso si falta el archivo o no hay firma previa (compat features sin spec).
pub fn is_spec_stale(paths: &HarnessPaths, feature: &Map<String, Value>) -> bool {
    let current = plan_signature(&spec_path(paths, feature), &paths.repo_root);
    let (Some(current), Some(last)) = (current, get_spec_sig(feature)) else {
        return false;
    };
    current.get("hash") != last.get("hash")
        || (sig_mtime(&current) - sig_mtime(last)).abs() > 1.0
}

pub fn spec_staleness_message(paths: &HarnessPaths, feature: &Map<String, Value>) -> String {
    let path = spec_path(paths, feature);
    let current = plan_signature(&path, &paths.repo_root);
    let last = get_spec_sig(feature);
    let Some(current) = current else {
        return format!("[!] No se pudo leer el spec actual: {}", path.display());
    };
    if last.is_none() {
        return "[!] Spec sin firma previa. Ejecuta sh harness_cli check-plan despues de start/advance."
            .to_string();
    }
    if is_spec_stale(paths, feature) {
        let last = last.unwrap_or(&current);
        return format!(
            "[!] SPEC ACTUALIZADO POR OTRO LLM (Claude/Gemini/Antigravity/Grok/Codex/etc.)\n    Spec en disco: {} (mtime={:.0}, hash={})\n    Ultima firma conocida: mtime={:.0}, hash={}\n    Accion requerida: Re-lee COMPLETAMENTE el spec actualizado en docs/.\n    Luego confirma con: sh harness_cli check-plan  (debe salir limpio)\n    Registra la re-sincronizacion: sh harness_cli advance --nota \"Re-sincronizado con spec actualizado por otro agente\"",
            py_str(current.get("path")),
            sig_mtime(&current),
            py_str(current.get("hash")),
            sig_mtime(last),
            py_str(last.get("hash")),
        );
    }
    "Spec fresco (sin cambios desde la ultima firma registrada).".to_string()
}

/// Estado declarado del spec. La aprobacion (draft -> approved) es EXCLUSIVA
/// del usuario; ningun agente puede editar la linea `Estado:`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpecState {
    /// No existe el archivo del spec.
    Missing,
    /// `Estado: draft` (pendiente de aprobacion del usuario).
    Draft,
    /// `Estado: approved` (el usuario autorizo implementar).
    Approved,
    /// Existe pero sin `Estado:` reconocible en las primeras 10 lineas.
    Other,
}

impl SpecState {
    pub fn label(self) -> &'static str {
        match self {
            SpecState::Missing => "ausente",
            SpecState::Draft => "draft",
            SpecState::Approved => "approved",
            SpecState::Other => "desconocido",
        }
    }
}

/// Deteccion (decidida por el lider en el plan): la PRIMERA linea cuyo trim
/// empiece con `Estado:` dentro de las primeras 10 lineas define el estado;
/// el valor es el resto tras `Estado:`, con trim y case-insensitive.
pub fn spec_state(paths: &HarnessPaths, feature: &Map<String, Value>) -> SpecState {
    let Ok(content) = std::fs::read_to_string(spec_path(paths, feature)) else {
        return SpecState::Missing;
    };
    for line in content.lines().take(10) {
        if let Some(value) = line.trim().strip_prefix("Estado:") {
            let value = value.trim();
            if value.eq_ignore_ascii_case("approved") {
                return SpecState::Approved;
            }
            if value.eq_ignore_ascii_case("draft") {
                return SpecState::Draft;
            }
            return SpecState::Other;
        }
    }
    SpecState::Other
}

/// Lee `rules.require_spec_approved` (default false: gate apagado para
/// instalaciones previas y features #1/#2).
pub fn require_spec_approved(data: &Value) -> bool {
    data.get("rules")
        .and_then(|r| r.get("require_spec_approved"))
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

/// El gate de close solo aplica al cierre "done"; blocked/pending son la
/// valvula de escape para abortar/aparcar sin spec aprobado.
pub fn close_requires_spec(status: &str) -> bool {
    status == "done"
}

/// Gate duro SDD: con la regla activa, cualquier estado != approved bloquea.
pub fn spec_gate(
    paths: &HarnessPaths,
    data: &Value,
    feature: &Map<String, Value>,
) -> Result<(), Exit> {
    if !require_spec_approved(data) {
        return Ok(());
    }
    let state = spec_state(paths, feature);
    if state == SpecState::Approved {
        return Ok(());
    }
    let path = spec_path(paths, feature);
    let rel = relpath(&path, &paths.repo_root).unwrap_or_else(|| path.clone());
    Err(Exit::msg(format!(
        "[GATE] Spec sin aprobar: {} (estado: {}).\n    La regla require_spec_approved esta activa: completa el spec y pide al usuario\n    aprobarlo editando `Estado: approved` (solo el usuario aprueba; los agentes no).",
        rel.display(),
        state.label()
    )))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;
    use serde_json::json;
    use std::path::Path;

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
    fn spec_template_should_declare_draft_and_sections() {
        let f = feature(3, "Pago QR");
        let t = spec_template(&f);
        assert!(t.starts_with(
            "# Spec - Feature #3: Pago QR\n\nEstado: draft\nPlan: docs/plan-feature-3-pago-qr.md\nConstitution: docs/constitution.md\n"
        ));
        assert!(t.contains("## Recorridos de usuario (priorizados)"));
        assert!(t.contains("- P1: Como <rol>, quiero <accion>, para <resultado>."));
        assert!(t.contains("## Criterios de aceptacion (Given/When/Then)"));
        assert!(t.contains("- AC-1: Given <contexto>, When <accion>, Then <resultado observable>."));
        assert!(t.contains("## No funcionales\n- SLOs:\n- Seguridad:\n- Observabilidad:\n"));
        assert!(t.contains("## Fuera de alcance"));
        assert!(t.contains("## Observaciones (decisiones pendientes)"));
        assert!(t.ends_with("-\n"));
    }

    #[test]
    fn spec_path_should_live_flat_next_to_plans() {
        let dir = tempfile::tempdir().unwrap();
        let paths = paths_in(dir.path());
        let f = feature(3, "Pago con QR (v2)");
        assert_eq!(
            spec_path(&paths, &f),
            dir.path().join("docs").join("spec-feature-3-pago-con-qr-v2.md")
        );
    }

    #[test]
    fn spec_state_should_detect_draft_approved_and_missing() {
        let dir = tempfile::tempdir().unwrap();
        let paths = paths_in(dir.path());
        let f = feature(1, "demo");
        // ausente: no existe el archivo
        assert_eq!(spec_state(&paths, &f), SpecState::Missing);
        // la plantilla nace draft (Estado: en la linea 3)
        write_spec(&paths, &f).unwrap();
        assert_eq!(spec_state(&paths, &f), SpecState::Draft);
        // aprobacion del usuario en la linea 3, valor case-insensitive + trim
        let p = spec_path(&paths, &f);
        let approved = std::fs::read_to_string(&p)
            .unwrap()
            .replace("Estado: draft", "Estado:   APPROVED  ");
        std::fs::write(&p, approved).unwrap();
        assert_eq!(spec_state(&paths, &f), SpecState::Approved);
        // valor no reconocido => desconocido (no aprobado)
        std::fs::write(&p, "# Spec\nEstado: en revision\n").unwrap();
        assert_eq!(spec_state(&paths, &f), SpecState::Other);
    }

    #[test]
    fn spec_state_should_ignore_estado_after_line_ten() {
        let dir = tempfile::tempdir().unwrap();
        let paths = paths_in(dir.path());
        let f = feature(1, "demo");
        let mut body = String::new();
        for _ in 0..10 {
            body.push_str("relleno\n");
        }
        body.push_str("Estado: approved\n"); // linea 11: fuera de la ventana
        std::fs::create_dir_all(&paths.plans).unwrap();
        std::fs::write(spec_path(&paths, &f), body).unwrap();
        assert_eq!(spec_state(&paths, &f), SpecState::Other);
        // la PRIMERA linea Estado: dentro de la ventana manda (trim del margen)
        std::fs::write(
            spec_path(&paths, &f),
            "# Spec\n  Estado: draft\nEstado: approved\n",
        )
        .unwrap();
        assert_eq!(spec_state(&paths, &f), SpecState::Draft);
    }

    #[test]
    fn is_spec_stale_should_tolerate_one_second_mtime_drift() {
        let dir = tempfile::tempdir().unwrap();
        let paths = paths_in(dir.path());
        let mut f = feature(1, "demo");
        write_spec(&paths, &f).unwrap();
        update_spec_sig(&paths, &mut f);
        assert!(!is_spec_stale(&paths, &f));
        // misma huella pero mtime corrido < 1s no es stale
        if let Some(Value::Object(sig)) = f.get_mut("last_spec_sig") {
            let m = sig.get("mtime").and_then(Value::as_f64).unwrap();
            sig.insert("mtime".into(), json!(m + 0.5));
        }
        assert!(!is_spec_stale(&paths, &f));
        // contenido distinto -> stale
        std::fs::write(spec_path(&paths, &f), "spec editado por otro LLM").unwrap();
        assert!(is_spec_stale(&paths, &f));
    }

    #[test]
    fn is_spec_stale_should_be_false_without_file_or_signature() {
        let dir = tempfile::tempdir().unwrap();
        let paths = paths_in(dir.path());
        let mut f = feature(1, "demo");
        // sin archivo ni firma (features #1/#2 done): nunca stale
        assert!(!is_spec_stale(&paths, &f));
        // archivo sin firma previa tampoco
        write_spec(&paths, &f).unwrap();
        assert!(!is_spec_stale(&paths, &f));
        // firma previa pero archivo borrado: sin firma actual -> falso
        update_spec_sig(&paths, &mut f);
        std::fs::remove_file(spec_path(&paths, &f)).unwrap();
        assert!(!is_spec_stale(&paths, &f));
    }

    #[test]
    fn update_spec_sig_should_store_python_key_order() {
        let dir = tempfile::tempdir().unwrap();
        let paths = paths_in(dir.path());
        let mut f = feature(1, "demo");
        write_spec(&paths, &f).unwrap();
        update_spec_sig(&paths, &mut f);
        let sig = get_spec_sig(&f).unwrap();
        let keys: Vec<&String> = sig.keys().collect();
        assert_eq!(keys, ["path", "mtime", "size", "hash"]);
        assert_eq!(sig.get("path"), Some(&json!("docs/spec-feature-1-demo.md")));
        assert_eq!(
            sig.get("hash").and_then(Value::as_str).map(str::len),
            Some(16)
        );
    }

    #[test]
    fn spec_gate_should_pass_when_rule_is_off() {
        let dir = tempfile::tempdir().unwrap();
        let paths = paths_in(dir.path());
        let f = feature(1, "demo"); // sin spec en disco
        // regla ausente (compat #1/#2 e instalaciones previas): advance/close pasan
        let data = json!({"features": [{"id": 1}]});
        assert!(spec_gate(&paths, &data, &f).is_ok());
        // regla explicita en false: idem
        let data = json!({"rules": {"require_spec_approved": false}});
        assert!(spec_gate(&paths, &data, &f).is_ok());
    }

    #[test]
    fn spec_gate_should_block_draft_or_missing_when_rule_is_on() {
        let dir = tempfile::tempdir().unwrap();
        let paths = paths_in(dir.path());
        let f = feature(1, "demo");
        let data = json!({"rules": {"require_spec_approved": true}});
        // spec ausente: bloquea (gate cerrado)
        let err = spec_gate(&paths, &data, &f).unwrap_err();
        assert_eq!(err.code, 1);
        assert!(err.message.unwrap().contains("ausente"));
        // spec draft: advance y close --status done fallan con mensaje accionable
        write_spec(&paths, &f).unwrap();
        let err = spec_gate(&paths, &data, &f).unwrap_err();
        assert_eq!(err.code, 1);
        let msg = err.message.unwrap();
        assert!(msg.contains("spec-feature-1-demo.md"));
        assert!(msg.contains("draft"));
        assert!(msg.contains("Estado: approved"));
        // el usuario aprueba: el gate abre
        let p = spec_path(&paths, &f);
        let approved = std::fs::read_to_string(&p)
            .unwrap()
            .replace("Estado: draft", "Estado: approved");
        std::fs::write(&p, approved).unwrap();
        assert!(spec_gate(&paths, &data, &f).is_ok());
    }

    #[test]
    fn spec_gate_should_block_unrecognized_estado_when_rule_is_on() {
        // Fail-closed: un `Estado:` no reconocido (ni draft ni approved) NO abre
        // el gate; solo `approved` lo hace. Cubre el camino SpecState::Other,
        // que antes solo se probaba a nivel de spec_state, no del gate.
        let dir = tempfile::tempdir().unwrap();
        let paths = paths_in(dir.path());
        let f = feature(1, "demo");
        let data = json!({"rules": {"require_spec_approved": true}});
        write_spec(&paths, &f).unwrap();
        let p = spec_path(&paths, &f);
        let other = std::fs::read_to_string(&p)
            .unwrap()
            .replace("Estado: draft", "Estado: pendiente");
        std::fs::write(&p, other).unwrap();
        assert_eq!(spec_state(&paths, &f), SpecState::Other);
        let err = spec_gate(&paths, &data, &f).unwrap_err();
        assert_eq!(err.code, 1);
        let msg = err.message.unwrap();
        assert!(msg.contains("spec-feature-1-demo.md"));
        assert!(msg.contains("desconocido"));
        assert!(msg.contains("Estado: approved"));
    }

    #[test]
    fn close_requires_spec_should_gate_only_done() {
        // close --status done gatea; blocked/pending son la valvula de escape
        assert!(close_requires_spec("done"));
        assert!(!close_requires_spec("blocked"));
        assert!(!close_requires_spec("pending"));
    }
}
