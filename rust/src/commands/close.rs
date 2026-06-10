//! `harness close --feature <id> --status <estado>` (paridad: cmd_close).

use std::io::Write;

use serde_json::{Value, json};

use crate::features::{feature_mut, find_feature_index, load_features, save_features};
use crate::memories::update_memories;
use crate::paths::HarnessPaths;
use crate::plan::{plan_path, slugify};
use crate::progress::{log, now_stamp};
use crate::pycompat::{py_str, relpath};

pub fn run(
    paths: &HarnessPaths,
    fid: &str,
    status: &str,
    note: Option<&str>,
) -> anyhow::Result<()> {
    let mut data = load_features(paths)?;
    let idx = find_feature_index(&data, fid)?;
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
