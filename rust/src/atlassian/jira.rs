//! Operaciones sobre Jira: plataforma v3 para issues, transiciones y
//! comentarios; Agile 1.0 para boards y sprints (spec #15, AC-15..AC-21).
//!
//! Los sprints son el punto ciego del MCP oficial (no expone boards ni
//! sprints), y por eso viven aca: es la parte que solo el ejecutor REST puede
//! hacer (decision OBS-2).

use anyhow::Context;
use serde_json::{Value, json};

use crate::atlassian::http::Client;

/// Texto plano -> ADF (Atlassian Document Format), que es lo que la API v3
/// espera en `description` y en los comentarios. Cada linea no vacia es un
/// parrafo; asi el texto del arnes se lee igual del otro lado.
pub fn adf(text: &str) -> Value {
    let mut content: Vec<Value> = text
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(|line| {
            json!({
                "type": "paragraph",
                "content": [{"type": "text", "text": line}]
            })
        })
        .collect();
    if content.is_empty() {
        content.push(json!({"type": "paragraph", "content": []}));
    }
    json!({"type": "doc", "version": 1, "content": content})
}

/// Crea un issue y devuelve su clave (`ADR-42`).
pub fn create_issue(
    client: &Client,
    project_key: &str,
    issue_type: &str,
    summary: &str,
    description: &str,
    parent: Option<&str>,
) -> anyhow::Result<String> {
    let mut fields = json!({
        "project": {"key": project_key},
        "issuetype": {"name": issue_type},
        "summary": truncate_summary(summary),
        "description": adf(description),
    });
    if let (Some(key), Some(obj)) = (parent, fields.as_object_mut()) {
        obj.insert("parent".to_string(), json!({"key": key}));
    }
    let res = client.post("/rest/api/3/issue", &json!({"fields": fields}))?;
    res.get("key")
        .and_then(Value::as_str)
        .map(str::to_string)
        .context("Jira creo el issue pero no devolvio su clave")
}

/// Jira corta los resumenes en 255 caracteres: recortamos antes para que el
/// error no venga del otro lado.
pub fn truncate_summary(summary: &str) -> String {
    let clean = summary.replace(['\n', '\r'], " ");
    let clean = clean.trim();
    if clean.chars().count() <= 255 {
        return clean.to_string();
    }
    let short: String = clean.chars().take(252).collect();
    format!("{short}...")
}

/// Busca la transicion que lleva al estado pedido (por nombre de destino o de
/// la propia transicion, sin distinguir mayusculas).
pub fn find_transition(client: &Client, key: &str, target: &str) -> anyhow::Result<Option<String>> {
    let res = client.get(&format!("/rest/api/3/issue/{key}/transitions"))?;
    let Some(list) = res.get("transitions").and_then(Value::as_array) else {
        return Ok(None);
    };
    let target_lower = target.to_lowercase();
    for t in list {
        let to_name = t
            .get("to")
            .and_then(|to| to.get("name"))
            .and_then(Value::as_str)
            .unwrap_or_default();
        let name = t.get("name").and_then(Value::as_str).unwrap_or_default();
        if to_name.to_lowercase() == target_lower || name.to_lowercase() == target_lower {
            if let Some(id) = t.get("id").and_then(Value::as_str) {
                return Ok(Some(id.to_string()));
            }
        }
    }
    Ok(None)
}

/// Mueve el issue al estado pedido. Si el board no tiene esa transicion
/// disponible, se informa sin romper (el intent queda pendiente con su error).
pub fn transition(client: &Client, key: &str, target: &str) -> anyhow::Result<()> {
    let Some(id) = find_transition(client, key, target)? else {
        anyhow::bail!(
            "el issue {key} no ofrece una transicion a '{target}'; revisa el flujo del board o ajusta el mapeo de estados en atlassian.json"
        );
    };
    client.post(
        &format!("/rest/api/3/issue/{key}/transitions"),
        &json!({"transition": {"id": id}}),
    )?;
    Ok(())
}

/// Comentario en el issue (la bitacora del arnes del otro lado).
pub fn add_comment(client: &Client, key: &str, body: &str) -> anyhow::Result<()> {
    client.post(
        &format!("/rest/api/3/issue/{key}/comment"),
        &json!({"body": adf(body)}),
    )?;
    Ok(())
}

/// Flag `Impediment` (customfield_10021): la representacion de `blocked`
/// elegida por el usuario (OBS-7). `on=false` lo quita.
pub fn set_impediment_flag(
    client: &Client,
    key: &str,
    on: bool,
    flag_value: &str,
) -> anyhow::Result<()> {
    let value = if on {
        json!([{"value": flag_value}])
    } else {
        json!([])
    };
    client.put(
        &format!("/rest/api/3/issue/{key}"),
        &json!({"fields": {"customfield_10021": value}}),
    )?;
    Ok(())
}

/// Enlaza el issue a su pagina de Confluence dejando el enlace como comentario
/// (AC-24). No usa remote links porque requieren scopes extra.
pub fn link_page(client: &Client, key: &str, title: &str, url: &str) -> anyhow::Result<()> {
    add_comment(client, key, &format!("Documento en Confluence: {title} - {url}"))
}

/// True si el proyecto existe y el usuario puede verlo (AC-18).
pub fn project_exists(client: &Client, project_key: &str) -> anyhow::Result<bool> {
    match client.get(&format!("/rest/api/3/project/{project_key}")) {
        Ok(_) => Ok(true),
        Err(err) => match err.downcast_ref::<crate::atlassian::http::ApiError>() {
            // 404 (no existe) y 403 (sin permiso) son respuestas, no fallas.
            Some(api) if api.status == 404 || api.status == 403 => Ok(false),
            _ => Err(err),
        },
    }
}

/// Busca un epic por titulo EXACTO dentro del proyecto (AC-29): permite adoptar
/// los epics que el equipo ya escribio a mano en vez de duplicarlos.
pub fn find_epic_by_title(
    client: &Client,
    project_key: &str,
    epic_type: &str,
    title: &str,
) -> anyhow::Result<Option<String>> {
    // El texto va como parametro de JQL entre comillas: se escapan las comillas
    // y las barras para no romper la consulta.
    let safe = title.replace('\\', "\\\\").replace('"', "\\\"");
    let jql = format!(
        "project = \"{project_key}\" AND issuetype = \"{epic_type}\" AND summary ~ \"{safe}\""
    );
    let res = client.post(
        "/rest/api/3/search/jql",
        &json!({"jql": jql, "fields": ["summary"], "maxResults": 50}),
    )?;
    let Some(list) = res.get("issues").and_then(Value::as_array) else {
        return Ok(None);
    };
    // `~` es busqueda por texto: confirmamos igualdad exacta del titulo.
    for issue in list {
        let summary = issue
            .get("fields")
            .and_then(|f| f.get("summary"))
            .and_then(Value::as_str)
            .unwrap_or_default();
        if summary.trim() == title.trim()
            && let Some(key) = issue.get("key").and_then(Value::as_str)
        {
            return Ok(Some(key.to_string()));
        }
    }
    Ok(None)
}

/// Crea el proyecto (AC-21). Requiere permiso de administracion; se usa SOLO
/// con `--create-project`.
pub fn create_project(
    client: &Client,
    key: &str,
    name: &str,
    lead_account_id: &str,
) -> anyhow::Result<String> {
    let res = client.post(
        "/rest/api/3/project",
        &json!({
            "key": key,
            "name": name,
            "projectTypeKey": "software",
            // Team-managed scrum: el mismo tipo de proyecto que ya usa el sitio.
            "projectTemplateKey": "com.pyxis.greenhopper.jira:gh-simplified-agility-scrum",
            "leadAccountId": lead_account_id,
            "description": "Creado por el arnes (harness_process) al configurar el binding.",
        }),
    )?;
    res.get("key")
        .and_then(Value::as_str)
        .map(str::to_string)
        .context("Jira creo el proyecto pero no devolvio su clave")
}

/// Cuenta del token (para poner al usuario como lead del proyecto nuevo).
pub fn my_account_id(client: &Client) -> anyhow::Result<String> {
    let res = client.get("/rest/api/3/myself")?;
    res.get("accountId")
        .and_then(Value::as_str)
        .map(str::to_string)
        .context("no pude resolver la cuenta del token")
}

// --------------------------------------------------------------------------
// Agile 1.0: boards y sprints
// --------------------------------------------------------------------------

/// Primer board del proyecto (los proyectos team-managed traen uno solo).
pub fn board_for_project(client: &Client, project_key: &str) -> anyhow::Result<i64> {
    let res = client.get(&format!(
        "/rest/agile/1.0/board?projectKeyOrId={project_key}"
    ))?;
    res.get("values")
        .and_then(Value::as_array)
        .and_then(|v| v.first())
        .and_then(|b| b.get("id"))
        .and_then(Value::as_i64)
        .with_context(|| format!("el proyecto {project_key} no tiene board visible"))
}

/// Crea un sprint futuro en el board y devuelve (id, nombre).
pub fn create_sprint(
    client: &Client,
    board_id: i64,
    name: &str,
    goal: Option<&str>,
    start: &str,
    end: &str,
) -> anyhow::Result<i64> {
    let mut body = json!({
        "name": name,
        "originBoardId": board_id,
        "startDate": start,
        "endDate": end,
    });
    if let (Some(goal), Some(obj)) = (goal, body.as_object_mut()) {
        obj.insert("goal".to_string(), json!(goal));
    }
    let res = client.post("/rest/agile/1.0/sprint", &body)?;
    res.get("id")
        .and_then(Value::as_i64)
        .context("Jira creo el sprint pero no devolvio su id")
}

/// Cambia el estado del sprint (`active` para arrancarlo, `closed` para
/// cerrarlo). Se usa el update PARCIAL para no pisar el resto de los campos.
pub fn set_sprint_state(client: &Client, sprint_id: i64, state: &str) -> anyhow::Result<()> {
    client.post(
        &format!("/rest/agile/1.0/sprint/{sprint_id}"),
        &json!({"state": state}),
    )?;
    Ok(())
}

/// Mueve issues al sprint. La API acepta 50 por llamada: se envia en lotes.
pub fn move_issues_to_sprint(client: &Client, sprint_id: i64, keys: &[String]) -> anyhow::Result<()> {
    for chunk in keys.chunks(50) {
        client.post(
            &format!("/rest/agile/1.0/sprint/{sprint_id}/issue"),
            &json!({"issues": chunk}),
        )?;
    }
    Ok(())
}

/// Issues del sprint con su estado, para reportar que queda sin terminar al
/// cerrarlo (AC-21).
pub fn sprint_issues(client: &Client, sprint_id: i64) -> anyhow::Result<Vec<(String, String)>> {
    let res = client.get(&format!(
        "/rest/agile/1.0/sprint/{sprint_id}/issue?fields=status&maxResults=100"
    ))?;
    let Some(list) = res.get("issues").and_then(Value::as_array) else {
        return Ok(Vec::new());
    };
    Ok(list
        .iter()
        .filter_map(|i| {
            let key = i.get("key").and_then(Value::as_str)?;
            let status = i
                .get("fields")
                .and_then(|f| f.get("status"))
                .and_then(|s| s.get("name"))
                .and_then(Value::as_str)
                .unwrap_or("?");
            Some((key.to_string(), status.to_string()))
        })
        .collect())
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;

    #[test]
    fn adf_should_wrap_each_line_in_a_paragraph() {
        let doc = adf("primera\n\nsegunda");
        assert_eq!(doc.get("type").and_then(Value::as_str), Some("doc"));
        let content = doc.get("content").and_then(Value::as_array).unwrap();
        assert_eq!(content.len(), 2);
        let text = content[1]["content"][0]["text"].as_str().unwrap();
        assert_eq!(text, "segunda");
    }

    #[test]
    fn adf_should_never_be_empty() {
        // Un comentario vacio igual tiene que ser un documento valido.
        let doc = adf("   \n  ");
        assert_eq!(doc["content"].as_array().map(Vec::len), Some(1));
    }

    #[test]
    fn truncate_summary_should_respect_jira_limit() {
        let long = "x".repeat(400);
        let out = truncate_summary(&long);
        assert_eq!(out.chars().count(), 255);
        assert!(out.ends_with("..."));
        assert_eq!(truncate_summary(" hola\nmundo "), "hola mundo");
    }
}
