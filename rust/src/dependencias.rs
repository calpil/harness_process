//! Feature #75: que feature espera a cual, y que feature se traba siempre.
//!
//! Dos preguntas distintas que comparten archivo porque comparten el dato
//! —el backlog— y nada mas. Las dos se responden con funciones PURAS: reciben
//! el backlog ya cargado y no tocan disco. Esa es la unica forma de probarlas
//! contra el `feature_list.json` REAL, que es lo que el AC-6 exige.
//!
//! Cuidado que ya costo una vez en este repo: `depends_on` **tambien** existe
//! en `graph/derive.rs`, como una de las relaciones del grafo del hub. Son dos
//! cosas distintas —aquella habla de codigo, esta de features— y reusar una por
//! la otra seria la clase de divergencia que las features #64, #67 y #69 ya
//! pagaron.

use serde_json::Value;

use crate::features::{feature_status, features_slice};
use crate::pycompat::py_str;

/// Los estados en los que una dependencia se considera SATISFECHA.
///
/// `blocked` y `pending` no estan: una feature trabada no habilita a la que la
/// espera. `superseded` y `resuelto-aguas-arriba` si, porque el trabajo existe
/// —en otra feature o en otro repo— y esperar a algo que ya no va a cerrar
/// nunca dejaria a la que depende colgada para siempre.
pub const ESTADOS_QUE_SATISFACEN: [&str; 3] = [
    "done",
    crate::commands::close::SUPERSEDED,
    crate::commands::close::AGUAS_ARRIBA,
];

/// Los ids de los que depende una feature. Vacio si no declara nada.
pub fn declaradas(feature: &Value) -> Vec<String> {
    feature
        .get("depends_on")
        .and_then(Value::as_array)
        .map(|a| a.iter().map(|v| py_str(Some(v))).collect())
        .unwrap_or_default()
}

/// Por que no se puede escribir este `depends_on`, o `None` si se puede.
///
/// PURA. Comprueba, en este orden: que los ids existan, que no se dependa de si
/// misma, y que no se forme un ciclo. El orden importa para el mensaje: decir
/// "ciclo" sobre un id que ni existe seria confundir dos problemas.
pub fn motivo_invalido(data: &Value, fid: &str, nuevas: &[String]) -> Option<String> {
    let existentes: Vec<String> = features_slice(data)
        .iter()
        .map(|f| py_str(f.get("id")))
        .collect();

    let inexistentes: Vec<&String> = nuevas.iter().filter(|d| !existentes.contains(d)).collect();
    if !inexistentes.is_empty() {
        let ids: Vec<String> = inexistentes.iter().map(|d| format!("#{d}")).collect();
        return Some(format!(
            "no existe(n) en el backlog: {}.\n    \
             Una dependencia a un id inventado no se guarda: el backlog quedaria afirmando\n    \
             una espera que nadie puede resolver.",
            ids.join(", ")
        ));
    }
    if nuevas.iter().any(|d| d == fid) {
        return Some(format!("la feature #{fid} no puede depender de si misma."));
    }
    if let Some(ciclo) = ciclo_que_formaria(data, fid, nuevas) {
        let camino: Vec<String> = ciclo.iter().map(|d| format!("#{d}")).collect();
        return Some(format!(
            "formaria un ciclo: {}.\n    \
             Un ciclo deja a las dos features esperandose para siempre, y `next` no\n    \
             ofreceria ninguna de las dos.",
            camino.join(" -> ")
        ));
    }
    None
}

/// El ciclo que se formaria al darle a `fid` estas dependencias, si lo hay.
///
/// Devuelve el camino completo (`fid -> a -> b -> fid`) porque un mensaje que
/// dice "hay un ciclo" sin decir cual obliga a buscarlo a mano.
fn ciclo_que_formaria(data: &Value, fid: &str, nuevas: &[String]) -> Option<Vec<String>> {
    // Recorrido en profundidad desde cada dependencia nueva, buscando volver a
    // `fid`. Se usan las dependencias YA guardadas del resto: son las unicas
    // que existen todavia.
    fn depende_de(data: &Value, de: &str) -> Vec<String> {
        features_slice(data)
            .iter()
            .find(|f| py_str(f.get("id")) == de)
            .map(declaradas)
            .unwrap_or_default()
    }
    fn buscar(
        data: &Value,
        actual: &str,
        objetivo: &str,
        camino: &mut Vec<String>,
        vistos: &mut Vec<String>,
    ) -> bool {
        if actual == objetivo {
            camino.push(actual.to_string());
            return true;
        }
        if vistos.contains(&actual.to_string()) {
            return false;
        }
        vistos.push(actual.to_string());
        camino.push(actual.to_string());
        for siguiente in depende_de(data, actual) {
            if buscar(data, &siguiente, objetivo, camino, vistos) {
                return true;
            }
        }
        camino.pop();
        false
    }

    for d in nuevas {
        let mut camino = vec![fid.to_string()];
        let mut vistos = Vec::new();
        if buscar(data, d, fid, &mut camino, &mut vistos) {
            return Some(camino);
        }
    }
    None
}

/// Una dependencia que todavia no esta satisfecha.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Abierta {
    pub id: String,
    pub nombre: String,
    pub estado: String,
}

impl Abierta {
    pub fn etiqueta(&self) -> String {
        format!("#{} {} ({})", self.id, self.nombre, self.estado)
    }
}

/// Las dependencias de una feature que NO estan satisfechas todavia.
///
/// PURA. Una dependencia a un id que ya no esta en el backlog se reporta como
/// abierta con estado `ausente`: desaparecer no es lo mismo que estar hecha, y
/// callarlo convertiria un backlog roto en uno que parece listo.
pub fn abiertas(data: &Value, feature: &Value) -> Vec<Abierta> {
    declaradas(feature)
        .into_iter()
        .filter_map(|id| {
            let Some(dep) = features_slice(data)
                .iter()
                .find(|f| py_str(f.get("id")) == id)
            else {
                return Some(Abierta {
                    id,
                    nombre: "(no esta en el backlog)".to_string(),
                    estado: "ausente".to_string(),
                });
            };
            let estado = feature_status(dep).unwrap_or("");
            if ESTADOS_QUE_SATISFACEN.contains(&estado) {
                return None;
            }
            Some(Abierta {
                id,
                nombre: py_str(dep.get("name")),
                estado: estado.to_string(),
            })
        })
        .collect()
}

// ---------------------------------------------------------------------------
// El contador de bloqueos (AC-4).
//
// Su condicion NUNCA se disparo: en 84 cierres reales, cero features se
// cerraron `blocked` mas de una vez, y los seis `blocked` del historial son una
// reclasificacion en bloque de 52 segundos. Se implementa por decision del
// usuario (2026-09-05) con esa medicion a la vista, y por eso sus dos tests son
// toda su evidencia: uno que lo dispara y otro que comprueba que no se dispara
// antes de tiempo.
// ---------------------------------------------------------------------------

/// Umbral de cierres `blocked` a partir del cual el arnes exige explicacion.
pub fn bloqueos_antes_de_decidir(data: &Value) -> u64 {
    data.get("rules")
        .and_then(|r| r.get("bloqueos_antes_de_decidir"))
        .and_then(Value::as_u64)
        .unwrap_or(2)
}

/// Cuantas veces se cerro `blocked` esta feature antes de ahora.
pub fn bloqueos_previos(feature: &Value) -> u64 {
    feature.get("bloqueos").and_then(Value::as_u64).unwrap_or(0)
}

/// Por que este cierre `blocked` necesita una nota, o `None` si no la necesita.
///
/// PURA: recibe cuantas veces se trabo y que nota trae, no las averigua.
pub fn motivo_exige_nota(previos: u64, umbral: u64, nota: &str) -> Option<String> {
    if previos < umbral || !nota.trim().is_empty() {
        return None;
    }
    let van = previos + 1;
    Some(format!(
        "esta feature se cierra como `blocked` por {van}a vez y el cierre no dice por que.\n    \
         El umbral es {umbral} (`rules.bloqueos_antes_de_decidir`).\n    \
         Una feature que se traba siempre por la misma causa y nadie la nombra se queda\n    \
         trabada para siempre. Deci si la causa es la misma o cambio:\n      \
         sh harness_cli close --feature <id> --status blocked --note \"<la causa>\""
    ))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;
    use serde_json::json;

    fn backlog(v: serde_json::Value) -> Value {
        json!({ "features": v })
    }

    fn feature_de(data: &Value, id: &str) -> Value {
        features_slice(data)
            .iter()
            .find(|f| py_str(f.get("id")) == id)
            .unwrap()
            .clone()
    }

    // --- AC-1: validacion ---------------------------------------------------

    #[test]
    fn una_dependencia_a_un_id_inexistente_no_se_guarda() {
        let d = backlog(json!([{"id": 1, "name": "A", "status": "pending"}]));
        let m = motivo_invalido(&d, "1", &["99".to_string()]).unwrap();
        assert!(m.contains("#99"), "nombra el id: {m}");
        assert!(m.contains("no existe"), "{m}");
    }

    #[test]
    fn una_dependencia_valida_se_acepta() {
        let d = backlog(json!([
            {"id": 1, "name": "A", "status": "done"},
            {"id": 2, "name": "B", "status": "pending"}
        ]));
        assert_eq!(motivo_invalido(&d, "2", &["1".to_string()]), None);
    }

    #[test]
    fn una_feature_no_puede_depender_de_si_misma() {
        let d = backlog(json!([{"id": 1, "name": "A", "status": "pending"}]));
        let m = motivo_invalido(&d, "1", &["1".to_string()]).unwrap();
        assert!(m.contains("de si misma"), "{m}");
    }

    // --- AC-5: ciclos -------------------------------------------------------

    #[test]
    fn un_ciclo_directo_se_rechaza_nombrando_el_camino() {
        // #1 ya depende de #2; darle a #2 dependencia de #1 cierra el ciclo.
        let d = backlog(json!([
            {"id": 1, "name": "A", "status": "pending", "depends_on": ["2"]},
            {"id": 2, "name": "B", "status": "pending"}
        ]));
        let m = motivo_invalido(&d, "2", &["1".to_string()]).unwrap();
        assert!(m.contains("ciclo"), "{m}");
        assert!(m.contains("#2 -> #1 -> #2"), "nombra el camino: {m}");
    }

    #[test]
    fn un_ciclo_transitivo_tambien_se_rechaza() {
        // #1 -> #2 -> #3, y ahora #3 -> #1.
        let d = backlog(json!([
            {"id": 1, "name": "A", "status": "pending", "depends_on": ["2"]},
            {"id": 2, "name": "B", "status": "pending", "depends_on": ["3"]},
            {"id": 3, "name": "C", "status": "pending"}
        ]));
        let m = motivo_invalido(&d, "3", &["1".to_string()]).unwrap();
        assert!(m.contains("#3 -> #1 -> #2 -> #3"), "el camino entero: {m}");
    }

    #[test]
    fn una_cadena_larga_sin_ciclo_no_se_confunde_con_uno() {
        // #1 -> #2 -> #3; darle a #4 dependencia de #1 NO cierra nada.
        let d = backlog(json!([
            {"id": 1, "name": "A", "status": "pending", "depends_on": ["2"]},
            {"id": 2, "name": "B", "status": "pending", "depends_on": ["3"]},
            {"id": 3, "name": "C", "status": "pending"},
            {"id": 4, "name": "D", "status": "pending"}
        ]));
        assert_eq!(motivo_invalido(&d, "4", &["1".to_string()]), None);
    }

    // --- AC-2: que dependencias quedan abiertas -----------------------------

    #[test]
    fn una_dependencia_cerrada_deja_de_estar_abierta() {
        let d = backlog(json!([
            {"id": 1, "name": "Base", "status": "done"},
            {"id": 2, "name": "Encima", "status": "pending", "depends_on": ["1"]}
        ]));
        assert!(abiertas(&d, &feature_de(&d, "2")).is_empty());
    }

    #[test]
    fn una_dependencia_pending_o_blocked_sigue_abierta() {
        for estado in ["pending", "in_progress", "blocked"] {
            let d = backlog(json!([
                {"id": 1, "name": "Base", "status": estado},
                {"id": 2, "name": "Encima", "status": "pending", "depends_on": ["1"]}
            ]));
            let ab = abiertas(&d, &feature_de(&d, "2"));
            assert_eq!(ab.len(), 1, "{estado} tendria que dejarla abierta");
            assert_eq!(ab[0].etiqueta(), format!("#1 Base ({estado})"));
        }
    }

    /// `superseded` y `resuelto-aguas-arriba` SATISFACEN: el trabajo existe en
    /// otro lado. Esperar a algo que no va a cerrar nunca dejaria a la que
    /// depende colgada para siempre.
    #[test]
    fn superseded_y_aguas_arriba_satisfacen_la_dependencia() {
        for estado in ["superseded", "resuelto-aguas-arriba"] {
            let d = backlog(json!([
                {"id": 1, "name": "Base", "status": estado},
                {"id": 2, "name": "Encima", "status": "pending", "depends_on": ["1"]}
            ]));
            assert!(
                abiertas(&d, &feature_de(&d, "2")).is_empty(),
                "{estado} tendria que satisfacer"
            );
        }
    }

    /// Una dependencia que ya no esta en el backlog NO cuenta como hecha.
    #[test]
    fn una_dependencia_ausente_se_reporta_abierta_y_no_satisfecha() {
        let d = backlog(json!([
            {"id": 2, "name": "Encima", "status": "pending", "depends_on": ["99"]}
        ]));
        let ab = abiertas(&d, &feature_de(&d, "2"));
        assert_eq!(ab.len(), 1);
        assert_eq!(ab[0].estado, "ausente");
    }

    #[test]
    fn sin_depends_on_no_hay_nada_abierto() {
        // AC-6: el campo es opcional y su ausencia no cambia nada.
        let d = backlog(json!([{"id": 1, "name": "Sola", "status": "pending"}]));
        assert!(declaradas(&feature_de(&d, "1")).is_empty());
        assert!(abiertas(&d, &feature_de(&d, "1")).is_empty());
    }

    // --- AC-4: el contador de bloqueos --------------------------------------

    #[test]
    fn el_umbral_default_es_dos() {
        assert_eq!(bloqueos_antes_de_decidir(&json!({})), 2);
        assert_eq!(
            bloqueos_antes_de_decidir(&json!({"rules": {"bloqueos_antes_de_decidir": 5}})),
            5
        );
    }

    /// El gate NO se dispara antes de tiempo: con el umbral en 2, el primer y
    /// el segundo cierre `blocked` pasan sin nota.
    #[test]
    fn debajo_del_umbral_no_se_exige_nota() {
        assert_eq!(motivo_exige_nota(0, 2, ""), None, "primer blocked");
        assert_eq!(motivo_exige_nota(1, 2, ""), None, "segundo blocked");
    }

    /// Y se dispara cuando corresponde. Es la unica evidencia de que este gate
    /// funciona: su condicion no se observo nunca en el backlog real.
    #[test]
    fn en_el_umbral_se_exige_nota_y_la_nota_la_satisface() {
        let m = motivo_exige_nota(2, 2, "").unwrap();
        assert!(m.contains("por 3a vez"), "{m}");
        assert!(m.contains("bloqueos_antes_de_decidir"), "nombra la regla: {m}");
        assert!(m.contains("--note"), "dice como resolverlo: {m}");
        // Con nota, no molesta.
        assert_eq!(motivo_exige_nota(2, 2, "la misma causa: falta el token"), None);
        // Y una nota en blanco no cuenta como nota.
        assert!(motivo_exige_nota(2, 2, "   ").is_some());
    }
}
