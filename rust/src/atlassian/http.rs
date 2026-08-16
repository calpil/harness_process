//! Cliente HTTP del ejecutor REST (spec #15, AC-15..AC-18).
//!
//! Articulo 4: el token llega SOLO por entorno o por `.harness.env` (ignorado
//! por git) y no se imprime, ni se guarda, ni viaja a la outbox ni al state.
//! Siempre HTTPS: `https_only(true)`, sin fallback a HTTP.

use std::time::Duration;

use serde_json::Value;

use crate::atlassian::binding::Binding;
use crate::paths::HarnessPaths;

/// Variables reconocidas para las credenciales.
pub const ENV_EMAIL: &str = "HARNESS_ATLASSIAN_EMAIL";
pub const ENV_TOKEN: &str = "HARNESS_ATLASSIAN_TOKEN";

/// Error de la API con el detalle que Atlassian devolvio: el spec pide errores
/// accionables (que fallo y que hacer), no un "request failed" (AC-17).
#[derive(Debug)]
pub struct ApiError {
    pub status: u16,
    pub detail: String,
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "HTTP {}: {}", self.status, self.detail)
    }
}

impl std::error::Error for ApiError {}

/// Credenciales del ejecutor REST.
#[derive(Clone)]
pub struct Credentials {
    pub email: String,
    token: String,
}

impl Credentials {
    /// Busca las credenciales primero en el entorno y despues en los mismos
    /// archivos de configuracion que ya lee el instalador, en el mismo orden:
    /// el `.harness.env` del proyecto y del arnes (por repo) y despues los
    /// globales `~/.config/harness/config` y `~/.harnessrc`, que permiten
    /// definir el token UNA vez para todos los proyectos de la maquina.
    /// Devuelve `None` si falta cualquiera de las dos.
    pub fn discover(paths: &HarnessPaths) -> Option<Credentials> {
        let from_env = |k: &str| std::env::var(k).ok().filter(|v| !v.trim().is_empty());
        let mut email = from_env(ENV_EMAIL);
        let mut token = from_env(ENV_TOKEN);
        if email.is_none() || token.is_none() {
            for file in config_files(paths) {
                let Ok(text) = std::fs::read_to_string(&file) else {
                    continue;
                };
                if email.is_none() {
                    email = read_env_key(&text, ENV_EMAIL);
                }
                if token.is_none() {
                    token = read_env_key(&text, ENV_TOKEN);
                }
            }
        }
        match (email, token) {
            (Some(email), Some(token)) => Some(Credentials { email, token }),
            _ => None,
        }
    }

    /// Header `Authorization` para Basic auth (email + API token).
    fn header(&self) -> String {
        format!(
            "Basic {}",
            base64_encode(format!("{}:{}", self.email, self.token).as_bytes())
        )
    }
}

/// Archivos de configuracion donde pueden vivir las credenciales, en orden de
/// precedencia. Es la MISMA lista (y el mismo orden) que `load_config_file` de
/// `setup_harness.sh` e `Import-HarnessConfiguration` de `setup_harness.ps1`:
/// lo local manda sobre lo global, y lo global evita repetir el token en cada
/// repo de la maquina.
fn config_files(paths: &HarnessPaths) -> Vec<std::path::PathBuf> {
    let mut files = vec![
        paths.repo_root.join(".harness.env"),
        paths.root.join(".harness.env"),
    ];
    if let Some(home) = crate::pycompat::home_dir() {
        files.push(home.join(".config/harness/config"));
        files.push(home.join(".harnessrc"));
    }
    files
}

/// Lee `CLAVE=valor` de un archivo tipo env, tolerando `export` y comillas.
fn read_env_key(text: &str, key: &str) -> Option<String> {
    for line in text.lines() {
        let line = line.trim();
        let line = line.strip_prefix("export ").unwrap_or(line);
        let Some((k, v)) = line.split_once('=') else {
            continue;
        };
        if k.trim() != key {
            continue;
        }
        let v = v.trim().trim_matches('"').trim_matches('\'').to_string();
        if !v.is_empty() {
            return Some(v);
        }
    }
    None
}

/// Cliente contra un sitio de Atlassian.
pub struct Client {
    agent: ureq::Agent,
    base: String,
    auth: String,
}

impl Client {
    pub fn new(binding: &Binding, creds: &Credentials) -> Client {
        let config = ureq::Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(30)))
            .https_only(true)
            // Los 4xx/5xx no son excepciones: queremos el cuerpo del error
            // para poder contarle al usuario que dijo Atlassian (AC-17).
            .http_status_as_error(false)
            .build();
        Client {
            agent: ureq::Agent::new_with_config(config),
            base: binding.base_url(),
            auth: creds.header(),
        }
    }

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base, path)
    }

    pub fn get(&self, path: &str) -> anyhow::Result<Value> {
        let mut res = self
            .agent
            .get(self.url(path))
            .header("Authorization", &self.auth)
            .header("Accept", "application/json")
            .call()?;
        Self::into_json(res.status().as_u16(), res.body_mut().read_to_string()?)
    }

    pub fn post(&self, path: &str, body: &Value) -> anyhow::Result<Value> {
        let mut res = self
            .agent
            .post(self.url(path))
            .header("Authorization", &self.auth)
            .header("Accept", "application/json")
            .send_json(body)?;
        Self::into_json(res.status().as_u16(), res.body_mut().read_to_string()?)
    }

    pub fn put(&self, path: &str, body: &Value) -> anyhow::Result<Value> {
        let mut res = self
            .agent
            .put(self.url(path))
            .header("Authorization", &self.auth)
            .header("Accept", "application/json")
            .send_json(body)?;
        Self::into_json(res.status().as_u16(), res.body_mut().read_to_string()?)
    }

    /// Traduce (status, cuerpo) a JSON o a un `ApiError` legible.
    fn into_json(status: u16, text: String) -> anyhow::Result<Value> {
        if (200..300).contains(&status) {
            if text.trim().is_empty() {
                return Ok(Value::Null);
            }
            return Ok(serde_json::from_str(&text).unwrap_or(Value::String(text)));
        }
        Err(ApiError {
            status,
            detail: summarize_error(&text),
        }
        .into())
    }
}

/// Resume el cuerpo de error de Atlassian (`errorMessages`, `errors`, o el
/// texto crudo recortado) para que el mensaje quepa en una linea util.
pub fn summarize_error(text: &str) -> String {
    if let Ok(value) = serde_json::from_str::<Value>(text) {
        let mut parts: Vec<String> = Vec::new();
        if let Some(msgs) = value.get("errorMessages").and_then(Value::as_array) {
            parts.extend(msgs.iter().filter_map(Value::as_str).map(str::to_string));
        }
        if let Some(errors) = value.get("errors").and_then(Value::as_object) {
            for (k, v) in errors {
                if let Some(s) = v.as_str() {
                    parts.push(format!("{k}: {s}"));
                }
            }
        }
        for key in ["message", "title", "detail"] {
            if let Some(s) = value.get(key).and_then(Value::as_str) {
                parts.push(s.to_string());
            }
        }
        if let Some(errors) = value.get("errors").and_then(Value::as_array) {
            for e in errors {
                if let Some(s) = e.get("title").and_then(Value::as_str) {
                    parts.push(s.to_string());
                }
            }
        }
        if !parts.is_empty() {
            return parts.join(" | ");
        }
    }
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return "sin cuerpo en la respuesta".to_string();
    }
    trimmed.chars().take(300).collect()
}

/// Base64 estandar. Se implementa aca a proposito: el Articulo 6 pide un ADR
/// por dependencia nueva y no vale la pena sumar un crate por 20 lineas.
pub fn base64_encode(input: &[u8]) -> String {
    const ALPHABET: &[u8; 64] =
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(input.len().div_ceil(3) * 4);
    for chunk in input.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = *chunk.get(1).unwrap_or(&0) as u32;
        let b2 = *chunk.get(2).unwrap_or(&0) as u32;
        let triple = (b0 << 16) | (b1 << 8) | b2;
        let idx = [
            (triple >> 18) & 0x3F,
            (triple >> 12) & 0x3F,
            (triple >> 6) & 0x3F,
            triple & 0x3F,
        ];
        out.push(ALPHABET[idx[0] as usize] as char);
        out.push(ALPHABET[idx[1] as usize] as char);
        out.push(if chunk.len() > 1 {
            ALPHABET[idx[2] as usize] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            ALPHABET[idx[3] as usize] as char
        } else {
            '='
        });
    }
    out
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;

    #[test]
    fn base64_should_match_known_vectors() {
        assert_eq!(base64_encode(b""), "");
        assert_eq!(base64_encode(b"f"), "Zg==");
        assert_eq!(base64_encode(b"fo"), "Zm8=");
        assert_eq!(base64_encode(b"foo"), "Zm9v");
        assert_eq!(base64_encode(b"foob"), "Zm9vYg==");
        assert_eq!(base64_encode(b"fooba"), "Zm9vYmE=");
        assert_eq!(base64_encode(b"foobar"), "Zm9vYmFy");
        assert_eq!(
            base64_encode(b"alan@calpil.cl:token123"),
            "YWxhbkBjYWxwaWwuY2w6dG9rZW4xMjM="
        );
    }

    #[test]
    fn read_env_key_should_tolerate_export_and_quotes() {
        let text = "# comentario\nexport HARNESS_ATLASSIAN_EMAIL=\"a@b.cl\"\nHARNESS_ATLASSIAN_TOKEN='xyz'\n";
        assert_eq!(read_env_key(text, ENV_EMAIL), Some("a@b.cl".to_string()));
        assert_eq!(read_env_key(text, ENV_TOKEN), Some("xyz".to_string()));
        assert_eq!(read_env_key(text, "OTRA"), None);
    }

    #[test]
    fn summarize_error_should_prefer_atlassian_messages() {
        // AC-17: el mensaje tiene que decir que paso, no "request failed".
        let body = r#"{"errorMessages":["Issue type is not valid"],"errors":{"project":"no existe"}}"#;
        let summary = summarize_error(body);
        assert!(summary.contains("Issue type is not valid"));
        assert!(summary.contains("project: no existe"));
    }

    #[test]
    fn summarize_error_should_fall_back_to_raw_text() {
        assert_eq!(summarize_error("  "), "sin cuerpo en la respuesta");
        assert!(summarize_error("<html>500</html>").contains("html"));
    }

    #[test]
    fn into_json_should_map_status_to_error() {
        let ok = Client::into_json(201, r#"{"key":"ADR-1"}"#.to_string()).unwrap();
        assert_eq!(ok.get("key").and_then(Value::as_str), Some("ADR-1"));
        assert!(Client::into_json(204, String::new()).unwrap().is_null());

        let err = Client::into_json(400, r#"{"errorMessages":["mal"]}"#.to_string()).unwrap_err();
        let api = err.downcast_ref::<ApiError>().unwrap();
        assert_eq!(api.status, 400);
        assert!(api.to_string().contains("mal"));
    }

    #[test]
    fn credentials_should_come_from_harness_env_file() {
        let dir = tempfile::tempdir().unwrap();
        let paths = HarnessPaths::from_root(dir.path().to_path_buf());
        std::fs::write(
            dir.path().join(".harness.env"),
            "HARNESS_ATLASSIAN_EMAIL=a@b.cl\nHARNESS_ATLASSIAN_TOKEN=secreto\n",
        )
        .unwrap();
        let creds = Credentials::discover(&paths).unwrap();
        assert_eq!(creds.email, "a@b.cl");
        // El token no se expone: solo viaja dentro del header.
        assert!(creds.header().starts_with("Basic "));
    }

    #[test]
    fn config_files_should_match_the_installer_order() {
        // El binario tiene que mirar los MISMOS archivos que el instalador, en
        // el mismo orden: local del proyecto, local del arnes y despues los
        // globales del usuario (para no repetir el token en cada repo).
        let dir = tempfile::tempdir().unwrap();
        let paths = HarnessPaths::from_root(dir.path().join("hp"));
        let files = config_files(&paths);
        let names: Vec<String> = files
            .iter()
            .map(|f| f.to_string_lossy().into_owned())
            .collect();
        assert!(names[0].ends_with(".harness.env"));
        assert!(names[1].ends_with("hp/.harness.env"));
        if crate::pycompat::home_dir().is_some() {
            assert!(
                names.iter().any(|n| n.ends_with(".config/harness/config")),
                "falta el config global: {names:?}"
            );
            assert!(
                names.iter().any(|n| n.ends_with(".harnessrc")),
                "falta ~/.harnessrc: {names:?}"
            );
        }
    }
}
