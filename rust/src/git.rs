//! Operaciones git del arnes (feature #47): ramas y worktrees por feature,
//! integracion GitFlow al cerrar.
//!
//! Dos reglas que valen para todo este modulo:
//!
//! - **Nunca reescribe historia ni fuerza nada**: sin `--force`, sin rebase,
//!   sin squash, sin borrar ramas. Un conflicto ABORTA y deja el repo como
//!   estaba (Articulo 4 / AC-18).
//! - **Los commits del arnes no llevan trailers de IA** (AC-16): lo exige
//!   `UPDATING.md` y lo verifica `commit_guard.sh`.
//!
//! Si el directorio no es un repo git, todas las consultas devuelven `None` y
//! el flujo sigue como siempre: el aislamiento es una mejora, no un requisito
//! (AC-5).

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Prefijos GitFlow por defecto.
pub const PREFIJO_FEATURE: &str = "feature/";
pub const PREFIJO_BUGFIX: &str = "bugfix/";
/// Ramas base candidatas, en orden: el arnes usa la primera que exista y
/// NUNCA crea ninguna (AC-22).
pub const BASES: [&str; 2] = ["develop", "main"];

/// Corre git en un directorio y devuelve stdout si el comando salio 0.
fn git(dir: &Path, args: &[&str]) -> Option<String> {
    let salida = Command::new("git")
        .args(args)
        .current_dir(dir)
        .stdin(Stdio::null())
        .output()
        .ok()?;
    if !salida.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&salida.stdout).trim().to_string())
}

/// Corre git devolviendo el error legible (stderr) cuando falla: lo usan las
/// operaciones que mutan, porque ahi el motivo importa.
fn git_check(dir: &Path, args: &[&str]) -> anyhow::Result<String> {
    let salida = Command::new("git")
        .args(args)
        .current_dir(dir)
        .stdin(Stdio::null())
        .output()?;
    let out = String::from_utf8_lossy(&salida.stdout).trim().to_string();
    if salida.status.success() {
        return Ok(out);
    }
    let err = String::from_utf8_lossy(&salida.stderr).trim().to_string();
    let detalle = if err.is_empty() { out } else { err };
    anyhow::bail!("git {}: {}", args.join(" "), detalle)
}

/// Raiz del arbol de trabajo actual (`--show-toplevel`), o `None` si no es repo.
pub fn toplevel(dir: &Path) -> Option<PathBuf> {
    git(dir, &["rev-parse", "--show-toplevel"]).map(PathBuf::from)
}

/// Directorio `.git` COMUN: en un worktree secundario apunta al `.git` del repo
/// principal, y por eso sirve para saber cual es el repo de verdad (AC-7).
pub fn common_dir(dir: &Path) -> Option<PathBuf> {
    let bruto = git(dir, &["rev-parse", "--path-format=absolute", "--git-common-dir"])?;
    Some(PathBuf::from(bruto))
}

/// Raiz del repo PRINCIPAL, incluso invocando desde un worktree secundario:
/// es el padre del `.git` comun (AC-7).
pub fn repo_principal(dir: &Path) -> Option<PathBuf> {
    let comun = common_dir(dir)?;
    // `.git` normal -> el padre es la raiz. Repo bare o raro -> sin respuesta.
    if comun.file_name().is_some_and(|n| n == ".git") {
        return comun.parent().map(Path::to_path_buf);
    }
    None
}

/// True si `dir` esta dentro de un worktree SECUNDARIO (no el principal).
pub fn es_worktree_secundario(dir: &Path) -> bool {
    match (toplevel(dir), repo_principal(dir)) {
        (Some(top), Some(principal)) => mismo_dir(&top, &principal) == Some(false),
        _ => false,
    }
}

/// Compara dos rutas por identidad real; `None` si alguna no se puede resolver.
fn mismo_dir(a: &Path, b: &Path) -> Option<bool> {
    match (std::fs::canonicalize(a), std::fs::canonicalize(b)) {
        (Ok(ca), Ok(cb)) => Some(ca == cb),
        _ => Some(a == b),
    }
}

/// True si la rama existe (local).
pub fn rama_existe(dir: &Path, rama: &str) -> bool {
    git(
        dir,
        &["rev-parse", "--verify", "--quiet", &format!("refs/heads/{rama}")],
    )
    .is_some()
}

/// Ramas locales del repo, para poder listarlas cuando el usuario tiene que
/// elegir a cual integrar (AC-14, AC-20).
pub fn ramas(dir: &Path) -> Vec<String> {
    git(dir, &["for-each-ref", "--format=%(refname:short)", "refs/heads/"])
        .map(|s| s.lines().map(str::to_string).collect())
        .unwrap_or_default()
}

/// Rama base para cortar una feature: la primera de `BASES` que exista, o la
/// rama actual si no hay ninguna. El arnes nunca crea la base (AC-22).
pub fn rama_base(dir: &Path, preferida: Option<&str>) -> Option<String> {
    if let Some(p) = preferida.filter(|p| rama_existe(dir, p)) {
        return Some(p.to_string());
    }
    for base in BASES {
        if rama_existe(dir, base) {
            return Some(base.to_string());
        }
    }
    git(dir, &["rev-parse", "--abbrev-ref", "HEAD"])
}

/// Nombre de la rama de una feature segun su tipo (AC-2).
pub fn nombre_rama(id: &str, slug: &str, kind: Option<&str>) -> String {
    let prefijo = if kind == Some("bug") {
        PREFIJO_BUGFIX
    } else {
        PREFIJO_FEATURE
    };
    format!("{prefijo}{id}-{slug}")
}

/// Carpeta del worktree de una feature: hermana del repo (decision OBS-7).
pub fn ruta_worktree(principal: &Path, id: &str, slug: &str) -> PathBuf {
    let nombre = principal
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "repo".to_string());
    let padre = principal.parent().unwrap_or(principal);
    padre.join(format!("{nombre}-wt")).join(format!("{id}-{slug}"))
}

/// Resultado de preparar el aislamiento de una feature.
pub struct Aislamiento {
    pub rama: String,
    pub worktree: PathBuf,
    /// True si ya existian (se reusaron sin tocar nada, AC-4).
    pub reusado: bool,
}

/// Crea (o reusa) la rama y el worktree de una feature. El checkout principal
/// NUNCA cambia de rama (AC-2, AC-3, AC-4).
pub fn preparar(
    principal: &Path,
    id: &str,
    slug: &str,
    kind: Option<&str>,
    base_preferida: Option<&str>,
) -> anyhow::Result<Aislamiento> {
    let rama = nombre_rama(id, slug, kind);
    let destino = ruta_worktree(principal, id, slug);

    // Worktree ya montado para esa ruta: no hay nada que hacer.
    if destino.join(".git").exists() {
        return Ok(Aislamiento {
            rama,
            worktree: destino,
            reusado: true,
        });
    }
    if let Some(padre) = destino.parent() {
        std::fs::create_dir_all(padre)?;
    }
    // `git worktree add` crea la rama con -b, o la reusa si ya existe.
    if rama_existe(principal, &rama) {
        git_check(
            principal,
            &["worktree", "add", &destino.to_string_lossy(), &rama],
        )?;
        return Ok(Aislamiento {
            rama,
            worktree: destino,
            reusado: true,
        });
    }
    let base = rama_base(principal, base_preferida)
        .ok_or_else(|| anyhow::anyhow!("no pude resolver la rama base del repo"))?;
    git_check(
        principal,
        &[
            "worktree",
            "add",
            "-b",
            &rama,
            &destino.to_string_lossy(),
            &base,
        ],
    )?;
    Ok(Aislamiento {
        rama,
        worktree: destino,
        reusado: false,
    })
}

/// Borra el worktree de una feature y limpia la referencia. La RAMA se conserva
/// (decision OBS-6 / AC-19).
pub fn borrar_worktree(principal: &Path, worktree: &Path) -> anyhow::Result<()> {
    if worktree.exists() {
        git_check(
            principal,
            &["worktree", "remove", "--force", &worktree.to_string_lossy()],
        )?;
    }
    let _ = git(principal, &["worktree", "prune"]);
    Ok(())
}

/// True si el worktree tiene cambios sin commitear.
pub fn hay_cambios(dir: &Path) -> bool {
    git(dir, &["status", "--porcelain"]).is_some_and(|s| !s.trim().is_empty())
}

/// Commitea lo que haya sin commitear en un worktree, con un mensaje del
/// arnes y SIN trailers de IA (AC-16). Devuelve `false` si no habia nada.
pub fn commit_todo(dir: &Path, mensaje: &str) -> anyhow::Result<bool> {
    if !hay_cambios(dir) {
        return Ok(false);
    }
    git_check(dir, &["add", "-A"])?;
    git_check(dir, &["commit", "-m", mensaje])?;
    Ok(true)
}

/// Mergea `rama` en `destino` sin reescribir historia y sin trailers de IA.
///
/// El merge se hace en un worktree TEMPORAL de la rama destino: asi el
/// checkout principal no cambia de rama y no importa si tiene cambios sin
/// commitear — el cierre de una feature no puede exigirte tener el escritorio
/// ordenado. Ante conflicto se aborta y no queda nada a medias (AC-15, AC-18).
pub fn merge_en(principal: &Path, destino: &str, rama: &str) -> anyhow::Result<()> {
    if !rama_existe(principal, destino) {
        anyhow::bail!(
            "la rama destino '{destino}' no existe. Ramas disponibles: {}",
            ramas(principal).join(", ")
        );
    }
    // Si el destino es la rama que el principal tiene abierta, se mergea ahi
    // mismo (git no permite dos worktrees sobre la misma rama).
    if rama_actual(principal).as_deref() == Some(destino) {
        return merge_aqui(principal, destino, rama);
    }
    let temporal = std::env::temp_dir().join(format!(
        "harness-merge-{}-{}",
        destino.replace('/', "-"),
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&temporal);
    git_check(
        principal,
        &["worktree", "add", &temporal.to_string_lossy(), destino],
    )?;
    let resultado = merge_aqui(&temporal, destino, rama);
    let _ = git(
        principal,
        &["worktree", "remove", "--force", &temporal.to_string_lossy()],
    );
    let _ = std::fs::remove_dir_all(&temporal);
    let _ = git(principal, &["worktree", "prune"]);
    resultado
}

/// El merge propiamente dicho, en el arbol que ya tiene `destino` abierto.
fn merge_aqui(dir: &Path, destino: &str, rama: &str) -> anyhow::Result<()> {
    let mensaje = format!("merge: {rama} -> {destino} (cierre de feature del arnes)");
    let resultado = git_check(dir, &["merge", "--no-ff", "-m", &mensaje, rama]);
    if let Err(err) = resultado {
        // AC-18: abortar y dejar todo como estaba.
        let _ = git(dir, &["merge", "--abort"]);
        return Err(err);
    }
    Ok(())
}

/// Publica la rama destino (AC-17). Nunca `--force`.
pub fn push(principal: &Path, rama: &str) -> anyhow::Result<()> {
    git_check(principal, &["push", "origin", rama])?;
    Ok(())
}

/// Rama actual del checkout.
pub fn rama_actual(dir: &Path) -> Option<String> {
    git(dir, &["rev-parse", "--abbrev-ref", "HEAD"])
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;

    /// Repo git de verdad en un tempdir, con un commit inicial.
    fn repo() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path();
        for args in [
            vec!["init", "-q", "-b", "main"],
            vec!["config", "user.email", "test@example.com"],
            vec!["config", "user.name", "Test"],
        ] {
            Command::new("git")
                .args(&args)
                .current_dir(p)
                .output()
                .unwrap();
        }
        std::fs::write(p.join("README.md"), "# demo\n").unwrap();
        Command::new("git").args(["add", "-A"]).current_dir(p).output().unwrap();
        Command::new("git")
            .args(["commit", "-q", "-m", "init"])
            .current_dir(p)
            .output()
            .unwrap();
        dir
    }

    #[test]
    fn should_return_none_outside_a_repo() {
        // AC-5: sin repo git no hay aislamiento, pero tampoco error.
        let dir = tempfile::tempdir().unwrap();
        assert!(toplevel(dir.path()).is_none());
        assert!(repo_principal(dir.path()).is_none());
        assert!(!es_worktree_secundario(dir.path()));
    }

    #[test]
    fn nombre_rama_should_follow_gitflow() {
        assert_eq!(nombre_rama("47", "paralelo", None), "feature/47-paralelo");
        assert_eq!(
            nombre_rama("45", "close-pisa", Some("bug")),
            "bugfix/45-close-pisa"
        );
    }

    #[test]
    fn ruta_worktree_should_be_a_sibling_of_the_repo() {
        // Decision OBS-7: hermanos del repo, no adentro.
        let ruta = ruta_worktree(Path::new("/x/proyecto"), "47", "paralelo");
        assert_eq!(ruta, Path::new("/x/proyecto-wt/47-paralelo"));
    }

    #[test]
    fn base_should_prefer_develop_then_main() {
        let dir = repo();
        let p = dir.path();
        // Solo existe main.
        assert_eq!(rama_base(p, None).as_deref(), Some("main"));
        Command::new("git")
            .args(["branch", "develop"])
            .current_dir(p)
            .output()
            .unwrap();
        // Con develop presente, GitFlow manda.
        assert_eq!(rama_base(p, None).as_deref(), Some("develop"));
        // Y una preferida explicita gana, si existe.
        assert_eq!(rama_base(p, Some("main")).as_deref(), Some("main"));
        assert_eq!(rama_base(p, Some("noexiste")).as_deref(), Some("develop"));
    }

    #[test]
    fn preparar_should_create_branch_and_worktree_and_reuse_them() {
        // AC-2, AC-3, AC-4.
        let dir = repo();
        let p = dir.path();
        let a = preparar(p, "47", "paralelo", None, None).unwrap();
        assert_eq!(a.rama, "feature/47-paralelo");
        assert!(!a.reusado);
        assert!(a.worktree.join("README.md").is_file(), "el worktree tiene el arbol");
        assert!(rama_existe(p, "feature/47-paralelo"));
        // El checkout principal NO cambio de rama.
        assert_eq!(rama_actual(p).as_deref(), Some("main"));

        // Reintento: reusa sin romper.
        let b = preparar(p, "47", "paralelo", None, None).unwrap();
        assert!(b.reusado);
        assert_eq!(b.worktree, a.worktree);
    }

    #[test]
    fn worktree_should_resolve_the_main_repo() {
        // AC-7: desde el worktree, el repo principal es el de siempre.
        let dir = repo();
        let p = dir.path();
        let a = preparar(p, "47", "paralelo", None, None).unwrap();
        assert!(es_worktree_secundario(&a.worktree));
        let principal = repo_principal(&a.worktree).unwrap();
        assert_eq!(
            std::fs::canonicalize(&principal).unwrap(),
            std::fs::canonicalize(p).unwrap()
        );
        // Y el principal no se considera worktree secundario de si mismo.
        assert!(!es_worktree_secundario(p));
    }

    #[test]
    fn merge_should_integrate_and_keep_history() {
        // AC-15: merge --no-ff, sin reescribir nada.
        let dir = repo();
        let p = dir.path();
        let a = preparar(p, "47", "paralelo", None, None).unwrap();
        std::fs::write(a.worktree.join("nuevo.txt"), "hola\n").unwrap();
        Command::new("git").args(["add", "-A"]).current_dir(&a.worktree).output().unwrap();
        Command::new("git")
            .args(["commit", "-q", "-m", "feat: algo"])
            .current_dir(&a.worktree)
            .output()
            .unwrap();

        merge_en(p, "main", &a.rama).unwrap();
        assert!(p.join("nuevo.txt").is_file(), "el archivo llego a main");
        // El merge commit no lleva trailers de IA (AC-16).
        let log = git(p, &["log", "-1", "--format=%B"]).unwrap();
        assert!(!log.to_lowercase().contains("co-authored-by"));
        assert!(!log.to_lowercase().contains("generated with"));
    }

    #[test]
    fn merge_should_abort_on_conflict_and_leave_everything_intact() {
        // AC-18: el conflicto no puede dejar el repo a medias.
        let dir = repo();
        let p = dir.path();
        let a = preparar(p, "47", "paralelo", None, None).unwrap();
        // Las dos ramas cambian la MISMA linea.
        std::fs::write(a.worktree.join("README.md"), "# desde la feature\n").unwrap();
        Command::new("git").args(["add", "-A"]).current_dir(&a.worktree).output().unwrap();
        Command::new("git")
            .args(["commit", "-q", "-m", "feat: readme feature"])
            .current_dir(&a.worktree)
            .output()
            .unwrap();
        std::fs::write(p.join("README.md"), "# desde main\n").unwrap();
        Command::new("git").args(["add", "-A"]).current_dir(p).output().unwrap();
        Command::new("git")
            .args(["commit", "-q", "-m", "docs: readme main"])
            .current_dir(p)
            .output()
            .unwrap();

        let err = merge_en(p, "main", &a.rama).unwrap_err();
        assert!(err.to_string().contains("git merge"), "el error nombra el merge: {err}");
        // Todo intacto: main sigue con su version y sin merge a medias.
        assert_eq!(
            std::fs::read_to_string(p.join("README.md")).unwrap(),
            "# desde main\n"
        );
        assert!(!hay_cambios(p), "no quedan restos del merge abortado");
        assert_eq!(rama_actual(p).as_deref(), Some("main"));
        assert!(a.worktree.is_dir(), "el worktree sigue ahi");
    }

    #[test]
    fn merge_should_refuse_an_unknown_target() {
        // AC-20: falla antes de tocar nada y lista las validas.
        let dir = repo();
        let p = dir.path();
        let a = preparar(p, "47", "paralelo", None, None).unwrap();
        let err = merge_en(p, "no-existe", &a.rama).unwrap_err();
        assert!(err.to_string().contains("no existe"));
        assert!(err.to_string().contains("main"), "lista las ramas: {err}");
    }

    #[test]
    fn borrar_worktree_should_keep_the_branch() {
        // AC-19: se borra la carpeta, la rama queda.
        let dir = repo();
        let p = dir.path();
        let a = preparar(p, "47", "paralelo", None, None).unwrap();
        borrar_worktree(p, &a.worktree).unwrap();
        assert!(!a.worktree.exists());
        assert!(rama_existe(p, &a.rama), "la rama se conserva");
    }
}
