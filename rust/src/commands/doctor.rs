//! `harness doctor [--json]` (feature #25).
//!
//! Diagnostica la INSTALACION. Para el proceso (spec, plan, PRDs, lecciones,
//! perfil, convenciones) esta `harness_check.sh`, y no se pisan: el pie de la
//! salida remite a el, y un test lo verifica (AC-14).
//!
//! **Solo lee.** El modulo no importa nada que escriba, y por eso la promesa se
//! sostiene sola (leccion `promesas-estructurales-vs-disciplina`).

use serde_json::json;

use crate::doctor::{self, Estado, Hallazgo};
use crate::exit::Exit;
use crate::paths::HarnessPaths;

pub fn run(paths: &HarnessPaths, as_json: bool) -> anyhow::Result<()> {
    let hallazgos = doctor::diagnosticar(paths);
    if as_json {
        emitir_json(paths, &hallazgos)?;
    } else {
        emitir_humano(paths, &hallazgos);
    }
    match doctor::exit_code(&hallazgos) {
        0 => Ok(()),
        code => Err(Exit { code, message: None }.into()),
    }
}

fn emitir_humano(paths: &HarnessPaths, hallazgos: &[Hallazgo]) {
    println!("== Harness Doctor: la instalacion ==");
    println!("   arnes: {}", paths.root.display());
    println!("   raiz:  {}\n", paths.repo_root.display());
    for h in hallazgos {
        println!("{} {:<13} {}", h.estado.simbolo(), h.area.etiqueta(), h.detalle);
        // El remedio va en su propia linea y es copiable tal cual (AC-2).
        if let Some(remedio) = &h.remedio
            && h.estado != Estado::Ok
        {
            println!("                   Remedio: {remedio}");
        }
    }
    let fallas = hallazgos.iter().filter(|h| h.estado.bloquea()).count();
    let avisos = hallazgos.iter().filter(|h| h.estado == Estado::Aviso).count();
    println!();
    if fallas == 0 {
        println!("[Ok] Instalacion sana ({avisos} aviso(s), que no impiden trabajar).");
    } else {
        println!("[!!] {fallas} problema(s) que impiden trabajar, {avisos} aviso(s).");
    }
    println!("Esto revisa la INSTALACION. Para el proceso (spec, plan, PRDs, lecciones,");
    println!("perfil, convenciones): bash harness_check.sh");
}

fn emitir_json(paths: &HarnessPaths, hallazgos: &[Hallazgo]) -> anyhow::Result<()> {
    let areas: Vec<_> = hallazgos
        .iter()
        .map(|h| {
            json!({
                "area": h.area.etiqueta(),
                "estado": h.estado.etiqueta(),
                "detalle": h.detalle,
                "remedio": h.remedio,
            })
        })
        .collect();
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "arnes": paths.root.display().to_string(),
            "raiz": paths.repo_root.display().to_string(),
            "sana": doctor::exit_code(hallazgos) == 0,
            "fallas": hallazgos.iter().filter(|h| h.estado.bloquea()).count(),
            "areas": areas,
        }))?
    );
    Ok(())
}
