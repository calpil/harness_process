//! `harness close --feature <id> --status <estado>` (paridad: cmd_close).

use std::io::Write;

use serde_json::{Value, json};

use crate::features::{feature_at, feature_mut, find_feature_index, load_features, save_features};
use crate::memories::update_memories;
use crate::paths::HarnessPaths;
use crate::plan::{plan_path, slugify};
use crate::prd;
use crate::progress::{log, now_stamp};
use crate::pycompat::{py_str, relpath};
use crate::spec::{close_requires_spec, spec_gate, spec_path};

pub fn run(
    paths: &HarnessPaths,
    fid: &str,
    status: &str,
    note: Option<&str>,
) -> anyhow::Result<()> {
    let mut data = load_features(paths)?;
    let idx = find_feature_index(&data, fid)?;
    // Gate SDD: cerrar como done exige spec aprobado por el usuario; se valida
    // ANTES de mutar la feature. blocked/pending no gatean (valvula de escape
    // para abortar/aparcar).
    if close_requires_spec(status) {
        let Some(feature) = feature_at(&data, idx).as_object() else {
            anyhow::bail!("feature_list.json: feature invalida");
        };
        spec_gate(paths, &data, feature)?;
    }
    let stamp = now_stamp();
    let note_text = note.unwrap_or_default().to_string();
    let (plan, feature_id, feature_name, slug) = {
        let feature = feature_mut(&mut data, idx)?;
        feature.insert("status".to_string(), json!(status));
        feature.insert("closed_at".to_string(), json!(stamp.clone()));
        if !note_text.is_empty() {
            feature.insert("note".to_string(), json!(note_text.clone()));
        }
        let name = feature
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        (
            plan_path(paths, feature),
            py_str(feature.get("id")),
            py_str(feature.get("name")),
            slugify(&name),
        )
    };
    save_features(paths, &data)?;
    // Vuelta al PRD: cerrar como done marca el hito y deja bitacora en el PRD de
    // origen. Nunca reescribe el cuerpo del documento (es del USUARIO) y nunca
    // bloquea el cierre: si el PRD no esta, avisa y sigue.
    if status == "done"
        && let Some(feature) = feature_at(&data, idx).as_object()
    {
        echo_to_prd(paths, feature, &stamp);
    }
    if plan.exists() {
        let mut f = std::fs::OpenOptions::new().append(true).open(&plan)?;
        write!(f, "\n---\nCerrado: {stamp} - status={status} - {note_text}\n")?;
    }
    std::fs::create_dir_all(&paths.progress)?;
    // No-destructivo: si current.md tiene estado real escrito a mano, archivalo
    // en docs/ ANTES de resetear.
    let mut archived_rel: Option<String> = None;
    if paths.current.exists() {
        let content = std::fs::read_to_string(&paths.current)?;
        if !content.trim().is_empty() && !content.contains("Sin feature activa") {
            std::fs::create_dir_all(&paths.plans)?;
            let archived = paths
                .plans
                .join(format!("estado-feature-{feature_id}-{slug}.md"));
            let mut body = format!(
                "# Estado archivado - Feature #{feature_id}: {feature_name}\n"
            );
            body.push_str(&format!(
                "Cerrada: {stamp} - status={status} - {note_text}\n\n---\n\n"
            ));
            body.push_str(&content);
            std::fs::write(&archived, body)?;
            archived_rel = Some(
                relpath(&archived, &paths.repo_root)
                    .unwrap_or_else(|| archived.clone())
                    .to_string_lossy()
                    .into_owned(),
            );
        }
    }
    let mut current = String::from("# Estado Actual\n\nSin feature activa.\n\n## Evidencia\n\n-\n");
    if let Some(rel) = &archived_rel {
        current.push_str(&format!(
            "\n_Estado de la feature #{feature_id} archivado en `{rel}`._\n"
        ));
    }
    std::fs::write(&paths.current, current)?;
    log(
        paths,
        &format!("close feature #{feature_id} status={status} note={note_text}"),
    )?;
    update_memories(
        "close",
        status,
        &format!("feature-{feature_id}"),
        &note_text,
        true,
        &paths.repo_root,
    );
    let _ = std::fs::remove_file(&paths.autocheck_stamp); // cierra el ciclo de checkpoints
    let mut msg = format!("Feature #{feature_id} cerrada como {status}.");
    if let Some(rel) = &archived_rel {
        msg.push_str(&format!(" Estado archivado en {rel}."));
    }
    println!("{msg}");
    Ok(())
}

/// Marca el hito y deja bitacora en el PRD de origen de la feature. Best-effort
/// por diseno: un PRD ausente o ilegible NO puede impedir cerrar una feature.
fn echo_to_prd(paths: &HarnessPaths, feature: &serde_json::Map<String, Value>, stamp: &str) {
    let slug = prd::normalize_parent(feature.get("prd").and_then(Value::as_str));
    let file = prd::file_for(paths, &prd::segments(&slug));
    let rel = prd::rel_path(&slug);
    if !file.is_file() {
        println!("[i] Sin vuelta al PRD: falta {rel}.");
        return;
    }
    let fid = py_str(feature.get("id"));
    let name = feature
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let date = stamp.get(..10).unwrap_or(stamp);
    let spec_rel = relpath(&spec_path(paths, feature), &paths.repo_root)
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .unwrap_or_default();
    let impl_rel = format!("docs/impl-{fid}.md");
    match prd::echo_close(&file, &fid, name, date, &spec_rel, &impl_rel) {
        Ok(echo) if echo.milestone_marked || echo.logged => {
            let what = if echo.milestone_marked {
                "hito marcado done + bitacora"
            } else {
                "bitacora"
            };
            println!("PRD actualizado ({what}): {rel}");
        }
        Ok(_) => println!("[i] El PRD {rel} ya tenia registrada esta feature."),
        Err(err) => println!("[i] No se pudo actualizar {rel}: {err}"),
    }
}
