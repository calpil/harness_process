//! `harness contexto --feature <id> | --tema "<texto>"`: el paquete de contexto
//! para EMPEZAR a implementar (feature #56).
//!
//! Gemelo de `revision`, del otro lado del flujo, y con una diferencia que es
//! el punto: cuando no hay material, lo dice. Es de SOLO LECTURA.

use crate::contexto::{MAX_LINEAS_DEFAULT, armar};
use crate::exit::Exit;
use crate::features::{active_feature_index_con_foco, feature_at, load_features};
use crate::paths::HarnessPaths;
use crate::pycompat::py_str;

pub struct Opts<'a> {
    pub feature: Option<&'a str>,
    pub tema: Option<&'a str>,
    pub max_lineas: Option<usize>,
    pub con_grafo: bool,
    pub json: bool,
}

pub fn run(paths: &HarnessPaths, opts: Opts<'_>) -> anyhow::Result<()> {
    // AC-3: sin ninguna de las dos formas, el error dice las dos.
    if opts.feature.is_none() && opts.tema.is_none() && !hay_feature_activa(paths) {
        return Err(Exit {
            code: 2,
            message: Some(
                "contexto: decime de que. Dos formas:\n  \
                 harness contexto --feature <id>     (usa el nombre, el servicio y el spec de la feature)\n  \
                 harness contexto --tema \"<texto>\"   (para un tema todavia sin feature)"
                    .into(),
            ),
        }
        .into());
    }

    let data = load_features(paths)?;
    // Con `--tema` suelto no hace falta feature; con `--feature` o con una
    // activa, los docs se resuelven DESDE la feature (features #47 y #49).
    let feature = match (opts.tema, opts.feature) {
        (Some(_), None) if !hay_feature_activa(paths) => None,
        _ => {
            let idx = active_feature_index_con_foco(paths, &data, opts.feature)?;
            feature_at(&data, idx).as_object().cloned()
        }
    };
    let paths_feature = match &feature {
        Some(f) => paths.para_feature(f),
        None => paths.para_feature(&serde_json::Map::new()),
    };

    let tema = match (opts.tema, &feature) {
        (Some(t), _) => t.to_string(),
        (None, Some(f)) => tema_de_feature(f),
        (None, None) => String::new(),
    };

    let paquete = armar(
        &paths_feature,
        feature.as_ref(),
        &tema,
        opts.max_lineas.unwrap_or(MAX_LINEAS_DEFAULT),
    );

    if opts.json {
        println!("{}", serde_json::to_string_pretty(&paquete.render_json())?);
        return Ok(());
    }
    print!("{}", paquete.render_texto());
    let (lineas, tokens) = paquete.tamano();
    println!("\n[paquete] {lineas} lineas, ~{tokens} tokens estimados.");
    if opts.con_grafo {
        consultar_grafo(&tema);
    }
    Ok(())
}

/// El tema de una feature: su nombre en palabras, mas el servicio. El slug con
/// guiones bajos no matchea nada en prosa.
pub fn tema_de_feature(f: &serde_json::Map<String, serde_json::Value>) -> String {
    let nombre = py_str(f.get("name")).replace(['_', '-'], " ");
    let servicio = py_str(f.get("service"));
    if servicio.is_empty() || servicio == "None" {
        nombre
    } else {
        format!("{nombre} {servicio}")
    }
}

fn hay_feature_activa(paths: &HarnessPaths) -> bool {
    load_features(paths).is_ok_and(|d| {
        d.get("features")
            .and_then(|f| f.as_array())
            .is_some_and(|arr| {
                arr.iter()
                    .filter_map(|f| f.as_object())
                    .any(|f| py_str(f.get("status")) == "in_progress")
            })
    })
}

/// OBS-1: `graphify query` solo con `--con-grafo`, y solo si el binario existe.
/// El paquete tiene que ser barato y predecible por default.
fn consultar_grafo(tema: &str) {
    let salida = std::process::Command::new("graphify")
        .arg("query")
        .arg(tema)
        .output();
    match salida {
        Ok(o) => {
            println!("\n== graphify query \"{tema}\" ==");
            print!("{}", String::from_utf8_lossy(&o.stdout));
        }
        Err(_) => println!(
            "\n[--con-grafo] `graphify` no esta en el PATH: instalalo o consulta el grafo a mano."
        ),
    }
}
