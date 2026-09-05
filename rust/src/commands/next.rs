//! `harness next` (paridad: harness.py cmd_next).

use crate::dependencias::abiertas;
use crate::features::{feature_status, features_slice, load_features};
use crate::paths::HarnessPaths;
use crate::pycompat::{py_json_pretty, py_str};

pub fn run(paths: &HarnessPaths) -> anyhow::Result<()> {
    let data = load_features(paths)?;
    // Feature #75 (AC-2): una feature cuyas dependencias no estan cerradas no
    // se ofrece. Y si NO se ofrece nada por ese motivo, se dice: un "no hay
    // features pending" sobre un backlog lleno de pendings seria exactamente el
    // silencio que esta feature vino a cerrar.
    let mut esperando: Vec<String> = Vec::new();
    for f in features_slice(&data) {
        if feature_status(f) != Some("pending") {
            continue;
        }
        let abiertas = abiertas(&data, f);
        if abiertas.is_empty() {
            println!("{}", py_json_pretty(f)?);
            return Ok(());
        }
        esperando.push(format!(
            "  #{} {} espera: {}",
            py_str(f.get("id")),
            py_str(f.get("name")),
            abiertas
                .iter()
                .map(crate::dependencias::Abierta::etiqueta)
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    if esperando.is_empty() {
        println!("No hay features pending.");
        return Ok(());
    }
    println!(
        "No hay features pending SIN dependencias abiertas. {} esperando:",
        esperando.len()
    );
    for linea in &esperando {
        println!("{linea}");
    }
    println!("Cerra de lo que dependen, o arrancala igual: harness start --feature <id>");
    Ok(())
}
