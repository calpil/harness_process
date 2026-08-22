//! `harness revision --feature <id>`: el paquete minimo de revision.
//!
//! Existe porque verificar lo implementado llego a costar 10 millones de
//! tokens (feature #51): el reviewer arranca de aca en vez de explorar el repo.
//! Es de SOLO LECTURA — no escribe archivos ni toca estado.

use crate::features::{active_feature_index_con_foco, feature_at, load_features};
use crate::paths::HarnessPaths;
use crate::revision::{MAX_LINEAS_DEFAULT, armar};

pub fn run(
    paths: &HarnessPaths,
    fid: Option<&str>,
    max_lineas: Option<usize>,
    json: bool,
) -> anyhow::Result<()> {
    let data = load_features(paths)?;
    let idx = active_feature_index_con_foco(paths, &data, fid)?;
    // Los docs se resuelven DESDE la feature (features #47 y #49).
    let paths = &match feature_at(&data, idx).as_object() {
        Some(f) => paths.para_feature(f),
        None => paths.para_feature(&serde_json::Map::new()),
    };
    let Some(feature) = feature_at(&data, idx).as_object() else {
        anyhow::bail!("feature_list.json: feature invalida");
    };

    let paquete = armar(paths, feature, max_lineas.unwrap_or(MAX_LINEAS_DEFAULT));
    if json {
        println!("{}", serde_json::to_string_pretty(&paquete.render_json())?);
        return Ok(());
    }
    print!("{}", paquete.render_texto());
    // El costo se ve ANTES de gastarlo (AC-12b).
    let (lineas, tokens) = paquete.tamano();
    println!("\n[paquete] {lineas} lineas, ~{tokens} tokens estimados.");
    Ok(())
}
