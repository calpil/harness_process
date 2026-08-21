//! Estado vivo: progress/history.md (append-only), stamps de autocheck/nudge.

use std::io::Write;

use crate::paths::HarnessPaths;

pub fn now_stamp() -> String {
    chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

/// `log()`: linea append-only en progress/history.md.
pub fn log(paths: &HarnessPaths, line: &str) -> anyhow::Result<()> {
    std::fs::create_dir_all(&paths.progress)?;
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&paths.history)?;
    writeln!(f, "- {} {}", now_stamp(), line)?;
    Ok(())
}

/// `_touch_stamp()`: linea base del checkpoint automatico (best-effort).
/// Stamp de autocheck POR FEATURE (feature #47 / AC-10): el checkpoint de una
/// feature no puede borrar ni pisar el de otra.
pub fn touch_autocheck_stamp_de(paths: &HarnessPaths, fid: &str) {
    if std::fs::create_dir_all(&paths.progress).is_ok() {
        let _ = std::fs::File::create(paths.autocheck_stamp_de(fid));
    }
}

/// Reescribe `progress/current.md` como INDICE de las features activas
/// (feature #47 / AC-9). Ya no es el estado de nadie: es la lista de lo que
/// esta abierto, con su rama y su worktree.
pub fn escribir_indice(paths: &HarnessPaths, data: &serde_json::Value) -> anyhow::Result<()> {
    use crate::features::{feature_status, features_slice};
    use crate::pycompat::py_str;

    std::fs::create_dir_all(&paths.progress)?;
    let activas: Vec<&serde_json::Value> = features_slice(data)
        .iter()
        .filter(|f| feature_status(f) == Some("in_progress"))
        .collect();

    let mut out = String::from("# Estado Actual\n\n");
    if activas.is_empty() {
        out.push_str("Sin feature activa.\n");
    } else {
        out.push_str(&format!(
            "{} feature(s) en curso. El estado vivo de cada una esta en su propio archivo.\n\n",
            activas.len()
        ));
        for f in &activas {
            let fid = py_str(f.get("id"));
            out.push_str(&format!(
                "- #{fid} {} -> `progress/current-{fid}.md`\n",
                py_str(f.get("name"))
            ));
            if let Some(rama) = f.get("branch").and_then(serde_json::Value::as_str) {
                out.push_str(&format!("  - rama: `{rama}`\n"));
            }
            if let Some(wt) = f.get("worktree").and_then(serde_json::Value::as_str) {
                out.push_str(&format!("  - worktree: `{wt}`\n"));
            }
        }
    }
    crate::features::write_text_atomic(&paths.current, &out)
}

pub fn touch_autocheck_stamp(paths: &HarnessPaths) {
    if std::fs::create_dir_all(&paths.progress).is_ok() {
        let _ = std::fs::File::create(&paths.autocheck_stamp);
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;

    #[test]
    fn log_should_append_dash_stamp_line() {
        let dir = tempfile::tempdir().unwrap();
        let paths = HarnessPaths::from_root(dir.path().to_path_buf());
        log(&paths, "add feature #1 demo").unwrap();
        log(&paths, "start feature #1 demo").unwrap();
        let text = std::fs::read_to_string(&paths.history).unwrap();
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].starts_with("- "));
        assert!(lines[0].ends_with(" add feature #1 demo"));
    }
}
