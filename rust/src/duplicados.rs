//! Feature #74: `add` no carga dos veces la misma feature.
//!
//! Medido antes de escribir esto (2026-09-07, 83 features reales): cero nombres
//! repetidos y cero pares parecidos. La defensa es para el ESCENARIO que no deja
//! huella en el backlog —dos sesiones que descubren el mismo problema, un `add`
//! re-corrido tras un corte, un script que carga hitos y se relanza—, no para
//! un problema que ya ocurrio. Por eso es chica: nombre normalizado identico,
//! y una clave opaca de idempotencia. Sin similitud parcial (OBS-2: un umbral
//! sin evidencia es un numero inventado).
//!
//! Modulo puro: no lee archivos, no escribe, no imprime.

use serde_json::Value;

use crate::pycompat::py_str;

/// Palabras que no distinguen una feature de otra. La lista es corta a
/// proposito: agregarle palabras cambia que cuenta como "el mismo nombre", asi
/// que es una decision, no un ajuste.
const VACIAS: [&str; 21] = [
    "el", "la", "los", "las", "un", "una", "unos", "unas", "de", "del", "al", "a", "y", "o",
    "que", "en", "con", "por", "para", "no", "se",
];

/// Estados en los que la feature sigue siendo LA feature: cargar otra igual es
/// duplicarla (`blocked` incluida: una feature bloqueada no desaparece, y
/// cargar otra igual es esconder el bloqueo). Todo lo demas cuenta como
/// cerrada.
pub const ABIERTAS: [&str; 3] = ["pending", "in_progress", "blocked"];

/// Nombre normalizado: minusculas, sin acentos, solo letras y digitos ASCII,
/// sin palabras vacias, palabras separadas por un espacio.
///
/// `"El Instalador, respalda el BACKLOG"` y `"instalador respalda backlog"`
/// normalizan igual; `"add no protege"` y `"close no protege"` no.
pub fn normalizar(nombre: &str) -> String {
    let plano: String = nombre.to_lowercase().chars().map(sin_acento).collect();
    plano
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|p| !p.is_empty() && !VACIAS.contains(p))
        .collect::<Vec<_>>()
        .join(" ")
}

fn sin_acento(c: char) -> char {
    match c {
        'á' | 'à' | 'ä' | 'â' => 'a',
        'é' | 'è' | 'ë' | 'ê' => 'e',
        'í' | 'ì' | 'ï' | 'î' => 'i',
        'ó' | 'ò' | 'ö' | 'ô' => 'o',
        'ú' | 'ù' | 'ü' | 'û' => 'u',
        'ñ' => 'n',
        'ç' => 'c',
        otro => otro,
    }
}

/// Que encontro `add` al buscar su nombre en el backlog.
///
/// Es un enum y no un `Option<&Value>` porque cada caso decide distinto: una
/// abierta RECHAZA el alta, una cerrada solo AVISA (una regresion es legitima),
/// y ninguna deja pasar. Una abierta gana sobre cualquier cerrada.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Coincidencia {
    Abierta {
        id: String,
        status: String,
        nombre: String,
    },
    Cerrada {
        id: String,
        status: String,
        fecha: String,
    },
    Ninguna,
}

/// Busca `nombre` (normalizado) entre `features`.
pub fn buscar(features: &[Value], nombre: &str) -> Coincidencia {
    let buscado = normalizar(nombre);
    if buscado.is_empty() {
        return Coincidencia::Ninguna;
    }
    let iguales = || {
        features
            .iter()
            .filter(|f| normalizar(&py_str(f.get("name"))) == buscado)
    };
    if let Some(f) = iguales().find(|f| ABIERTAS.contains(&py_str(f.get("status")).as_str())) {
        return Coincidencia::Abierta {
            id: py_str(f.get("id")),
            status: py_str(f.get("status")),
            nombre: py_str(f.get("name")),
        };
    }
    match iguales().next() {
        Some(f) => Coincidencia::Cerrada {
            id: py_str(f.get("id")),
            status: py_str(f.get("status")),
            // `closed_at` es un timestamp ISO; la fecha alcanza para citarla.
            fecha: py_str(f.get("closed_at")).chars().take(10).collect(),
        },
        None => Coincidencia::Ninguna,
    }
}

/// Id de la feature que ya lleva esta `clave` (idempotencia, como el
/// idempotency-key de Hermes en `kanban create`). La clave es opaca.
pub fn por_clave(features: &[Value], clave: &str) -> Option<String> {
    features
        .iter()
        .find(|f| f.get("clave").and_then(Value::as_str) == Some(clave))
        .map(|f| py_str(f.get("id")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn feature(id: u64, name: &str, status: &str) -> Value {
        json!({"id": id, "name": name, "status": status, "closed_at": "2026-09-05T10:00:00Z"})
    }

    #[test]
    fn normalizar_nombre_should_ignore_case_accents_punctuation_and_empty_words() {
        assert_eq!(
            normalizar("  El Instalador, respalda  él BACKLOG!! "),
            normalizar("instalador respalda backlog")
        );
    }

    #[test]
    fn normalizar_nombre_should_keep_names_that_differ_in_one_word_apart() {
        assert_ne!(normalizar("add no protege"), normalizar("close no protege"));
    }

    #[test]
    fn normalizar_nombre_should_return_empty_for_only_empty_words() {
        assert_eq!(normalizar("el de la y"), "");
    }

    #[test]
    fn buscar_should_prefer_an_open_feature_over_a_closed_one_with_the_same_name() {
        let fs = [feature(1, "Guard sin trailers", "done"), feature(2, "guard sin trailers", "blocked")];
        assert_eq!(
            buscar(&fs, "GUARD sin trailers"),
            Coincidencia::Abierta {
                id: "2".into(),
                status: "blocked".into(),
                nombre: "guard sin trailers".into()
            }
        );
    }

    #[test]
    fn buscar_should_report_the_closed_feature_with_its_date_when_none_is_open() {
        let fs = [feature(7, "El checkout tiene capacidad uno", "done")];
        assert_eq!(
            buscar(&fs, "checkout tiene capacidad uno"),
            Coincidencia::Cerrada {
                id: "7".into(),
                status: "done".into(),
                fecha: "2026-09-05".into()
            }
        );
    }

    #[test]
    fn buscar_should_return_ninguna_for_a_new_name_or_an_empty_one() {
        let fs = [feature(1, "algo", "pending")];
        assert_eq!(buscar(&fs, "otra cosa"), Coincidencia::Ninguna);
        assert_eq!(buscar(&fs, "el de"), Coincidencia::Ninguna);
    }

    #[test]
    fn por_clave_should_find_the_feature_that_carries_the_key() {
        let mut con = feature(3, "hito", "pending");
        con["clave"] = json!("prd/mora/3");
        let fs = [feature(1, "otra", "pending"), con];
        assert_eq!(por_clave(&fs, "prd/mora/3"), Some("3".into()));
        assert_eq!(por_clave(&fs, "prd/mora/4"), None);
    }
}
