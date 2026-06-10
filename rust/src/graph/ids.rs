//! Identificadores del hub (paridad: graph_memory.py qualify/artifact_id/is_repo_root).

use std::path::Path;
use std::process::{Command, Stdio};

use sha1::{Digest, Sha1};

pub fn qualify(project: &str, name: &str) -> String {
    if name.contains('/') {
        name.to_string()
    } else {
        format!("{project}/{name}")
    }
}

pub fn artifact_id(path: &str) -> String {
    let sep = std::path::MAIN_SEPARATOR.to_string();
    let safe = path
        .replace(&sep, "__")
        .replace('/', "__")
        .replace('.', "_");
    let digest = hex::encode(Sha1::digest(path.as_bytes()));
    let short = digest.get(..8).unwrap_or(&digest);
    format!("{safe}_{short}")
}

/// True si `path` es la raiz de un repo git (compara realpaths como Python).
pub fn is_repo_root(path: &Path) -> bool {
    let output = Command::new("git")
        .arg("-C")
        .arg(path)
        .args(["rev-parse", "--show-toplevel"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output();
    match output {
        Ok(out) => {
            if !out.status.success() {
                return false;
            }
            let top = String::from_utf8_lossy(&out.stdout).trim().to_string();
            match (std::fs::canonicalize(&top), std::fs::canonicalize(path)) {
                (Ok(a), Ok(b)) => a == b,
                _ => false,
            }
        }
        Err(_) => path.join(".git").is_dir(),
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;

    #[test]
    fn qualify_should_prefix_project_when_unqualified() {
        assert_eq!(qualify("demo", "ms-auth-service"), "demo/ms-auth-service");
        assert_eq!(qualify("demo", "otro/servicio"), "otro/servicio");
    }

    #[test]
    fn artifact_id_should_match_python_shape() {
        // Python: "a/b.py" -> "a__b_py_" + sha1("a/b.py")[:8]
        let id = artifact_id("a/b.py");
        assert!(id.starts_with("a__b_py_"));
        assert_eq!(id.len(), "a__b_py_".len() + 8);
    }

    #[test]
    fn is_repo_root_should_detect_a_real_git_repo() {
        let dir = tempfile::tempdir().unwrap();
        assert!(!is_repo_root(dir.path()));
        let status = Command::new("git")
            .arg("init")
            .arg("-q")
            .arg(dir.path())
            .status()
            .unwrap();
        assert!(status.success());
        assert!(is_repo_root(dir.path()));
        let nested = dir.path().join("sub");
        std::fs::create_dir(&nested).unwrap();
        assert!(!is_repo_root(&nested));
    }
}
