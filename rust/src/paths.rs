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
    /// Worktree secundario desde el que se invoco el arnes (feature #47), o
    /// `None` cuando se trabaja en el checkout principal.
    pub worktree: Option<PathBuf>,
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

    /// Rutas vistas DESDE una feature concreta: `plans` (docs/) apunta al
    /// worktree de ESA feature, para que su spec, su plan y su evidencia vivan
    /// en su rama y viajen con el merge (feature #47).
    ///
    /// Sin worktree (modo clasico o repo sin git) devuelve las rutas de
    /// siempre. El ESTADO (`feature_list.json`, `progress/`) no cambia nunca:
    /// es unico y del repo principal (AC-7).
    pub fn para_feature(&self, feature: &serde_json::Map<String, serde_json::Value>) -> HarnessPaths {
        let plans = feature
            .get("worktree")
            .and_then(serde_json::Value::as_str)
            .map(|wt| PathBuf::from(wt).join("docs"))
            .filter(|docs| docs.parent().is_some_and(Path::exists))
            .unwrap_or_else(|| self.plans.clone());
        HarnessPaths {
            root: self.root.clone(),
            features: self.features.clone(),
            progress: self.progress.clone(),
            current: self.current.clone(),
            history: self.history.clone(),
            repo_root: self.repo_root.clone(),
            plans,
            autocheck_stamp: self.autocheck_stamp.clone(),
            nudge_stamp: self.nudge_stamp.clone(),
            nudge_lecciones: self.nudge_lecciones.clone(),
            worktree: self.worktree.clone(),
        }
    }

    /// Directorio desde el que se EJECUTAN los comandos de una feature
    /// (feature #57): su worktree cuando lo tiene, la raiz de siempre cuando no.
    ///
    /// Es la contracara de `para_feature`. Esa resuelve donde se LEEN y se
    /// ESCRIBEN los documentos de la feature; esta, donde corre el codigo que
    /// esos documentos dicen verificar. Que las dos existieran pero solo una
    /// tuviera nombre fue el bug: `verify` leia el spec del worktree y corria
    /// `cargo test` en el checkout principal, donde el codigo de la feature
    /// todavia no existe. No fallaba —eso habria sido facil de ver—: salia
    /// VERDE habiendo ejecutado cero casos, que es el peor resultado posible de
    /// un gate. Se descubrio cerrando la feature #56.
    ///
    /// Un `worktree` anotado que ya no esta en el disco cae a la raiz: la
    /// alternativa seria correr en un directorio inexistente y traducir eso a
    /// un rojo que no dice nada del codigo.
    pub fn raiz_de_ejecucion(
        &self,
        feature: &serde_json::Map<String, serde_json::Value>,
    ) -> PathBuf {
        feature
            .get("worktree")
            .and_then(serde_json::Value::as_str)
            .map(PathBuf::from)
            .filter(|wt| wt.is_dir())
            .unwrap_or_else(|| self.repo_root.clone())
    }

    /// Estado vivo de UNA feature (feature #47 / AC-8): cada una escribe el
    /// suyo, asi que cerrar una no puede pisar el de otra.
    pub fn current_de(&self, fid: &str) -> PathBuf {
        self.progress.join(format!("current-{fid}.md"))
    }

    /// Stamp de autocheck por feature (AC-10).
    pub fn autocheck_stamp_de(&self, fid: &str) -> PathBuf {
        self.progress.join(format!(".last_autocheck-{fid}"))
    }

    pub fn from_root(root: PathBuf) -> Self {
        // Paridad harness.py: el valor del env NO se normaliza con abspath.
        let repo_root = match env_nonempty("HARNESS_REPO_ROOT") {
            Some(v) => PathBuf::from(v),
            None => repo_root_from_marker(&root),
        };
        let progress = root.join("progress");
        // Feature #47: el ESTADO (feature_list.json, progress/) es unico y vive
        // en el checkout principal — esta gitignorado, no viaja con las ramas —
        // pero los DOCS (spec, plan, impl, review) son archivos versionados que
        // tienen que quedar en la rama de la feature. Por eso, cuando se invoca
        // desde un worktree, `plans` apunta al docs/ de ESE worktree (AC-7).
        let worktree = worktree_actual(&repo_root);
        let plans = match &worktree {
            Some(wt) => wt.join("docs"),
            None => repo_root.join("docs"),
        };
        HarnessPaths {
            features: root.join("feature_list.json"),
            current: progress.join("current.md"),
            history: progress.join("history.md"),
            autocheck_stamp: progress.join(".last_autocheck"),
            nudge_stamp: progress.join(".last_nudge"),
            nudge_lecciones: progress.join(".nudge_lecciones"),
            plans,
            progress,
            repo_root,
            root,
            worktree,
        }
    }
}

/// Worktree secundario desde el que se esta invocando el arnes, si lo hay.
///
/// Se mira el directorio ACTUAL: es el que dice en que feature esta trabajando
/// quien corre el comando (AC-12). Un worktree cuyo repo principal es el mismo
/// `repo_root` no cuenta: ese es el checkout de siempre.
fn worktree_actual(repo_root: &Path) -> Option<PathBuf> {
    // Override explicito para tests y para casos raros.
    if let Some(v) = env_nonempty("HARNESS_WORKTREE") {
        return Some(PathBuf::from(v));
    }
    let cwd = std::env::current_dir().ok()?;
    if !crate::git::es_worktree_secundario(&cwd) {
        return None;
    }
    let top = crate::git::toplevel(&cwd)?;
    // Si el "worktree" resulta ser la propia raiz, no hay nada que redirigir.
    if top == repo_root {
        return None;
    }
    // El worktree tiene que ser DEL MISMO repo que este arnes. Sin esta
    // comprobacion, correr el binario de un proyecto parado en el worktree de
    // OTRO repo le desvia los docs a un arbol ajeno — paso de verdad al correr
    // la suite de tests desde un worktree: los sandboxes escribieron sus specs
    // en el docs/ del worktree real (feature #51).
    let mismo_repo = crate::git::repo_principal(&cwd)
        .zip(crate::git::repo_principal(repo_root))
        .is_some_and(|(a, b)| mismo_dir(&a, &b));
    if !mismo_repo {
        return None;
    }
    Some(top)
}

/// Compara dos rutas por identidad real, con fallback lexico.
fn mismo_dir(a: &Path, b: &Path) -> bool {
    match (std::fs::canonicalize(a), std::fs::canonicalize(b)) {
        (Ok(ca), Ok(cb)) => ca == cb,
        _ => a == b,
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
///
/// En Windows el binario y los scripts no coincidian sobre que es "$HOME":
/// `home_dir()` mira `USERPROFILE` y los cuatro scripts sh miran `HOME`, que en
/// Git Bash puede ser otro directorio. Con las dos mitades del arnes en
/// desacuerdo, la misma instalacion resolvia la raiz distinto segun quien
/// preguntara, y la guarda —que existe para no sembrar el arnes sobre la
/// carpeta del usuario— se saltaba en el lado que no miraba. Aca se aceptan
/// LAS DOS: la guarda protege de mas, nunca de menos, que es lo que
/// corresponde a un chequeo de seguridad.
fn parent_is_home(parent: &Path) -> bool {
    if env_nonempty("HARNESS_ALLOW_HOME_SURFACE").as_deref() == Some("1") {
        return false;
    }
    let candidatos = [
        crate::pycompat::home_dir(),
        env_nonempty("HOME").map(PathBuf::from),
    ];
    candidatos
        .iter()
        .flatten()
        .any(|home| same_dir(parent, home))
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
    use serde_json::{Map, Value};

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

    /// Feature #57: donde CORREN los comandos de una feature.
    fn paths_con_worktree(worktree: Option<&Path>) -> (tempfile::TempDir, HarnessPaths, Map<String, Value>) {
        let dir = tempfile::tempdir().unwrap();
        let harness = dir.path().join("harness_process");
        std::fs::create_dir_all(&harness).unwrap();
        let paths = HarnessPaths::from_root(harness);
        let mut feature = Map::new();
        if let Some(wt) = worktree {
            feature.insert(
                "worktree".to_string(),
                Value::String(wt.to_string_lossy().into_owned()),
            );
        }
        (dir, paths, feature)
    }

    #[test]
    fn raiz_de_ejecucion_should_be_the_worktree_of_the_feature() {
        let base = tempfile::tempdir().unwrap();
        let wt = base.path().join("repo-wt/57-demo");
        std::fs::create_dir_all(&wt).unwrap();
        let (_dir, paths, feature) = paths_con_worktree(Some(&wt));
        assert_eq!(paths.raiz_de_ejecucion(&feature), wt);
    }

    #[test]
    fn raiz_de_ejecucion_should_be_the_root_without_worktree() {
        // Modo clasico (sin la #47) o repo sin git: la raiz de siempre.
        let (_dir, paths, feature) = paths_con_worktree(None);
        assert_eq!(paths.raiz_de_ejecucion(&feature), paths.repo_root);
    }

    #[test]
    fn raiz_de_ejecucion_should_fall_back_when_the_worktree_is_gone() {
        // Un worktree anotado que alguien borro a mano. Correr en un directorio
        // que no existe daria un rojo que no habla del codigo, sino del disco.
        let base = tempfile::tempdir().unwrap();
        let (_dir, paths, feature) = paths_con_worktree(Some(&base.path().join("no-esta")));
        assert_eq!(paths.raiz_de_ejecucion(&feature), paths.repo_root);
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
