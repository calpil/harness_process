//! `harness status` (paridad: harness.py cmd_status).

use serde_json::Value;

use crate::features::{active_indices, feature_status, features_slice, load_features};
use crate::paths::HarnessPaths;
use crate::plan::{get_plan_sig, is_plan_stale};
use crate::pycompat::py_str;
use crate::spec::{is_spec_stale, spec_state};

/// Los estados que tienen bucket propio en la cabecera. Lo que no este aca cae
/// en `otros=N` y se VE — antes desaparecia sin dejar rastro.
pub(crate) const ESTADOS_CON_BUCKET: [&str; 6] = [
    "in_progress",
    "pending",
    "blocked",
    "done",
    crate::commands::close::SUPERSEDED,
    crate::commands::close::AGUAS_ARRIBA,
];

pub fn run(paths: &HarnessPaths) -> anyhow::Result<()> {
    let data = load_features(paths)?;
    let features = features_slice(&data);
    let count = |status: &str| {
        features
            .iter()
            .filter(|f| feature_status(f) == Some(status))
            .count()
    };
    // La cabecera SUMA (feature #65). Antes enumeraba cuatro estados de los
    // cinco y `superseded` desaparecia: con dos features absorbidas imprimia
    // "4 feature(s) | active=0 pending=0 blocked=0 done=2" y los numeros no
    // daban. Un resumen que no suma invita a buscar las que faltan en otro lado.
    // `otros` existe para que agregar un estado nuevo no vuelva a romper esto en
    // silencio: lo que no tenga su bucket cae ahi y se ve.
    let conocidos = ESTADOS_CON_BUCKET;
    let otros = features
        .iter()
        .filter(|f| !feature_status(f).is_some_and(|s| conocidos.contains(&s)))
        .count();
    let mut linea = format!(
        "Backlog: {} feature(s) | active={} pending={} blocked={} done={} superseded={} aguas-arriba={}",
        features.len(),
        count("in_progress"),
        count("pending"),
        count("blocked"),
        count("done"),
        count(crate::commands::close::SUPERSEDED),
        count(crate::commands::close::AGUAS_ARRIBA)
    );
    if otros > 0 {
        linea.push_str(&format!(" otros={otros}"));
    }
    println!("{linea}");
    for f in features {
        let services = f
            .get("microservicios")
            .and_then(Value::as_array)
            .map(|a| {
                a.iter()
                    .map(|s| s.as_str().unwrap_or_default())
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .unwrap_or_default();
        let services = if services.is_empty() {
            "sin servicios".to_string()
        } else {
            services
        };
        // `superseded` se lee con quien la absorbio: sin eso, el estado no
        // dice nada mas que `blocked` (feature #37).
        let estado = match f.get("superseded_by").and_then(Value::as_str) {
            Some(por) if py_str(f.get("status")) == "superseded" => {
                format!("superseded por #{por}")
            }
            _ => match f.get("resuelto_en").and_then(Value::as_str) {
                // "sin verificar" NO es cortesia: es la unica parte del renglon
                // que el arnes puede garantizar. La referencia vive en otro repo
                // y no la puede abrir; decirla sin la marca seria afirmar lo que
                // no comprobo, que es lo que la #63 cerro (feature #65).
                Some(r) if py_str(f.get("status")) == crate::commands::close::AGUAS_ARRIBA => {
                    format!("resuelto aguas arriba en {r}, sin verificar")
                }
                _ => py_str(f.get("status")),
            },
        };
        println!(
            "  #{} [{estado}] {} ({services})",
            py_str(f.get("id")),
            py_str(f.get("name"))
        );
    }
    if paths.current.exists() {
        let content = std::fs::read_to_string(&paths.current)?;
        let content = content.trim();
        if !content.is_empty() {
            println!("\nprogress/current.md:");
            println!("{content}");
        }
    }

    // Reporte de frescura de planes (importante para multi-LLM)
    for idx in active_indices(&data) {
        let Some(f) = features[idx].as_object() else {
            continue;
        };
        // El estado global vive en el principal, pero el spec y el plan son
        // documentos de la feature. `check-spec` usa esta misma resolución:
        // hacerlo dentro del loop impide que una feature vea el docs/ de otra.
        let paths_feature = paths.para_feature(f);
        if get_plan_sig(f).is_some() {
            if is_plan_stale(&paths_feature, f) {
                println!(
                    "  [!] #{} PLAN STALE - actualizado por otro agente/LLM. Ejecuta: harness.py check-plan",
                    py_str(f.get("id"))
                );
            } else {
                println!("  [plan] #{} fresco", py_str(f.get("id")));
            }
        }
        // Estado del spec SDD (draft/approved/ausente) + frescura multi-LLM.
        let spec_fresh = if is_spec_stale(&paths_feature, f) { "STALE" } else { "fresco" };
        println!(
            "  [spec] #{} {} ({spec_fresh})",
            py_str(f.get("id")),
            spec_state(&paths_feature, f).label()
        );
    }
    Ok(())
}
