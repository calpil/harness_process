//! `harness atlassian ...` (spec #15): binding, outbox y los dos ejecutores.
//!
//! - `bind`    a que proyecto Jira y a que space pertenece este repo (AC-1/5)
//! - `status`  binding, token, mapeo y pendientes (AC-12)
//! - `drain`   plan de llamadas MCP, sin mutar nada (AC-9)
//! - `ack`     el agente devuelve la clave creada (AC-10)
//! - `apply`   ejecutor REST con token (AC-15..AC-18)
//! - `sprint`  abrir y cerrar sprints via Agile API (AC-19..AC-21)
//! - `publish` PRD, SDD y specs a Confluence (AC-22..AC-24)

use serde_json::{Value, json};

use crate::atlassian::binding::{Binding, ConfluenceBinding, JiraBinding};
use crate::atlassian::confluence;
use crate::atlassian::emit;
use crate::atlassian::http::{Client, Credentials};
use crate::atlassian::jira;
use crate::atlassian::markdown;
use crate::atlassian::outbox::{self, Intent, IntentKind};
use crate::atlassian::state::{SprintRemote, State};
use crate::exit::Exit;
use crate::features::{features_slice, load_features};
use crate::paths::HarnessPaths;
use crate::pycompat::py_str;

/// Mensaje canonico cuando el arnes no sabe a donde pertenece el repo (AC-5).
const ASK_USER: &str = concat!(
    "No se a que proyecto de Jira ni a que space de Confluence pertenece este repo, y no lo voy a adivinar.\n",
    "  Preguntale al USUARIO las dos cosas y registralo:\n",
    "    sh harness_cli atlassian bind --site <sitio>.atlassian.net --jira-project <KEY> --confluence-space <KEY>\n",
    "  (o corre el instalador con --atlassian-site/--jira-project/--confluence-space)"
);

/// Carga el binding activo o corta con exit 2 explicando que preguntar.
fn require_binding(paths: &HarnessPaths) -> Result<Binding, Exit> {
    match Binding::load(paths) {
        Some(b) if b.is_active() => Ok(b),
        Some(_) => Err(Exit::msg(format!(
            "[Atlassian] el binding existe pero esta apagado o sin proyecto ({}).\n  Reactivalo con: sh harness_cli atlassian bind --enable",
            Binding::path(paths).display()
        ))),
        None => Err(Exit {
            code: 2,
            message: Some(format!("[Atlassian] {ASK_USER}")),
        }),
    }
}

/// Cliente REST o exit 2 indicando la alternativa con agente (AC-18).
fn require_client(paths: &HarnessPaths, binding: &Binding) -> Result<Client, Exit> {
    match Credentials::discover(paths) {
        Some(creds) => Ok(Client::new(binding, &creds)),
        None => Err(Exit {
            code: 2,
            message: Some(format!(
                concat!(
                    "[Atlassian] no hay credenciales para hablar con la API.\n",
                    "  Opcion 1 (sin token): sh harness_cli atlassian drain  y que el agente con MCP ejecute el plan.\n",
                    "  Opcion 2 (con token): define {} y {} en .harness.env (no versionado) o en el entorno."
                ),
                crate::atlassian::http::ENV_EMAIL,
                crate::atlassian::http::ENV_TOKEN
            )),
        }),
    }
}

// --------------------------------------------------------------------------
// bind / status
// --------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
pub fn bind(
    paths: &HarnessPaths,
    site: Option<&str>,
    project: Option<&str>,
    space: Option<&str>,
    issue_type: Option<&str>,
    enable: bool,
    disable: bool,
) -> anyhow::Result<()> {
    let previous = Binding::load(paths);
    let site = site
        .map(str::to_string)
        .or_else(|| previous.as_ref().map(|b| b.site.clone()));
    let project = project
        .map(str::to_string)
        .or_else(|| previous.as_ref().map(|b| b.jira.project_key.clone()))
        .filter(|p| !p.trim().is_empty());
    let space = space
        .map(str::to_string)
        .or_else(|| previous.as_ref().map(|b| b.confluence.space_key.clone()))
        .filter(|s| !s.trim().is_empty());

    // AC-5: sin proyecto (o sin sitio) no se inventa nada; se dice que preguntar.
    let (Some(site), Some(project)) = (site, project) else {
        return Err(Exit {
            code: 2,
            message: Some(format!("[Atlassian] {ASK_USER}")),
        }
        .into());
    };
    if space.is_none() {
        eprintln!(
            "[Atlassian] aviso: sin space de Confluence, `publish` no va a poder publicar el PRD ni el SDD.\n  Preguntale al USUARIO a que space pertenece y agregalo con --confluence-space <KEY>."
        );
    }

    let mut binding = previous.unwrap_or(Binding {
        site: site.clone(),
        cloud_id: None,
        enabled: true,
        jira: JiraBinding::default(),
        confluence: ConfluenceBinding::default(),
    });
    binding.site = site;
    binding.jira.project_key = project;
    binding.confluence.space_key = space.unwrap_or_default();
    if let Some(t) = issue_type {
        binding.jira.issue_types.feature = t.to_string();
    }
    if enable {
        binding.enabled = true;
    }
    if disable {
        binding.enabled = false;
    }
    binding.save(paths)?;

    println!("[Atlassian] binding registrado en {}", Binding::path(paths).display());
    println!("  sitio    : {}", binding.site);
    println!(
        "  Jira     : proyecto {} (feature -> {}, PRD -> {}, AC-n -> {})",
        binding.jira.project_key,
        binding.jira.issue_types.feature,
        binding.jira.issue_types.epic,
        binding.jira.issue_types.ac
    );
    println!(
        "  Confluence: space {}",
        if binding.confluence.space_key.is_empty() {
            "(sin definir)"
        } else {
            &binding.confluence.space_key
        }
    );
    println!(
        "  estado   : {}",
        if binding.enabled { "activo" } else { "apagado" }
    );
    Ok(())
}

pub fn status(paths: &HarnessPaths) -> anyhow::Result<()> {
    let Some(binding) = Binding::load(paths) else {
        println!("[Atlassian] este repo no tiene binding: la integracion esta apagada.");
        println!("{ASK_USER}");
        return Ok(());
    };
    let state = State::load(paths);
    let pending = outbox::pending(paths);

    println!("== Atlassian ==");
    println!("Sitio      : {}", binding.site);
    println!(
        "Jira       : {} ({})",
        binding.jira.project_key,
        if binding.is_active() { "activo" } else { "apagado" }
    );
    println!(
        "Confluence : {}",
        if binding.confluence.space_key.is_empty() {
            "(sin definir)".to_string()
        } else {
            binding.confluence.space_key.clone()
        }
    );
    // AC-16: del token solo se dice si esta, nunca su valor.
    println!(
        "Token      : {}",
        match Credentials::discover(paths) {
            Some(c) => format!("presente ({})", c.email),
            None => "ausente (usa `drain` + agente con MCP)".to_string(),
        }
    );
    match &state.sprint {
        Some(s) => println!("Sprint     : #{} {} ({})", s.id, s.name, s.state),
        None => println!("Sprint     : ninguno vigente"),
    }

    println!("\nMapeo local -> remoto:");
    if state.prds.is_empty() && state.features.is_empty() && state.pages.is_empty() {
        println!("  (todavia nada publicado)");
    }
    for (slug, key) in &state.prds {
        println!("  PRD {slug} -> {key}");
    }
    for (fid, remote) in &state.features {
        let issue = remote.issue.as_deref().unwrap_or("-");
        println!("  feature #{fid} -> {issue} ({} subtask/s)", remote.acs.len());
    }
    for (doc, page) in &state.pages {
        println!("  {doc} -> pagina {} (v{})", page.id, page.version);
    }

    println!("\nIntents pendientes: {}", pending.len());
    for intent in pending.iter().take(20) {
        println!("  [{}] {} ({})", intent.id, intent.kind.label(), intent.origin);
    }
    if pending.len() > 20 {
        println!("  ... y {} mas", pending.len() - 20);
    }
    Ok(())
}

// --------------------------------------------------------------------------
// drain / ack (ejecutor con agente MCP)
// --------------------------------------------------------------------------

/// Resumen del issue de una feature: `#15 nombre`.
fn feature_summary(fid: &str, name: &str) -> String {
    format!("#{fid} {name}")
}

/// Resumen de la subtask de un AC: la convencion que el proyecto SCRUM ya usa.
fn ac_summary(ac: &str, text: &str) -> String {
    format!("{ac} · {text}")
}

/// Descripcion de la historia a partir de sus criterios de aceptacion.
fn feature_description(acceptance: &[String]) -> String {
    if acceptance.is_empty() {
        return "Feature del backlog del arnes.".to_string();
    }
    let mut out = String::from("Criterio de aceptacion del backlog:\n");
    for a in acceptance {
        out.push_str(a);
        out.push('\n');
    }
    out
}

/// Traduce un intent a la llamada MCP que el agente tiene que ejecutar.
/// `None` en `args.parent` significa que todavia falta el ack del padre.
fn mcp_call(binding: &Binding, state: &State, intent: &Intent) -> Value {
    let types = &binding.jira.issue_types;
    let project = &binding.jira.project_key;
    match &intent.kind {
        IntentKind::PrdEpic { title, body, .. } => json!({
            "tool": "createJiraIssue",
            "args": {
                "projectKey": project,
                "issueTypeName": types.epic,
                "summary": jira::truncate_summary(title),
                "description": body,
            }
        }),
        IntentKind::FeatureCreate {
            fid,
            name,
            acceptance,
            prd,
        } => {
            let parent = prd.as_ref().and_then(|s| state.prds.get(s)).cloned();
            json!({
                "tool": "createJiraIssue",
                "args": {
                    "projectKey": project,
                    "issueTypeName": types.feature,
                    "summary": jira::truncate_summary(&feature_summary(fid, name)),
                    "description": feature_description(acceptance),
                    "parent": parent,
                },
                "needs": parent.is_none().then(|| format!(
                    "el epic del PRD {} todavia no tiene clave: ejecuta antes su intent y hace ack",
                    prd.clone().unwrap_or_default()
                )),
            })
        }
        IntentKind::AcSubtask { fid, ac, text } => {
            let parent = state.feature_issue(fid).map(str::to_string);
            json!({
                "tool": "createJiraIssue",
                "args": {
                    "projectKey": project,
                    "issueTypeName": types.ac,
                    "summary": jira::truncate_summary(&ac_summary(ac, text)),
                    "description": text,
                    "parent": parent,
                },
                "needs": parent.is_none().then(|| format!(
                    "la historia de la feature #{fid} todavia no tiene clave"
                )),
            })
        }
        IntentKind::Transition { fid, to } => {
            let key = state.feature_issue(fid).map(str::to_string);
            json!({
                "tool": "transitionJiraIssue",
                "args": {"issueIdOrKey": key, "targetStatusName": to},
                "note": "resolve el id con getTransitionsForJiraIssue y pasalo como transition.id",
                "needs": key.is_none().then(|| format!("la feature #{fid} todavia no tiene issue")),
            })
        }
        IntentKind::Comment { fid, body } => {
            let key = state.feature_issue(fid).map(str::to_string);
            json!({
                "tool": "addCommentToJiraIssue",
                "args": {"issueIdOrKey": key, "commentBody": body},
                "needs": key.is_none().then(|| format!("la feature #{fid} todavia no tiene issue")),
            })
        }
        IntentKind::BlockedFlag { fid, on } => {
            let key = state.feature_issue(fid).map(str::to_string);
            let value = if *on {
                json!([{"value": binding.jira.statuses.blocked_flag}])
            } else {
                json!([])
            };
            json!({
                "tool": "editJiraIssue",
                "args": {
                    "issueIdOrKey": key,
                    "additional_fields": {"customfield_10021": value},
                },
                "needs": key.is_none().then(|| format!("la feature #{fid} todavia no tiene issue")),
            })
        }
    }
}

/// AC-9: imprime el plan ordenado por dependencia y NO muta nada.
pub fn drain(paths: &HarnessPaths) -> anyhow::Result<()> {
    let binding = require_binding(paths)?;
    let state = State::load(paths);
    let pending = outbox::pending(paths);

    let plan: Vec<Value> = pending
        .iter()
        .map(|intent| {
            json!({
                "intent": intent.id,
                "key": intent.key,
                "origin": intent.origin,
                "what": intent.kind.label(),
                "call": mcp_call(&binding, &state, intent),
                "ack": format!("sh harness_cli atlassian ack --intent {} --key <CLAVE>", intent.id),
            })
        })
        .collect();

    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "site": binding.site,
            "project": binding.jira.project_key,
            "pending": plan.len(),
            "plan": plan,
            "note": "ejecuta en orden; despues de cada llamada que cree un issue, registra su clave con `atlassian ack`",
        }))?
    );
    Ok(())
}

/// AC-10: el agente devuelve la clave creada y el intent pasa a aplicado.
pub fn ack(paths: &HarnessPaths, intent_id: &str, key: Option<&str>) -> anyhow::Result<()> {
    require_binding(paths)?;
    let pending = outbox::pending(paths);
    let Some(intent) = pending.iter().find(|i| i.id == intent_id) else {
        // Idempotente: un ack repetido no es un error (AC-10).
        println!("[Atlassian] el intent {intent_id} ya no esta pendiente (nada que hacer).");
        return Ok(());
    };
    let mut state = State::load(paths);
    record_result(&mut state, intent, key)?;
    state.mark_applied(&intent.key);
    state.save(paths)?;
    outbox::archive(paths, intent)?;
    println!(
        "[Atlassian] intent {intent_id} aplicado: {}{}",
        intent.kind.label(),
        key.map(|k| format!(" -> {k}")).unwrap_or_default()
    );
    Ok(())
}

/// Guarda en el state la clave remota que corresponde al tipo de intent.
fn record_result(state: &mut State, intent: &Intent, key: Option<&str>) -> anyhow::Result<()> {
    match (&intent.kind, key) {
        (IntentKind::PrdEpic { slug, .. }, Some(k)) => {
            state.prds.insert(slug.clone(), k.to_string());
        }
        (IntentKind::FeatureCreate { fid, .. }, Some(k)) => state.set_feature_issue(fid, k),
        (IntentKind::AcSubtask { fid, ac, .. }, Some(k)) => state.set_ac_issue(fid, ac, k),
        (IntentKind::PrdEpic { .. }, None)
        | (IntentKind::FeatureCreate { .. }, None)
        | (IntentKind::AcSubtask { .. }, None) => {
            anyhow::bail!(
                "este intent crea un issue: pasa la clave devuelta por Jira con --key <CLAVE>"
            );
        }
        // Transiciones, comentarios y flags no crean nada nuevo.
        _ => {}
    }
    Ok(())
}

// --------------------------------------------------------------------------
// apply (ejecutor REST)
// --------------------------------------------------------------------------

/// AC-15..AC-18: ejecuta los pendientes contra la API y solo marca aplicado lo
/// que Atlassian confirmo. Un fallo deja el intent pendiente y sale con 1.
pub fn apply(paths: &HarnessPaths) -> anyhow::Result<()> {
    let binding = require_binding(paths)?;
    let client = require_client(paths, &binding)?;
    let pending = outbox::pending(paths);
    if pending.is_empty() {
        println!("[Atlassian] no hay intents pendientes.");
        return Ok(());
    }

    let mut state = State::load(paths);
    let mut failures: Vec<String> = Vec::new();
    let mut done = 0usize;

    for intent in &pending {
        match execute(&client, &binding, &mut state, intent) {
            Ok(key) => {
                state.mark_applied(&intent.key);
                state.save(paths)?;
                outbox::archive(paths, intent)?;
                done += 1;
                println!(
                    "[Atlassian] {}{}",
                    intent.kind.label(),
                    key.map(|k| format!(" -> {k}")).unwrap_or_default()
                );
            }
            Err(err) => {
                // El intent QUEDA pendiente con su error legible (AC-17).
                failures.push(format!("  [{}] {}: {err:#}", intent.id, intent.kind.label()));
            }
        }
    }

    println!("[Atlassian] aplicados {done} de {} intents.", pending.len());
    if !failures.is_empty() {
        eprintln!("[Atlassian] quedaron {} sin aplicar:", failures.len());
        for f in &failures {
            eprintln!("{f}");
        }
        eprintln!("  Corregi la causa y volve a correr `atlassian apply` (lo ya aplicado no se repite).");
        return Err(Exit::code(1).into());
    }
    Ok(())
}

/// Ejecuta UN intent. Devuelve la clave creada, si creo algo.
fn execute(
    client: &Client,
    binding: &Binding,
    state: &mut State,
    intent: &Intent,
) -> anyhow::Result<Option<String>> {
    let types = &binding.jira.issue_types;
    let project = &binding.jira.project_key;
    match &intent.kind {
        IntentKind::PrdEpic { slug, title, body } => {
            let key = jira::create_issue(client, project, &types.epic, title, body, None)?;
            state.prds.insert(slug.clone(), key.clone());
            Ok(Some(key))
        }
        IntentKind::FeatureCreate {
            fid,
            name,
            acceptance,
            prd,
        } => {
            let parent = prd.as_ref().and_then(|s| state.prds.get(s)).cloned();
            let key = jira::create_issue(
                client,
                project,
                &types.feature,
                &feature_summary(fid, name),
                &feature_description(acceptance),
                parent.as_deref(),
            )?;
            state.set_feature_issue(fid, &key);
            // AC-20: si hay sprint vigente, la historia entra al sprint.
            if let Some(sprint) = &state.sprint {
                if sprint.state == "active" {
                    jira::move_issues_to_sprint(client, sprint.id, std::slice::from_ref(&key))?;
                }
            }
            Ok(Some(key))
        }
        IntentKind::AcSubtask { fid, ac, text } => {
            let parent = state
                .feature_issue(fid)
                .map(str::to_string)
                .ok_or_else(|| anyhow::anyhow!("la feature #{fid} todavia no tiene issue"))?;
            let key = jira::create_issue(
                client,
                project,
                &types.ac,
                &ac_summary(ac, text),
                text,
                Some(&parent),
            )?;
            state.set_ac_issue(fid, ac, &key);
            Ok(Some(key))
        }
        IntentKind::Transition { fid, to } => {
            let key = require_issue(state, fid)?;
            jira::transition(client, &key, to)?;
            Ok(None)
        }
        IntentKind::Comment { fid, body } => {
            let key = require_issue(state, fid)?;
            jira::add_comment(client, &key, body)?;
            Ok(None)
        }
        IntentKind::BlockedFlag { fid, on } => {
            let key = require_issue(state, fid)?;
            jira::set_impediment_flag(client, &key, *on, &binding.jira.statuses.blocked_flag)?;
            Ok(None)
        }
    }
}

fn require_issue(state: &State, fid: &str) -> anyhow::Result<String> {
    state
        .feature_issue(fid)
        .map(str::to_string)
        .ok_or_else(|| anyhow::anyhow!("la feature #{fid} todavia no tiene issue en Jira"))
}

// --------------------------------------------------------------------------
// sprint
// --------------------------------------------------------------------------

/// AC-19: abre un sprint en el board del proyecto y lo deja vigente.
pub fn sprint_start(
    paths: &HarnessPaths,
    name: &str,
    goal: Option<&str>,
    days: i64,
) -> anyhow::Result<()> {
    let mut binding = require_binding(paths)?;
    let client = require_client(paths, &binding)?;

    let board_id = match binding.jira.board_id {
        Some(id) => id,
        None => {
            let id = jira::board_for_project(&client, &binding.jira.project_key)?;
            binding.jira.board_id = Some(id);
            binding.save(paths)?;
            id
        }
    };

    let now = chrono::Utc::now();
    let end = now + chrono::TimeDelta::days(days.max(1));
    let fmt = |d: chrono::DateTime<chrono::Utc>| d.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();
    let sprint_id = jira::create_sprint(&client, board_id, name, goal, &fmt(now), &fmt(end))?;
    jira::set_sprint_state(&client, sprint_id, "active")?;

    let mut state = State::load(paths);
    state.sprint = Some(SprintRemote {
        id: sprint_id,
        name: name.to_string(),
        board_id,
        state: "active".to_string(),
    });
    state.save(paths)?;

    println!("[Atlassian] sprint #{sprint_id} '{name}' abierto en el board {board_id} ({days} dias).");
    println!("  Las features que arranques con `start` entran a este sprint.");
    Ok(())
}

/// AC-21: cierra el sprint vigente y reporta lo que quedo sin terminar.
pub fn sprint_close(paths: &HarnessPaths) -> anyhow::Result<()> {
    let binding = require_binding(paths)?;
    let client = require_client(paths, &binding)?;
    let mut state = State::load(paths);
    let Some(sprint) = state.sprint.clone() else {
        return Err(Exit::msg("[Atlassian] no hay ningun sprint vigente registrado.").into());
    };

    let issues = jira::sprint_issues(&client, sprint.id).unwrap_or_default();
    let done_name = binding.jira.statuses.done.to_lowercase();
    let pendientes: Vec<&(String, String)> = issues
        .iter()
        .filter(|(_, status)| status.to_lowercase() != done_name)
        .collect();

    jira::set_sprint_state(&client, sprint.id, "closed")?;
    state.sprint = None;
    state.save(paths)?;

    println!("[Atlassian] sprint #{} '{}' cerrado.", sprint.id, sprint.name);
    if pendientes.is_empty() {
        println!("  Todas las historias del sprint quedaron en {}.", binding.jira.statuses.done);
    } else {
        println!("  Quedaron sin terminar {}:", pendientes.len());
        for (key, status) in pendientes {
            println!("    {key} [{status}]");
        }
    }
    Ok(())
}

// --------------------------------------------------------------------------
// publish (Confluence)
// --------------------------------------------------------------------------

/// Titulo de la pagina: el primer `# ` del documento, o el nombre del archivo.
fn doc_title(file: &std::path::Path, text: &str) -> String {
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("# ") {
            let t = rest.trim();
            if !t.is_empty() {
                return t.to_string();
            }
        }
    }
    file.file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "Documento".to_string())
}

/// Ruta del documento relativa a la raiz del repo (la clave en `state.pages`).
fn doc_key(paths: &HarnessPaths, file: &std::path::Path) -> String {
    pathdiff::diff_paths(file, &paths.repo_root)
        .unwrap_or_else(|| file.to_path_buf())
        .to_string_lossy()
        .replace('\\', "/")
}

/// Todo lo que `publish_doc` necesita del entorno (el resto es del documento).
struct PublishCtx<'a> {
    client: &'a Client,
    paths: &'a HarnessPaths,
    binding: &'a Binding,
    space_id: &'a str,
}

/// Publica un documento y devuelve el id de su pagina.
fn publish_doc(
    ctx: &PublishCtx<'_>,
    state: &mut State,
    file: &std::path::Path,
    parent_id: Option<&str>,
    extra_header: Option<&str>,
) -> anyhow::Result<Option<String>> {
    let PublishCtx { client, paths, binding, space_id } = *ctx;
    let Ok(text) = std::fs::read_to_string(file) else {
        return Ok(None);
    };
    let key = doc_key(paths, file);
    let title = doc_title(file, &text);
    let mut body = String::new();
    if let Some(header) = extra_header {
        body.push_str(header);
        body.push('\n');
        body.push('\n');
    }
    body.push_str(&text);
    let storage = markdown::to_storage(
        &body,
        Some(&format!("Publicado por el arnes desde `{key}` (fuente de verdad en el repo).")),
    );
    let hash = confluence::content_hash(&storage);

    // AC-23: sin cambios, no se toca la pagina.
    if let Some(known) = state.pages.get(&key) {
        if known.hash == hash {
            println!("  = {key} (sin cambios)");
            return Ok(Some(known.id.clone()));
        }
    }

    let existing = match state.pages.get(&key) {
        Some(known) => Some(confluence::RemotePage {
            id: known.id.clone(),
            version: known.version,
            webui: None,
        }),
        None => confluence::find_page(client, space_id, &title)?,
    };

    let page = match existing {
        Some(previous) => {
            // La version que manda es la del servidor, no la recordada.
            let fresh = confluence::find_page(client, space_id, &title)?.unwrap_or(previous);
            let updated = confluence::update_page(client, &fresh, &title, &storage, parent_id)?;
            println!("  ~ {key} -> pagina {} (v{})", updated.id, updated.version);
            updated
        }
        None => {
            let created = confluence::create_page(client, space_id, &title, parent_id, &storage)?;
            println!("  + {key} -> pagina {} (nueva)", created.id);
            created
        }
    };

    state.pages.insert(
        key,
        crate::atlassian::state::PageRemote {
            id: page.id.clone(),
            version: page.version,
            hash,
            title: Some(title),
        },
    );
    let url = confluence::page_url(&binding.base_url(), &page);
    Ok(Some(format!("{}|{}", page.id, url)))
}

/// Separa el `id|url` que devuelve `publish_doc`.
fn split_id_url(value: &str) -> (String, String) {
    match value.split_once('|') {
        Some((id, url)) => (id.to_string(), url.to_string()),
        None => (value.to_string(), String::new()),
    }
}

/// AC-22..AC-24: PRD maestro, PRDs anidados, SDD y specs como arbol de paginas.
pub fn publish(paths: &HarnessPaths) -> anyhow::Result<()> {
    let binding = require_binding(paths)?;
    if binding.confluence.space_key.trim().is_empty() {
        return Err(Exit {
            code: 2,
            message: Some(
                "[Atlassian] este repo no tiene space de Confluence.\n  Preguntale al USUARIO a que space pertenece y registralo:\n    sh harness_cli atlassian bind --confluence-space <KEY>"
                    .to_string(),
            ),
        }
        .into());
    }
    let client = require_client(paths, &binding)?;
    let space_id = confluence::space_id(&client, &binding.confluence.space_key)?;
    let ctx = PublishCtx {
        client: &client,
        paths,
        binding: &binding,
        space_id: &space_id,
    };
    let mut state = State::load(paths);

    println!(
        "[Atlassian] publicando en el space {} ({})",
        binding.confluence.space_key, binding.site
    );

    // 1. PRDs: el maestro primero; `scan` ya devuelve los padres antes que los
    //    hijos (orden alfabetico de la cadena de slugs).
    let mut prd_pages: std::collections::BTreeMap<String, String> = std::collections::BTreeMap::new();
    for prd in crate::prd::scan(paths) {
        let slug = emit::prd_key_slug(&prd.slug);
        let parent = prd
            .parent_slug()
            .map(|p| emit::prd_key_slug(&p))
            .and_then(|p| prd_pages.get(&p).cloned());
        let header = state
            .prds
            .get(&slug)
            .map(|issue| format!("> Epic en Jira: {}", binding.browse_url(issue)));
        if let Some(value) = publish_doc(
            &ctx,
            &mut state,
            &prd.file,
            parent.as_deref(),
            header.as_deref(),
        )? {
            let (id, _) = split_id_url(&value);
            prd_pages.insert(slug, id);
        }
        state.save(paths)?;
    }

    // 2. SDD maestro: hermano del PRD maestro.
    let sdd = crate::prd::prd_dir(paths).join("SDD-master.md");
    if sdd.is_file() {
        publish_doc(&ctx, &mut state, &sdd, None, None)?;
        state.save(paths)?;
    }

    // 3. Specs: hijos del PRD que los origina, con enlace cruzado al issue.
    let data = load_features(paths)?;
    for feature in features_slice(&data) {
        let Some(map) = feature.as_object() else {
            continue;
        };
        let file = crate::spec::spec_path(paths, map);
        if !file.is_file() {
            continue;
        }
        let fid = py_str(map.get("id"));
        let slug = emit::prd_key_slug(&crate::prd::feature_prd_slug(feature));
        let parent = prd_pages.get(&slug).cloned();
        let issue = state.feature_issue(&fid).map(str::to_string);
        let header = issue
            .as_ref()
            .map(|k| format!("> Historia en Jira: {}", binding.browse_url(k)));
        let published = publish_doc(
            &ctx,
            &mut state,
            &file,
            parent.as_deref(),
            header.as_deref(),
        )?;
        state.save(paths)?;

        // AC-24: el issue tambien queda con el enlace a su pagina.
        if let (Some(value), Some(key)) = (published, issue) {
            let (_, url) = split_id_url(&value);
            let link_key = format!("page-link:{fid}");
            if !url.is_empty() && !state.is_applied(&link_key) {
                if let Err(err) = jira::link_page(&client, &key, &doc_key(paths, &file), &url) {
                    eprintln!("  [aviso] no pude enlazar la pagina en {key}: {err:#}");
                } else {
                    state.mark_applied(&link_key);
                    state.save(paths)?;
                }
            }
        }
    }

    println!("[Atlassian] publicacion terminada.");
    Ok(())
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;
    use crate::atlassian::binding::{Binding, ConfluenceBinding, JiraBinding};

    fn binding() -> Binding {
        Binding {
            site: "calpil.atlassian.net".to_string(),
            cloud_id: None,
            enabled: true,
            jira: JiraBinding {
                project_key: "ADR".to_string(),
                ..Default::default()
            },
            confluence: ConfluenceBinding {
                space_key: "SD".to_string(),
                space_id: None,
            },
        }
    }

    #[test]
    fn mcp_call_should_flag_missing_parent() {
        // AC-9: si el epic todavia no tiene clave, el plan lo dice en vez de
        // inventar un parent.
        let state = State::default();
        let intent = Intent::new(
            "0002".to_string(),
            "feature:15:create".to_string(),
            "add",
            IntentKind::FeatureCreate {
                fid: "15".to_string(),
                name: "demo".to_string(),
                acceptance: vec!["algo".to_string()],
                prd: Some("master".to_string()),
            },
        );
        let call = mcp_call(&binding(), &state, &intent);
        assert_eq!(call["tool"], "createJiraIssue");
        assert!(call["args"]["parent"].is_null());
        assert!(call["needs"].as_str().unwrap().contains("master"));
    }

    #[test]
    fn mcp_call_should_use_parent_once_acked() {
        let mut state = State::default();
        state.prds.insert("master".to_string(), "ADR-1".to_string());
        let intent = Intent::new(
            "0002".to_string(),
            "feature:15:create".to_string(),
            "add",
            IntentKind::FeatureCreate {
                fid: "15".to_string(),
                name: "demo".to_string(),
                acceptance: vec![],
                prd: Some("master".to_string()),
            },
        );
        let call = mcp_call(&binding(), &state, &intent);
        assert_eq!(call["args"]["parent"], "ADR-1");
        assert!(call["needs"].is_null());
        assert_eq!(call["args"]["summary"], "#15 demo");
        assert_eq!(call["args"]["issueTypeName"], "Story");
    }

    #[test]
    fn ac_subtask_summary_should_follow_the_scrum_convention() {
        assert_eq!(ac_summary("AC-1", "Given algo"), "AC-1 · Given algo");
    }

    #[test]
    fn record_result_should_require_a_key_for_creating_intents() {
        let mut state = State::default();
        let intent = Intent::new(
            "0001".to_string(),
            "prd:master:epic".to_string(),
            "add",
            IntentKind::PrdEpic {
                slug: "master".to_string(),
                title: "PRD maestro".to_string(),
                body: String::new(),
            },
        );
        assert!(record_result(&mut state, &intent, None).is_err());
        record_result(&mut state, &intent, Some("ADR-1")).unwrap();
        assert_eq!(state.prds.get("master"), Some(&"ADR-1".to_string()));
    }

    #[test]
    fn record_result_should_ignore_key_for_comments() {
        let mut state = State::default();
        let intent = Intent::new(
            "0009".to_string(),
            "feature:15:comment:abc".to_string(),
            "advance",
            IntentKind::Comment {
                fid: "15".to_string(),
                body: "nota".to_string(),
            },
        );
        record_result(&mut state, &intent, None).unwrap();
        assert!(state.features.is_empty());
    }

    #[test]
    fn doc_title_should_prefer_the_first_heading() {
        let file = std::path::Path::new("/tmp/PRD-master.md");
        assert_eq!(doc_title(file, "# PRD maestro\n\ntexto\n"), "PRD maestro");
        assert_eq!(doc_title(file, "sin titulo\n"), "PRD-master");
    }
}
