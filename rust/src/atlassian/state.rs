//! `progress/atlassian/state.json`: el mapa local -> remoto (spec #15).
//!
//! Es el UNICO lugar donde se escriben claves de Jira e ids de Confluence, y
//! por eso tambien es el candado de idempotencia: si una clave de dedupe ya
//! tiene destino remoto, el intent correspondiente nace aplicado (AC-11).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::paths::HarnessPaths;

/// Directorio de la integracion dentro de `progress/`.
pub fn atlassian_dir(paths: &HarnessPaths) -> PathBuf {
    paths.progress.join("atlassian")
}

pub fn state_path(paths: &HarnessPaths) -> PathBuf {
    atlassian_dir(paths).join("state.json")
}

/// Mapeo de una feature del backlog a su historia y sus subtasks.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FeatureRemote {
    /// Clave de la historia (`ADR-42`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub issue: Option<String>,
    /// Clave de cada subtask por AC-n.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub acs: BTreeMap<String, String>,
}

/// Pagina publicada en Confluence: id, version y hash del contenido con el que
/// se publico (AC-23: sin cambios no se crea version nueva).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PageRemote {
    pub id: String,
    #[serde(default)]
    pub version: i64,
    #[serde(default)]
    pub hash: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

/// Sprint vigente (AC-19/AC-20).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SprintRemote {
    pub id: i64,
    pub name: String,
    #[serde(default)]
    pub board_id: i64,
    #[serde(default)]
    pub state: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct State {
    #[serde(default = "default_version")]
    pub version: u32,
    /// feature id (como texto) -> destino remoto.
    #[serde(default)]
    pub features: BTreeMap<String, FeatureRemote>,
    /// slug de PRD -> clave del epic.
    #[serde(default)]
    pub prds: BTreeMap<String, String>,
    /// ruta del documento (relativa al repo) -> pagina publicada.
    #[serde(default)]
    pub pages: BTreeMap<String, PageRemote>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sprint: Option<SprintRemote>,
    /// Claves de dedupe ya satisfechas.
    #[serde(default)]
    pub applied: Vec<String>,
}

fn default_version() -> u32 {
    1
}

impl State {
    pub fn load(paths: &HarnessPaths) -> State {
        Self::load_from(&state_path(paths))
    }

    pub fn load_from(file: &Path) -> State {
        std::fs::read_to_string(file)
            .ok()
            .and_then(|t| serde_json::from_str(&t).ok())
            .unwrap_or_else(|| State {
                version: default_version(),
                ..Default::default()
            })
    }

    pub fn save(&self, paths: &HarnessPaths) -> anyhow::Result<()> {
        std::fs::create_dir_all(atlassian_dir(paths))?;
        let text = format!("{}\n", serde_json::to_string_pretty(self)?);
        crate::features::write_text_atomic(&state_path(paths), &text)
    }

    /// True si esa clave de dedupe ya fue satisfecha.
    pub fn is_applied(&self, key: &str) -> bool {
        self.applied.iter().any(|k| k == key)
    }

    /// Marca una clave como aplicada (idempotente: repetir es inofensivo).
    pub fn mark_applied(&mut self, key: &str) {
        if !self.is_applied(key) {
            self.applied.push(key.to_string());
        }
    }

    /// Clave de la historia de una feature, si ya existe.
    pub fn feature_issue(&self, fid: &str) -> Option<&str> {
        self.features.get(fid).and_then(|f| f.issue.as_deref())
    }

    pub fn set_feature_issue(&mut self, fid: &str, key: &str) {
        self.features
            .entry(fid.to_string())
            .or_default()
            .issue = Some(key.to_string());
    }

    pub fn set_ac_issue(&mut self, fid: &str, ac: &str, key: &str) {
        self.features
            .entry(fid.to_string())
            .or_default()
            .acs
            .insert(ac.to_string(), key.to_string());
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;

    #[test]
    fn load_should_give_empty_state_without_file() {
        let dir = tempfile::tempdir().unwrap();
        let paths = HarnessPaths::from_root(dir.path().to_path_buf());
        let state = State::load(&paths);
        assert_eq!(state.version, 1);
        assert!(state.features.is_empty());
        assert!(state.applied.is_empty());
    }

    #[test]
    fn mark_applied_should_be_idempotent() {
        // AC-11: la clave de dedupe es el candado; repetirla no duplica nada.
        let mut state = State::default();
        assert!(!state.is_applied("feature:15:create"));
        state.mark_applied("feature:15:create");
        state.mark_applied("feature:15:create");
        assert!(state.is_applied("feature:15:create"));
        assert_eq!(state.applied.len(), 1);
    }

    #[test]
    fn save_and_load_should_roundtrip_the_map() {
        let dir = tempfile::tempdir().unwrap();
        let paths = HarnessPaths::from_root(dir.path().to_path_buf());
        let mut state = State::default();
        state.set_feature_issue("15", "ADR-42");
        state.set_ac_issue("15", "AC-1", "ADR-43");
        state.mark_applied("feature:15:create");
        state.sprint = Some(SprintRemote {
            id: 7,
            name: "Sprint 1".to_string(),
            board_id: 3,
            state: "active".to_string(),
        });
        state.save(&paths).unwrap();

        let back = State::load(&paths);
        assert_eq!(back.feature_issue("15"), Some("ADR-42"));
        assert_eq!(
            back.features.get("15").and_then(|f| f.acs.get("AC-1")),
            Some(&"ADR-43".to_string())
        );
        assert!(back.is_applied("feature:15:create"));
        assert_eq!(back.sprint.map(|s| s.id), Some(7));
    }
}
