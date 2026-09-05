//! `harness depende` (feature #75): declarar que una feature espera a otra.
//!
//! Existe porque `add --depends-on` no alcanza: una dependencia se descubre
//! casi siempre DESPUES de que las dos features existen. El recorrido P1 del
//! spec —"Alan declara que la #21 depende de la #17"— es imposible desde `add`,
//! porque las dos ya estaban creadas. Y sin este camino la deteccion de ciclos
//! del AC-5 seria codigo inalcanzable: por `add` el grafo es un DAG por
//! construccion (una feature nueva solo puede depender de ids anteriores).
//!
//! Peldano 3 de la escalera de huella (comando nuevo) porque ningun comando
//! existente edita una feature: `add` crea y `close` cierra.

use serde_json::{Value, json};

use crate::exit::Exit;
use crate::features::{feature_mut, find_feature_index, load_features, save_features};
use crate::paths::HarnessPaths;
use crate::progress::log;
use crate::pycompat::py_str;

pub fn run(paths: &HarnessPaths, fid: &str, de: &[String], quitar: bool) -> anyhow::Result<()> {
    let mut data = load_features(paths)?;
    let idx = find_feature_index(&data, fid)?;
    let actuales = crate::dependencias::declaradas(crate::features::feature_at(&data, idx));

    let nuevas: Vec<String> = if quitar {
        actuales.iter().filter(|d| !de.contains(d)).cloned().collect()
    } else {
        let mut v = actuales.clone();
        for d in de {
            if !v.contains(d) {
                v.push(d.clone());
            }
        }
        v
    };

    // AC-1 y AC-5: se valida el conjunto RESULTANTE, no solo lo que se agrega.
    // Validar el delta dejaria pasar un ciclo formado entre dos altas seguidas.
    if !quitar
        && let Some(motivo) = crate::dependencias::motivo_invalido(&data, fid, &nuevas)
    {
        return Err(Exit {
            code: 2,
            message: Some(format!(
                "No se puede declarar esa dependencia para la feature #{fid}: {motivo}\n    \
                 El backlog no se toco."
            )),
        }
        .into());
    }

    {
        let feature = feature_mut(&mut data, idx)?;
        if nuevas.is_empty() {
            feature.remove("depends_on");
        } else {
            feature.insert(
                "depends_on".to_string(),
                Value::Array(nuevas.iter().map(|d| json!(d)).collect()),
            );
        }
    }
    save_features(paths, &data)?;

    let verbo = if quitar { "quitada(s)" } else { "declarada(s)" };
    println!(
        "Feature #{fid} {}: depende de {}",
        verbo,
        if nuevas.is_empty() {
            "(nada)".to_string()
        } else {
            nuevas
                .iter()
                .map(|d| format!("#{d}"))
                .collect::<Vec<_>>()
                .join(", ")
        }
    );
    let abiertas = crate::dependencias::abiertas(&data, crate::features::feature_at(&data, idx));
    if !abiertas.is_empty() {
        println!(
            "  Sin cerrar todavia: {}",
            abiertas
                .iter()
                .map(crate::dependencias::Abierta::etiqueta)
                .collect::<Vec<_>>()
                .join(", ")
        );
        println!("  `next` no la va a ofrecer hasta que cierren; `start` te deja arrancarla igual.");
    }
    log(
        paths,
        &format!(
            "depende feature #{fid} {verbo} de={} nombre={}",
            de.join(","),
            py_str(crate::features::feature_at(&data, idx).get("name"))
        ),
    )?;
    Ok(())
}
