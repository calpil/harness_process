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
