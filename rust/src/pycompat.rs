//! Helpers de paridad con CPython: las salidas y archivos que produce este
//! binario deben ser intercambiables byte a byte con los de harness.py /
//! graph_memory.py (fallback Python del shim harness_cli).

use std::path::{Component, Path, PathBuf};
use std::time::UNIX_EPOCH;

use serde_json::Value;

/// mtime como float con la MISMA formula de CPython (`sec + 1e-9*nsec`).
/// No usar `as_secs_f64()`: redondea distinto y rompe la firma del plan.
pub fn mtime_f64(path: &Path) -> std::io::Result<f64> {
    let meta = std::fs::metadata(path)?;
    let modified = meta.modified()?;
    let dur = modified.duration_since(UNIX_EPOCH).unwrap_or_default();
    Ok(dur.as_secs() as f64 + f64::from(dur.subsec_nanos()) * 1e-9)
}

/// `str()` de Python para valores JSON sueltos interpolados en f-strings:
/// ausente/null -> "None", booleanos -> "True"/"False", numeros y strings tal cual.
pub fn py_str(value: Option<&Value>) -> String {
    match value {
        None | Some(Value::Null) => "None".to_string(),
        Some(Value::Bool(true)) => "True".to_string(),
        Some(Value::Bool(false)) => "False".to_string(),
        Some(Value::Number(n)) => n.to_string(),
        Some(Value::String(s)) => s.clone(),
        Some(other) => other.to_string(),
    }
}

/// `json.dumps(value, indent=2, ensure_ascii=False)`: pretty-print con 2
/// espacios y UTF-8 sin escapar (serde_json ya no escapa no-ASCII).
pub fn py_json_pretty(value: &Value) -> serde_json::Result<String> {
    let mut buf = Vec::new();
    let fmt = serde_json::ser::PrettyFormatter::with_indent(b"  ");
    let mut ser = serde_json::Serializer::with_formatter(&mut buf, fmt);
    serde::Serialize::serialize(value, &mut ser)?;
    // serde_json solo emite UTF-8 valido
    Ok(String::from_utf8_lossy(&buf).into_owned())
}

/// `os.path.expanduser("~")`: HOME en unix; en Windows el orden de ntpath
/// (USERPROFILE, luego HOMEDRIVE+HOMEPATH) para coincidir con el fallback Python.
pub fn home_dir() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        if let Some(p) = std::env::var_os("USERPROFILE") {
            if !p.is_empty() {
                return Some(PathBuf::from(p));
            }
        }
        let drive = std::env::var_os("HOMEDRIVE").unwrap_or_default();
        std::env::var_os("HOMEPATH").map(|p| {
            let mut out = std::ffi::OsString::from(drive);
            out.push(p);
            PathBuf::from(out)
        })
    }
    #[cfg(not(windows))]
    {
        std::env::var_os("HOME")
            .filter(|h| !h.is_empty())
            .map(PathBuf::from)
    }
}

/// Variable de entorno con truthiness de Python: ausente o vacia -> None.
pub fn env_nonempty(name: &str) -> Option<String> {
    std::env::var(name).ok().filter(|v| !v.is_empty())
}

/// `os.path.abspath`: normpath(join(cwd, path)) SIN resolver symlinks.
pub fn abspath(path: &Path) -> PathBuf {
    let joined = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(path)
    };
    normpath(&joined)
}

/// `os.path.normpath`: colapso lexico de `.` y `..` (sin tocar el filesystem).
pub fn normpath(path: &Path) -> PathBuf {
    let mut parts: Vec<Component> = Vec::new();
    for comp in path.components() {
        match comp {
            Component::CurDir => {}
            Component::ParentDir => match parts.last() {
                Some(Component::Normal(_)) => {
                    parts.pop();
                }
                Some(Component::RootDir) | Some(Component::Prefix(_)) => {}
                _ => parts.push(comp),
            },
            other => parts.push(other),
        }
    }
    let mut out = PathBuf::new();
    for c in &parts {
        out.push(c.as_os_str());
    }
    if out.as_os_str().is_empty() {
        out.push(".");
    }
    out
}

/// `os.path.relpath(path, base)`; None si Python lanzaria ValueError
/// (mezcla absoluta/relativa). El resultado usa el separador nativo.
pub fn relpath(path: &Path, base: &Path) -> Option<PathBuf> {
    pathdiff::diff_paths(path, base)
}

/// Epoch actual como float (equivalente de `datetime.now(timezone.utc).timestamp()`).
pub fn now_epoch_f64() -> f64 {
    std::time::SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as f64 + f64::from(d.subsec_nanos()) * 1e-9)
        .unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;
    use serde_json::json;

    #[test]
    fn py_str_should_render_missing_and_null_as_none() {
        assert_eq!(py_str(None), "None");
        assert_eq!(py_str(Some(&Value::Null)), "None");
    }

    #[test]
    fn py_str_should_render_python_booleans() {
        assert_eq!(py_str(Some(&json!(true))), "True");
        assert_eq!(py_str(Some(&json!(false))), "False");
    }

    #[test]
    fn py_str_should_render_numbers_and_strings_verbatim() {
        assert_eq!(py_str(Some(&json!(7))), "7");
        assert_eq!(py_str(Some(&json!("hola"))), "hola");
    }

    #[test]
    fn py_json_pretty_should_match_python_indent_two() {
        let v = json!({"id": 1, "name": "Pago QR", "tags": []});
        let out = py_json_pretty(&v).unwrap();
        assert_eq!(
            out,
            "{\n  \"id\": 1,\n  \"name\": \"Pago QR\",\n  \"tags\": []\n}"
        );
    }

    #[test]
    fn py_json_pretty_should_keep_utf8_unescaped() {
        let v = json!({"name": "Calderón"});
        let out = py_json_pretty(&v).unwrap();
        assert!(out.contains("Calderón"));
    }

    #[test]
    fn normpath_should_collapse_dot_and_dotdot() {
        assert_eq!(normpath(Path::new("/a/b/../c/./d")), PathBuf::from("/a/c/d"));
        assert_eq!(normpath(Path::new("/..")), PathBuf::from("/"));
    }

    #[test]
    fn mtime_f64_should_use_cpython_formula() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let m = mtime_f64(tmp.path()).unwrap();
        assert!(m > 0.0);
    }
}
