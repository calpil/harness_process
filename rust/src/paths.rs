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
    /// Contador de invocaciones del nudge por feature (`<id>:<n>`), feature #18.
    pub nudge_lecciones: PathBuf,
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
            nudge_lecciones: progress.join(".nudge_lecciones"),
            plans: repo_root.join("docs"),
            progress,
            repo_root,
            root,
        }
    }
}

/// Rutas que siembra el instalador en la raiz de una instalacion: su presencia
/// en el padre es la "huella de instalacion". Mismas cuatro que usan
/// harness_check.sh / harness_status.sh / init.sh / commit_guard.sh.
const INSTALL_FOOTPRINTS: [&str; 4] = [
    "docs/constitution.md",
    "CLAUDE.md",
    "AGENTS.md",
    ".claude/settings.json",
];

/// Resuelve la raiz multi-repo segun `.harness_layout`. Tres casos EXCLUYENTES
/// (feature #10), identicos a los de los 4 scripts sh:
///
/// 1. marker == "subdir" -> el padre de `root`, salvo el guardrail de checkout
///    fuente (feature #7, decision usuario 2026-07-28): un clon de la fuente es
///    identico a una instalacion subdir y solo el ENTORNO los distingue; con
///    senales de fuente en `root` (`templates/harness_cli` + `rust/`) y un
///    padre sin huella de instalacion (o `$HOME` sin
///    `HARNESS_ALLOW_HOME_SURFACE=1`) el marker es incoherente y la raiz es el
///    propio arnes, con aviso `[i]`.
/// 2. marker AUSENTE (decision usuario 2026-07-29) -> la feature #7
///    des-versiono el marker, asi que toda instalacion subdir que hizo
///    `git pull` se quedo sin el. Si el padre tiene huella de instalacion (y no
///    es `$HOME`), se infiere layout subdir y la raiz es el padre, con aviso
///    `[i]`; sin huella no hay evidencia para inferir nada.
/// 3. marker presente con cualquier otro valor ("root") -> se respeta tal cual:
///    la raiz es `root`, sin inferencia y sin aviso.
pub fn repo_root_from_marker(root: &Path) -> PathBuf {
    let marker = root.join(".harness_layout");
    if marker.is_file() {
        // Marker EXPLICITO: manda tal cual (nunca se infiere sobre el).
        let content = std::fs::read_to_string(&marker).unwrap_or_default();
        if content.trim() == "subdir" {
            if let Some(parent) = non_empty_parent(root) {
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
        return root.to_path_buf();
    }
    // Marker AUSENTE: inferencia por huella del padre.
    if let Some(parent) = non_empty_parent(root) {
        if parent_has_footprint(parent) && !parent_is_home(parent) {
            eprintln!(
                "[i] .harness_layout ausente: layout subdir inferido por la huella de instalacion del padre: REPO_ROOT={}. Re-corre el instalador (setup_harness.sh / setup_harness.ps1) para regenerar el marker.",
                parent.display()
            );
            return parent.to_path_buf();
        }
    }
    root.to_path_buf()
}

/// Padre de `root` descartando el padre vacio de una ruta relativa de un solo
/// componente (`Path::new("harness").parent() == Some("")`), que no designa
/// ningun directorio utilizable como raiz.
fn non_empty_parent(root: &Path) -> Option<&Path> {
    root.parent().filter(|p| !p.as_os_str().is_empty())
}

/// True si `parent` tiene al menos una huella de instalacion del arnes.
fn parent_has_footprint(parent: &Path) -> bool {
    INSTALL_FOOTPRINTS
        .iter()
        .any(|fp| parent.join(fp).is_file())
}

/// True si `parent` es `$HOME` y no esta el escape
/// `HARNESS_ALLOW_HOME_SURFACE=1` (paridad con la guarda del instalador).
fn parent_is_home(parent: &Path) -> bool {
    env_nonempty("HARNESS_ALLOW_HOME_SURFACE").as_deref() != Some("1")
        && crate::pycompat::home_dir().is_some_and(|home| same_dir(parent, &home))
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
    !parent_has_footprint(parent) || parent_is_home(parent)
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
        // El padre es un tempdir VACIO (sin huella de instalacion): sin marker
        // y sin evidencia, la raiz es el propio dir del arnes.
        let dir = tempfile::tempdir().unwrap();
        let harness = dir.path().join("harness_process");
        std::fs::create_dir(&harness).unwrap();
        assert_eq!(repo_root_from_marker(&harness), harness);
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

    /// Arma `<parent>/harness_process` SIN marker (el estado en que queda una
    /// instalacion subdir tras el `git pull` que borro `.harness_layout`).
    fn lost_marker_fixture(parent: &Path) -> PathBuf {
        let harness = parent.join("harness_process");
        std::fs::create_dir_all(&harness).unwrap();
        assert!(!harness.join(".harness_layout").exists());
        harness
    }

    #[test]
    fn repo_root_should_infer_subdir_without_marker_when_parent_has_footprint() {
        // Feature #10 / AC-1: sin marker pero con huella de instalacion en el
        // padre, la raiz es el PADRE (el proyecto), no el dir del arnes.
        let dir = tempfile::tempdir().unwrap();
        let harness = lost_marker_fixture(dir.path());
        std::fs::write(dir.path().join("CLAUDE.md"), "# proyecto\n").unwrap();
        assert_eq!(repo_root_from_marker(&harness), dir.path());
    }

    #[test]
    fn repo_root_inference_should_accept_any_single_footprint_file() {
        // La inferencia usa las MISMAS cuatro huellas que el guardrail: basta
        // cualquiera de ellas (aqui la que no es un .md de la raiz).
        let dir = tempfile::tempdir().unwrap();
        let harness = lost_marker_fixture(dir.path());
        std::fs::create_dir_all(dir.path().join(".claude")).unwrap();
        std::fs::write(dir.path().join(".claude/settings.json"), "{}\n").unwrap();
        assert_eq!(repo_root_from_marker(&harness), dir.path());
        // docs/constitution.md tambien, en una fixture limpia.
        let dir2 = tempfile::tempdir().unwrap();
        let harness2 = lost_marker_fixture(dir2.path());
        std::fs::create_dir_all(dir2.path().join("docs")).unwrap();
        std::fs::write(dir2.path().join("docs/constitution.md"), "# c\n").unwrap();
        assert_eq!(repo_root_from_marker(&harness2), dir2.path());
    }

    #[test]
    fn repo_root_should_not_infer_without_parent_footprint() {
        // Feature #10 / AC-4: sin marker y sin huella no hay evidencia para
        // inferir; la raiz sigue siendo el propio dir del arnes.
        let dir = tempfile::tempdir().unwrap();
        let harness = lost_marker_fixture(dir.path());
        std::fs::write(dir.path().join("README.md"), "# no es huella\n").unwrap();
        assert_eq!(repo_root_from_marker(&harness), harness);
    }

    #[test]
    fn repo_root_should_not_infer_when_marker_says_root() {
        // Feature #10 / AC-3: un marker EXPLICITO distinto de 'subdir' manda
        // aunque el padre tenga huella: sin inferencia, la raiz es el arnes.
        let dir = tempfile::tempdir().unwrap();
        let harness = lost_marker_fixture(dir.path());
        std::fs::write(harness.join(".harness_layout"), "root\n").unwrap();
        std::fs::write(dir.path().join("CLAUDE.md"), "# proyecto\n").unwrap();
        std::fs::create_dir_all(dir.path().join("docs")).unwrap();
        std::fs::write(dir.path().join("docs/constitution.md"), "# c\n").unwrap();
        assert_eq!(repo_root_from_marker(&harness), harness);
    }

    #[test]
    fn repo_root_should_not_infer_when_marker_is_empty_or_unknown() {
        // Mismo caso AC-3 con valores raros: el archivo EXISTE, asi que se
        // respeta como marker explicito (nunca se infiere sobre el).
        for value in ["", "\n", "  ", "flat", "subdirectorio"] {
            let dir = tempfile::tempdir().unwrap();
            let harness = lost_marker_fixture(dir.path());
            std::fs::write(harness.join(".harness_layout"), value).unwrap();
            std::fs::write(dir.path().join("AGENTS.md"), "# proyecto\n").unwrap();
            assert_eq!(repo_root_from_marker(&harness), harness, "valor {value:?}");
        }
    }

    #[test]
    fn repo_root_should_not_infer_without_usable_parent() {
        // Ruta relativa de un solo componente: Path::parent() devuelve "" (no
        // designa directorio alguno) y la inferencia no aplica.
        let root = Path::new("harness_process_inexistente_para_test");
        assert_eq!(repo_root_from_marker(root), root);
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
