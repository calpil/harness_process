//! `harness close --feature <id> --status <estado>` (paridad: cmd_close).

use std::io::Write;

use serde_json::{Value, json};

use crate::exit::Exit;
use crate::features::{feature_at, feature_mut, find_feature_index, load_features, save_features};
use crate::lecciones;
use crate::memories::update_memories;
use crate::paths::HarnessPaths;
use crate::plan::{plan_path, slugify};
use crate::prd;
use crate::progress::{log, now_stamp};
use crate::pycompat::{py_str, relpath};
use crate::spec::{close_requires_spec, spec_gate, spec_path};

/// Estado de una entrada cuyo trabajo se hizo en OTRA feature (#37). No es
/// `done` (nunca tuvo spec ni evidencia propia) ni `blocked` (no esta trabada).
pub const SUPERSEDED: &str = "superseded";
use crate::verificacion;

pub fn run(
    paths: &HarnessPaths,
    fid: &str,
    status: &str,
    note: Option<&str>,
    absorbida_por: Option<&str>,
    leccion: Option<&str>,
    leccion_motivo: Option<&str>,
) -> anyhow::Result<()> {
    let mut data = load_features(paths)?;
    let idx = find_feature_index(&data, fid)?;
    // Estado `superseded` (feature #37): el trabajo se hizo en OTRA feature.
    // Exige decir cual, y esa referencia se valida: una entrada que dice
    // "absorbida" sin decir por quien es una nota en prosa, no trazabilidad.
    // No pasa por los gates de `done` a proposito — el spec, la leccion, el
    // reporte de verify y la propuesta de documentos viven en la que absorbio.
    let absorbida = if status == SUPERSEDED {
        let Some(por) = absorbida_por.map(str::trim).filter(|s| !s.is_empty()) else {
            return Err(Exit {
                code: 2,
                message: Some(format!(
                    "--status superseded exige --absorbida-por <id>: hay que decir QUE feature\n    \
                     absorbio este trabajo, o el estado no significa nada.\n    \
                     Ejemplo: sh harness_cli close --feature {fid} --status superseded --absorbida-por 36"
                )),
            }
            .into());
        };
        if find_feature_index(&data, por).is_err() {
            return Err(Exit {
                code: 2,
                message: Some(format!(
                    "--absorbida-por {por}: esa feature no existe. Una referencia rota es peor que ninguna."
                )),
            }
            .into());
        }
        if por == fid {
            return Err(Exit {
                code: 2,
                message: Some(format!("--absorbida-por {por}: una feature no se absorbe a si misma.")),
            }
            .into());
        }
        Some(por.to_string())
    } else {
        None
    };
    // Gate SDD: cerrar como done exige spec aprobado por el usuario; se valida
    // ANTES de mutar la feature. blocked/pending no gatean (valvula de escape
    // para abortar/aparcar).
    if close_requires_spec(status) {
        let Some(feature) = feature_at(&data, idx).as_object() else {
            anyhow::bail!("feature_list.json: feature invalida");
        };
        spec_gate(paths, &data, feature)?;
    }
    // Gate de verificacion (feature #23): si el spec declara comandos y la regla
    // esta activa, exige el reporte verde y fresco. LEE el reporte; cerrar nunca
    // ejecuta un comando.
    if close_requires_spec(status) {
        let Some(feature) = feature_at(&data, idx).as_object() else {
            anyhow::bail!("feature_list.json: feature invalida");
        };
        verificacion::gate(paths, &data, status, &spec_path(paths, feature), fid)?;
    }
    // Gate de documentos (feature #29): si la regla esta activa, el PRD, el SDD
    // y architecture.md tienen que reflejar lo implementado, con la propuesta
    // aprobada por el USUARIO. Solo LEE: escribir es `prd apply --yes`.
    if close_requires_spec(status) {
        let feature = feature_at(&data, idx).clone();
        crate::documentos::gate(paths, &data, status, &feature, fid)?;
    }
    // Gate de aprendizaje (feature #17): cerrar como done declara que se
    // aprendio. Se valida tambien ANTES de mutar, por la misma razon.
    let declaracion = lecciones::gate(paths, &data, status, leccion, leccion_motivo)?;
    let stamp = now_stamp();
    let note_text = note.unwrap_or_default().to_string();
    let (plan, feature_id, feature_name, slug) = {
        let feature = feature_mut(&mut data, idx)?;
        feature.insert("status".to_string(), json!(status));
        feature.insert("closed_at".to_string(), json!(stamp.clone()));
        if !note_text.is_empty() {
            feature.insert("note".to_string(), json!(note_text.clone()));
        }
        if let Some(por) = &absorbida {
            feature.insert("superseded_by".to_string(), json!(por));
        }
        // Campos OPCIONALES (feature #17): sin declaracion la entrada queda como
        // siempre, asi que las features ya cerradas no se migran ni se tocan.
        if let Some(decl) = &declaracion {
            feature.insert("leccion".to_string(), json!(decl.clase));
            if let Some(motivo) = &decl.motivo {
                feature.insert("leccion_motivo".to_string(), json!(motivo));
            }
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
    // Feature #15: transicion al estado final (o flag Impediment si quedo
    // bloqueada) y comentario con la nota de cierre (AC-8).
    if let Some(feature) = feature_at(&data, idx).as_object() {
        crate::atlassian::emit::on_close(paths, feature, status, Some(&note_text));
    }
    crate::atlassian::push::push_bg(paths);
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
    let leccion_log = match &declaracion {
        Some(decl) => format!(" leccion={}", decl.resumen()),
        None => String::new(),
    };
    log(
        paths,
        &format!("close feature #{feature_id} status={status}{leccion_log} note={note_text}"),
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
    if let Some(decl) = &declaracion {
        match &decl.motivo {
            Some(motivo) => println!("  Leccion declarada: ninguna ({motivo})."),
            None => println!(
                "  Leccion declarada: {} ({}).",
                decl.clase,
                lecciones::rel_path(&decl.clase)
            ),
        }
    }
    // Contrato de aprendizaje (feature #18): si la feature cerro como done SIN
    // declarar nada, se le pone delante el metodo. Va al FINAL y a stderr, con
    // el stdout y el exit code ya fijados: emitir el contrato no puede cambiar
    // el resultado de un cierre (AC-10).
    if status == "done" && declaracion.is_none() && lecciones::dir(paths).is_dir() {
        let _ = std::io::stderr().write_all(lecciones::texto_contrato_de_cierre(paths).as_bytes());
    }
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
            // El PRD es una ruta protegida (feature #26) y esta escritura la
            // hizo el ARNES, no el agente: se registra para que la red de
            // seguridad no la reporte como violacion en el turno siguiente.
            crate::commands::rutas::registrar_escritura_del_arnes(paths, &rel);
        }
        Ok(_) => println!("[i] El PRD {rel} ya tenia registrada esta feature."),
        Err(err) => println!("[i] No se pudo actualizar {rel}: {err}"),
    }
}
