//! Resolucion de rutas con la MISMA semantica que los modulos Python:
//! - harness.py: REPO_ROOT = env HARNESS_REPO_ROOT *verbatim* (sin abspath)
//!   o el padre si `.harness_layout` == "subdir".
//! - graph_memory.py: REPO_ROOT = abspath(env) o el marker (ver graph::GraphEnv).

use std::path::{Path, PathBuf};

use anyhow::Context;

use crate::pycompat::env_nonempty;

/// Rutas del lado harness.py (ROOT = directorio del ejecutable, equivalente a
/// `os.path.dirname(os.path.abspath(__file__))`).
pub struct HarnessPaths {
    pub root: PathBuf,
    pub features: PathBuf,
    pub progress: PathBuf,
    pub current: PathBuf,
    pub history: PathBuf,
    pub repo_root: PathBuf,
    pub plans: PathBuf,
    pub autocheck_stamp: PathBuf,
    pub nudge_stamp: PathBuf,
}

impl HarnessPaths {
    pub fn resolve() -> anyhow::Result<Self> {
        let exe = std::env::current_exe().context("no se pudo resolver el ejecutable")?;
        let root = exe
            .parent()
            .context("el ejecutable no tiene directorio padre")?
            .to_path_buf();
        Ok(Self::from_root(root))
    }

    pub fn from_root(root: PathBuf) -> Self {
        // Paridad harness.py: el valor del env NO se normaliza con abspath.
        let repo_root = match env_nonempty("HARNESS_REPO_ROOT") {
            Some(v) => PathBuf::from(v),
            None => repo_root_from_marker(&root),
        };
        let progress = root.join("progress");
        HarnessPaths {
            features: root.join("feature_list.json"),
            current: progress.join("current.md"),
            history: progress.join("history.md"),
            autocheck_stamp: progress.join(".last_autocheck"),
            nudge_stamp: progress.join(".last_nudge"),
            plans: repo_root.join("docs"),
            progress,
            repo_root,
            root,
        }
    }
}

/// Lee `.harness_layout`: "subdir" -> el padre de `root` es la raiz multi-repo.
pub fn repo_root_from_marker(root: &Path) -> PathBuf {
    let marker = root.join(".harness_layout");
    if let Ok(content) = std::fs::read_to_string(&marker) {
        if content.trim() == "subdir" {
            if let Some(parent) = root.parent() {
                return parent.to_path_buf();
            }
        }
    }
    root.to_path_buf()
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;

    #[test]
    fn repo_root_should_be_root_without_marker() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(repo_root_from_marker(dir.path()), dir.path());
    }

    #[test]
    fn repo_root_should_be_parent_with_subdir_marker() {
        let dir = tempfile::tempdir().unwrap();
        let harness = dir.path().join("harness_process");
        std::fs::create_dir(&harness).unwrap();
        std::fs::write(harness.join(".harness_layout"), "subdir\n").unwrap();
        assert_eq!(repo_root_from_marker(&harness), dir.path());
    }

    #[test]
    fn repo_root_should_ignore_other_marker_values() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join(".harness_layout"), "root").unwrap();
        assert_eq!(repo_root_from_marker(dir.path()), dir.path());
    }
}
