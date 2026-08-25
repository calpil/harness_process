//! Arbol CLI (paridad con los argparse de harness.py y graph_memory.py).
//! Divergencia aceptada: clap no abrevia flags (--feat) y los textos de
//! usage/error difieren; los exit codes (2 en error de uso) coinciden.

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

use crate::paths::HarnessPaths;
use crate::{atlassian, commands, graph, graphify};

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
    /// Inicia una feature (crea plan + firma, y su rama + worktree)
    Start {
        #[arg(long)]
        feature: String,
        /// No crear rama ni worktree: trabajar en el checkout actual
        #[arg(long = "sin-worktree")]
        sin_worktree: bool,
    },
    /// Cierra una feature (archiva estado, refresca memorias)
    Close {
        #[arg(long)]
        feature: String,
        #[arg(long, value_parser = clap::builder::PossibleValuesParser::new(["done", "blocked", "pending", "superseded"]))]
        status: String,
        /// Feature que absorbio este trabajo (obligatorio con --status superseded)
        #[arg(long = "absorbida-por")]
        absorbida_por: Option<String>,
        #[arg(long)]
        note: Option<String>,
        /// Rama a la que se integra el cierre `done` (GitFlow). Sin esto el
        /// arnes se niega: la decide el USUARIO.
        #[arg(long = "to")]
        to: Option<String>,
        /// Que se aprendio: la clase de `docs/lecciones/`, o `ninguna` (#17).
        #[arg(long)]
        leccion: Option<String>,
        /// Por que no hubo nada que aprender (obligatorio con `--leccion ninguna`).
        #[arg(long = "leccion-motivo")]
        leccion_motivo: Option<String>,
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
        /// Que es esto en Jira: `feature` (default), `bug` o `task`.
        #[arg(long)]
        kind: Option<String>,
    },
    /// PRDs anidados: el arbol de producto de docs/prd/
    Prd {
        #[command(subcommand)]
        command: PrdCommand,
    },
    /// Lecciones: la memoria procedural de docs/lecciones/
    Leccion {
        #[command(subcommand)]
        command: LeccionCommand,
    },
    /// Busca en los artefactos del proceso (specs, planes, lecciones, bitacora)
    Buscar {
        /// Terminos a buscar (en cualquier orden, sin importar mayusculas)
        consulta: Vec<String>,
        #[arg(long)]
        json: bool,
        /// No cortar en los primeros 20 resultados
        #[arg(long)]
        todos: bool,
    },
    /// Rutas protegidas: lista la config, o consulta si una ruta lo esta (exit 2)
    Rutas {
        /// Ruta(s) a consultar. Sin ninguna, lista la configuracion vigente.
        #[arg(long = "check")]
        check: Vec<String>,
        /// Rutas protegidas modificadas y sin commitear (exit 2 si hay alguna)
        #[arg(long)]
        violaciones: bool,
        /// Toma el estado actual como linea de base (para adoptar la proteccion
        /// con trabajo ya en curso). Lo corre una persona, nunca un hook.
        #[arg(long = "aceptar-estado-actual")]
        aceptar: bool,
        #[arg(long)]
        json: bool,
    },
    /// Diagnostica la INSTALACION (binario, hooks, superficies, marker, hub)
    Doctor {
        #[arg(long)]
        json: bool,
    },
    /// Ejecuta los comandos que los AC del spec declaran (exige spec aprobado)
    Verify {
        #[arg(long)]
        feature: String,
        #[arg(long)]
        json: bool,
        /// Correr solo un AC (por ejemplo: --solo AC-7)
        #[arg(long)]
        solo: Option<String>,
    },
    /// Mapa de lo aprendido: linea de tiempo, enlaces y huecos (solo lectura)
    Journey {
        #[arg(long)]
        json: bool,
    },
    /// Curador de la biblioteca de lecciones (ciclo de vida, pin, backup)
    Lecciones {
        #[command(subcommand)]
        command: LeccionesCommand,
    },
    /// Perfil: como quiere trabajar el usuario (docs/perfil-usuario.md)
    Perfil {
        #[command(subcommand)]
        command: PerfilCommand,
    },
    /// Paquete de contexto para EMPEZAR a implementar (solo lectura)
    Contexto {
        #[arg(long)]
        feature: Option<String>,
        /// Tema a mapear cuando todavia no hay feature
        #[arg(long)]
        tema: Option<String>,
        /// Presupuesto de lineas del mapa (default 300)
        #[arg(long = "max-lineas")]
        max_lineas: Option<usize>,
        /// Ademas del paquete, corre `graphify query` (cuesta: por eso no es el default)
        #[arg(long = "con-grafo")]
        con_grafo: bool,
        #[arg(long)]
        json: bool,
    },
    /// Paquete minimo de revision de una feature (solo lectura)
    Revision {
        #[arg(long)]
        feature: Option<String>,
        /// Presupuesto de lineas del diff (default 400)
        #[arg(long = "max-lineas")]
        max_lineas: Option<usize>,
        #[arg(long)]
        json: bool,
    },
    /// Integracion con Atlassian: binding, outbox, Jira, sprints y Confluence
    Atlassian {
        #[command(subcommand)]
        command: AtlassianCommand,
    },
    /// Memory Hub PostgreSQL (port de graph_memory.py)
    Graph {
        #[command(subcommand)]
        command: GraphCommand,
    },
    /// Interno: worker detached del envio a Atlassian (no usar a mano)
    #[command(name = "atlassian-worker", hide = true)]
    AtlassianWorker {
        #[arg(long)]
        root: PathBuf,
        #[arg(long)]
        lock: PathBuf,
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
pub enum AtlassianCommand {
    /// Registra a que proyecto Jira y a que space pertenece este repo
    Bind {
        /// Host del sitio (`calpil.atlassian.net`)
        #[arg(long)]
        site: Option<String>,
        /// Clave del proyecto Jira (`ADR`)
        #[arg(long = "jira-project")]
        jira_project: Option<String>,
        /// Clave del space de Confluence (`SD`)
        #[arg(long = "confluence-space")]
        confluence_space: Option<String>,
        /// Tipo de issue para una feature del backlog (default `Story`)
        #[arg(long = "issue-type")]
        issue_type: Option<String>,
        /// Reactiva la integracion sin perder el mapeo
        #[arg(long)]
        enable: bool,
        /// Apaga la integracion sin perder el mapeo
        #[arg(long)]
        disable: bool,
        /// Si el proyecto Jira no existe, crearlo (requiere permiso de admin)
        #[arg(long = "create-project")]
        create_project: bool,
        /// Si el space de Confluence no existe, crearlo (requiere permiso)
        #[arg(long = "create-space")]
        create_space: bool,
    },
    /// Binding vigente, mapeo local -> remoto y pendientes
    Status,
    /// Carga en Jira lo que ya existe en el repo (PRDs y backlog)
    Backfill {
        /// No bajar las subtasks de los AC-n (util en repos grandes)
        #[arg(long = "sin-acs")]
        sin_acs: bool,
    },
    /// Plan de llamadas MCP para que lo ejecute un agente (no muta nada)
    Drain,
    /// Registra la clave que devolvio Jira para un intent
    Ack {
        #[arg(long)]
        intent: String,
        /// Clave creada (`ADR-42`); no hace falta en comentarios y transiciones
        #[arg(long)]
        key: Option<String>,
    },
    /// Ejecuta los intents pendientes contra la API (requiere token)
    Apply,
    /// Sprints via Agile API (lo que el MCP no puede hacer)
    Sprint {
        #[command(subcommand)]
        command: SprintCommand,
    },
    /// Publica PRD, SDD y specs en Confluence
    Publish,
}

#[derive(Subcommand)]
pub enum SprintCommand {
    /// Abre un sprint en el board del proyecto y lo deja vigente
    Start {
        #[arg(long)]
        name: String,
        #[arg(long)]
        goal: Option<String>,
        /// Duracion en dias (default 14)
        #[arg(long, default_value_t = 14)]
        days: i64,
    },
    /// Cierra el sprint vigente y reporta lo que quedo sin terminar
    Close,
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
    /// Siembra la propuesta de documentos al dia (una pregunta por documento)
    Propose {
        #[arg(long)]
        feature: String,
    },
    /// Aplica la propuesta a los documentos del USUARIO (exige --yes)
    Apply {
        #[arg(long)]
        feature: String,
        /// Confirmacion explicita del USUARIO: sin este flag el comando se niega
        #[arg(long)]
        yes: bool,
    },
    /// Dibuja el arbol con hitos y estado de features
    Tree {
        /// Subarbol a dibujar (`master` por defecto)
        #[arg(long)]
        prd: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum LeccionCommand {
    /// Catalogo de lecciones, ordenado por uso
    List {
        #[arg(long)]
        json: bool,
        /// Lista las archivadas en vez del catalogo activo
        #[arg(long)]
        archivadas: bool,
    },
    /// Imprime una leccion entera
    Show { nombre: String },
    /// Crea una leccion de CLASE (el ULTIMO recurso: primero patchea una existente)
    Nueva { nombre: String },
    /// Deja rastro de que una leccion sirvio (+1 uso)
    Usar { nombre: String },
}

#[derive(Subcommand)]
pub enum LeccionesCommand {
    /// Que esta vivo, que se enfria y que esta por vencer
    Status {
        #[arg(long)]
        json: bool,
    },
    /// Pasada del curador. Sin --aplicar solo INFORMA: no toca ningun archivo
    Curar {
        /// Aplica las transiciones (respalda antes y deja reporte)
        #[arg(long)]
        aplicar: bool,
    },
    /// Detecta lecciones solapadas con un LLM. Sin --aplicar solo INFORMA
    Consolidar {
        /// Prepara un borrador de paraguas; no archiva ni fusiona miembros
        #[arg(long)]
        preparar: bool,
        /// Aplica la fusion (respalda antes y archiva las miembros)
        #[arg(long)]
        aplicar: bool,
        /// Leccion paraguas que queda (puede ser una de las miembros)
        #[arg(long)]
        en: Option<String>,
        /// Lecciones a fusionar, separadas por coma
        #[arg(long)]
        de: Option<String>,
        /// Por que se fusionan (obligatorio con --aplicar)
        #[arg(long)]
        motivo: Option<String>,
    },
    /// Congela una leccion: ninguna transicion automatica la toca
    Pin { nombre: String },
    /// Devuelve una leccion al ciclo de vida normal
    Unpin { nombre: String },
    /// Mueve una leccion a docs/lecciones/archivo/ (no la borra)
    Archivar { nombre: String },
    /// Devuelve una leccion archivada al catalogo activo
    Restaurar { nombre: String },
    /// Deshace una pasada del curador (tambien es reversible)
    Rollback {
        /// Backup puntual; sin esto se usa el mas reciente
        #[arg(long)]
        id: Option<String>,
        /// Lista los backups disponibles en vez de restaurar
        #[arg(long)]
        list: bool,
    },
}

#[derive(Subcommand)]
pub enum PerfilCommand {
    /// Imprime el perfil y cuanto ocupa
    Show,
    /// Agrega una entrada (exige el SI del usuario)
    Add {
        #[arg(long)]
        texto: String,
        /// Confirmacion explicita del USUARIO: sin este flag el comando se niega
        #[arg(long)]
        yes: bool,
    },
    /// Reemplaza una entrada, ubicada por subcadena unica (exige el SI)
    Replace {
        /// Fragmento que identifica UNA entrada
        #[arg(long)]
        old: String,
        #[arg(long)]
        texto: String,
        #[arg(long)]
        yes: bool,
    },
    /// Quita una entrada, ubicada por subcadena unica (exige el SI)
    Remove {
        #[arg(long)]
        old: String,
        #[arg(long)]
        yes: bool,
    },
    /// Junta la evidencia ya escrita en el repo y emite el contrato (no escribe)
    Sugerir,
    /// Valida el perfil (limite y formato); lo usa harness_check.sh
    Check,
    /// Interno: imprime el bloque que el instalador inyecta en las superficies
    #[command(hide = true)]
    Bloque,
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
        Command::Start {
            feature,
            sin_worktree,
        } => commands::start::run(&HarnessPaths::resolve()?, &feature, sin_worktree),
        Command::Close {
            feature,
            status,
            note,
            to,
            absorbida_por,
            leccion,
            leccion_motivo,
        } => commands::close::run(
            &HarnessPaths::resolve()?,
            &feature,
            commands::close::CierreOpts {
                status: &status,
                note: note.as_deref(),
                absorbida_por: absorbida_por.as_deref(),
                leccion: leccion.as_deref(),
                leccion_motivo: leccion_motivo.as_deref(),
                to: to.as_deref(),
            },
        ),
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
            kind,
        } => commands::add::run(
            &HarnessPaths::resolve()?,
            &name,
            &service,
            &acceptance,
            prd.as_deref(),
            kind.as_deref(),
        ),
        Command::Prd { command } => match command {
            PrdCommand::Add { name, parent } => {
                commands::prd::add(&HarnessPaths::resolve()?, &name, parent.as_deref())
            }
            PrdCommand::Propose { feature } => {
                commands::prd::propose(&HarnessPaths::resolve()?, &feature)
            }
            PrdCommand::Apply { feature, yes } => {
                commands::prd::apply(&HarnessPaths::resolve()?, &feature, yes)
            }
            PrdCommand::Tree { prd } => {
                commands::prd::tree(&HarnessPaths::resolve()?, prd.as_deref())
            }
        },
        Command::Leccion { command } => {
            let paths = HarnessPaths::resolve()?;
            match command {
                LeccionCommand::List { json, archivadas } => {
                    commands::leccion::list(&paths, json, archivadas)
                }
                LeccionCommand::Show { nombre } => commands::leccion::show(&paths, &nombre),
                LeccionCommand::Nueva { nombre } => commands::leccion::nueva(&paths, &nombre),
                LeccionCommand::Usar { nombre } => commands::leccion::usar(&paths, &nombre),
            }
        }
        Command::Buscar {
            consulta,
            json,
            todos,
        } => commands::buscar::run(&HarnessPaths::resolve()?, &consulta.join(" "), json, todos),
        Command::Rutas {
            check,
            violaciones,
            aceptar,
            json,
        } => commands::rutas::run(
            &HarnessPaths::resolve()?,
            &check,
            violaciones,
            aceptar,
            json,
        ),
        Command::Doctor { json } => commands::doctor::run(&HarnessPaths::resolve()?, json),
        Command::Verify {
            feature,
            json,
            solo,
        } => commands::verify::run(&HarnessPaths::resolve()?, &feature, json, solo.as_deref()),
        Command::Journey { json } => commands::journey::run(&HarnessPaths::resolve()?, json),
        Command::Lecciones { command } => {
            let paths = HarnessPaths::resolve()?;
            match command {
                LeccionesCommand::Status { json } => commands::leccion::status(&paths, json),
                LeccionesCommand::Curar { aplicar } => commands::leccion::curar(&paths, aplicar),
                LeccionesCommand::Consolidar {
                    preparar,
                    aplicar,
                    en,
                    de,
                    motivo,
                } => commands::leccion::consolidar(
                    &paths,
                    preparar,
                    aplicar,
                    en.as_deref(),
                    de.as_deref(),
                    motivo.as_deref(),
                ),
                LeccionesCommand::Pin { nombre } => commands::leccion::pin(&paths, &nombre, true),
                LeccionesCommand::Unpin { nombre } => {
                    commands::leccion::pin(&paths, &nombre, false)
                }
                LeccionesCommand::Archivar { nombre } => {
                    commands::leccion::archivar(&paths, &nombre)
                }
                LeccionesCommand::Restaurar { nombre } => {
                    commands::leccion::restaurar(&paths, &nombre)
                }
                LeccionesCommand::Rollback { id, list } => {
                    commands::leccion::rollback(&paths, id.as_deref(), list)
                }
            }
        }
        Command::Perfil { command } => {
            let paths = HarnessPaths::resolve()?;
            match command {
                PerfilCommand::Show => commands::perfil::show(&paths),
                PerfilCommand::Add { texto, yes } => commands::perfil::add(&paths, &texto, yes),
                PerfilCommand::Replace { old, texto, yes } => {
                    commands::perfil::replace(&paths, &old, &texto, yes)
                }
                PerfilCommand::Remove { old, yes } => commands::perfil::remove(&paths, &old, yes),
                PerfilCommand::Sugerir => commands::perfil::sugerir(&paths),
                PerfilCommand::Check => commands::perfil::check(&paths),
                PerfilCommand::Bloque => commands::perfil::bloque(&paths),
            }
        }
        Command::Atlassian { command } => {
            let paths = HarnessPaths::resolve()?;
            match command {
                AtlassianCommand::Bind {
                    site,
                    jira_project,
                    confluence_space,
                    issue_type,
                    enable,
                    disable,
                    create_project,
                    create_space,
                } => commands::atlassian::bind(
                    &paths,
                    site.as_deref(),
                    jira_project.as_deref(),
                    confluence_space.as_deref(),
                    issue_type.as_deref(),
                    enable,
                    disable,
                    create_project,
                    create_space,
                ),
                AtlassianCommand::Backfill { sin_acs } => {
                    commands::atlassian::backfill(&paths, sin_acs)
                }
                AtlassianCommand::Status => commands::atlassian::status(&paths),
                AtlassianCommand::Drain => commands::atlassian::drain(&paths),
                AtlassianCommand::Ack { intent, key } => {
                    commands::atlassian::ack(&paths, &intent, key.as_deref())
                }
                AtlassianCommand::Apply => commands::atlassian::apply(&paths),
                AtlassianCommand::Sprint { command } => match command {
                    SprintCommand::Start { name, goal, days } => {
                        commands::atlassian::sprint_start(&paths, &name, goal.as_deref(), days)
                    }
                    SprintCommand::Close => commands::atlassian::sprint_close(&paths),
                },
                AtlassianCommand::Publish => commands::atlassian::publish(&paths),
            }
        }
        Command::Contexto {
            feature,
            tema,
            max_lineas,
            con_grafo,
            json,
        } => commands::contexto::run(
            &HarnessPaths::resolve()?,
            commands::contexto::Opts {
                feature: feature.as_deref(),
                tema: tema.as_deref(),
                max_lineas,
                con_grafo,
                json,
            },
        ),
        Command::Revision {
            feature,
            max_lineas,
            json,
        } => commands::revision::run(
            &HarnessPaths::resolve()?,
            feature.as_deref(),
            max_lineas,
            json,
        ),
        Command::Graph { command } => graph::run(command),
        Command::AtlassianWorker { root, lock } => atlassian::push::worker(&root, &lock),
        Command::GraphifyWorker { root, stale, lock } => graphify::worker(&root, &stale, &lock),
    }
}
