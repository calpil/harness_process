//! `atlassian.json`: a que proyecto Jira y a que space de Confluence pertenece
//! ESTE repo (spec #15, AC-1..AC-5). Es el interruptor de toda la integracion:
//! sin el archivo no existe ningun camino nuevo.
//!
//! Vive en la raiz del PROYECTO (`repo_root`), es versionable a proposito y
//! jamas lleva credenciales: solo nombres de proyecto y space (Articulo 4).

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::paths::HarnessPaths;

/// Nombre del archivo de binding en la raiz del proyecto.
pub const BINDING_FILE: &str = "atlassian.json";

/// Tipos de issue con los que el arnes representa cada pieza del flujo.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueTypes {
    /// PRD -> epic.
    #[serde(default = "default_epic")]
    pub epic: String,
    /// Feature del backlog -> historia (decision OBS-6: `Story` por default,
    /// unico tipo presente tanto en ADR como en SCRUM; `Feature` es opcional).
    #[serde(default = "default_feature")]
    pub feature: String,
    /// AC-n del spec -> subtask.
    #[serde(default = "default_ac")]
    pub ac: String,
}

fn default_epic() -> String {
    "Epic".to_string()
}

fn default_feature() -> String {
    "Story".to_string()
}

fn default_ac() -> String {
    "Subtask".to_string()
}

impl Default for IssueTypes {
    fn default() -> Self {
        IssueTypes {
            epic: default_epic(),
            feature: default_feature(),
            ac: default_ac(),
        }
    }
}

/// Nombres de estado del board a los que se mapea el estado del backlog.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusMap {
    #[serde(default = "default_todo")]
    pub pending: String,
    #[serde(default = "default_in_progress")]
    pub in_progress: String,
    #[serde(default = "default_done")]
    pub done: String,
    /// Decision OBS-7: `blocked` no tiene columna propia; se marca con el flag
    /// `Impediment` (customfield_10021) dejando la historia donde esta.
    #[serde(default = "default_blocked_flag")]
    pub blocked_flag: String,
}

fn default_todo() -> String {
    "To Do".to_string()
}

fn default_in_progress() -> String {
    "In Progress".to_string()
}

fn default_done() -> String {
    "Done".to_string()
}

fn default_blocked_flag() -> String {
    "Impediment".to_string()
}

impl Default for StatusMap {
    fn default() -> Self {
        StatusMap {
            pending: default_todo(),
            in_progress: default_in_progress(),
            done: default_done(),
            blocked_flag: default_blocked_flag(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct JiraBinding {
    /// Clave del proyecto (`ADR`, `SCRUM`, ...). Sin esto no hay binding.
    pub project_key: String,
    #[serde(default)]
    pub issue_types: IssueTypes,
    #[serde(default)]
    pub statuses: StatusMap,
    /// Id del board del proyecto, si ya se resolvio (Agile API).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub board_id: Option<i64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConfluenceBinding {
    /// Clave del space (`SD`, `~712020...`). Sin esto no se publica nada.
    pub space_key: String,
    /// Id numerico del space, si ya se resolvio.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub space_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Binding {
    /// Host del sitio (`calpil.atlassian.net`), sin esquema.
    pub site: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cloud_id: Option<String>,
    /// Interruptor: `false` apaga la integracion sin perder el mapeo.
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub jira: JiraBinding,
    #[serde(default)]
    pub confluence: ConfluenceBinding,
}

fn default_true() -> bool {
    true
}

impl Binding {
    /// Ruta del binding para una instalacion dada.
    pub fn path(paths: &HarnessPaths) -> PathBuf {
        paths.repo_root.join(BINDING_FILE)
    }

    /// Lee el binding si existe. Un archivo ilegible o corrupto NO es un error
    /// fatal para el flujo: se trata como "sin binding" y el comando sigue
    /// (AC-4); solo los subcomandos `atlassian` lo reportan.
    pub fn load(paths: &HarnessPaths) -> Option<Binding> {
        Self::load_from(&Self::path(paths))
    }

    pub fn load_from(file: &Path) -> Option<Binding> {
        let text = std::fs::read_to_string(file).ok()?;
        serde_json::from_str(&text).ok()
    }

    /// Lee el binding solo si esta activo (interruptor del spec).
    pub fn load_active(paths: &HarnessPaths) -> Option<Binding> {
        Self::load(paths).filter(|b| b.is_active())
    }

    /// Un binding sirve cuando esta habilitado y sabe a que proyecto pertenece.
    pub fn is_active(&self) -> bool {
        self.enabled && !self.jira.project_key.trim().is_empty()
    }

    pub fn save(&self, paths: &HarnessPaths) -> anyhow::Result<()> {
        let text = format!("{}\n", serde_json::to_string_pretty(self)?);
        crate::features::write_text_atomic(&Self::path(paths), &text)
    }

    /// URL base del sitio, siempre HTTPS (nunca hay fallback a HTTP).
    pub fn base_url(&self) -> String {
        let host = self
            .site
            .trim()
            .trim_start_matches("https://")
            .trim_start_matches("http://")
            .trim_end_matches('/');
        format!("https://{host}")
    }

    /// Enlace navegable a un issue (para los enlaces cruzados con Confluence).
    pub fn browse_url(&self, key: &str) -> String {
        format!("{}/browse/{}", self.base_url(), key)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;

    fn paths_in(dir: &Path) -> HarnessPaths {
        HarnessPaths::from_root(dir.to_path_buf())
    }

    #[test]
    fn load_should_return_none_without_file() {
        // AC-4: un repo sin binding no tiene integracion y no falla.
        let dir = tempfile::tempdir().unwrap();
        assert!(Binding::load(&paths_in(dir.path())).is_none());
    }

    #[test]
    fn load_should_return_none_for_corrupt_file() {
        // Un binding roto NO puede romper el flujo: se lee como ausente.
        let dir = tempfile::tempdir().unwrap();
        let paths = paths_in(dir.path());
        std::fs::write(Binding::path(&paths), "{ esto no es json").unwrap();
        assert!(Binding::load(&paths).is_none());
    }

    #[test]
    fn save_and_load_should_roundtrip_with_defaults() {
        let dir = tempfile::tempdir().unwrap();
        let paths = paths_in(dir.path());
        let binding = Binding {
            site: "calpil.atlassian.net".to_string(),
            cloud_id: None,
            enabled: true,
            jira: JiraBinding {
                project_key: "ADR".to_string(),
                ..Default::default()
            },
            confluence: ConfluenceBinding {
                space_key: "SD".to_string(),
                space_id: None,
            },
        };
        binding.save(&paths).unwrap();
        let back = Binding::load(&paths).unwrap();
        assert_eq!(back.jira.project_key, "ADR");
        assert_eq!(back.confluence.space_key, "SD");
        // Decision OBS-6: `Story` por default.
        assert_eq!(back.jira.issue_types.feature, "Story");
        assert_eq!(back.jira.issue_types.epic, "Epic");
        assert_eq!(back.jira.issue_types.ac, "Subtask");
        // Decision OBS-7: blocked se marca con el flag Impediment.
        assert_eq!(back.jira.statuses.blocked_flag, "Impediment");
        assert!(back.is_active());
    }

    #[test]
    fn disabled_or_projectless_binding_should_not_be_active() {
        let mut binding = Binding {
            site: "calpil.atlassian.net".to_string(),
            cloud_id: None,
            enabled: false,
            jira: JiraBinding {
                project_key: "ADR".to_string(),
                ..Default::default()
            },
            confluence: ConfluenceBinding::default(),
        };
        assert!(!binding.is_active(), "enabled=false apaga la integracion");
        binding.enabled = true;
        binding.jira.project_key = "   ".to_string();
        assert!(!binding.is_active(), "sin proyecto no hay binding");
    }

    #[test]
    fn base_url_should_normalize_scheme_and_slash() {
        let binding = Binding {
            site: "https://calpil.atlassian.net/".to_string(),
            cloud_id: None,
            enabled: true,
            jira: JiraBinding::default(),
            confluence: ConfluenceBinding::default(),
        };
        assert_eq!(binding.base_url(), "https://calpil.atlassian.net");
        assert_eq!(
            binding.browse_url("ADR-42"),
            "https://calpil.atlassian.net/browse/ADR-42"
        );
    }
}
