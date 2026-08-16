//! `progress/atlassian/outbox/`: la intencion antes que la llamada (spec #15).
//!
//! El binario NO habla MCP — el MCP vive en el agente — asi que cada
//! transicion del flujo escribe QUE deberia existir del otro lado y hay dos
//! ejecutores para lo mismo: `drain` (agente con MCP) y `apply` (REST con
//! token). La outbox es el contrato comun entre ambos.
//!
//! Reglas duras del spec:
//! - emitir es best-effort: jamas cambia el exit code del comando del flujo;
//! - un intent nace aplicado si su clave de dedupe ya esta satisfecha (AC-11);
//! - los intents aplicados no se borran: se archivan en `applied/`.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::atlassian::state::{State, atlassian_dir};
use crate::paths::HarnessPaths;
use crate::progress::now_stamp;

pub fn outbox_dir(paths: &HarnessPaths) -> PathBuf {
    atlassian_dir(paths).join("outbox")
}

pub fn applied_dir(paths: &HarnessPaths) -> PathBuf {
    atlassian_dir(paths).join("applied")
}

/// Que deberia existir del otro lado. El intent guarda datos SEMANTICOS, no la
/// llamada ya armada: las referencias (clave del epic padre, de la historia)
/// se resuelven contra `state.json` recien en el momento de ejecutar, que es
/// cuando ya existen.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum IntentKind {
    /// PRD -> epic.
    PrdEpic {
        slug: String,
        title: String,
        #[serde(default)]
        body: String,
    },
    /// Feature del backlog -> historia bajo el epic de su PRD.
    FeatureCreate {
        fid: String,
        name: String,
        #[serde(default)]
        acceptance: Vec<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        prd: Option<String>,
    },
    /// AC-n del spec -> subtask de la historia.
    AcSubtask { fid: String, ac: String, text: String },
    /// Estado del backlog -> columna del board.
    Transition { fid: String, to: String },
    /// Bitacora -> comentario en la historia.
    Comment { fid: String, body: String },
    /// `blocked` -> flag Impediment (decision OBS-7).
    BlockedFlag { fid: String, on: bool },
}

impl IntentKind {
    /// Orden de dependencia: primero el epic, despues la historia, despues las
    /// subtasks, y solo entonces transiciones y comentarios (AC-9).
    pub fn rank(&self) -> u8 {
        match self {
            IntentKind::PrdEpic { .. } => 0,
            IntentKind::FeatureCreate { .. } => 1,
            IntentKind::AcSubtask { .. } => 2,
            IntentKind::Transition { .. } => 3,
            IntentKind::BlockedFlag { .. } => 4,
            IntentKind::Comment { .. } => 5,
        }
    }

    /// Descripcion corta para `status` y `drain`.
    pub fn label(&self) -> String {
        match self {
            IntentKind::PrdEpic { slug, .. } => format!("epic del PRD {slug}"),
            IntentKind::FeatureCreate { fid, name, .. } => {
                format!("historia de la feature #{fid} ({name})")
            }
            IntentKind::AcSubtask { fid, ac, .. } => format!("subtask {ac} de la feature #{fid}"),
            IntentKind::Transition { fid, to } => format!("feature #{fid} -> {to}"),
            IntentKind::Comment { fid, .. } => format!("comentario en la feature #{fid}"),
            IntentKind::BlockedFlag { fid, on } => {
                let verbo = if *on { "marca" } else { "quita" };
                format!("{verbo} el flag Impediment en la feature #{fid}")
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Intent {
    /// Identificador estable dentro de la outbox (`0007`).
    pub id: String,
    /// Clave de dedupe: el candado contra `state.json` (AC-11).
    pub key: String,
    /// Comando del flujo que lo origino (`add`, `start`, `close`, ...).
    pub origin: String,
    pub created_at: String,
    #[serde(flatten)]
    pub kind: IntentKind,
}

impl Intent {
    pub fn new(id: String, key: String, origin: &str, kind: IntentKind) -> Intent {
        Intent {
            id,
            key,
            origin: origin.to_string(),
            created_at: now_stamp(),
            kind,
        }
    }

    fn file_name(&self) -> String {
        format!("{}-{}.json", self.id, self.origin)
    }
}

/// Lee los intents pendientes, ordenados por dependencia y despues por id.
pub fn pending(paths: &HarnessPaths) -> Vec<Intent> {
    let dir = outbox_dir(paths);
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return Vec::new();
    };
    let mut out: Vec<Intent> = entries
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|x| x == "json"))
        .filter_map(|e| std::fs::read_to_string(e.path()).ok())
        .filter_map(|t| serde_json::from_str::<Intent>(&t).ok())
        .collect();
    out.sort_by(|a, b| a.kind.rank().cmp(&b.kind.rank()).then(a.id.cmp(&b.id)));
    out
}

/// Siguiente id de la outbox (cuenta pendientes + archivados para no repetir).
fn next_id(paths: &HarnessPaths) -> String {
    let count = [outbox_dir(paths), applied_dir(paths)]
        .iter()
        .filter_map(|d| std::fs::read_dir(d).ok())
        .flat_map(|rd| rd.filter_map(|e| e.ok()))
        .filter(|e| e.path().extension().is_some_and(|x| x == "json"))
        .count();
    format!("{:04}", count + 1)
}

/// True si ya hay un intent pendiente con esa clave de dedupe.
fn already_pending(paths: &HarnessPaths, key: &str) -> bool {
    pending(paths).iter().any(|i| i.key == key)
}

/// Escribe un intent salvo que su clave ya este satisfecha o ya pendiente.
/// Devuelve `Ok(true)` si escribio algo.
pub fn emit(paths: &HarnessPaths, key: &str, origin: &str, kind: IntentKind) -> anyhow::Result<bool> {
    let state = State::load(paths);
    if state.is_applied(key) || already_pending(paths, key) {
        return Ok(false);
    }
    let dir = outbox_dir(paths);
    std::fs::create_dir_all(&dir)?;
    let intent = Intent::new(next_id(paths), key.to_string(), origin, kind);
    let text = format!("{}\n", serde_json::to_string_pretty(&intent)?);
    crate::features::write_text_atomic(&dir.join(intent.file_name()), &text)?;
    Ok(true)
}

/// Emision best-effort: el spec exige que Atlassian NUNCA rompa el flujo
/// (AC-4). Si algo falla, se avisa por stderr y el comando sigue su curso.
pub fn emit_best_effort(paths: &HarnessPaths, key: &str, origin: &str, kind: IntentKind) {
    if let Err(err) = emit(paths, key, origin, kind) {
        eprintln!("[Atlassian] no se pudo registrar el intent ({key}): {err}");
    }
}

/// Archiva un intent ya ejecutado: sale de `outbox/` y queda en `applied/`.
/// No se borra nada (la outbox es el rastro de lo que se pidio).
pub fn archive(paths: &HarnessPaths, intent: &Intent) -> anyhow::Result<()> {
    let from = outbox_dir(paths).join(intent.file_name());
    let to_dir = applied_dir(paths);
    std::fs::create_dir_all(&to_dir)?;
    let to = to_dir.join(intent.file_name());
    if from.exists() {
        // Rename dentro del mismo arbol; si falla (FS distinto), copia + borra.
        if std::fs::rename(&from, &to).is_err() {
            std::fs::copy(&from, &to)?;
            std::fs::remove_file(&from)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;

    fn paths_in(dir: &std::path::Path) -> HarnessPaths {
        HarnessPaths::from_root(dir.to_path_buf())
    }

    fn feature_kind(fid: &str) -> IntentKind {
        IntentKind::FeatureCreate {
            fid: fid.to_string(),
            name: "demo".to_string(),
            acceptance: vec!["algo verificable".to_string()],
            prd: None,
        }
    }

    #[test]
    fn emit_should_write_one_intent_and_dedupe_the_second() {
        // AC-11: la clave de dedupe es el candado.
        let dir = tempfile::tempdir().unwrap();
        let paths = paths_in(dir.path());
        assert!(emit(&paths, "feature:15:create", "add", feature_kind("15")).unwrap());
        assert!(!emit(&paths, "feature:15:create", "add", feature_kind("15")).unwrap());
        assert_eq!(pending(&paths).len(), 1);
    }

    #[test]
    fn emit_should_skip_when_state_already_has_the_key() {
        // Si el destino remoto ya existe, el intent nace aplicado: no se emite.
        let dir = tempfile::tempdir().unwrap();
        let paths = paths_in(dir.path());
        let mut state = State::default();
        state.mark_applied("feature:15:create");
        state.save(&paths).unwrap();
        assert!(!emit(&paths, "feature:15:create", "add", feature_kind("15")).unwrap());
        assert!(pending(&paths).is_empty());
    }

    #[test]
    fn pending_should_sort_by_dependency_then_id() {
        // AC-9: primero el epic, despues la historia, despues las subtasks,
        // y al final transiciones y comentarios.
        let dir = tempfile::tempdir().unwrap();
        let paths = paths_in(dir.path());
        emit(
            &paths,
            "feature:15:comment:1",
            "advance",
            IntentKind::Comment {
                fid: "15".to_string(),
                body: "nota".to_string(),
            },
        )
        .unwrap();
        emit(
            &paths,
            "ac:15:AC-1",
            "start",
            IntentKind::AcSubtask {
                fid: "15".to_string(),
                ac: "AC-1".to_string(),
                text: "Given ...".to_string(),
            },
        )
        .unwrap();
        emit(&paths, "feature:15:create", "add", feature_kind("15")).unwrap();
        emit(
            &paths,
            "prd:master:epic",
            "add",
            IntentKind::PrdEpic {
                slug: "master".to_string(),
                title: "PRD maestro".to_string(),
                body: String::new(),
            },
        )
        .unwrap();

        let order: Vec<u8> = pending(&paths).iter().map(|i| i.kind.rank()).collect();
        assert_eq!(order, vec![0, 1, 2, 5]);
    }

    #[test]
    fn archive_should_move_the_intent_out_of_the_outbox() {
        let dir = tempfile::tempdir().unwrap();
        let paths = paths_in(dir.path());
        emit(&paths, "feature:15:create", "add", feature_kind("15")).unwrap();
        let intent = pending(&paths).remove(0);
        archive(&paths, &intent).unwrap();
        assert!(pending(&paths).is_empty());
        assert!(applied_dir(&paths).join(intent.file_name()).is_file());
    }

    #[test]
    fn ids_should_not_repeat_after_archiving() {
        // El contador mira pendientes + archivados: un id no se reusa.
        let dir = tempfile::tempdir().unwrap();
        let paths = paths_in(dir.path());
        emit(&paths, "k1", "add", feature_kind("1")).unwrap();
        let first = pending(&paths).remove(0);
        archive(&paths, &first).unwrap();
        emit(&paths, "k2", "add", feature_kind("2")).unwrap();
        let second = pending(&paths).remove(0);
        assert_ne!(first.id, second.id);
    }

    #[test]
    fn emit_best_effort_should_not_panic_on_unwritable_dir() {
        // AC-4: la emision jamas puede romper el comando del flujo.
        let paths = HarnessPaths::from_root(PathBuf::from("/dev/null/no-existe"));
        emit_best_effort(&paths, "feature:1:create", "add", feature_kind("1"));
    }
}
