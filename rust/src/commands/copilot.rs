//! `harness copilot instalar|quitar` (feature #85): lo llaman los dos
//! instaladores. No usa `HarnessPaths`: la raiz viene por `--raiz` porque el
//! instalador ya la resolvio y este comando corre ANTES de que exista marker.
//!
//! El prefijo del hook puede venir por `HARNESS_COPILOT_HOOK` en vez de
//! `--hook`: Windows PowerShell 5.1 no sabe pasar a un ejecutable un argumento
//! con comillas adentro, y el prefijo las lleva (la ruta de `harness-hook.ps1`).

use std::path::Path;

use crate::copilot;
use crate::exit::Exit;

pub const HOOK_ENV: &str = "HARNESS_COPILOT_HOOK";

pub fn instalar(raiz: &Path, hook: Option<&str>, shell: &str, arnes: &str) -> anyhow::Result<()> {
    let prefijo = match hook.map(str::trim).filter(|h| !h.is_empty()) {
        Some(h) => h.to_string(),
        None => std::env::var(HOOK_ENV)
            .ok()
            .filter(|h| !h.trim().is_empty())
            .ok_or_else(|| Exit {
                code: 2,
                message: Some(format!(
                    "Falta el comando del hook: --hook \"<prefijo>\" o la variable {HOOK_ENV}."
                )),
            })?,
    };
    let (cambios, avisos) = copilot::instalar(raiz, &prefijo, shell, arnes)?;
    for a in avisos {
        eprintln!("[i] {a}");
    }
    for c in cambios {
        println!("{}", c.linea());
    }
    Ok(())
}

pub fn quitar(raiz: &Path) -> anyhow::Result<()> {
    let (cambios, avisos) = copilot::quitar(raiz)?;
    for a in avisos {
        eprintln!("[i] {a}");
    }
    if cambios.is_empty() {
        println!(
            "Nada que quitar: no hay {} ni {}.",
            copilot::CONFIG,
            copilot::INSTRUCCIONES
        );
        return Ok(());
    }
    for c in cambios {
        println!("{}", c.linea());
    }
    Ok(())
}
