//! `harness start --feature <id>` (paridad: harness.py cmd_start).

use serde_json::{Value, json};

use crate::features::{
    feature_at, feature_mut, feature_status, features_slice, find_feature_index, load_features,
    save_features,
};
use crate::memories::update_memories;
use crate::paths::HarnessPaths;
use crate::plan::{update_plan_sig, write_plan};
use crate::progress::{log, now_stamp};
use crate::pycompat::{py_str, relpath};
use crate::spec::{update_spec_sig, write_spec};

/// Crea (o reusa) la rama y el worktree de la feature y los guarda en el
/// backlog. Best-effort: si no hay repo git, se avisa y se sigue (AC-5).
fn preparar_aislamiento(
    paths: &HarnessPaths,
    data: &mut Value,
    idx: usize,
    sin_worktree: bool,
) -> anyhow::Result<Option<crate::git::Aislamiento>> {
    if sin_worktree {
        return Ok(None);
    }
    // El aislamiento es del repo del PROYECTO, no del dir del arnes.
    let Some(principal) = crate::git::repo_principal(&paths.repo_root) else {
        return Ok(None);
    };
    let (fid, slug, kind) = {
        let feature = feature_mut(data, idx)?;
        (
            py_str(feature.get("id")),
            crate::plan::slugify(feature.get("name").and_then(Value::as_str).unwrap_or_default()),
            feature
                .get("kind")
                .and_then(Value::as_str)
                .map(str::to_string),
        )
    };
    match crate::git::preparar(&principal, &fid, &slug, kind.as_deref(), None) {
        Ok(a) => {
            let feature = feature_mut(data, idx)?;
            feature.insert("branch".to_string(), json!(a.rama));
            feature.insert(
                "worktree".to_string(),
                json!(a.worktree.to_string_lossy().to_string()),
            );
            Ok(Some(a))
        }
        Err(err) => {
            // Nunca bloquea el arranque: se avisa y se trabaja como siempre.
            println!("[i] Sin worktree para la feature #{fid}: {err:#}");
            Ok(None)
        }
    }
}

pub fn run(paths: &HarnessPaths, fid: &str, sin_worktree: bool) -> anyhow::Result<()> {
    let mut data = load_features(paths)?;
    let idx = find_feature_index(&data, fid)?;
    // Feature #47 (AC-1): varias features pueden estar in_progress a la vez.
    // Ya no se rechaza la segunda: cada una se aisla en su rama y su worktree,
    // y su estado vivo es un archivo propio.
    let otras_activas: Vec<String> = features_slice(&data)
        .iter()
        .filter(|f| feature_status(f) == Some("in_progress") && py_str(f.get("id")) != fid)
        .map(|f| format!("#{} {}", py_str(f.get("id")), py_str(f.get("name"))))
        .collect();
    {
        let feature = feature_mut(&mut data, idx)?;
        feature.insert("status".to_string(), json!("in_progress"));
        feature.insert("started_at".to_string(), json!(now_stamp()));
    }
    save_features(paths, &data)?;

    // Feature #47: rama + worktree por feature ANTES de escribir nada, para que
    // el plan, el spec y toda la evidencia nazcan DENTRO del worktree de esta
    // feature y viajen con su rama. Sin repo git (o con --sin-worktree) se
    // trabaja como siempre (AC-5, AC-6).
    let aislamiento = preparar_aislamiento(paths, &mut data, idx, sin_worktree)?;
    save_features(paths, &data)?;
    // Las rutas de docs se resuelven DESDE la feature: su worktree manda, no el
    // directorio donde se ejecuto el comando.
    let paths = &{
        let feature = feature_mut(&mut data, idx)?;
        paths.para_feature(feature)
    };

    let (rel_plan, rel_spec, feature_id, feature_name, services, meta_name) = {
        let feature = feature_mut(&mut data, idx)?;
        let plan = write_plan(paths, feature)?;
        let base_rel = paths
            .plans
            .parent()
            .map(std::path::Path::to_path_buf)
            .unwrap_or_else(|| paths.repo_root.clone());
        let rel_plan = relpath(&plan, &base_rel)
            .unwrap_or_else(|| plan.clone())
            .to_string_lossy()
            .into_owned();
        // Capturar firma del plan para detectar ediciones por otros LLMs
        update_plan_sig(paths, feature);
        // Spec SDD: se siembra SIEMPRE (nace draft); el gate lo controla solo
        // la regla require_spec_approved. Firma igual que el plan.
        let spec = write_spec(paths, feature)?;
        let rel_spec = relpath(&spec, &base_rel)
            .unwrap_or_else(|| spec.clone())
            .to_string_lossy()
            .into_owned();
        update_spec_sig(paths, feature);
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
            rel_spec,
            py_str(feature.get("id")),
            py_str(feature.get("name")),
            services,
            meta_name,
        )
    };
    save_features(paths, &data)?;
    // Feature #15: la historia entra a In Progress y cada AC-n del spec baja
    // como subtask (AC-7). El spec ya esta escrito en disco a esta altura.
    if let Some(feature) = feature_at(&data, idx).as_object() {
        crate::atlassian::emit::on_start(paths, feature);
    }
    crate::atlassian::push::push_bg(paths);


    std::fs::create_dir_all(&paths.progress)?;
    let mut current = format!("# Feature #{feature_id}: {feature_name}\n\n");
    current.push_str("Estado: in_progress\n");
    current.push_str(&format!("Plan: {rel_plan}\n"));
    current.push_str(&format!("Spec: {rel_spec}\n\n"));
    current.push_str("Microservicios:\n");
    for service in &services {
        current.push_str(&format!("- {service}\n"));
    }
    current.push_str("\nEvidencia:\n- \n");
    // AC-8: cada feature escribe SU estado vivo; nadie pisa a nadie.
    std::fs::write(paths.current_de(&feature_id), current)?;
    // AC-9: current.md pasa a ser el indice de lo que hay abierto.
    crate::progress::escribir_indice(paths, &data)?;
    log(paths, &format!("start feature #{feature_id} {feature_name}"))?;
    update_memories(
        "start",
        "in_progress",
        &format!("feature-{feature_id}"),
        &meta_name,
        false,
        &paths.repo_root,
    );
    // Linea base del checkpoint, por feature (AC-10): el plan recien creado no
    // dispara autocheck y no toca el stamp de las otras.
    crate::progress::touch_autocheck_stamp_de(paths, &feature_id);
    println!("Feature #{feature_id} iniciada. Plan: {rel_plan}");
    match &aislamiento {
        Some(a) => {
            let verbo = if a.reusado { "reusados" } else { "creados" };
            println!("  Rama y worktree {verbo}: {} en {}", a.rama, a.worktree.display());
            println!("  Trabaja ahi: cd {}", a.worktree.display());
        }
        None if sin_worktree => println!("  (--sin-worktree: se trabaja en el checkout actual)"),
        None => println!("  (sin aislamiento: no hay repo git utilizable; se trabaja en el checkout actual)"),
    }
    if !otras_activas.is_empty() {
        println!(
            "  En paralelo con: {} (cada una en su worktree; el backlog es uno solo)",
            otras_activas.join(", ")
        );
    }
    println!("  (firma del plan registrada para deteccion de actualizaciones por otros agentes)");
    println!("Spec (draft) generado: {rel_spec}");
    println!(
        "  Completa recorridos y AC-n; despues mostrale el spec al USUARIO, preguntale si lo"
    );
    println!("  aprueba y con su SI registra: sh harness_cli approve-spec --yes");
    println!(
        "  Con la regla require_spec_approved activa, advance y close --status done bloquean sin esa aprobacion."
    );
    if let Ok(feature) = feature_mut(&mut data, idx) {
        imprimir_contexto(paths, &feature.clone());
    }
    Ok(())
}

/// El resumen del contexto, SIEMPRE (feature #56, OBS-3 del spec).
///
/// Sale aca y no detras de un flag porque el caso en que mas importa —el
/// paquete vacio, el mapa que no cubre el tema— es justo el que nadie pediria.
/// La leccion `promesas-estructurales-vs-disciplina` lo dice completo: si
/// depende de acordarse, no es un invariante.
///
/// Nunca falla el `start`: el resumen es informacion, no un gate.
fn imprimir_contexto(paths: &HarnessPaths, feature: &serde_json::Map<String, serde_json::Value>) {
    // `paths` ya viene resuelto contra la feature (arriba en `run`).
    let tema = crate::commands::contexto::tema_de_feature(feature);
    let paquete = crate::contexto::armar(
        paths,
        Some(feature),
        &tema,
        crate::contexto::MAX_LINEAS_DEFAULT,
    );
    println!();
    print!("{}", paquete.resumen());
    if let Some(aviso) = paquete.aviso_de_cobertura() {
        println!("\n{aviso}");
    }
}
