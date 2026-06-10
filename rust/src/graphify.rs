//! Refresh de graphify (paridad: harness.py _graphify_refresh / _bg).
//! La variante bg reemplaza el trampolin `bash -c` de Python por un
//! self-spawn detached del subcomando oculto `graphify-worker` (multi-OS).

use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Duration;

use wait_timeout::ChildExt;

fn graphify_available(repo_root: &Path) -> bool {
    which::which("graphify").is_ok()
        && repo_root.join("graphify-out").join("graph.json").exists()
}

fn run_graphify_update(repo_root: &Path) -> bool {
    let child = Command::new("graphify")
        .arg("update")
        .arg(repo_root)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
    let Ok(mut child) = child else {
        return false;
    };
    match child.wait_timeout(Duration::from_secs(300)) {
        Ok(Some(status)) => status.success(),
        Ok(None) => {
            // timeout: mata el update y marca stale (como subprocess.run)
            let _ = child.kill();
            let _ = child.wait();
            false
        }
        Err(_) => false,
    }
}

fn mark_stale(stale: &Path) {
    // open(stale, "a").close(): crea sin truncar
    let _ = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(stale);
}

/// Refresh sincrono bajo el mismo lock que el hook post-commit.
pub fn refresh_sync(repo_root: &Path) {
    if !graphify_available(repo_root) {
        return;
    }
    let lock = repo_root.join("graphify-out").join(".update.lock");
    if std::fs::create_dir(&lock).is_err() {
        return; // ya hay un update en curso; no dupliques
    }
    let stale = repo_root.join("graphify-out").join(".graphify_stale");
    if run_graphify_update(repo_root) {
        let _ = std::fs::remove_file(&stale); // grafo fresco: limpia el marcador
    } else {
        mark_stale(&stale); // update fallo o timeout: marca stale
    }
    let _ = std::fs::remove_dir(&lock);
}

/// Refresh detached: lanza `harness graphify-worker` y retorna de inmediato
/// para no colgar el turno cuando lo dispara un hook.
pub fn refresh_bg(repo_root: &Path) {
    if !graphify_available(repo_root) {
        return;
    }
    let lock = repo_root.join("graphify-out").join(".update.lock");
    if std::fs::create_dir(&lock).is_err() {
        return; // ya hay un refresh en curso
    }
    let stale = repo_root.join("graphify-out").join(".graphify_stale");
    let Ok(exe) = std::env::current_exe() else {
        let _ = std::fs::remove_dir(&lock);
        return;
    };
    let mut cmd = Command::new(exe);
    cmd.arg("graphify-worker")
        .arg("--root")
        .arg(repo_root)
        .arg("--stale")
        .arg(&stale)
        .arg("--lock")
        .arg(&lock)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        cmd.process_group(0); // equivalente de start_new_session=True
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        // CREATE_NO_WINDOW | CREATE_NEW_PROCESS_GROUP
        cmd.creation_flags(0x0800_0000 | 0x0000_0200);
    }
    if cmd.spawn().is_err() {
        let _ = std::fs::remove_dir(&lock);
    }
}

/// Cuerpo del worker detached (subcomando oculto). Hace el trabajo del script
/// bash de Python: update + marcador stale + liberar el lock SIEMPRE.
pub fn worker(root: &Path, stale: &Path, lock: &Path) -> anyhow::Result<()> {
    if run_graphify_update(root) {
        let _ = std::fs::remove_file(stale);
    } else {
        // ": > $STALE" trunca/crea
        let _ = std::fs::File::create(stale);
    }
    let _ = std::fs::remove_dir(lock);
    Ok(())
}
