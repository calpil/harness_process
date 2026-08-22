//! `harness buscar "<consulta>"` (feature #20).
//!
//! Solo lectura: no escribe un byte, no abre red y no consulta el hub. Exit 0
//! con o sin resultados (no encontrar no es un error); exit 2 solo por uso
//! invalido.

use serde_json::json;

use crate::buscar::{self, Resultado, TOPE};
use crate::exit::Exit;
use crate::paths::HarnessPaths;

pub fn run(paths: &HarnessPaths, consulta: &str, as_json: bool, todos: bool) -> anyhow::Result<()> {
    if buscar::terminos(consulta).is_empty() {
        return Err(Exit {
            code: 2,
            message: Some(
                "Falta la consulta.\n    \
                 Uso: sh harness_cli buscar \"<terminos>\" [--json] [--todos]\n    \
                 Ejemplo: sh harness_cli buscar \"ureq adr\""
                    .to_string(),
            ),
        }
        .into());
    }
    let res = buscar::buscar(paths, consulta);
    if as_json {
        return emitir_json(&res, todos);
    }
    emitir_humano(&res, todos, consulta)
}

fn emitir_json(res: &Resultado, todos: bool) -> anyhow::Result<()> {
    let corte = if todos { res.hallazgos.len() } else { TOPE.min(res.hallazgos.len()) };
    let rows: Vec<_> = res.hallazgos[..corte]
        .iter()
        .map(|h| {
            json!({
                "archivo": h.archivo,
                "linea": h.linea,
                "feature": h.feature,
                "fecha": h.fecha,
                "fuente": h.fuente.etiqueta(),
                "texto": h.texto,
                "score": h.score,
                "repetido": h.repetido,
            })
        })
        .collect();
    // JSON valido tambien sin resultados: un script no deberia manejar dos
    // formatos distintos (AC-11).
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "resultados": rows,
            "total": res.hallazgos.len(),
            "mostrados": corte,
            "parcial": res.parcial,
            "archivos_revisados": res.archivos,
        }))?
    );
    Ok(())
}

fn emitir_humano(res: &Resultado, todos: bool, consulta: &str) -> anyhow::Result<()> {
    if res.archivos == 0 {
        println!("No hay corpus que buscar todavia (falta docs/ o esta vacio).");
        return Ok(());
    }
    if res.hallazgos.is_empty() {
        println!("Sin coincidencias para '{consulta}' en {} archivo(s).", res.archivos);
        println!("  Proba con menos terminos, o con una palabra mas general.");
        return Ok(());
    }
    if res.parcial {
        println!(
            "[i] Ninguna linea tiene TODOS los terminos: se muestran las que tienen alguno."
        );
    }
    let corte = if todos { res.hallazgos.len() } else { TOPE.min(res.hallazgos.len()) };
    println!(
        "{} resultado(s) en {} archivo(s):",
        res.hallazgos.len(),
        res.archivos
    );
    for h in &res.hallazgos[..corte] {
        let mut meta = h.fuente.etiqueta().to_string();
        if !h.feature.is_empty() {
            meta.push_str(&format!(" #{}", h.feature));
        }
        if !h.fecha.is_empty() {
            meta.push_str(&format!(" {}", h.fecha));
        }
        // Feature #39: la linea aparecia identica en varios documentos y se
        // mostro una sola vez. Se dice cuantas, porque "esto esta en cuatro
        // lugares" es un dato sobre el tema, y un dedup callado lo perderia.
        if h.repetido > 0 {
            meta.push_str(&format!(" +{} archivo(s)", h.repetido));
        }
        println!("\n  {}:{}  [{meta}]", h.archivo, h.linea);
        println!("    {}", buscar::recorta(&h.texto));
    }
    // Nunca se trunca en silencio (AC-9).
    let fuera = res.hallazgos.len() - corte;
    if fuera > 0 {
        println!("\n  ... {fuera} resultado(s) mas. Vealos con --todos.");
    }
    Ok(())
}
