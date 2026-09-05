//! `harness start --feature <id>` (paridad: harness.py cmd_start).

use serde_json::{Value, json};

use crate::aislamiento::{Contexto, Decision, NoAislado, Ocupacion, Rechazo};
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

/// Las otras features `in_progress`, con el arbol en que escribe cada una.
///
/// Es lo que `aislamiento::decidir` necesita para saber si este arranque
/// pisaria a alguien (feature #72 / AC-1).
fn ocupaciones(data: &Value, fid: &str) -> Vec<Ocupacion> {
    features_slice(data)
        .iter()
        .filter(|f| feature_status(f) == Some("in_progress") && py_str(f.get("id")) != fid)
        .map(|f| Ocupacion {
            id: py_str(f.get("id")),
            nombre: py_str(f.get("name")),
            // AC-1 pide verificar la IDENTIDAD del worktree, no creerle al
            // backlog: un `worktree` declarado cuya carpeta ya no existe no
            // aisla a nadie. Sin esta comprobacion, una feature que perdio su
            // worktree seguia contando como aislada y le abria la puerta a la
            // siguiente para compartir checkout.
            worktree: f
                .get("worktree")
                .and_then(Value::as_str)
                .filter(|w| !w.trim().is_empty())
                .map(std::path::PathBuf::from)
                .filter(|w| w.is_dir()),
        })
        .collect()
}

/// Prepara el aislamiento de la feature SIN tocar el backlog.
///
/// Feature #72: antes esto se llamaba despues de haber marcado `in_progress`, y
/// cualquier fallo se imprimia con `[i]` y seguia de largo. Un arranque que no
/// consigue aislamiento ahora devuelve `Err`, y como el estado todavia no se
/// escribio, el backlog queda exactamente como estaba (AC-1).
fn resolver_aislamiento(
    paths: &HarnessPaths,
    data: &Value,
    idx: usize,
    sin_worktree: bool,
) -> anyhow::Result<Resuelto> {
    // El aislamiento es del repo del PROYECTO, no del dir del arnes.
    let principal = crate::git::repo_principal(&paths.repo_root);
    let feature = crate::features::feature_at(data, idx);
    let fid = py_str(feature.get("id"));
    let slug = crate::plan::slugify(feature.get("name").and_then(Value::as_str).unwrap_or_default());
    let kind = feature.get("kind").and_then(Value::as_str);
    let destino = principal
        .as_deref()
        .map(|p| crate::git::ruta_worktree(p, &fid, &slug));
    let otras = ocupaciones(data, &fid);

    let ctx = Contexto {
        repo: principal.as_deref(),
        destino,
        otras: &otras,
        sin_worktree,
    };
    match crate::aislamiento::decidir(&ctx) {
        Decision::Rechazar(r) => Err(rechazo(&fid, &r)),
        Decision::Seguir(motivo) => Ok(Resuelto::SinAislar(motivo)),
        Decision::Aislar => {
            let principal = principal.unwrap_or_else(|| paths.repo_root.clone());
            let a = crate::git::preparar(&principal, &fid, &slug, kind, None).map_err(|err| {
                // El fallback silencioso del AC-1: esto era un `println!`.
                rechazo(
                    &fid,
                    &Rechazo::FalloDeGit {
                        detalle: format!("{err:#}"),
                    },
                )
            })?;
            let docs = preparar_docs(paths, &fid, &slug, kind)?;
            Ok(Resuelto::Aislada(a, docs))
        }
    }
}

/// El worktree del repo `docs/`, cuando docs es un repo aparte (AC-2).
///
/// Si docs es su propio repo y no se le puede dar worktree, el arranque se
/// RECHAZA: caer al `docs/` compartido seria escribir el spec de esta feature
/// en el arbol de todas, que es justo lo que el AC-2 prohibe. Un `docs/` que
/// viaja con el repo principal devuelve `None` y no hay nada que preparar.
fn preparar_docs(
    paths: &HarnessPaths,
    fid: &str,
    slug: &str,
    kind: Option<&str>,
) -> anyhow::Result<Option<std::path::PathBuf>> {
    let Some(repo_docs) = crate::git::repo_de_docs(&paths.repo_root) else {
        return Ok(None);
    };
    match crate::git::preparar(&repo_docs, fid, slug, kind, None) {
        Ok(a) => Ok(Some(a.worktree)),
        Err(err) => Err(rechazo(
            fid,
            &Rechazo::FalloDeGit {
                detalle: format!("docs/ es un repo aparte y no se le pudo dar worktree: {err:#}"),
            },
        )),
    }
}

/// Un rechazo del AC-1, con la forma que ya usa el resto del arnes: exit 1 y
/// un mensaje que dice que hacer.
fn rechazo(fid: &str, r: &Rechazo) -> anyhow::Error {
    crate::exit::Exit::msg(format!(
        "[GATE] No se arranca la feature #{fid}: {}\n\nEl backlog no se toco: la feature sigue como estaba.",
        r.mensaje()
    ))
    .into()
}

/// Lo que quedo resuelto para esta feature.
enum Resuelto {
    /// Rama y worktree propios, y —si docs es un repo aparte— su worktree.
    Aislada(crate::git::Aislamiento, Option<std::path::PathBuf>),
    SinAislar(NoAislado),
}

pub fn run(paths: &HarnessPaths, fid: &str, sin_worktree: bool) -> anyhow::Result<()> {
    let mut data = load_features(paths)?;
    let idx = find_feature_index(&data, fid)?;
    // Feature #47 (AC-1): varias features pueden estar in_progress a la vez.
    // Ya no se rechaza la segunda: cada una se aisla en su rama y su worktree,
    // y su estado vivo es un archivo propio.
    let otras_activas: Vec<String> = ocupaciones(&data, fid)
        .iter()
        .map(|o| format!("#{} {}", o.id, o.nombre))
        .collect();

    // Feature #72 (AC-1): el aislamiento se resuelve ANTES de escribir nada.
    // Este orden es el arreglo: mientras el estado se marcaba primero, un
    // arranque que no conseguia worktree dejaba igual la feature `in_progress`,
    // y asi es como el diagnostico encontro a #98, #122 y #126 compartiendo
    // checkout. Un `?` aca sale sin haber tocado `feature_list.json`.
    let resuelto = resolver_aislamiento(paths, &data, idx, sin_worktree)?;

    // Recien ahora, con el aislamiento ya conseguido, la feature pasa a activa.
    {
        let feature = feature_mut(&mut data, idx)?;
        feature.insert("status".to_string(), json!("in_progress"));
        feature.insert("started_at".to_string(), json!(now_stamp()));
        match &resuelto {
            Resuelto::Aislada(a, docs) => {
                feature.insert("branch".to_string(), json!(a.rama));
                feature.insert(
                    "worktree".to_string(),
                    json!(a.worktree.to_string_lossy().to_string()),
                );
                feature.insert("aislada".to_string(), json!(true));
                // AC-2: donde vive el docs/ de ESTA feature. Sin este campo,
                // `para_feature` apuntaria al docs vacio del worktree.
                if let Some(d) = docs {
                    feature.insert(
                        "docs_worktree".to_string(),
                        json!(d.to_string_lossy().to_string()),
                    );
                }
            }
            // Queda ESCRITO que no esta aislada: es lo que despues lee
            // `ocupaciones` para negarle el paralelo a la siguiente.
            Resuelto::SinAislar(_) => {
                feature.insert("aislada".to_string(), json!(false));
            }
        }
    }
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
    match &resuelto {
        Resuelto::Aislada(a, docs) => {
            let verbo = if a.reusado { "reusados" } else { "creados" };
            println!("  Rama y worktree {verbo}: {} en {}", a.rama, a.worktree.display());
            println!("  Trabaja ahi: cd {}", a.worktree.display());
            if let Some(d) = docs {
                println!("  docs/ es un repo aparte: su worktree es {}", d.display());
            }
        }
        // AC-1: no aislada se DICE, no se insinua entre parentesis.
        Resuelto::SinAislar(motivo) => println!("{}", motivo.aviso()),
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
