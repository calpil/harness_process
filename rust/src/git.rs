//! Operaciones git del arnes (feature #47): ramas y worktrees por feature,
//! integracion GitFlow al cerrar.
//!
//! Dos reglas que valen para todo este modulo:
//!
//! - **Nunca reescribe historia ni fuerza nada**: sin `--force`, sin rebase,
//!   sin squash, sin borrar ramas. Un conflicto ABORTA y deja el repo como
//!   estaba (Articulo 4 / AC-18).
//! - **El merge NUNCA corre en el checkout del usuario** (feature #61). Se hace
//!   siempre en un worktree temporal `--detach`, este el destino checkouteado o
//!   no. Antes habia una excepcion silenciosa —si el destino era la rama abierta
//!   se mergeaba ahi mismo— y era justo el caso mas comun: cerrar hacia `main`
//!   estando en `main`. Ahi la promesa "el cierre no puede exigirte tener el
//!   escritorio ordenado" se caia con el texto crudo de git, y despues de haber
//!   commiteado el worktree de la feature.
//!
//!   Queda UN caso que no se puede resolver sin decidir por el usuario: que el
//!   merge cambie un archivo que el tiene modificado sin commitear. Ahi el arnes
//!   no elige entre su merge y el trabajo ajeno: lo DETECTA antes de tocar nada
//!   (`colisiones`) y se detiene nombrando los archivos. No stashea ni descarta
//!   (decision USUARIO 2026-08-27), y tampoco avanza la rama dejando el arbol
//!   atras: eso deja `git status` mostrando la REVERSION del merge, y un commit
//!   distraido desharia lo recien integrado.
//! - **Los commits del arnes no llevan trailers de IA** (AC-16): lo exige
//!   `UPDATING.md` y lo verifica `commit_guard.sh`.
//!
//! Si el directorio no es un repo git, todas las consultas devuelven `None` y
//! el flujo sigue como siempre: el aislamiento es una mejora, no un requisito
//! (AC-5).

use anyhow::Context;
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

/// El repo del `docs/` del proyecto, cuando docs es un repo git DISTINTO del
/// principal (feature #72 / AC-2).
///
/// Este es el caso que rompio el aislamiento de la feature #98: en el worktree
/// del repo principal, un `docs/` que pertenece a otro repo NO viaja — queda
/// vacio o directamente ausente. El coordinador vio un `docs/` vacio, lo leyo
/// como "aca no se puede trabajar" y arranco con `--sin-worktree`, que es como
/// termino escribiendo en el checkout compartido.
///
/// Devuelve `None` cuando `docs/` no existe, no es repo, o es parte del MISMO
/// repo principal (el caso comun: ahi viaja con el worktree y no hay nada que
/// preparar).
pub fn repo_de_docs(repo_root: &Path) -> Option<PathBuf> {
    let docs = repo_root.join("docs");
    if !docs.is_dir() {
        return None;
    }
    // Tiene que ser la RAIZ de su propio repo: un `docs/` que es subdirectorio
    // del principal devuelve el toplevel del principal, y ahi no hay nada que
    // aislar aparte.
    let propio = toplevel(&docs)?;
    if mismo_dir(&propio, &docs) != Some(true) {
        return None;
    }
    let principal = repo_principal(repo_root);
    match principal {
        Some(p) if mismo_dir(&p, &propio) == Some(true) => None,
        _ => Some(propio),
    }
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

/// Archivos con cambios sin commitear en un checkout (modificados, borrados,
/// staged o sin trackear). Solo consulta.
pub fn sucios(dir: &Path) -> Vec<String> {
    let Some(salida) = git(dir, &["status", "--porcelain"]) else {
        return Vec::new();
    };
    salida.lines().filter_map(ruta_de_status).collect()
}

/// La ruta de una linea de `git status --porcelain` (`XY ruta`, o
/// `R  vieja -> nueva`). Devuelve la ruta DESTINO, que es la que el merge
/// podria pisar.
///
/// NO se corta por posicion fija: `git()` le hace `trim()` a la salida, asi que
/// la primera linea pierde el espacio de la columna X (` M a.md` llega como
/// `M a.md`) y un corte en la columna 3 se comeria las primeras letras del
/// nombre. Se parte por el primer espacio despues de los codigos, que funciona
/// con y sin ese espacio, y deja intactas las rutas con espacios.
fn ruta_de_status(linea: &str) -> Option<String> {
    let (_estado, resto) = linea.trim_start().split_once(' ')?;
    let resto = resto.trim();
    let ruta = resto.rsplit(" -> ").next().unwrap_or(resto);
    let ruta = ruta.trim().trim_matches('"');
    if ruta.is_empty() {
        return None;
    }
    Some(ruta.to_string())
}

/// Archivos que el merge de `rama` en `destino` cambiaria: lo que la rama toco
/// desde que se separo (`destino...rama`), mas lo que todavia esta sin
/// commitear en su worktree — porque el cierre lo va a commitear antes de
/// mergear.
pub fn archivos_del_merge(dir: &Path, destino: &str, rama: &str, worktree: Option<&Path>) -> Vec<String> {
    let mut out: Vec<String> = git(
        dir,
        &["diff", "--name-only", &format!("{destino}...{rama}")],
    )
    .map(|s| s.lines().map(str::trim).filter(|l| !l.is_empty()).map(str::to_string).collect())
    .unwrap_or_default();
    if let Some(wt) = worktree {
        out.extend(sucios(wt));
    }
    out.sort();
    out.dedup();
    out
}

/// Los archivos que el usuario tiene sin commitear en su checkout Y que el
/// merge cambiaria: los unicos que de verdad impiden integrar.
///
/// SOLO CONSULTA: no muta nada, no necesita hacer el merge y por eso se puede
/// llamar ANTES de commitear nada (feature #61 / AC-7).
///
/// Vacio cuando el principal no tiene el destino abierto: ahi el merge ocurre
/// en un worktree temporal y el arbol del usuario ni se entera.
pub fn colisiones(
    principal: &Path,
    destino: &str,
    rama: &str,
    worktree: Option<&Path>,
) -> Vec<String> {
    if rama_actual(principal).as_deref() != Some(destino) {
        return Vec::new();
    }
    let sucios = sucios(principal);
    if sucios.is_empty() {
        return Vec::new();
    }
    let del_merge = archivos_del_merge(principal, destino, rama, worktree);
    let mut choques: Vec<String> = sucios
        .into_iter()
        .filter(|s| del_merge.iter().any(|m| m == s))
        .collect();
    choques.sort();
    choques.dedup();
    choques
}

/// Un commit del rango que la integracion se llevaria.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Commit {
    pub sha: String,
    pub titulo: String,
    /// La rama de OTRA feature que tambien contiene este commit, si la hay.
    /// `Some` significa: esto no es trabajo de la feature que se esta cerrando.
    pub ajeno: Option<String>,
}

/// Todo lo que un merge de `rama` en `destino` se llevaria, commit por commit
/// (feature #72 / AC-3).
///
/// El motivo es un incidente verificado: se publico `9750cc2` (el arreglo de
/// #117) y con el se fue `2fd6c5f` (#106), que se habia acordado dejar local.
/// Nadie mintio — el commit propio simplemente tenia por padre uno ajeno, y el
/// cierre solo hablaba de "la rama". Un rango que no se muestra es un rango que
/// no se revisa.
///
/// `otras_ramas` son las ramas de las demas features: un commit alcanzable
/// desde alguna de ellas se marca `ajeno`. No se adivina por autor ni por
/// fecha, que son atributos que cualquiera puede reescribir.
pub fn rango_de_integracion(
    principal: &Path,
    destino: &str,
    rama: &str,
    otras_ramas: &[String],
) -> Vec<Commit> {
    let Some(salida) = git(
        principal,
        &[
            "log",
            "--reverse",
            "--format=%H\x1f%s",
            &format!("{destino}..{rama}"),
        ],
    ) else {
        return Vec::new();
    };
    // Un set por rama ajena: lo que esa rama aporta sobre el mismo destino.
    let ajenos: Vec<(String, Vec<String>)> = otras_ramas
        .iter()
        .filter(|o| o.as_str() != rama)
        .map(|o| {
            let shas = git(principal, &["rev-list", &format!("{destino}..{o}")])
                .map(|s| s.lines().map(str::to_string).collect())
                .unwrap_or_default();
            (o.clone(), shas)
        })
        .collect();

    salida
        .lines()
        .filter(|l| !l.trim().is_empty())
        .filter_map(|linea| {
            let (sha, titulo) = linea.split_once('\x1f')?;
            let ajeno = ajenos
                .iter()
                .find(|(_, shas)| shas.iter().any(|s| s == sha))
                .map(|(rama, _)| rama.clone());
            Some(Commit {
                sha: sha.to_string(),
                titulo: titulo.to_string(),
                ajeno,
            })
        })
        .collect()
}

/// Candado por rama destino: dos cierres del arnes sobre el MISMO destino no
/// corren a la vez (AC-3).
///
/// Vive en el `.git` comun, asi que lo comparten todos los worktrees del repo
/// —que es exactamente el conjunto de features que podrian pisarse—. Se libera
/// al soltarlo; si un proceso muere sin soltarlo queda un archivo suelto, y el
/// mensaje dice como borrarlo, porque un candado que nadie puede abrir es peor
/// que no tenerlo.
#[derive(Debug)]
pub struct CandadoDeIntegracion {
    ruta: PathBuf,
}

impl CandadoDeIntegracion {
    pub fn tomar(principal: &Path, destino: &str) -> anyhow::Result<Self> {
        let dir = common_dir(principal)
            .ok_or_else(|| anyhow::anyhow!("no se pudo resolver el .git de {}", principal.display()))?;
        let ruta = dir.join(format!("harness-integracion-{}.lock", destino.replace('/', "-")));
        match std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&ruta)
        {
            Ok(_) => Ok(Self { ruta }),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => anyhow::bail!(
                "ya hay una integracion en curso hacia '{destino}'.\n    \
                 Espera a que termine. Si estas seguro de que no queda ninguna corriendo,\n    \
                 borra el candado: rm {}",
                ruta.display()
            ),
            Err(e) => anyhow::bail!("no se pudo tomar el candado de integracion: {e}"),
        }
    }
}

impl Drop for CandadoDeIntegracion {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.ruta);
    }
}

/// Mergea `rama` en `destino` sin reescribir historia y sin trailers de IA.
///
/// El merge se hace SIEMPRE en un worktree temporal `--detach` (feature #61):
/// el checkout del usuario no participa, no cambia de rama y no queda en estado
/// de merge pase lo que pase. Despues se avanza la rama destino; si el usuario
/// la tiene abierta, `git reset --keep` mueve rama y arbol conservando lo que
/// tenga sin commitear. Ante conflicto se aborta y no queda nada a medias
/// (AC-15, AC-18).
pub fn merge_en(principal: &Path, destino: &str, rama: &str) -> anyhow::Result<()> {
    if !rama_existe(principal, destino) {
        anyhow::bail!(
            "la rama destino '{destino}' no existe. Ramas disponibles: {}",
            ramas(principal).join(", ")
        );
    }
    let Some(viejo) = git(principal, &["rev-parse", destino]) else {
        anyhow::bail!("no se pudo resolver el commit de '{destino}'");
    };
    // Directorio UNICO por invocacion. Con `<destino>-<pid>` dos merges del
    // mismo proceso (o dos tests en paralelo) se pisaban: uno borraba el
    // worktree del otro a mitad del merge.
    let base = tempfile::Builder::new()
        .prefix(&format!("harness-merge-{}-", destino.replace('/', "-")))
        .tempdir()
        .context("no se pudo crear el directorio temporal del merge")?;
    // `worktree add` exige que el destino NO exista todavia.
    let temporal = base.path().join("wt");
    // `--detach` es la clave: git no deja dos worktrees sobre la MISMA rama,
    // pero si deja uno en HEAD detached sobre su commit. Por eso ya no hace
    // falta la excepcion de mergear en el checkout principal.
    git_check(
        principal,
        &[
            "worktree",
            "add",
            "--detach",
            &temporal.to_string_lossy(),
            destino,
        ],
    )?;
    let resultado = merge_aqui(&temporal, destino, rama)
        .and_then(|()| git(&temporal, &["rev-parse", "HEAD"]).context("no se pudo leer el merge"));
    let _ = git(
        principal,
        &["worktree", "remove", "--force", &temporal.to_string_lossy()],
    );
    drop(base);
    let _ = git(principal, &["worktree", "prune"]);
    // El merge quedo en un commit suelto; recien ahora se mueve la rama. Si
    // fallo, no se movio nada y el commit huerfano se lo lleva el `gc`.
    avanzar_rama(principal, destino, &resultado?, &viejo)
}

/// Mueve `destino` al commit del merge.
///
/// Si el usuario tiene esa rama abierta, `reset --keep` mueve la rama Y
/// actualiza el arbol preservando sus cambios sin commitear; si alguno chocara,
/// aborta sin tocar nada (por eso `colisiones` lo detecta antes: para poder
/// explicarlo en castellano y no dejar el cierre a mitad de camino). Si no la
/// tiene abierta, se mueve la referencia con guarda de valor viejo, que falla si
/// alguien la movio mientras tanto.
fn avanzar_rama(principal: &Path, destino: &str, nuevo: &str, viejo: &str) -> anyhow::Result<()> {
    if rama_actual(principal).as_deref() == Some(destino) {
        git_check(principal, &["reset", "--keep", nuevo])?;
        return Ok(());
    }
    git_check(
        principal,
        &["update-ref", &format!("refs/heads/{destino}"), nuevo, viejo],
    )?;
    Ok(())
}

/// El merge propiamente dicho, en el directorio que se le pase (siempre el
/// worktree temporal desde la feature #61). Ante conflicto aborta y deja ese
/// arbol como estaba.
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
mod tests_colisiones {
    #![allow(clippy::unwrap_used)]
    use super::*;
    use std::process::Command;

    fn git_raw(dir: &Path, args: &[&str]) {
        Command::new("git").args(args).current_dir(dir).output().unwrap();
    }

    /// Repo con `main` y una rama `feature` que toca `A.md`.
    fn repo_con_rama() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path();
        git_raw(p, &["init", "-q", "-b", "main"]);
        git_raw(p, &["config", "user.email", "t@e.c"]);
        git_raw(p, &["config", "user.name", "T"]);
        std::fs::write(p.join("A.md"), "base\n").unwrap();
        std::fs::write(p.join("B.md"), "base\n").unwrap();
        git_raw(p, &["add", "-A"]);
        git_raw(p, &["commit", "-qm", "init"]);
        git_raw(p, &["checkout", "-q", "-b", "feature"]);
        std::fs::write(p.join("A.md"), "lo de la rama\n").unwrap();
        git_raw(p, &["commit", "-qam", "rama toca A"]);
        git_raw(p, &["checkout", "-q", "main"]);
        dir
    }

    /// AC-7: `colisiones` SOLO consulta. Se llama con el arbol sucio y despues
    /// el arbol sigue exactamente igual: no mergea, no stashea, no resetea.
    #[test]
    fn colisiones_solo_consulta_y_no_muta() {
        let dir = repo_con_rama();
        let p = dir.path();
        std::fs::write(p.join("A.md"), "lo mio sin commitear\n").unwrap();

        let antes_head = git(p, &["rev-parse", "HEAD"]).unwrap();
        let antes_status = git(p, &["status", "--porcelain"]).unwrap();

        let choques = colisiones(p, "main", "feature", None);

        assert_eq!(choques, vec!["A.md".to_string()], "A.md choca");
        assert_eq!(git(p, &["rev-parse", "HEAD"]).unwrap(), antes_head, "no movio HEAD");
        assert_eq!(git(p, &["status", "--porcelain"]).unwrap(), antes_status, "no toco el arbol");
        assert_eq!(
            std::fs::read_to_string(p.join("A.md")).unwrap(),
            "lo mio sin commitear\n"
        );
        assert!(!p.join(".git/MERGE_HEAD").exists(), "no mergeo nada");
    }

    /// Lo sucio que el merge NO toca no es una colision: ese es todo el punto.
    #[test]
    fn colisiones_ignora_lo_sucio_que_el_merge_no_toca() {
        let dir = repo_con_rama();
        let p = dir.path();
        std::fs::write(p.join("B.md"), "lo mio sin commitear\n").unwrap();
        assert!(colisiones(p, "main", "feature", None).is_empty());
    }

    /// Si el destino no esta checkouteado, el merge ocurre en un worktree
    /// temporal y el arbol del usuario ni se entera: nunca hay colision.
    #[test]
    fn colisiones_vacias_si_el_destino_no_esta_abierto() {
        let dir = repo_con_rama();
        let p = dir.path();
        std::fs::write(p.join("A.md"), "lo mio\n").unwrap();
        // Parado en otra rama: el destino `main` no es la rama abierta.
        git_raw(p, &["stash"]);
        git_raw(p, &["checkout", "-q", "-b", "otra"]);
        std::fs::write(p.join("A.md"), "lo mio\n").unwrap();
        assert!(
            colisiones(p, "main", "feature", None).is_empty(),
            "sin el destino abierto no hay nada que pisar"
        );
    }

    /// Lo que la feature todavia NO commiteo cuenta: el cierre lo va a
    /// commitear justo antes de mergear.
    #[test]
    fn archivos_del_merge_incluye_lo_sin_commitear_del_worktree() {
        let dir = repo_con_rama();
        let p = dir.path();
        let wt = dir.path().join("wt");
        git_raw(p, &["worktree", "add", "-q", &wt.to_string_lossy(), "feature"]);
        std::fs::write(wt.join("B.md"), "la feature todavia no lo commiteo\n").unwrap();

        let archivos = archivos_del_merge(p, "main", "feature", Some(&wt));
        assert!(archivos.contains(&"A.md".to_string()), "lo ya commiteado");
        assert!(archivos.contains(&"B.md".to_string()), "y lo que esta por commitearse");
    }

    /// Las rutas de `git status --porcelain`, incluido el rename.
    #[test]
    fn ruta_de_status_lee_las_formas_de_porcelain() {
        assert_eq!(ruta_de_status(" M docs/a.md"), Some("docs/a.md".to_string()));
        // La forma que llega cuando `git()` ya trimeo la salida: sin el espacio
        // de la columna X. Cortar por posicion fija devolvia ".md" y la
        // colision no se detectaba (bug encontrado por el test de la #61).
        assert_eq!(ruta_de_status("M docs/a.md"), Some("docs/a.md".to_string()));
        assert_eq!(ruta_de_status("M A.md"), Some("A.md".to_string()));
        // Rutas con espacios.
        assert_eq!(ruta_de_status(" M mi archivo.md"), Some("mi archivo.md".to_string()));
        assert_eq!(ruta_de_status("?? nuevo.md"), Some("nuevo.md".to_string()));
        assert_eq!(ruta_de_status("M  staged.md"), Some("staged.md".to_string()));
        // En un rename manda el destino: es el que el merge podria pisar.
        assert_eq!(
            ruta_de_status("R  viejo.md -> nuevo.md"),
            Some("nuevo.md".to_string())
        );
        assert_eq!(ruta_de_status(""), None);
    }
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

    /// AC-3, el incidente del 2026-09-03 reproducido: la rama de la feature que
    /// se cierra lleva un commit PROPIO cuyo padre es un commit de OTRA feature
    /// que se habia acordado dejar local. Integrar la primera se lleva las dos.
    #[test]
    fn el_rango_delata_el_commit_ajeno_que_viaja_de_padre() {
        let dir = repo();
        let p = dir.path();

        // Feature A (la que se queda local): su commit sale de main.
        Command::new("git").args(["checkout", "-q", "-b", "feature/106-a"]).current_dir(p).output().unwrap();
        std::fs::write(p.join("a.txt"), "de la 106\n").unwrap();
        Command::new("git").args(["add", "-A"]).current_dir(p).output().unwrap();
        Command::new("git").args(["commit", "-q", "-m", "feat: lo de la 106"]).current_dir(p).output().unwrap();
        let sha_ajeno = git(p, &["rev-parse", "HEAD"]).unwrap();

        // Feature B se corta DESDE A: su commit tiene por padre el de A.
        Command::new("git").args(["checkout", "-q", "-b", "feature/117-b"]).current_dir(p).output().unwrap();
        std::fs::write(p.join("b.txt"), "de la 117\n").unwrap();
        Command::new("git").args(["add", "-A"]).current_dir(p).output().unwrap();
        Command::new("git").args(["commit", "-q", "-m", "fix: lo de la 117"]).current_dir(p).output().unwrap();
        let sha_propio = git(p, &["rev-parse", "HEAD"]).unwrap();
        Command::new("git").args(["checkout", "-q", "main"]).current_dir(p).output().unwrap();

        let otras = vec!["feature/106-a".to_string()];
        let rango = rango_de_integracion(p, "main", "feature/117-b", &otras);

        // El rango tiene DOS commits, no uno: eso es lo que el cierre no decia.
        assert_eq!(rango.len(), 2, "el rango completo: {rango:?}");
        assert_eq!(rango[0].sha, sha_ajeno, "el padre va primero (--reverse)");
        assert_eq!(
            rango[0].ajeno.as_deref(),
            Some("feature/106-a"),
            "y esta marcado como ajeno"
        );
        assert_eq!(rango[1].sha, sha_propio);
        assert_eq!(rango[1].ajeno, None, "el propio no es ajeno");
        assert_eq!(rango[1].titulo, "fix: lo de la 117", "el titulo viaja entero");
    }

    /// Sin ramas ajenas alrededor, un rango limpio no inventa culpables.
    #[test]
    fn el_rango_limpio_no_marca_nada_como_ajeno() {
        let dir = repo();
        let p = dir.path();
        let a = preparar(p, "47", "paralelo", None, None).unwrap();
        std::fs::write(a.worktree.join("x.txt"), "hola\n").unwrap();
        Command::new("git").args(["add", "-A"]).current_dir(&a.worktree).output().unwrap();
        Command::new("git").args(["commit", "-q", "-m", "feat: x"]).current_dir(&a.worktree).output().unwrap();

        let rango = rango_de_integracion(p, "main", &a.rama, &[]);
        assert_eq!(rango.len(), 1);
        assert_eq!(rango[0].ajeno, None);
    }

    /// AC-3: dos integraciones al MISMO destino no corren a la vez.
    #[test]
    fn el_candado_de_integracion_serializa_por_destino() {
        let dir = repo();
        let p = dir.path();
        let primero = CandadoDeIntegracion::tomar(p, "main").unwrap();
        let err = CandadoDeIntegracion::tomar(p, "main").unwrap_err();
        assert!(err.to_string().contains("ya hay una integracion en curso"), "{err}");
        // Otro destino no se estorba: el candado es POR destino.
        let otro = CandadoDeIntegracion::tomar(p, "develop");
        assert!(otro.is_ok(), "develop es otro candado");
        drop(otro);
        // Y al soltarlo, el destino vuelve a estar libre.
        drop(primero);
        assert!(CandadoDeIntegracion::tomar(p, "main").is_ok());
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
