//! Los enganches del flujo (spec #15, AC-6/7/8): add, start, advance,
//! approve-spec y close dejan su intent y siguen su curso.
//!
//! Todo lo de aca es best-effort por contrato: si no hay binding activo no se
//! hace nada, y si algo falla se avisa sin cambiar el exit code del comando.

use serde_json::{Map, Value};
use sha1::{Digest, Sha1};

use crate::atlassian::binding::Binding;
use crate::atlassian::outbox::{IntentKind, emit_best_effort};
use crate::paths::HarnessPaths;
use crate::pycompat::py_str;

/// Clave de dedupe de la historia de una feature.
pub fn key_feature(fid: &str) -> String {
    format!("feature:{fid}:create")
}

/// Clave de dedupe del epic de un PRD. El maestro tiene slug vacio en el
/// arbol; en las claves se escribe `master` para que se lea.
pub fn key_prd(slug: &str) -> String {
    format!("prd:{}:epic", prd_key_slug(slug))
}

/// Slug legible de un PRD (`master` para la raiz del arbol).
pub fn prd_key_slug(slug: &str) -> String {
    if slug.trim().is_empty() {
        crate::prd::MASTER.to_string()
    } else {
        slug.to_string()
    }
}

/// Clave de dedupe de la subtask de un AC-n.
pub fn key_ac(fid: &str, ac: &str) -> String {
    format!("ac:{fid}:{ac}")
}

/// Clave de dedupe de una transicion de estado.
pub fn key_status(fid: &str, to: &str) -> String {
    format!("feature:{fid}:status:{to}")
}

/// Clave de dedupe de un comentario: el hash del cuerpo evita que la misma
/// nota se publique dos veces y deja pasar dos notas distintas.
pub fn key_comment(fid: &str, body: &str) -> String {
    let mut hasher = Sha1::new();
    hasher.update(body.as_bytes());
    let digest = hex::encode(hasher.finalize());
    format!("feature:{fid}:comment:{}", &digest[..12])
}

/// Clave de dedupe del flag Impediment.
pub fn key_flag(fid: &str, on: bool) -> String {
    let estado = if on { "on" } else { "off" };
    format!("feature:{fid}:flag:{estado}")
}

/// Binding activo o nada: es el interruptor de todos los enganches (AC-4).
fn active(paths: &HarnessPaths) -> Option<Binding> {
    Binding::load_active(paths)
}

/// Titulo del epic de un PRD: el H1 del documento si se puede leer, y si no
/// una etiqueta derivada del slug.
fn prd_title(paths: &HarnessPaths, slug: &str) -> String {
    let file = paths.repo_root.join(crate::prd::rel_path(slug));
    if let Ok(text) = std::fs::read_to_string(&file) {
        for line in text.lines() {
            if let Some(rest) = line.strip_prefix("# ") {
                let title = rest.trim();
                if !title.is_empty() {
                    return title.to_string();
                }
            }
        }
    }
    if slug == "master" {
        "PRD maestro".to_string()
    } else {
        format!("PRD {slug}")
    }
}

/// Emite el epic del PRD del que sale la feature (si todavia no existe).
fn emit_prd_epic(paths: &HarnessPaths, feature: &Map<String, Value>, origin: &str) -> String {
    let slug = crate::prd::feature_prd_slug(&Value::Object(feature.clone()));
    emit_best_effort(
        paths,
        &key_prd(&slug),
        origin,
        IntentKind::PrdEpic {
            slug: prd_key_slug(&slug),
            title: prd_title(paths, &slug),
            body: format!(
                "Epic derivado del PRD `{}` del repo.",
                crate::prd::rel_path(&slug)
            ),
        },
    );
    prd_key_slug(&slug)
}

/// `add`: el PRD nace como epic y la feature como historia bajo el (AC-6).
pub fn on_add(paths: &HarnessPaths, feature: &Map<String, Value>) {
    if active(paths).is_none() {
        return;
    }
    let fid = py_str(feature.get("id"));
    let slug = emit_prd_epic(paths, feature, "add");
    let acceptance: Vec<String> = feature
        .get("acceptance")
        .and_then(Value::as_array)
        .map(|a| a.iter().map(|v| py_str(Some(v))).collect())
        .unwrap_or_default();
    emit_best_effort(
        paths,
        &key_feature(&fid),
        "add",
        IntentKind::FeatureCreate {
            fid: fid.clone(),
            name: py_str(feature.get("name")),
            acceptance,
            prd: Some(slug),
            issue_kind: feature
                .get("kind")
                .and_then(Value::as_str)
                .map(str::to_string),
        },
    );
}

/// `prd add`: un PRD nuevo nace como epic sin esperar a su primera feature
/// (feature #16, AC-3).
pub fn on_prd_add(paths: &HarnessPaths, slug: &str) {
    if active(paths).is_none() {
        return;
    }
    let key_slug = prd_key_slug(slug);
    emit_best_effort(
        paths,
        &key_prd(slug),
        "prd add",
        IntentKind::PrdEpic {
            slug: key_slug,
            title: prd_title(paths, slug),
            body: format!(
                "Epic derivado del PRD `{}` del repo.",
                crate::prd::rel_path(slug)
            ),
        },
    );
}

/// `start`: la historia entra a In Progress y cada AC-n del spec baja como
/// subtask (AC-7).
pub fn on_start(paths: &HarnessPaths, feature: &Map<String, Value>) {
    let Some(binding) = active(paths) else {
        return;
    };
    let fid = py_str(feature.get("id"));
    for (ac, text) in spec_acceptance_criteria(paths, feature) {
        emit_best_effort(
            paths,
            &key_ac(&fid, &ac),
            "start",
            IntentKind::AcSubtask {
                fid: fid.clone(),
                ac,
                text,
            },
        );
    }
    emit_best_effort(
        paths,
        &key_status(&fid, &binding.jira.statuses.in_progress),
        "start",
        IntentKind::Transition {
            fid,
            to: binding.jira.statuses.in_progress.clone(),
        },
    );
}

/// `advance`: la nota de la bitacora viaja como comentario (AC-8).
pub fn on_advance(paths: &HarnessPaths, feature: &Map<String, Value>, nota: &str) {
    if active(paths).is_none() {
        return;
    }
    let fid = py_str(feature.get("id"));
    let body = format!("Avance registrado por el arnes: {nota}");
    emit_best_effort(
        paths,
        &key_comment(&fid, &body),
        "advance",
        IntentKind::Comment { fid, body },
    );
}

/// `approve-spec`: queda el comentario de la aprobacion del USUARIO (AC-8) y
/// bajan las subtasks de los AC-n (AC-7).
///
/// Las subtasks se emiten TAMBIEN aca a proposito: `start` genera el spec como
/// plantilla y los AC-n reales los escribe el lider despues, asi que al
/// arrancar todavia no hay nada que bajar. Cuando el USUARIO aprueba, el spec
/// ya declara sus AC-n — y el dedupe por `ac:<fid>:<AC-n>` garantiza que lo que
/// ya salio en `start` no se duplique.
pub fn on_approve_spec(paths: &HarnessPaths, feature: &Map<String, Value>, stamp_line: &str) {
    if active(paths).is_none() {
        return;
    }
    let fid = py_str(feature.get("id"));
    for (ac, text) in spec_acceptance_criteria(paths, feature) {
        emit_best_effort(
            paths,
            &key_ac(&fid, &ac),
            "approve-spec",
            IntentKind::AcSubtask {
                fid: fid.clone(),
                ac,
                text,
            },
        );
    }
    let body = format!("Spec aprobado por el USUARIO. {stamp_line}");
    emit_best_effort(
        paths,
        &key_comment(&fid, &body),
        "approve-spec",
        IntentKind::Comment { fid, body },
    );
}

/// `close`: transicion al estado final y comentario con la nota de cierre
/// (AC-8). `blocked` no transiciona: marca el flag Impediment (OBS-7).
pub fn on_close(paths: &HarnessPaths, feature: &Map<String, Value>, status: &str, note: Option<&str>) {
    let Some(binding) = active(paths) else {
        return;
    };
    let fid = py_str(feature.get("id"));
    let statuses = &binding.jira.statuses;

    match status {
        "blocked" => {
            emit_best_effort(
                paths,
                &key_flag(&fid, true),
                "close",
                IntentKind::BlockedFlag {
                    fid: fid.clone(),
                    on: true,
                },
            );
        }
        "done" => {
            emit_best_effort(
                paths,
                &key_status(&fid, &statuses.done),
                "close",
                IntentKind::Transition {
                    fid: fid.clone(),
                    to: statuses.done.clone(),
                },
            );
        }
        _ => {
            emit_best_effort(
                paths,
                &key_status(&fid, &statuses.pending),
                "close",
                IntentKind::Transition {
                    fid: fid.clone(),
                    to: statuses.pending.clone(),
                },
            );
        }
    }

    let nota = note.unwrap_or("").trim();
    let body = if nota.is_empty() {
        format!("Feature cerrada por el arnes con estado `{status}`.")
    } else {
        format!("Feature cerrada por el arnes con estado `{status}`: {nota}")
    };
    emit_best_effort(
        paths,
        &key_comment(&fid, &body),
        "close",
        IntentKind::Comment { fid, body },
    );
}

/// Backfill (feature #16): baja los AC-n del spec de una feature que ya existe,
/// sin importar en que estado este. El dedupe evita repetir lo ya mapeado.
pub fn on_backfill_acs(paths: &HarnessPaths, feature: &Map<String, Value>) {
    if active(paths).is_none() {
        return;
    }
    let fid = py_str(feature.get("id"));
    for (ac, text) in spec_acceptance_criteria(paths, feature) {
        emit_best_effort(
            paths,
            &key_ac(&fid, &ac),
            "backfill",
            IntentKind::AcSubtask {
                fid: fid.clone(),
                ac,
                text,
            },
        );
    }
}

/// Backfill: lleva la historia al estado que la feature tiene HOY en el
/// backlog, para que el board sea espejo del repo (AC-24, AC-27).
pub fn on_backfill_status(
    paths: &HarnessPaths,
    feature: &Map<String, Value>,
    binding: &Binding,
) {
    if active(paths).is_none() {
        return;
    }
    let fid = py_str(feature.get("id"));
    let statuses = &binding.jira.statuses;
    let status = feature
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("pending");
    // `pending` es el estado inicial del board: no hace falta transicionar.
    let target = match status {
        "in_progress" => Some(statuses.in_progress.clone()),
        "done" => Some(statuses.done.clone()),
        _ => None,
    };
    if let Some(to) = target {
        emit_best_effort(
            paths,
            &key_status(&fid, &to),
            "backfill",
            IntentKind::Transition { fid: fid.clone(), to },
        );
    }
    if status == "blocked" {
        emit_best_effort(
            paths,
            &key_flag(&fid, true),
            "backfill",
            IntentKind::BlockedFlag { fid, on: true },
        );
    }
}

/// Extrae los `AC-n` del spec de la feature: `- AC-1: <texto>` con sus lineas
/// de continuacion indentadas. Los AC de la plantilla sin completar (los que
/// conservan los marcadores `<...>`) se ignoran: no tiene sentido crear una
/// subtask que diga "Given <contexto>".
pub fn spec_acceptance_criteria(
    paths: &HarnessPaths,
    feature: &Map<String, Value>,
) -> Vec<(String, String)> {
    let file = crate::spec::spec_path(paths, feature);
    let Ok(text) = std::fs::read_to_string(&file) else {
        return Vec::new();
    };
    parse_acceptance_criteria(&text)
}

/// Parser del bloque de AC-n (separado para poder testearlo sin archivos).
pub fn parse_acceptance_criteria(text: &str) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = Vec::new();
    let mut current: Option<(String, String)> = None;

    for line in text.lines() {
        if let Some((id, rest)) = split_ac_line(line) {
            if let Some(prev) = current.take() {
                push_ac(&mut out, prev);
            }
            current = Some((id, rest.to_string()));
            continue;
        }
        let Some((_, body)) = current.as_mut() else {
            continue;
        };
        // Continuacion: linea indentada y no vacia. Cualquier otra cosa (una
        // linea en blanco, un encabezado, otro item) cierra el AC en curso.
        if line.starts_with("  ") && !line.trim().is_empty() {
            body.push(' ');
            body.push_str(line.trim());
        } else if let Some(prev) = current.take() {
            push_ac(&mut out, prev);
        }
    }
    if let Some(prev) = current.take() {
        push_ac(&mut out, prev);
    }
    out
}

/// `- AC-12: texto` -> ("AC-12", "texto"). Acepta sufijos como `AC-10 bis`.
fn split_ac_line(line: &str) -> Option<(String, &str)> {
    let rest = line.strip_prefix("- ")?;
    let rest = rest.strip_prefix("AC-")?;
    let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() {
        return None;
    }
    let after = &rest[digits.len()..];
    let (extra, body) = match after.find(':') {
        Some(pos) => (after[..pos].trim(), after[pos + 1..].trim()),
        None => return None,
    };
    let id = if extra.is_empty() {
        format!("AC-{digits}")
    } else {
        format!("AC-{digits} {extra}")
    };
    Some((id, body))
}

fn push_ac(out: &mut Vec<(String, String)>, (id, body): (String, String)) {
    let body = body.trim().to_string();
    if body.is_empty() || is_template_placeholder(&body) {
        return;
    }
    out.push((id, body));
}

/// La plantilla del spec trae `Given <contexto>, When <accion>, ...`: esos
/// marcadores delatan un AC sin completar.
fn is_template_placeholder(body: &str) -> bool {
    body.contains("<contexto>") || body.contains("<accion>") || body.contains("<resultado")
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;

    #[test]
    fn should_parse_single_line_criteria() {
        let text = "## Criterios\n- AC-1: Given un repo, When corro add, Then pasa algo.\n";
        let acs = parse_acceptance_criteria(text);
        assert_eq!(acs.len(), 1);
        assert_eq!(acs[0].0, "AC-1");
        assert!(acs[0].1.starts_with("Given un repo"));
    }

    #[test]
    fn should_join_continuation_lines() {
        let text = "- AC-2: Given algo,\n  When otra cosa,\n  Then el final.\n\n## Otra seccion\n";
        let acs = parse_acceptance_criteria(text);
        assert_eq!(acs.len(), 1);
        assert_eq!(acs[0].1, "Given algo, When otra cosa, Then el final.");
    }

    #[test]
    fn should_ignore_template_placeholders() {
        // La plantilla sin completar no genera subtasks.
        let text = "- AC-1: Given <contexto>, When <accion>, Then <resultado observable>.\n";
        assert!(parse_acceptance_criteria(text).is_empty());
    }

    #[test]
    fn should_keep_ids_with_suffix_and_ignore_other_bullets() {
        let text = concat!(
            "- AC-10: primero.\n",
            "- AC-10 bis: segundo.\n",
            "- otra vinieta que no es AC.\n",
            "- AC-11: tercero.\n",
        );
        let acs = parse_acceptance_criteria(text);
        let ids: Vec<&str> = acs.iter().map(|(id, _)| id.as_str()).collect();
        assert_eq!(ids, vec!["AC-10", "AC-10 bis", "AC-11"]);
    }

    #[test]
    fn comment_keys_should_differ_by_body_and_repeat_by_content() {
        let a = key_comment("15", "una nota");
        let b = key_comment("15", "otra nota");
        assert_ne!(a, b, "dos notas distintas no pueden colisionar");
        assert_eq!(a, key_comment("15", "una nota"), "la misma nota, la misma clave");
    }

    #[test]
    fn hooks_should_do_nothing_without_binding() {
        // AC-4: sin binding no se crea ni la carpeta de la outbox.
        let dir = tempfile::tempdir().unwrap();
        let paths = HarnessPaths::from_root(dir.path().to_path_buf());
        let mut feature = Map::new();
        feature.insert("id".to_string(), Value::from(15));
        feature.insert("name".to_string(), Value::from("demo"));
        on_add(&paths, &feature);
        on_start(&paths, &feature);
        on_advance(&paths, &feature, "nota");
        on_close(&paths, &feature, "done", Some("cerrada"));
        assert!(crate::atlassian::outbox::pending(&paths).is_empty());
        assert!(!crate::atlassian::state::atlassian_dir(&paths).exists());
    }
}
