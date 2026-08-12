//! Arbol CLI (paridad con los argparse de harness.py y graph_memory.py).
//! Divergencia aceptada: clap no abrevia flags (--feat) y los textos de
//! usage/error difieren; los exit codes (2 en error de uso) coinciden.

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

use crate::paths::HarnessPaths;
use crate::{commands, graph, graphify};

#[derive(Parser)]
#[command(
    name = "harness",
    version,
    about = "Harness Process: ciclo de vida de features + Memory Hub PostgreSQL"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Backlog + estado vivo + frescura de planes
    Status,
    /// Primera feature pending (JSON)
    Next,
    /// Inicia una feature (crea plan + firma)
    Start {
        #[arg(long)]
        feature: String,
    },
    /// Cierra una feature (archiva estado, refresca memorias)
    Close {
        #[arg(long)]
        feature: String,
        #[arg(long, value_parser = clap::builder::PossibleValuesParser::new(["done", "blocked", "pending"]))]
        status: String,
        #[arg(long)]
        note: Option<String>,
    },
    /// Registra un hito intermedio de la feature activa
    Advance {
        #[arg(long)]
        feature: Option<String>,
        #[arg(long)]
        nota: String,
        #[arg(long = "no-graphify")]
        no_graphify: bool,
    },
    /// Checkpoint automatico para hooks (silencioso, best-effort)
    Autocheck {
        #[arg(long = "no-graphify")]
        no_graphify: bool,
    },
    /// Aviso no bloqueante para hooks post-tool
    Nudge,
    /// Gate multi-LLM: exit 2 si el plan fue editado por otro agente
    #[command(name = "check-plan")]
    CheckPlan {
        #[arg(long)]
        feature: Option<String>,
    },
    /// Gate SDD: exit 2 si el spec esta stale o (regla activa) sin aprobar
    #[command(name = "check-spec")]
    CheckSpec {
        #[arg(long)]
        feature: Option<String>,
    },
    /// Registra la aprobacion del USUARIO sobre el spec (exige --yes)
    #[command(name = "approve-spec")]
    ApproveSpec {
        #[arg(long)]
        feature: Option<String>,
        /// Confirmacion explicita del USUARIO: sin este flag el comando se niega
        #[arg(long)]
        yes: bool,
        #[arg(long)]
        nota: Option<String>,
    },
    /// Agrega una feature al backlog
    Add {
        #[arg(long)]
        name: String,
        #[arg(long = "service")]
        service: Vec<String>,
        #[arg(long = "acceptance")]
        acceptance: Vec<String>,
        /// PRD del que sale este hito (ruta `cobranza/mora`, ultimo segmento si
        /// es unico, o `master`). Sin el, la feature cuenta para el maestro.
        #[arg(long)]
        prd: Option<String>,
    },
    /// PRDs anidados: el arbol de producto de docs/prd/
    Prd {
        #[command(subcommand)]
        command: PrdCommand,
    },
    /// Memory Hub PostgreSQL (port de graph_memory.py)
    Graph {
        #[command(subcommand)]
        command: GraphCommand,
    },
    /// Interno: worker detached del refresh de graphify (no usar a mano)
    #[command(name = "graphify-worker", hide = true)]
    GraphifyWorker {
        #[arg(long)]
        root: PathBuf,
        #[arg(long)]
        stale: PathBuf,
        #[arg(long)]
        lock: PathBuf,
    },
}

#[derive(Subcommand)]
pub enum PrdCommand {
    /// Crea un PRD hijo desde plantilla y lo enlaza en su padre
    Add {
        #[arg(long)]
        name: String,
        /// PRD padre (`master` por defecto)
        #[arg(long)]
        parent: Option<String>,
    },
    /// Dibuja el arbol con hitos y estado de features
    Tree {
        /// Subarbol a dibujar (`master` por defecto)
        #[arg(long)]
        prd: Option<String>,
    },
}

/// graph_memory.py acepta TODOS los flags en cualquier comando (argparse
/// global); cada subcomando comparte el mismo set para no romper llamadas.
#[derive(Args, Clone)]
pub struct GraphOpts {
    #[arg(long, default_value = "raiz")]
    pub microservicio: String,
    #[arg(long)]
    pub destino: Option<String>,
    #[arg(long)]
    pub transversal: bool,
    #[arg(long)]
    pub artefacto: Option<String>,
    #[arg(long)]
    pub meta: Option<String>,
    #[arg(long, default_value = "AgentCLI")]
    pub agente: String,
    #[arg(long)]
    pub accion: Option<String>,
    #[arg(long)]
    pub estado: Option<String>,
}

#[derive(Subcommand)]
pub enum GraphCommand {
    /// Descubre microservicios (repos git) bajo la raiz multi-repo
    Descubrir(GraphOpts),
    /// Mapa completo del hub
    Mapa(GraphOpts),
    /// Quien depende del microservicio dado
    Impacto(GraphOpts),
    /// Registra dependencia consumidor -> destino
    Vincular(GraphOpts),
    /// Quita la marca transversal
    Desmarcar(GraphOpts),
    /// Sincroniza un commit (lo llama el hook post-commit)
    #[command(name = "sync_git")]
    SyncGit(GraphOpts),
    /// Deriva dependencias desde graphify-out/graph.json
    #[command(name = "vincular-grafo")]
    VincularGrafo(GraphOpts),
    /// Registra un evento agente->artefacto
    Registrar(GraphOpts),
    /// Consulta un artefacto (JSON)
    Consultar(GraphOpts),
}

pub fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Status => commands::status::run(&HarnessPaths::resolve()?),
        Command::Next => commands::next::run(&HarnessPaths::resolve()?),
        Command::Start { feature } => commands::start::run(&HarnessPaths::resolve()?, &feature),
        Command::Close {
            feature,
            status,
            note,
        } => commands::close::run(&HarnessPaths::resolve()?, &feature, &status, note.as_deref()),
        Command::Advance {
            feature,
            nota,
            no_graphify,
        } => commands::advance::run(
            &HarnessPaths::resolve()?,
            feature.as_deref(),
            &nota,
            no_graphify,
        ),
        Command::Autocheck { no_graphify } => {
            commands::autocheck::run(&HarnessPaths::resolve()?, no_graphify)
        }
        Command::Nudge => commands::nudge::run(&HarnessPaths::resolve()?),
        Command::CheckPlan { feature } => {
            commands::check_plan::run(&HarnessPaths::resolve()?, feature.as_deref())
        }
        Command::CheckSpec { feature } => {
            commands::check_spec::run(&HarnessPaths::resolve()?, feature.as_deref())
        }
        Command::ApproveSpec { feature, yes, nota } => commands::approve_spec::run(
            &HarnessPaths::resolve()?,
            feature.as_deref(),
            yes,
            nota.as_deref(),
        ),
        Command::Add {
            name,
            service,
            acceptance,
            prd,
        } => commands::add::run(
            &HarnessPaths::resolve()?,
            &name,
            &service,
            &acceptance,
            prd.as_deref(),
        ),
        Command::Prd { command } => match command {
            PrdCommand::Add { name, parent } => {
                commands::prd::add(&HarnessPaths::resolve()?, &name, parent.as_deref())
            }
            PrdCommand::Tree { prd } => {
                commands::prd::tree(&HarnessPaths::resolve()?, prd.as_deref())
            }
        },
        Command::Graph { command } => graph::run(command),
        Command::GraphifyWorker { root, stale, lock } => graphify::worker(&root, &stale, &lock),
    }
}
