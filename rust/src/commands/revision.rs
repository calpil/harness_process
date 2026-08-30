//! `harness revision --feature <id>`: el paquete minimo de revision.
//!
//! Existe porque verificar lo implementado llego a costar 10 millones de
//! tokens (feature #51): el reviewer arranca de aca en vez de explorar el repo.
//! De SOLO LECTURA en su forma base. Con `--veredicto` hace lo unico que
//! escribe: estampa la linea que el gate del cierre lee (feature #64). La
//! parte que DECIDE (`acs_sin_fila`) es pura y corre antes; la que escribe
//! solo ejecuta un veredicto ya validado.

use crate::features::{active_feature_index_con_foco, feature_at, load_features};
use crate::paths::HarnessPaths;
use crate::revision::{
    MAX_LINEAS_DEFAULT, VEREDICTOS, acs_sin_fila, armar, linea_sello, review_path, review_rel,
};

pub fn run(
    paths: &HarnessPaths,
    fid: Option<&str>,
    max_lineas: Option<usize>,
    veredicto: Option<&str>,
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

    if let Some(v) = veredicto {
        return estampar(paths, feature, v);
    }

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

/// Estampa el veredicto en `docs/review-<id>.md` y deja el rastro en
/// `progress/history.md`.
///
/// Se niega —sin escribir nada— si el review no responde por cada AC-n que
/// declara el SPEC con una cita `archivo:linea`. Esa es la parte del gate que
/// un archivo tipeado en cinco segundos no puede satisfacer.
fn estampar(
    paths: &HarnessPaths,
    feature: &serde_json::Map<String, serde_json::Value>,
    veredicto: &str,
) -> anyhow::Result<()> {
    if !VEREDICTOS.contains(&veredicto) {
        anyhow::bail!(
            "Veredicto invalido: `{veredicto}`. Los de roles/reviewer.md son: {}.",
            VEREDICTOS.join(", ")
        );
    }
    let fid = feature
        .get("id")
        .map(std::string::ToString::to_string)
        .unwrap_or_default();
    let rel = review_rel(&fid);
    let ruta = review_path(paths, &fid);
    let Ok(review) = std::fs::read_to_string(&ruta) else {
        anyhow::bail!(
            "Falta {rel}: escribi el veredicto ANTES de estamparlo.\n    \
             Arranca por el paquete: sh harness_cli revision --feature {fid}"
        );
    };
    // Un spec ilegible o sin AC no es "cubierto": es que no hay contra que medir.
    // Sin esta guarda, `unwrap_or_default()` + `parsear("")` = 0 AC = lista vacia
    // de faltantes, y el comando estampaba `approved` sobre un review que decia
    // "nada" (lo encontro el reviewer de esta feature).
    let spec_path = crate::spec::spec_path(paths, feature);
    let Ok(spec) = std::fs::read_to_string(&spec_path) else {
        anyhow::bail!(
            "No se pudo leer el spec de la feature #{fid}: {}.\n    \
             Sin spec no hay AC contra que medir el review.",
            spec_path.display()
        );
    };
    if crate::verificacion::parsear(&spec).is_empty() {
        anyhow::bail!(
            "El spec de la feature #{fid} no declara ningun AC-n.\n    \
             Sin AC no hay contra que medir el review: completa el spec primero."
        );
    }

    // La parte que DECIDE. Si falla, no se escribe nada.
    let faltan = acs_sin_fila(&crate::revision::raices_de_citas(paths), &spec, &review);
    if !faltan.is_empty() {
        anyhow::bail!(
            "{rel} no responde por {} AC del spec: {}.\n    \
             Cada AC-n necesita una fila que lo nombre y cite `archivo:linea`,\n    \
             con un archivo que exista y una linea que exista en el.\n    \
             Una fila sin cita que resuelva es una afirmacion, no una verificacion.",
            faltan.len(),
            faltan.join(", ")
        );
    }

    // La parte que ESCRIBE. Idempotente: reemplaza el sello anterior si lo hay.
    let stamp = crate::progress::now_stamp();
    let sello = linea_sello(veredicto, &stamp);
    // Se saca el sello anterior, pero SOLO fuera de los bloques ```: un review
    // que documenta el formato del sello lo cita, y borrar esa cita seria mutar
    // la prosa del reviewer.
    let mut en_bloque = false;
    let mut cuerpo_lineas: Vec<&str> = Vec::new();
    for l in review.lines() {
        let t = l.trim_start();
        if t.starts_with("```") || t.starts_with("~~~") {
            en_bloque = !en_bloque;
            cuerpo_lineas.push(l);
            continue;
        }
        if !en_bloque && l.trim_start().starts_with(crate::revision::SELLO_REVIEW) {
            continue;
        }
        cuerpo_lineas.push(l);
    }
    let cuerpo = cuerpo_lineas.join("\n");
    // El sello va tras la primera linea (el titulo), donde se lee sin scrollear
    // — pero NUNCA dentro de un bloque ```: ahi el gate no lo veria y el
    // comando estaria diciendo "registrado" sobre algo que su propio gate niega.
    // Si el review arranca con un fence, se sella arriba de todo.
    let arranca_en_bloque = cuerpo
        .lines()
        .next()
        .is_some_and(|l| l.trim_start().starts_with("```") || l.trim_start().starts_with("~~~"));
    let mut out = String::new();
    let mut lineas = cuerpo.lines();
    if !arranca_en_bloque && let Some(titulo) = lineas.next() {
        out.push_str(titulo);
        out.push('\n');
    }
    out.push_str(&sello);
    out.push('\n');
    for l in lineas {
        out.push_str(l);
        out.push('\n');
    }
    std::fs::write(&ruta, &out)?;
    // Se re-lee con el MISMO parser del gate: el comando no afirma "registrado"
    // sin comprobar que lo registrado se pueda leer.
    if crate::revision::veredicto_estampado(&out).as_deref() != Some(veredicto) {
        anyhow::bail!(
            "El sello quedo escrito pero el gate no lo puede leer en {rel}.\n    \
             Revisa que el review no lo deje dentro de un bloque de codigo."
        );
    }

    let _ = crate::progress::log(
        paths,
        &format!("revision feature #{fid} veredicto={veredicto}"),
    );
    println!("[OK] Veredicto registrado en {rel}: {veredicto}");
    println!("    {sello}");
    if veredicto != "approved" {
        println!("    Un cierre `done` exige `approved`: el gate del cierre lo va a rechazar.");
    }
    Ok(())
}
