//! Specs por feature (Spec-Driven Development) y gate de aprobacion.
//! Espejo de plan.rs: la firma anti-conflicto multi-LLM REUSA
//! `plan::plan_signature` (mismo dict path/mtime/size/hash) sobre la clave
//! `last_spec_sig`. La DECISION de aprobar (draft -> approved) es del USUARIO;
//! el agente la registra con `approve-spec --yes` tras su si explicito y nunca
//! por iniciativa propia.

use std::path::PathBuf;

use anyhow::Context;
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

/// Ruta del spec relativa a la RAIZ del repo, con `/` en todas las
/// plataformas: la forma que tiene DESPUES del merge, sin importar en que
/// worktree se escribio.
///
/// No se calcula con `relpath` contra el spec del worktree: eso produce
/// `../<repo>-wt/<id>-<slug>/docs/spec-*.md`, un puntero al arbol que el propio
/// cierre borra segundos despues (feature #60, bug #92).
pub fn spec_rel_raiz(feature: &Map<String, Value>) -> String {
    let id = py_str(feature.get("id"));
    let name = feature
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or_default();
    format!("docs/spec-feature-{id}-{}.md", slugify(name))
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
        // PRD de origen: el que declaro `add --prd`, o el maestro (las features
        // sin PRD explicito cuentan para el producto entero).
        format!("PRD: {}", crate::prd::feature_prd_rel(feature)),
        "Constitution: docs/constitution.md".to_string(),
        "Metodo: docs/prd/COMO-ESCRIBIR-UN-PRD.md (este spec es el PRD del cambio)".to_string(),
    ];
    for tail in [
        "",
        "## La historia (antes -> despues)",
        "<!-- El corazon del spec: contala en palabras, sin tecnicismos, con una",
        "     persona con nombre y un momento concreto. Si la historia no convence,",
        "     el resto no importa. -->",
        "ANTES: <que le pasa hoy a <persona>, y por que duele>.",
        "DESPUES: <que vive esa misma persona cuando esto exista>.",
        "",
        "## Hoy -> Como va a funcionar",
        "<!-- El flujo, dibujado dos veces: dibujar el HOY obliga a reusar lo que ya",
        "     existe en vez de inventar arquitectura nueva. -->",
        "```",
        "HOY                      DESPUES",
        "<evento> -> (nada)       <evento> -> <lo que ahora ocurre>",
        "                              |__ <componente> -> <componente>",
        "```",
        "",
        "## Recorridos de usuario (priorizados)",
        "<!-- P1 imprescindible, P2 importante. Cada recorrido independientemente testeable. -->",
        "- P1: Como <rol>, quiero <accion>, para <resultado>.",
        "",
        "## Criterios de aceptacion (Given/When/Then)",
        "<!-- La Delegacion del plan cita estos IDs (AC-1, AC-2, ...); el reviewer exige evidencia por AC.",
        "     OPCIONAL: debajo de un AC podes declarar COMO se prueba, y",
        "     `sh harness_cli verify --feature <id>` lo ejecuta y deja",
        "     docs/verify-<id>.md. Un AC sin comando lo verifica el reviewer,",
        "     como siempre: no declarar comando NO es un fallo. -->",
        "- AC-1: Given <contexto>, When <accion>, Then <resultado observable>.",
        "  Comando: `<como se prueba, ejecutable desde la raiz>`",
        "",
        "## Los datos que se tocan",
        "<!-- El plano de los datos: que dispara el flujo, que interruptor lo apaga y",
        "     que candado evita que pase dos veces. Entidades y campos en palabras. -->",
        "- disparador: <el evento o cambio de estado que arranca el flujo>",
        "- interruptor: <flag por cliente/entorno para apagarlo en 1 clic>",
        "- candado: <campo que evita repetir la accion sobre el mismo caso>",
        "",
        "## Pseudo-codigo (el acuerdo)",
        "<!-- La receta en palabras: que lo dispara, que lo frena y que promete.",
        "     SIN CODIGO FINAL: el spec fija la estructura, no la implementacion. -->",
        "```",
        "CUANDO <ocurre el disparador>",
        "",
        "  ¿<esta activado para este caso>?  -> si no, no hacemos nada",
        "  ¿<ya lo hicimos antes>?           -> si si, no hacemos nada",
        "",
        "  ENTONCES <que hacemos, en una frase>,",
        "           con <la restriccion que lo hace aceptable>.",
        "```",
        "Promesas: <una sola vez por caso> · <limite temporal> · <que NO hace>.",
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

/// Estado declarado del spec. La decision de aprobar (draft -> approved) es
/// EXCLUSIVA del usuario; el agente solo la escribe via `approve_spec`, y solo
/// despues de su confirmacion explicita.
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

/// Resultado de registrar la aprobacion del usuario (AC-1 / AC-4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApprovalOutcome {
    /// Se escribio `Estado: approved` y se inserto el sello de aprobacion.
    Registered,
    /// Ya estaba aprobado: no se reescribe el archivo ni se duplica el sello.
    AlreadyApproved,
}

/// Sello que acompana a `Estado: approved`: deja auditable quien aprobo, cuando
/// y con que nota (el spec es la fuente de verdad; history.md lo espeja).
pub fn approval_stamp_line(stamp: &str, nota: &str) -> String {
    let base = format!("Aprobado: {stamp} por USUARIO (confirmacion explicita)");
    match nota.trim() {
        "" => base,
        nota => format!("{base} - {nota}"),
    }
}

/// Registra en el spec la aprobacion del USUARIO: reescribe la PRIMERA linea
/// `Estado:` de la ventana de 10 lineas a `Estado: approved` e inserta el sello
/// debajo. La DECISION sigue siendo del usuario; esta funcion solo la persiste
/// despues de su confirmacion explicita (el comando la exige con `--yes`).
/// Idempotente: si ya estaba aprobado no toca el archivo.
pub fn approve_spec(
    paths: &HarnessPaths,
    feature: &Map<String, Value>,
    stamp: &str,
    nota: &str,
) -> anyhow::Result<ApprovalOutcome> {
    if spec_state(paths, feature) == SpecState::Approved {
        return Ok(ApprovalOutcome::AlreadyApproved);
    }
    let path = spec_path(paths, feature);
    let content = std::fs::read_to_string(&path)
        .with_context(|| format!("no se pudo leer el spec: {}", path.display()))?;
    let ends_with_newline = content.ends_with('\n');
    let mut lines: Vec<String> = content.lines().map(str::to_string).collect();
    let stamp_line = approval_stamp_line(stamp, nota);
    match lines
        .iter()
        .take(10)
        .position(|l| l.trim().starts_with("Estado:"))
    {
        Some(i) => {
            // Preserva la indentacion original: spec_state hace trim, y un spec
            // escrito a mano puede tener margen.
            let indent: String = lines[i].chars().take_while(|c| c.is_whitespace()).collect();
            lines[i] = format!("{indent}Estado: approved");
            lines.insert(i + 1, stamp_line);
        }
        None => {
            // Spec sin linea `Estado:`: se siembra al tope (tras el titulo) para
            // que caiga dentro de las 10 lineas que lee spec_state.
            let at = usize::from(lines.first().is_some_and(|l| l.trim_start().starts_with('#')));
            lines.splice(at..at, ["Estado: approved".to_string(), stamp_line]);
        }
    }
    let mut out = lines.join("\n");
    if ends_with_newline {
        out.push('\n');
    }
    std::fs::write(&path, out)?;
    Ok(ApprovalOutcome::Registered)
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
    // Exit 2, igual que los otros dos gates de `close` (leccion y verify) y que
    // el resto del binario cuando el arnes te frena. Salia 1 por un `Exit::msg`
    // heredado; unificado en la feature #36.
    Err(Exit {
        code: 2,
        message: Some(format!(
        "[GATE] Spec sin aprobar: {} (estado: {}).\n    La regla require_spec_approved esta activa. Flujo de aprobacion:\n      1) Mostrale el spec al USUARIO (contenido en el chat + abriselo en su editor).\n      2) Preguntale si lo aprueba.\n      3) Solo con su SI: sh harness_cli approve-spec --yes\n    La decision es del usuario; el agente solo la registra.",
        rel.display(),
        state.label()
    ))})
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
            "# Spec - Feature #3: Pago QR\n\nEstado: draft\nPlan: docs/plan-feature-3-pago-qr.md\nPRD: docs/prd/PRD-master.md\nConstitution: docs/constitution.md\n"
        ));
        // `Estado:` sigue en la linea 3: spec_state solo mira las primeras diez.
        assert_eq!(t.lines().nth(2), Some("Estado: draft"));
        assert!(t.contains("Metodo: docs/prd/COMO-ESCRIBIR-UN-PRD.md"));
        // El spec es el PRD del cambio: historia, hoy->despues, datos y acuerdo.
        assert!(t.contains("## La historia (antes -> despues)"));
        assert!(t.contains("ANTES: "));
        assert!(t.contains("DESPUES: "));
        assert!(t.contains("## Hoy -> Como va a funcionar"));
        assert!(t.contains("## Recorridos de usuario (priorizados)"));
        assert!(t.contains("- P1: Como <rol>, quiero <accion>, para <resultado>."));
        assert!(t.contains("## Criterios de aceptacion (Given/When/Then)"));
        assert!(t.contains("- AC-1: Given <contexto>, When <accion>, Then <resultado observable>."));
        assert!(t.contains("## Los datos que se tocan"));
        assert!(t.contains("- disparador: "));
        assert!(t.contains("- interruptor: "));
        assert!(t.contains("- candado: "));
        assert!(t.contains("## Pseudo-codigo (el acuerdo)"));
        assert!(t.contains("CUANDO <ocurre el disparador>"));
        assert!(t.contains("  ENTONCES <que hacemos, en una frase>,"));
        assert!(t.contains("Promesas: "));
        assert!(t.contains("## No funcionales\n- SLOs:\n- Seguridad:\n- Observabilidad:\n"));
        assert!(t.contains("## Fuera de alcance"));
        assert!(t.contains("## Observaciones (decisiones pendientes)"));
        assert!(t.ends_with("-\n"));
    }

    #[test]
    fn spec_template_sections_should_keep_the_prd_order() {
        let t = spec_template(&feature(7, "Gracias post-venta"));
        let order: Vec<&str> = t
            .lines()
            .filter(|l| l.starts_with("## "))
            .collect();
        assert_eq!(
            order,
            vec![
                "## La historia (antes -> despues)",
                "## Hoy -> Como va a funcionar",
                "## Recorridos de usuario (priorizados)",
                "## Criterios de aceptacion (Given/When/Then)",
                "## Los datos que se tocan",
                "## Pseudo-codigo (el acuerdo)",
                "## No funcionales",
                "## Fuera de alcance",
                "## Observaciones (decisiones pendientes)",
            ]
        );
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
    fn approve_spec_should_write_approved_with_stamp_preserving_indent() {
        // AC-1: la primera linea Estado: de la ventana pasa a approved y el
        // sello queda justo debajo, sin perder el margen del spec original.
        let dir = tempfile::tempdir().unwrap();
        let paths = paths_in(dir.path());
        let f = feature(1, "demo");
        std::fs::create_dir_all(&paths.plans).unwrap();
        let p = spec_path(&paths, &f);
        std::fs::write(&p, "# Spec\n  Estado: draft\nPlan: docs/plan.md\n").unwrap();
        let outcome = approve_spec(&paths, &f, "2026-07-24T00:00:00Z", "").unwrap();
        assert_eq!(outcome, ApprovalOutcome::Registered);
        assert_eq!(spec_state(&paths, &f), SpecState::Approved);
        assert_eq!(
            std::fs::read_to_string(&p).unwrap(),
            "# Spec\n  Estado: approved\nAprobado: 2026-07-24T00:00:00Z por USUARIO (confirmacion explicita)\nPlan: docs/plan.md\n"
        );
    }

    #[test]
    fn approve_spec_should_record_nota_in_the_stamp() {
        // AC-6: la nota del usuario queda en el sello (auditoria en el spec).
        let dir = tempfile::tempdir().unwrap();
        let paths = paths_in(dir.path());
        let f = feature(1, "demo");
        write_spec(&paths, &f).unwrap();
        approve_spec(&paths, &f, "2026-07-24T00:00:00Z", "  aprobado en chat  ").unwrap();
        let text = std::fs::read_to_string(spec_path(&paths, &f)).unwrap();
        assert!(text.contains(
            "Aprobado: 2026-07-24T00:00:00Z por USUARIO (confirmacion explicita) - aprobado en chat\n"
        ));
    }

    #[test]
    fn approve_spec_should_be_idempotent_without_duplicating_the_stamp() {
        // AC-4: re-aprobar no reescribe ni agrega un segundo sello.
        let dir = tempfile::tempdir().unwrap();
        let paths = paths_in(dir.path());
        let f = feature(1, "demo");
        write_spec(&paths, &f).unwrap();
        approve_spec(&paths, &f, "2026-07-24T00:00:00Z", "primera").unwrap();
        let first = std::fs::read_to_string(spec_path(&paths, &f)).unwrap();
        let outcome = approve_spec(&paths, &f, "2026-07-24T11:11:11Z", "segunda").unwrap();
        assert_eq!(outcome, ApprovalOutcome::AlreadyApproved);
        let second = std::fs::read_to_string(spec_path(&paths, &f)).unwrap();
        assert_eq!(first, second);
        assert_eq!(second.matches("Aprobado: ").count(), 1);
    }

    #[test]
    fn approve_spec_should_seed_estado_when_the_window_has_no_line() {
        // Borde: spec sin linea Estado: (SpecState::Other). Se siembra tras el
        // titulo para que caiga dentro de la ventana de 10 lineas.
        let dir = tempfile::tempdir().unwrap();
        let paths = paths_in(dir.path());
        let f = feature(1, "demo");
        std::fs::create_dir_all(&paths.plans).unwrap();
        let p = spec_path(&paths, &f);
        std::fs::write(&p, "# Spec\ncuerpo sin estado\n").unwrap();
        assert_eq!(spec_state(&paths, &f), SpecState::Other);
        approve_spec(&paths, &f, "2026-07-24T00:00:00Z", "").unwrap();
        assert_eq!(spec_state(&paths, &f), SpecState::Approved);
        assert!(std::fs::read_to_string(&p).unwrap().starts_with(
            "# Spec\nEstado: approved\nAprobado: 2026-07-24T00:00:00Z por USUARIO"
        ));
    }

    #[test]
    fn approve_spec_plus_resign_should_leave_the_spec_fresh() {
        // AC-2 a nivel funcion: aprobar cambia el hash (stale), y la re-firma
        // que hace el comando lo deja fresco. Sin esto, la aprobacion del propio
        // usuario se reporta como "SPEC ACTUALIZADO POR OTRO LLM".
        let dir = tempfile::tempdir().unwrap();
        let paths = paths_in(dir.path());
        let mut f = feature(1, "demo");
        write_spec(&paths, &f).unwrap();
        update_spec_sig(&paths, &mut f);
        assert!(!is_spec_stale(&paths, &f));
        approve_spec(&paths, &f, "2026-07-24T00:00:00Z", "").unwrap();
        assert!(is_spec_stale(&paths, &f));
        update_spec_sig(&paths, &mut f);
        assert!(!is_spec_stale(&paths, &f));
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
        // spec ausente: bloquea (gate cerrado). Exit 2, igual que los otros dos
        // gates de `close`, desde la feature #36.
        let err = spec_gate(&paths, &data, &f).unwrap_err();
        assert_eq!(err.code, 2);
        assert!(err.message.unwrap().contains("ausente"));
        // spec draft: advance y close --status done fallan con mensaje accionable
        write_spec(&paths, &f).unwrap();
        let err = spec_gate(&paths, &data, &f).unwrap_err();
        assert_eq!(err.code, 2);
        let msg = err.message.unwrap();
        assert!(msg.contains("spec-feature-1-demo.md"));
        assert!(msg.contains("draft"));
        // El mensaje instruye el ritual de aprobacion, no la edicion manual.
        assert!(msg.contains("Mostrale el spec al USUARIO"));
        assert!(msg.contains("approve-spec --yes"));
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
        assert_eq!(err.code, 2);
        let msg = err.message.unwrap();
        assert!(msg.contains("spec-feature-1-demo.md"));
        assert!(msg.contains("desconocido"));
        assert!(msg.contains("approve-spec --yes"));
    }

    #[test]
    fn close_requires_spec_should_gate_only_done() {
        // close --status done gatea; blocked/pending son la valvula de escape
        assert!(close_requires_spec("done"));
        assert!(!close_requires_spec("blocked"));
        assert!(!close_requires_spec("pending"));
    }
}
