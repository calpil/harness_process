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
/// Cuerpo del review sin los sellos anteriores, lo que hace idempotente a
/// `estampar`.
///
/// Se saca el sello solo de las lineas que estan FUERA de un bloque: un review
/// que documenta el formato del sello lo cita, y borrar esa cita seria mutar la
/// prosa del reviewer.
///
/// Esta suelta —y no inline en `estampar`— para que el test exhaustivo del AC-4
/// pueda comparar lo que ve ESTE consumidor contra la clasificacion del parser
/// unico. Inline, el unico que podia mirarla era un E2E con un caso a mano, y un
/// caso a mano es exactamente como se colo la divergencia.
pub(crate) fn cuerpo_sin_sellos(review: &str) -> String {
    crate::markdown::lineas_clasificadas(review)
        .into_iter()
        .filter(|(l, clase)| {
            // El MISMO predicado que usa el gate para leer el sello, y no
            // `starts_with(SELLO_REVIEW)`: con ese, una linea de prosa del
            // reviewer que empezara con `Revisado:` se borraba del archivo sin
            // aviso, aunque el gate jamas la habria contado como sello.
            *clase != crate::markdown::Clase::Fuera
                || crate::revision::veredicto_de_sello(l).is_none()
        })
        .map(|(l, _)| l)
        .collect::<Vec<_>>()
        .join("\n")
}

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
    // La MISMA pregunta que hace el gate del cierre, con la misma respuesta.
    // Antes cada uno tenia la suya y discrepaban: el gate rechazaba un spec con
    // un AC ilegible y `estampar` lo estampaba igual, asi que el sello decia
    // "approved" sobre una cobertura que el cierre no iba a aceptar.
    if let Some(motivo) = crate::revision::motivo_spec_inservible(&fid.to_string(), &spec) {
        anyhow::bail!("{motivo}");
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
    // Se saca el sello anterior, pero SOLO de las lineas que estan FUERA de un
    // bloque: un review que documenta el formato del sello lo cita, y borrar esa
    // cita seria mutar la prosa del reviewer.
    //
    // Feature #67: esto tenia su propio parser (toggle con cualquier fence) que
    // discrepaba con el del gate (fences emparejados). Medido: segun la paridad
    // de fences ajenos citados, o borraba la cita del reviewer o dejaba DOS
    // sellos contradictorios en el archivo. Ahora los dos preguntan lo mismo.
    let cuerpo = cuerpo_sin_sellos(&review);
    // El sello va tras la primera linea (el titulo), donde se lee sin scrollear
    // — pero NUNCA dentro de un bloque ```: ahi el gate no lo veria y el
    // comando estaria diciendo "registrado" sobre algo que su propio gate niega.
    // Si el review arranca con un fence, se sella arriba de todo.
    // La primera linea ya viene clasificada por el parser unico: no hace falta
    // un tercer chequeo de fences con su propia idea de que es un bloque
    // (feature #67).
    let arranca_en_bloque = crate::markdown::lineas_clasificadas(&cuerpo)
        .first()
        .is_some_and(|(_, c)| *c != crate::markdown::Clase::Fuera);
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
    // Se comprueba con el MISMO parser del gate ANTES de escribir. El chequeo
    // ya operaba sobre `out` en memoria, no sobre el archivo, asi que correrlo
    // primero no pierde nada y saca la escritura de en medio: lo irreversible
    // va ultimo (AC-11 de la #67). No se pudo reproducir un caso donde el orden
    // viejo dejara el archivo pisado con el comando en error —el `bail` posterior
    // no revertia nada, pero el contenido escrito era el bueno— asi que esto es
    // cambio de forma, no arreglo de bug: la forma en que un comando no deberia
    // poder fallar DESPUES de haber tocado el disco.
    if crate::revision::veredicto_estampado(&out).as_deref() != Some(veredicto) {
        anyhow::bail!(
            "El sello no se puede leer con el parser del gate, asi que no se escribio {rel}.\n    \
             Revisa que el review no lo deje dentro de un bloque de codigo."
        );
    }
    std::fs::write(&ruta, &out)?;

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

#[cfg(test)]
mod tests_sello {
    use crate::revision::linea_sello;

    #[test]
    fn prosa_que_arranca_con_revisado_sobrevive() {
        // Hallazgo de la revision adversarial de la #67. El limpiador borraba
        // CUALQUIER linea de afuera que empezara con `Revisado:`, aunque el gate
        // jamas la hubiera contado como sello: una linea de prosa del reviewer
        // desaparecia del archivo al estampar, sin aviso. Es la misma falla que
        // el resto de la feature —dos partes que no coinciden en QUE ES un
        // sello— un nivel mas abajo que los fences.
        let review = "# Review\nRevisado: el parser unico esta bien resuelto, pero el tope miente.\nRevisado a mano por un humano, sin el binario.\n| AC-1 | a.md:1 | ok |\n";
        let out = super::cuerpo_sin_sellos(review);
        assert!(
            out.contains("el parser unico esta bien resuelto"),
            "se borro prosa del reviewer:\n{out}"
        );
        assert!(out.contains("Revisado a mano por un humano"), "y la otra tambien");
    }

    #[test]
    fn el_sello_de_verdad_se_sigue_borrando() {
        // La otra mitad: si el limpiador dejara de sacar el sello anterior,
        // `estampar` dejaria DOS sellos contradictorios y romperia su promesa de
        // idempotencia. Un predicado mas estricto no puede costar eso.
        let viejo = linea_sello("changes_requested", "2026-01-01 00:00");
        let review = format!("# Review\n{viejo}\n| AC-1 | a.md:1 | ok |\n");
        let out = super::cuerpo_sin_sellos(&review);
        assert!(!out.contains("changes_requested"), "quedo el sello anterior:\n{out}");

        // Y un sello con veredicto valido escrito a mano tambien se saca: es
        // indistinguible de uno real y el gate lo leeria como veredicto, asi que
        // dejarlo seria dejar DOS.
        let review = "# Review\nRevisado: approved · a mano · sin binario\n";
        assert!(!super::cuerpo_sin_sellos(review).contains("approved"));
    }
}
