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
///
/// Guardrail checkout fuente (decision usuario 2026-07-28): un clon de la
/// fuente es identico a una instalacion subdir; solo el ENTORNO los
/// distingue. Con senales de fuente en `root` (`templates/harness_cli` +
/// `rust/`) y un padre sin huella de instalacion (o `$HOME` sin
/// `HARNESS_ALLOW_HOME_SURFACE=1`), el marker 'subdir' es incoherente:
/// fallback al propio arnes con aviso informativo `[i]` (ni fallo duro ni
/// silencioso). Misma regla que harness_check.sh / harness_status.sh /
/// init.sh / commit_guard.sh.
pub fn repo_root_from_marker(root: &Path) -> PathBuf {
    let marker = root.join(".harness_layout");
    if let Ok(content) = std::fs::read_to_string(&marker) {
        if content.trim() == "subdir" {
            if let Some(parent) = root.parent() {
                if source_checkout_mismatch(root, parent) {
                    eprintln!(
                        "[i] Checkout fuente del arnes detectado (.harness_layout=subdir sin huella de instalacion en el padre): REPO_ROOT={}",
                        root.display()
                    );
                    return root.to_path_buf();
                }
                return parent.to_path_buf();
            }
        }
    }
    root.to_path_buf()
}

/// True si `root` parece el checkout FUENTE del arnes y `parent` no parece la
/// raiz de una instalacion subdir legitima (sin huella de instalacion, o es
/// `$HOME` sin el escape `HARNESS_ALLOW_HOME_SURFACE=1`).
fn source_checkout_mismatch(root: &Path, parent: &Path) -> bool {
    // Senales de fuente en el propio dir (las mismas que usa el instalador
    // para ASSET_DIR y las que exige el smoke del clon simulado).
    if !(root.join("templates/harness_cli").is_file() && root.join("rust").is_dir()) {
        return false;
    }
    let footprints = [
        "docs/constitution.md",
        "CLAUDE.md",
        "AGENTS.md",
        ".claude/settings.json",
    ];
    let parent_has_footprint = footprints.iter().any(|fp| parent.join(fp).is_file());
    let parent_is_home = env_nonempty("HARNESS_ALLOW_HOME_SURFACE").as_deref() != Some("1")
        && crate::pycompat::home_dir().is_some_and(|home| same_dir(parent, &home));
    !parent_has_footprint || parent_is_home
}

/// Compara dos directorios por identidad real (canonicalize) con fallback a
/// igualdad lexica si alguno no se puede resolver.
fn same_dir(a: &Path, b: &Path) -> bool {
    match (std::fs::canonicalize(a), std::fs::canonicalize(b)) {
        (Ok(ca), Ok(cb)) => ca == cb,
        _ => a == b,
    }
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

    /// Arma `<parent>/harness_process` con marker subdir y senales de FUENTE
    /// (templates/harness_cli + rust/), como un clon del repo del arnes.
    fn source_checkout_fixture(parent: &Path) -> PathBuf {
        let harness = parent.join("harness_process");
        std::fs::create_dir_all(harness.join("templates")).unwrap();
        std::fs::create_dir_all(harness.join("rust")).unwrap();
        std::fs::write(harness.join("templates/harness_cli"), "#!/bin/sh\n").unwrap();
        std::fs::write(harness.join(".harness_layout"), "subdir\n").unwrap();
        harness
    }

    #[test]
    fn repo_root_should_stay_local_for_source_checkout_without_parent_footprint() {
        // Marker subdir + senales de fuente, padre SIN huella de instalacion:
        // la raiz es el propio checkout (no el padre).
        let dir = tempfile::tempdir().unwrap();
        let harness = source_checkout_fixture(dir.path());
        assert_eq!(repo_root_from_marker(&harness), harness);
    }

    #[test]
    fn repo_root_should_resolve_parent_for_subdir_install_with_footprint() {
        // La MISMA fixture con huella de instalacion en el padre es una
        // instalacion subdir legitima: cero regresion, la raiz es el padre.
        let dir = tempfile::tempdir().unwrap();
        let harness = source_checkout_fixture(dir.path());
        std::fs::write(dir.path().join("CLAUDE.md"), "# proyecto\n").unwrap();
        assert_eq!(repo_root_from_marker(&harness), dir.path());
    }

    #[test]
    fn repo_root_should_accept_any_single_footprint_file() {
        // Cualquiera de las 4 huellas basta para tratar al padre como raiz.
        let dir = tempfile::tempdir().unwrap();
        let harness = source_checkout_fixture(dir.path());
        std::fs::create_dir_all(dir.path().join(".claude")).unwrap();
        std::fs::write(dir.path().join(".claude/settings.json"), "{}\n").unwrap();
        assert_eq!(repo_root_from_marker(&harness), dir.path());
    }

    #[test]
    fn repo_root_should_resolve_parent_without_source_signals() {
        // Marker subdir SIN senales de fuente (instalacion recortada, o la
        // fixture historica de los tests de integracion): comportamiento de
        // siempre, la raiz es el padre aunque no haya huella.
        let dir = tempfile::tempdir().unwrap();
        let harness = dir.path().join("harness_process");
        std::fs::create_dir_all(&harness).unwrap();
        std::fs::write(harness.join(".harness_layout"), "subdir\n").unwrap();
        assert_eq!(repo_root_from_marker(&harness), dir.path());
    }
}
