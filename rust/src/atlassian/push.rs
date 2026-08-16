//! Envio automatico (feature #16): el flujo empuja solo.
//!
//! Cada transicion que emite un intent lanza — si corresponde — un worker
//! DETACHED que corre el `apply` + `publish` ya probados de la feature #15. El
//! comando del flujo vuelve al instante y jamas cambia su exit code: si
//! Atlassian esta lento o caido, lo pendiente queda en la outbox y la proxima
//! transicion lo reintenta.
//!
//! Mismo patron que `graphify::refresh_bg`: lock como directorio, proceso en su
//! propio grupo y liberacion del lock en todos los caminos.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::atlassian::binding::Binding;
use crate::atlassian::http::Credentials;
use crate::atlassian::outbox;
use crate::atlassian::state::atlassian_dir;
use crate::paths::HarnessPaths;
use crate::progress::now_stamp;
use crate::pycompat::env_nonempty;

/// Variable de entorno que apaga el envio automatico por corrida (AC-14).
pub const ENV_AUTO: &str = "HARNESS_ATLASSIAN_AUTO";

pub fn lock_path(paths: &HarnessPaths) -> PathBuf {
    atlassian_dir(paths).join(".push.lock")
}

pub fn log_path(paths: &HarnessPaths) -> PathBuf {
    atlassian_dir(paths).join("last-push.log")
}

/// Por que NO se empujo: sirve para el aviso al usuario y para `status`.
#[derive(Debug, PartialEq, Eq)]
pub enum Skip {
    /// Sin binding activo: la integracion no existe en este repo.
    NoBinding,
    /// El interruptor del binding (`"auto": false`).
    DisabledInBinding,
    /// `HARNESS_ATLASSIAN_AUTO=0` en el entorno.
    DisabledInEnv,
    /// Sin credenciales: la outbox espera al agente con MCP.
    NoToken,
}

impl Skip {
    /// Mensaje para el usuario, o `None` cuando no hay nada que decir.
    pub fn note(&self) -> Option<&'static str> {
        match self {
            // Sin binding no hay integracion: no corresponde decir nada.
            Skip::NoBinding => None,
            Skip::DisabledInBinding | Skip::DisabledInEnv => Some(
                "[Atlassian] envio automatico apagado: lo pendiente queda en la outbox (`atlassian apply` para enviarlo).",
            ),
            Skip::NoToken => Some(
                "[Atlassian] sin token: el intent quedo en la outbox. Drenalo con `atlassian drain` + tu MCP, o define las credenciales para que se envie solo.",
            ),
        }
    }
}

/// Decide si corresponde empujar. El orden es el del spec: entorno primero
/// (una corrida puntual), despues el binding (este repo) y por ultimo las
/// credenciales (sin ellas no hay envio posible).
pub fn should_push(paths: &HarnessPaths) -> Result<Binding, Skip> {
    should_push_with(paths, Credentials::discover(paths).is_some())
}

/// La decision, con la presencia del token como dato de entrada. Existe
/// separada para poder testearla sin depender de la configuracion REAL de la
/// maquina (el token global de `~/.config/harness/config` es del usuario, no
/// del test).
pub fn should_push_with(paths: &HarnessPaths, has_token: bool) -> Result<Binding, Skip> {
    let Some(binding) = Binding::load_active(paths) else {
        return Err(Skip::NoBinding);
    };
    if env_nonempty(ENV_AUTO).as_deref() == Some("0") {
        return Err(Skip::DisabledInEnv);
    }
    if !binding.auto {
        return Err(Skip::DisabledInBinding);
    }
    if !has_token {
        return Err(Skip::NoToken);
    }
    Ok(binding)
}

/// Lanza el worker detached tras una transicion del flujo. Best-effort puro:
/// cualquier problema se traga (el comando del flujo no puede fallar por esto).
pub fn push_bg(paths: &HarnessPaths) {
    match should_push(paths) {
        Ok(_) => {}
        Err(skip) => {
            // El aviso solo tiene sentido si hay algo esperando.
            if !outbox::pending(paths).is_empty()
                && let Some(note) = skip.note()
            {
                eprintln!("{note}");
            }
            return;
        }
    }

    let lock = lock_path(paths);
    if std::fs::create_dir_all(atlassian_dir(paths)).is_err() {
        return;
    }
    if std::fs::create_dir(&lock).is_err() {
        // Ya hay un worker corriendo: el va a levantar tambien lo nuevo en su
        // segunda pasada (AC-4b), asi que no hay nada que hacer.
        return;
    }
    let Ok(exe) = std::env::current_exe() else {
        let _ = std::fs::remove_dir(&lock);
        return;
    };

    // La salida del worker ES el log del ultimo envio (AC-5): se trunca en cada
    // corrida y nunca lleva el token, porque nadie lo imprime.
    let log = log_path(paths);
    let (out, err) = match std::fs::File::create(&log) {
        Ok(f) => match f.try_clone() {
            Ok(clone) => (Stdio::from(f), Stdio::from(clone)),
            Err(_) => (Stdio::null(), Stdio::null()),
        },
        Err(_) => (Stdio::null(), Stdio::null()),
    };

    let mut cmd = Command::new(exe);
    cmd.arg("atlassian-worker")
        .arg("--root")
        .arg(&paths.root)
        .arg("--lock")
        .arg(&lock)
        .stdin(Stdio::null())
        .stdout(out)
        .stderr(err);
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        cmd.process_group(0); // que sobreviva al cierre de la terminal
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

/// Cuerpo del worker detached (subcomando oculto `atlassian-worker`).
///
/// Hace dos pasadas: la segunda levanta los intents que aparecieron MIENTRAS
/// corria la primera (AC-4b), para que nada quede esperando al proximo comando.
/// El lock se libera SIEMPRE.
pub fn worker(root: &Path, lock: &Path) -> anyhow::Result<()> {
    let paths = HarnessPaths::from_root(root.to_path_buf());
    println!("== Atlassian push {} ==", now_stamp());

    // AC-24: la PRIMERA vez (nada mapeado todavia) se carga lo que ya existe en
    // el repo, para que el board no arranque vacio al lado de una wiki completa.
    // Despues es incremental: el dedupe corta lo ya mapeado (AC-25).
    let state = crate::atlassian::state::State::load(&paths);
    if state.prds.is_empty() && state.features.is_empty() {
        println!("[backfill] primer envio: cargando PRDs y backlog existentes");
        if let Err(err) = crate::commands::atlassian::backfill(&paths, false) {
            println!("[backfill] {err:#}");
        }
    }

    for pass in 1..=2 {
        let pending = outbox::pending(&paths).len();
        if pending == 0 {
            if pass == 1 {
                println!("[pasada {pass}] sin intents pendientes");
            }
            break;
        }
        println!("[pasada {pass}] {pending} intent(s) pendiente(s)");
        if let Err(err) = crate::commands::atlassian::apply(&paths) {
            // `apply` sale con Exit(1) cuando algo quedo sin aplicar: eso ya se
            // detallo en la salida, y los intents siguen en la outbox.
            println!("[pasada {pass}] con pendientes: {err:#}");
        }
    }

    // La publicacion corre en cada transicion por decision del usuario (OBS-7):
    // el hash de la #15 evita tocar la red cuando nada cambio.
    if let Err(err) = crate::commands::atlassian::publish(&paths) {
        println!("[publish] {err:#}");
    }

    println!("== fin {} ==", now_stamp());
    let _ = std::fs::remove_dir(lock);
    Ok(())
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;
    use crate::atlassian::binding::{ConfluenceBinding, JiraBinding};

    fn paths_in(dir: &Path) -> HarnessPaths {
        HarnessPaths::from_root(dir.to_path_buf())
    }

    fn write_binding(paths: &HarnessPaths, auto: bool) {
        let binding = Binding {
            site: "calpil.atlassian.net".to_string(),
            cloud_id: None,
            enabled: true,
            auto,
            jira: JiraBinding {
                project_key: "ADR".to_string(),
                ..Default::default()
            },
            confluence: ConfluenceBinding {
                space_key: "SD".to_string(),
                space_id: None,
            },
        };
        binding.save(paths).unwrap();
    }

    #[test]
    fn should_skip_without_binding() {
        // AC-15: sin binding no se empuja nada y no se avisa nada.
        let dir = tempfile::tempdir().unwrap();
        let paths = paths_in(dir.path());
        assert_eq!(should_push_with(&paths, true).err(), Some(Skip::NoBinding));
        assert!(Skip::NoBinding.note().is_none());
    }

    #[test]
    fn should_skip_without_token() {
        // AC-12: con binding pero sin credenciales, la outbox espera al agente.
        let dir = tempfile::tempdir().unwrap();
        let paths = paths_in(dir.path());
        write_binding(&paths, true);
        assert_eq!(should_push_with(&paths, false).err(), Some(Skip::NoToken));
        assert!(Skip::NoToken.note().unwrap().contains("atlassian drain"));
    }

    #[test]
    fn should_skip_when_binding_disables_auto() {
        // AC-13: `"auto": false` apaga el envio sin perder el binding.
        let dir = tempfile::tempdir().unwrap();
        let paths = paths_in(dir.path());
        write_binding(&paths, false);
        assert_eq!(should_push_with(&paths, true).err(), Some(Skip::DisabledInBinding));
    }

    #[test]
    fn should_push_with_binding_token_and_auto() {
        let dir = tempfile::tempdir().unwrap();
        let paths = paths_in(dir.path());
        write_binding(&paths, true);
        assert!(should_push_with(&paths, true).is_ok());
        // Y el token real de la maquina no influye en la decision testeada.
        assert_eq!(should_push_with(&paths, false).err(), Some(Skip::NoToken));
    }

    #[test]
    fn push_bg_should_do_nothing_when_locked() {
        // AC-4: con el lock tomado no se lanza un segundo worker (y el lock
        // ajeno queda intacto).
        let dir = tempfile::tempdir().unwrap();
        let paths = paths_in(dir.path());
        write_binding(&paths, true);
        std::fs::write(
            paths.repo_root.join(".harness.env"),
            "HARNESS_ATLASSIAN_EMAIL=a@b.cl\nHARNESS_ATLASSIAN_TOKEN=secreto\n",
        )
        .unwrap();
        std::fs::create_dir_all(atlassian_dir(&paths)).unwrap();
        std::fs::create_dir(lock_path(&paths)).unwrap();
        push_bg(&paths);
        assert!(lock_path(&paths).is_dir(), "el lock ajeno sigue ahi");
        // Y no se creo el log: nunca se lanzo un worker.
        assert!(!log_path(&paths).exists());
    }

    #[test]
    fn push_bg_should_never_panic_without_binding() {
        // AC-2: el disparo es best-effort puro.
        let dir = tempfile::tempdir().unwrap();
        push_bg(&paths_in(dir.path()));
        push_bg(&HarnessPaths::from_root(PathBuf::from("/dev/null/no-existe")));
    }
}
