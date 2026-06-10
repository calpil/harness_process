//! Memory Hub PostgreSQL (paridad: graph_memory.py). El manager se construye
//! ANTES de validar argumentos por comando, replicando el orden de Python
//! (los errores de credenciales/conexion salen con exit 1 antes que los de
//! uso, que salen con exit 2).

pub mod commands;
pub mod derive;
pub mod ids;
pub mod store;
pub mod tls;

use std::path::PathBuf;

use anyhow::Context;

use crate::cli::GraphCommand;
use crate::exit::Exit;
use crate::pycompat::{abspath, env_nonempty, home_dir};

use store::PgGraphStore;

/// Entorno del lado graph_memory.py (BASE_DIR = dir del ejecutable).
pub struct GraphEnv {
    pub base_dir: PathBuf,
    pub repo_root: PathBuf,
    pub project: String,
    pub hub_dir: PathBuf,
    pub lock_file: PathBuf,
}

impl GraphEnv {
    pub fn resolve() -> anyhow::Result<Self> {
        let exe = std::env::current_exe().context("no se pudo resolver el ejecutable")?;
        let base_dir = exe
            .parent()
            .context("el ejecutable no tiene directorio padre")?
            .to_path_buf();
        // Paridad graph_memory.py: el env SI se normaliza con abspath.
        let repo_root = match env_nonempty("HARNESS_REPO_ROOT") {
            Some(v) => abspath(&PathBuf::from(v)),
            None => crate::paths::repo_root_from_marker(&base_dir),
        };
        let project = env_nonempty("HARNESS_PROJECT").unwrap_or_else(|| {
            repo_root
                .file_name()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_default()
        });
        let hub_dir = match env_nonempty("HARNESS_HUB") {
            Some(v) => PathBuf::from(v),
            None => home_dir().unwrap_or_default().join(".harness-hub"),
        };
        Ok(GraphEnv {
            lock_file: hub_dir.join(".lock"),
            base_dir,
            repo_root,
            project,
            hub_dir,
        })
    }
}

pub struct GraphMemoryManager {
    pub env: GraphEnv,
    pub store: PgGraphStore,
    pub hub_location: String,
}

impl GraphMemoryManager {
    pub fn new() -> anyhow::Result<Self> {
        let env = GraphEnv::resolve()?;
        // .env del hub: setdefault -> el entorno del proceso SIEMPRE gana,
        // incluso con valor vacio (truthiness se evalua despues).
        let env_file = env.hub_dir.join(".env");
        let mut file_vars: Vec<(String, String)> = Vec::new();
        if env_file.exists() {
            let text = std::fs::read_to_string(&env_file)
                .with_context(|| format!("no se pudo leer {}", env_file.display()))?;
            for line in text.lines() {
                let line = line.trim();
                if !line.is_empty() && !line.starts_with('#') && line.contains('=') {
                    let (key, val) = line.split_once('=').unwrap_or((line, ""));
                    file_vars.push((
                        key.trim().to_string(),
                        val.trim()
                            .trim_matches(|c| c == '\'' || c == '"')
                            .to_string(),
                    ));
                }
            }
        }
        let lookup = |name: &str| -> Option<String> {
            match std::env::var(name) {
                Ok(v) => Some(v),
                Err(_) => file_vars
                    .iter()
                    .find(|(k, _)| k == name)
                    .map(|(_, v)| v.clone()),
            }
        };
        let required = ["DB_HOST", "DB_USER", "DB_PASSWORD"];
        let missing: Vec<&str> = required
            .iter()
            .copied()
            .filter(|name| lookup(name).filter(|v| !v.is_empty()).is_none())
            .collect();
        if !missing.is_empty() {
            return Err(Exit::msg(format!(
                "El Memory Hub PostgreSQL requiere: {}",
                missing.join(", ")
            ))
            .into());
        }
        let get = |name: &str| lookup(name).unwrap_or_default();
        let dbname = lookup("DB_NAME")
            .filter(|v| !v.is_empty())
            .unwrap_or_else(|| "postgres".to_string());
        let port = lookup("DB_PORT")
            .filter(|v| !v.is_empty())
            .unwrap_or_else(|| "5432".to_string());
        let sslmode = lookup("DB_SSL_MODE")
            .filter(|v| !v.is_empty())
            .unwrap_or_else(|| "require".to_string());
        let host = get("DB_HOST");
        let store = PgGraphStore::new(
            &dbname,
            &get("DB_USER"),
            &get("DB_PASSWORD"),
            &host,
            &port,
            &sslmode,
        )?;
        let hub_location = format!("{dbname}@{host}:{port}");
        std::fs::create_dir_all(&env.hub_dir)?;
        Ok(GraphMemoryManager {
            env,
            store,
            hub_location,
        })
    }

    /// `hub_lock()`: flock exclusivo sobre HUB_DIR/.lock alrededor de `f`.
    pub fn locked<T>(
        &mut self,
        f: impl FnOnce(&mut Self) -> anyhow::Result<T>,
    ) -> anyhow::Result<T> {
        std::fs::create_dir_all(&self.env.hub_dir)?;
        let file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true) // open(LOCK_FILE, "w")
            .open(&self.env.lock_file)?;
        let mut lock = fd_lock::RwLock::new(file);
        let _guard = lock.write()?;
        f(self)
    }
}

/// `parser.error(...)` de argparse: mensaje a stderr + exit 2.
fn usage_error(msg: &str) -> anyhow::Error {
    eprintln!("error: {msg}");
    Exit::code(2).into()
}

pub fn run(command: GraphCommand) -> anyhow::Result<()> {
    // Manager primero (paridad con el orden de graph_memory.py main()).
    let mut manager = GraphMemoryManager::new()?;
    match &command {
        GraphCommand::Registrar(o) => {
            let (Some(accion), Some(estado), Some(artefacto)) =
                (nonempty(&o.accion), nonempty(&o.estado), nonempty(&o.artefacto))
            else {
                return Err(usage_error("registrar requiere --accion, --estado y --artefacto"));
            };
            manager.record_event(&o.agente, accion, artefacto, estado, o.meta.as_deref())
        }
        GraphCommand::Consultar(o) => {
            let Some(artefacto) = nonempty(&o.artefacto) else {
                return Err(usage_error("--artefacto es requerido para consultar"));
            };
            manager.query_state(artefacto, &o.microservicio)
        }
        GraphCommand::Descubrir(_) => manager.discover(),
        GraphCommand::Mapa(_) => manager.map(),
        GraphCommand::Impacto(o) => manager.impact(&o.microservicio),
        GraphCommand::Vincular(o) => {
            let Some(destino) = nonempty(&o.destino) else {
                return Err(usage_error("--destino es requerido para vincular"));
            };
            manager.link(&o.microservicio, destino, o.transversal)
        }
        GraphCommand::Desmarcar(o) => manager.unmark(&o.microservicio),
        GraphCommand::SyncGit(o) => {
            let Some(artefacto) = nonempty(&o.artefacto) else {
                return Err(usage_error("--artefacto es requerido para sync_git"));
            };
            // Python: args.meta.split(",") if args.meta else []
            let files: Vec<&str> = match o.meta.as_deref() {
                Some(m) if !m.is_empty() => m.split(',').collect(),
                _ => Vec::new(),
            };
            manager.sync_git(artefacto, &files, &o.microservicio)
        }
        GraphCommand::VincularGrafo(_) => manager.derive_from_graphify(),
    }
}

/// Truthiness de Python para Option<String> de argparse.
fn nonempty(opt: &Option<String>) -> Option<&str> {
    opt.as_deref().filter(|s| !s.is_empty())
}
