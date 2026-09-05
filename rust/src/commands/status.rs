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
    // Feature #72 / AC-10: el preflight de la actualizacion. Las features
    // abiertas SIN worktree escriben en el checkout compartido, y son las que la
    // regla de aislamiento va a afectar. Se INVENTARIAN y nada mas: el arnes no
    // mueve commits, no cambia ramas, no borra worktrees y no para procesos.
    // Migrar trabajo vivo se coordina con el usuario, no se automatiza.
    if let Some(aviso) = aviso_de_features_sin_aislar(features) {
        println!("{aviso}");
    }
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

/// Las features abiertas que escriben en el checkout compartido (AC-10).
///
/// PURA: recibe las features y devuelve el aviso, o `None` si no hay ninguna.
/// Solo informa — es un inventario, no una migracion. Vive en `status` y no en
/// `doctor` a proposito: `doctor` diagnostica la INSTALACION y el estado del
/// proceso es de otro lado (frontera de la feature #25).
pub fn aviso_de_features_sin_aislar(features: &[Value]) -> Option<String> {
    let sin_aislar: Vec<String> = features
        .iter()
        .filter(|f| feature_status(f) == Some("in_progress"))
        .filter(|f| {
            f.get("worktree")
                .and_then(Value::as_str)
                .is_none_or(|w| w.trim().is_empty())
        })
        .map(|f| format!("#{} {}", py_str(f.get("id")), py_str(f.get("name"))))
        .collect();
    if sin_aislar.is_empty() {
        return None;
    }
    Some(format!(
        "[!] {} feature(s) abiertas SIN worktree, escribiendo en el checkout compartido: {}.\n    \
         Su trabajo NO esta aislado. El arnes no las migra solo: no mueve commits, no cambia\n    \
         ramas, no borra worktrees y no para procesos. Coordinalo con el usuario antes de tocarlas.",
        sin_aislar.len(),
        sin_aislar.join(", ")
    ))
}

#[cfg(test)]
mod tests_72 {
    #![allow(clippy::unwrap_used)]
    use super::*;
    use serde_json::json;

    fn features(v: serde_json::Value) -> Vec<Value> {
        v.as_array().unwrap().clone()
    }

    /// AC-10: el inventario nombra a las que quedaron sin aislar y a nadie mas.
    #[test]
    fn el_preflight_nombra_solo_las_abiertas_sin_worktree() {
        let f = features(json!([
            {"id": 98, "name": "Sin aislar", "status": "in_progress"},
            {"id": 122, "name": "Tampoco", "status": "in_progress", "worktree": "   "},
            {"id": 121, "name": "Aislada", "status": "in_progress", "worktree": "/tmp/wt/121"},
            {"id": 1, "name": "Cerrada", "status": "done"}
        ]));
        let a = aviso_de_features_sin_aislar(&f).unwrap();
        assert!(a.contains("#98 Sin aislar"), "{a}");
        assert!(a.contains("#122 Tampoco"), "un worktree en blanco no aisla: {a}");
        assert!(!a.contains("#121"), "la aislada no entra: {a}");
        assert!(!a.contains("Cerrada"), "las cerradas no entran: {a}");
        assert!(a.contains("2 feature(s)"), "{a}");
        // Y dice lo que NO va a hacer, que es la mitad del AC-10.
        assert!(a.contains("no mueve commits"), "{a}");
        assert!(a.contains("Coordinalo con el usuario"), "{a}");
    }

    #[test]
    fn el_preflight_calla_cuando_todo_esta_aislado() {
        let f = features(json!([
            {"id": 1, "name": "A", "status": "in_progress", "worktree": "/tmp/wt/1"},
            {"id": 2, "name": "B", "status": "pending"}
        ]));
        assert_eq!(aviso_de_features_sin_aislar(&f), None);
    }
}
