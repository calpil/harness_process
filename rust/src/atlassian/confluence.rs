//! Confluence v2: PRD, SDD y specs publicados como arbol de paginas
//! (spec #15, AC-22..AC-24).
//!
//! Idempotencia en dos capas: por titulo dentro del space (no duplicar) y por
//! hash del contenido en `state.json` (no crear versiones nuevas cuando el
//! documento no cambio).

use anyhow::Context;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::atlassian::http::Client;

/// Pagina existente en el space.
#[derive(Debug, Clone)]
pub struct RemotePage {
    pub id: String,
    pub version: i64,
    pub webui: Option<String>,
}

/// Id numerico del space a partir de su clave (`SD`).
pub fn space_id(client: &Client, space_key: &str) -> anyhow::Result<String> {
    let res = client.get(&format!("/wiki/api/v2/spaces?keys={space_key}&limit=1"))?;
    res.get("results")
        .and_then(Value::as_array)
        .and_then(|v| v.first())
        .and_then(|s| s.get("id"))
        .and_then(value_as_id)
        .with_context(|| format!("no encontre el space '{space_key}' en este sitio"))
}

/// La API devuelve ids como numero o como texto segun el recurso.
fn value_as_id(value: &Value) -> Option<String> {
    match value {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        _ => None,
    }
}

/// Busca una pagina por titulo exacto dentro del space.
pub fn find_page(
    client: &Client,
    space_id: &str,
    title: &str,
) -> anyhow::Result<Option<RemotePage>> {
    let encoded = url_encode(title);
    let res = client.get(&format!(
        "/wiki/api/v2/spaces/{space_id}/pages?title={encoded}&limit=25"
    ))?;
    let Some(list) = res.get("results").and_then(Value::as_array) else {
        return Ok(None);
    };
    for page in list {
        if page.get("title").and_then(Value::as_str) != Some(title) {
            continue;
        }
        let Some(id) = page.get("id").and_then(value_as_id) else {
            continue;
        };
        return Ok(Some(RemotePage {
            id,
            version: page
                .get("version")
                .and_then(|v| v.get("number"))
                .and_then(Value::as_i64)
                .unwrap_or(1),
            webui: page
                .get("_links")
                .and_then(|l| l.get("webui"))
                .and_then(Value::as_str)
                .map(str::to_string),
        }));
    }
    Ok(None)
}

/// True si el space existe y es visible para el token (AC-18).
pub fn space_exists(client: &Client, space_key: &str) -> anyhow::Result<bool> {
    match space_id(client, space_key) {
        Ok(_) => Ok(true),
        Err(err) => match err.downcast_ref::<crate::atlassian::http::ApiError>() {
            Some(api) if api.status == 404 || api.status == 403 => Ok(false),
            // `space_id` tambien falla con "no encontre el space" (lista vacia).
            _ if err.to_string().contains("no encontre el space") => Ok(false),
            _ => Err(err),
        },
    }
}

/// Crea el space (AC-22). Requiere permiso para crear spaces; se usa SOLO con
/// `--create-space`.
pub fn create_space(client: &Client, key: &str, name: &str) -> anyhow::Result<String> {
    let res = client.post(
        "/wiki/api/v2/spaces",
        &json!({
            "key": key,
            "name": name,
            "description": {
                "value": "Creado por el arnes (harness_process) al configurar el binding.",
                "representation": "plain"
            },
        }),
    )?;
    res.get("id")
        .and_then(value_as_id)
        .context("Confluence creo el space pero no devolvio su id")
}

/// Crea la pagina (publicada) y devuelve su id y version.
pub fn create_page(
    client: &Client,
    space_id: &str,
    title: &str,
    parent_id: Option<&str>,
    storage: &str,
) -> anyhow::Result<RemotePage> {
    let mut body = json!({
        "spaceId": space_id,
        "status": "current",
        "title": title,
        "body": {"representation": "storage", "value": storage},
    });
    if let (Some(parent), Some(obj)) = (parent_id, body.as_object_mut()) {
        obj.insert("parentId".to_string(), json!(parent));
    }
    let res = client.post("/wiki/api/v2/pages", &body)?;
    let id = res
        .get("id")
        .and_then(value_as_id)
        .context("Confluence creo la pagina pero no devolvio su id")?;
    Ok(RemotePage {
        id,
        version: res
            .get("version")
            .and_then(|v| v.get("number"))
            .and_then(Value::as_i64)
            .unwrap_or(1),
        webui: res
            .get("_links")
            .and_then(|l| l.get("webui"))
            .and_then(Value::as_str)
            .map(str::to_string),
    })
}

/// Actualiza la pagina subiendo la version en uno (lo exige la API v2).
pub fn update_page(
    client: &Client,
    page: &RemotePage,
    title: &str,
    storage: &str,
    parent_id: Option<&str>,
) -> anyhow::Result<RemotePage> {
    let next = page.version + 1;
    let mut body = json!({
        "id": page.id,
        "status": "current",
        "title": title,
        "body": {"representation": "storage", "value": storage},
        "version": {"number": next, "message": "Actualizado por el arnes"},
    });
    if let (Some(parent), Some(obj)) = (parent_id, body.as_object_mut()) {
        obj.insert("parentId".to_string(), json!(parent));
    }
    let res = client.put(&format!("/wiki/api/v2/pages/{}", page.id), &body)?;
    Ok(RemotePage {
        id: page.id.clone(),
        version: res
            .get("version")
            .and_then(|v| v.get("number"))
            .and_then(Value::as_i64)
            .unwrap_or(next),
        webui: res
            .get("_links")
            .and_then(|l| l.get("webui"))
            .and_then(Value::as_str)
            .map(str::to_string)
            .or_else(|| page.webui.clone()),
    })
}

/// URL navegable de una pagina.
pub fn page_url(base_url: &str, page: &RemotePage) -> String {
    match &page.webui {
        Some(webui) => format!("{base_url}/wiki{webui}"),
        None => format!("{base_url}/wiki/pages/viewpage.action?pageId={}", page.id),
    }
}

/// Hash del contenido publicado: el candado que evita versiones inutiles.
pub fn content_hash(storage: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(storage.as_bytes());
    hex::encode(hasher.finalize())[..16].to_string()
}

/// Percent-encoding minimo para el titulo en el query string.
pub fn url_encode(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for byte in text.as_bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(*byte as char)
            }
            b' ' => out.push_str("%20"),
            other => out.push_str(&format!("%{other:02X}")),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;

    #[test]
    fn url_encode_should_escape_spaces_and_accents() {
        assert_eq!(url_encode("PRD maestro"), "PRD%20maestro");
        assert_eq!(url_encode("a/b?c"), "a%2Fb%3Fc");
        // Multibyte: cada byte se escapa por separado.
        assert_eq!(url_encode("ñ"), "%C3%B1");
    }

    #[test]
    fn content_hash_should_change_with_content() {
        let a = content_hash("<p>uno</p>");
        assert_eq!(a, content_hash("<p>uno</p>"), "mismo contenido, mismo hash");
        assert_ne!(a, content_hash("<p>dos</p>"));
        assert_eq!(a.len(), 16);
    }

    #[test]
    fn page_url_should_prefer_webui_link() {
        let page = RemotePage {
            id: "262237".to_string(),
            version: 2,
            webui: Some("/spaces/SD/pages/262237/PRD".to_string()),
        };
        assert_eq!(
            page_url("https://calpil.atlassian.net", &page),
            "https://calpil.atlassian.net/wiki/spaces/SD/pages/262237/PRD"
        );
        let sin_link = RemotePage {
            id: "9".to_string(),
            version: 1,
            webui: None,
        };
        assert!(page_url("https://x.cl", &sin_link).contains("pageId=9"));
    }
}
