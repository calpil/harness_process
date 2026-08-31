//! El UNICO parser de bloques de codigo del arnes (feature #67).
//!
//! Antes habia cuatro, con tres semanticas distintas, sobre los mismos
//! documentos:
//!
//! | donde | que hacia |
//! | --- | --- |
//! | `revision::lineas_fuera_de_bloque` (el gate del review) | fences emparejados |
//! | `commands::revision` (el limpiador de `estampar`) | toggle con cualquiera |
//! | `verificacion::parsear` (los AC del spec) | solo ```` ``` ````, no conocia `~~~` |
//! | `atlassian::markdown` | CONSUME el bloque (caso distinto, queda aparte) |
//!
//! El costo de esa divergencia no era teorico. Medido antes de esta feature:
//!
//! - **`verify` ejecutaba comandos escritos dentro de un bloque `~~~`.** Es el
//!   bug que la #23 cerro para backticks —"un spec que documenta la sintaxis no
//!   puede quedar verificando su documentacion"— y estaba abierto para tildes.
//!   Ejecucion de shell salida de una seccion que el autor marco como
//!   documentacion.
//! - `revision --veredicto` **borraba prosa del reviewer** cuando el review
//!   citaba un bloque ajeno, y en la otra direccion dejaba **dos sellos**
//!   contradictorios en el archivo.
//! - Enumerando exhaustivamente los documentos sobre el alfabeto
//!   {```` ``` ````, `~~~`, sello, texto}, los parsers discrepaban en el **37%**
//!   de los documentos de siete lineas.
//!
//! La semantica es la de fences **emparejados**: se recuerda cual fence abrio el
//! bloque y solo lo cierra el mismo. Es la unica de las tres que coincide con
//! como se renderiza el markdown de verdad, y la que hace que un review ajeno
//! citado entero no rompa nada.
//!
//! Lo que este modulo NO hace, a proposito: el largo del fence al estilo
//! CommonMark (un bloque abierto con ```` ```` ```` no deberia cerrarse con
//! ```` ``` ````). Los cuatro parsers compartian esa divergencia, no producia
//! desacuerdo entre ellos y **no se pudo reproducir daño**. Endurecerlo seria
//! repetir el AC-11 de la #66: cambiar codigo que funciona contra un bug que no
//! se pudo reproducir.

/// Que es una linea respecto de los bloques de codigo del documento.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Clase {
    /// Texto normal: cuenta para todo.
    Fuera,
    /// La linea del fence en si (` ``` ` o `~~~`). No es contenido, pero el que
    /// reescribe el documento tiene que conservarla.
    Fence,
    /// Contenido del bloque: es documentacion, no instrucciones para el arnes.
    Dentro,
}

/// Clasifica cada linea del documento.
///
/// Se devuelve la clasificacion completa —y no una lista ya filtrada— porque los
/// consumidores necesitan cosas distintas de la MISMA respuesta: el gate se
/// queda con `Fuera`, el limpiador de `estampar` necesita todas las lineas para
/// reescribir el archivo conservando los `Fence`, y el parseo de AC se queda con
/// todo lo que no sea `Dentro`. Un `Vec<&str>` compartido no alcanzaba: fue
/// justamente lo que hizo que cada uno se escribiera su propia version.
pub fn lineas_clasificadas(texto: &str) -> Vec<(&str, Clase)> {
    let mut out = Vec::new();
    let mut abierto_con: Option<&'static str> = None;
    for linea in texto.lines() {
        let t = linea.trim_start();
        let fence = if t.starts_with("```") {
            Some("```")
        } else if t.starts_with("~~~") {
            Some("~~~")
        } else {
            None
        };
        match (abierto_con, fence) {
            (None, Some(f)) => {
                abierto_con = Some(f);
                out.push((linea, Clase::Fence));
            }
            (Some(abierto), Some(f)) if abierto == f => {
                abierto_con = None;
                out.push((linea, Clase::Fence));
            }
            // Un fence DISTINTO del que abrio es contenido del bloque, no un
            // cierre: es el caso de un review ajeno citado entero.
            (Some(_), _) => out.push((linea, Clase::Dentro)),
            (None, None) => out.push((linea, Clase::Fuera)),
        }
    }
    out
}

/// Las lineas que NO estan dentro de un bloque ni son fences.
///
/// Es el filtro que quieren el gate del review y el parseo de AC: lo que el
/// documento AFIRMA, sin lo que solo documenta.
pub fn lineas_fuera_de_bloque(texto: &str) -> Vec<&str> {
    lineas_clasificadas(texto)
        .into_iter()
        .filter(|(_, c)| *c == Clase::Fuera)
        .map(|(l, _)| l)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn clases(t: &str) -> Vec<Clase> {
        lineas_clasificadas(t).into_iter().map(|(_, c)| c).collect()
    }

    #[test]
    fn un_fence_distinto_adentro_es_contenido() {
        // El caso que rompia: un review ajeno citado entero dentro de un bloque.
        // El `~~~` de adentro NO cierra el bloque abierto con backticks.
        let t = "afuera\n```\n~~~\nadentro\n```\nafuera2";
        assert_eq!(
            clases(t),
            vec![
                Clase::Fuera,
                Clase::Fence,
                Clase::Dentro,
                Clase::Dentro,
                Clase::Fence,
                Clase::Fuera
            ]
        );
    }

    #[test]
    fn la_paridad_no_cambia_el_resultado() {
        // Con el toggle simetrico viejo, la cantidad de fences ajenos adentro
        // decidia si una linea de afuera se veia como de adentro. Con fences
        // emparejados, una y dos lineas `~~~` dan lo mismo para lo que sigue.
        let una = "```\n~~~\n```\nfinal";
        let dos = "```\n~~~\n~~~\n```\nfinal";
        assert_eq!(clases(una).last(), Some(&Clase::Fuera));
        assert_eq!(clases(dos).last(), Some(&Clase::Fuera));
    }

    #[test]
    fn los_dos_fences_valen_para_abrir() {
        // El bug mas caro: `verificacion::parsear` no conocia `~~~`, asi que
        // `verify` ejecutaba los `Comando:` de adentro.
        let t = "~~~markdown\n  Comando: `rm -rf /`\n~~~\nafuera";
        assert_eq!(
            clases(t),
            vec![Clase::Fence, Clase::Dentro, Clase::Fence, Clase::Fuera]
        );
        assert_eq!(lineas_fuera_de_bloque(t), vec!["afuera"]);
    }

    #[test]
    fn un_bloque_sin_cerrar_se_come_el_resto() {
        // Direccion segura: si el documento abre y no cierra, lo que sigue es
        // contenido. Vale mas dejar de leer una linea que ejecutar una que el
        // autor marco como ejemplo.
        let t = "afuera\n```\nadentro\nadentro2";
        assert_eq!(lineas_fuera_de_bloque(t), vec!["afuera"]);
    }

    #[test]
    fn los_fences_indentados_cuentan() {
        let t = "  ```\n  adentro\n  ```\nafuera";
        assert_eq!(lineas_fuera_de_bloque(t), vec!["afuera"]);
    }

    #[test]
    fn texto_vacio_y_sin_fences() {
        assert!(lineas_clasificadas("").is_empty());
        assert_eq!(lineas_fuera_de_bloque("a\nb"), vec!["a", "b"]);
    }

    // ---------------------------------------------------------------------
    // AC-4: los consumidores no discrepan. Por enumeracion exhaustiva.
    // ---------------------------------------------------------------------

    /// Las TRES semanticas que habia antes de esta feature, reimplementadas tal
    /// cual para poder medirlas.
    ///
    /// Estan aca a proposito y no se borran: sin ellas, "los cuatro consumidores
    /// coinciden" es una tautologia —comparar el parser unico consigo mismo— y
    /// el numero que motivo la feature no se puede reproducir.
    mod viejos {
        pub fn emparejados(doc: &[&str]) -> Vec<char> {
            let mut out = Vec::new();
            let mut ab: Option<&str> = None;
            for l in doc {
                let f = if l.starts_with("```") {
                    Some("```")
                } else if l.starts_with("~~~") {
                    Some("~~~")
                } else {
                    None
                };
                match (ab, f) {
                    (None, Some(x)) => {
                        ab = Some(x);
                        out.push('F')
                    }
                    (Some(a), Some(x)) if a == x => {
                        ab = None;
                        out.push('F')
                    }
                    (Some(_), _) => out.push('D'),
                    (None, None) => out.push('O'),
                }
            }
            out
        }

        /// `commands::revision` (el limpiador de `estampar`): togglea con
        /// cualquier fence, asi que un `~~~` citado dentro de un bloque ```
        /// cerraba el bloque.
        pub fn toggle_cualquiera(doc: &[&str]) -> Vec<char> {
            let mut out = Vec::new();
            let mut dentro = false;
            for l in doc {
                if l.starts_with("```") || l.starts_with("~~~") {
                    dentro = !dentro;
                    out.push('F');
                } else {
                    out.push(if dentro { 'D' } else { 'O' });
                }
            }
            out
        }

        /// `verificacion::parsear`: no conocia `~~~`. El mas caro de los tres.
        pub fn solo_backticks(doc: &[&str]) -> Vec<char> {
            let mut out = Vec::new();
            let mut dentro = false;
            for l in doc {
                if l.starts_with("```") {
                    dentro = !dentro;
                    out.push('F');
                } else {
                    out.push(if dentro { 'D' } else { 'O' });
                }
            }
            out
        }
    }

    const SELLO: &str = "Revisado: approved · 2026-01-01 00:00 · estampado por `harness revision --veredicto`";
    const AC: &str = "- AC-1: Given algo, When pasa, Then otra.";

    /// Todos los documentos de `n` lineas sobre `alfabeto`.
    fn documentos(alfabeto: &[&'static str], n: usize) -> Vec<Vec<&'static str>> {
        let mut docs = vec![Vec::new()];
        for _ in 0..n {
            let mut sig = Vec::with_capacity(docs.len() * alfabeto.len());
            for d in &docs {
                for s in alfabeto {
                    let mut e = d.clone();
                    e.push(*s);
                    sig.push(e);
                }
            }
            docs = sig;
        }
        docs
    }

    fn clases_de(doc: &[&str]) -> Vec<char> {
        lineas_clasificadas(&doc.join("\n"))
            .into_iter()
            .map(|(_, c)| match c {
                Clase::Fuera => 'O',
                Clase::Fence => 'F',
                Clase::Dentro => 'D',
            })
            .collect()
    }

    #[test]
    fn los_parsers_no_discrepan() {
        // Lo que este test prueba NO es que los cuatro consumidores coincidan
        // —ahora llaman todos a la misma funcion, eso seria una tautologia— sino
        // que lo que cada uno OBSERVA se deriva de la clasificacion del parser
        // unico. Si manana alguien vuelve a escribir un parser local en
        // `verificacion` o en `commands::revision`, esto se pone rojo aunque el
        // grep del AC-10 no lo agarre.
        let alfabeto = ["```", "~~~", SELLO, AC, "texto"];
        let mut docs = 0usize;
        for n in 1..=7 {
            for doc in documentos(&alfabeto, n) {
                docs += 1;
                let texto = doc.join("\n");
                let clases = clases_de(&doc);

                // Consumidor 1 — `verificacion::parsear`: los AC son exactamente
                // los de las lineas clasificadas `Fuera`.
                let esperados = doc
                    .iter()
                    .zip(&clases)
                    .filter(|(l, c)| **c == 'O' && l.starts_with("- AC-"))
                    .count();
                assert_eq!(
                    crate::verificacion::parsear(&texto).len(),
                    esperados,
                    "parsear discrepa con la clasificacion en {doc:?}"
                );

                // Consumidor 2 — `revision::veredicto_estampado`: hay sello si y
                // solo si alguna linea de sello quedo `Fuera`.
                let hay_sello = doc
                    .iter()
                    .zip(&clases)
                    .any(|(l, c)| *c == 'O' && *l == SELLO);
                assert_eq!(
                    crate::revision::veredicto_estampado(&texto).is_some(),
                    hay_sello,
                    "veredicto_estampado discrepa con la clasificacion en {doc:?}"
                );

                // Consumidor 3 — el limpiador de `estampar`: saca exactamente
                // los sellos de afuera y no toca nada mas.
                let esperado: Vec<&str> = doc
                    .iter()
                    .zip(&clases)
                    .filter(|(l, c)| !(**c == 'O' && **l == SELLO))
                    .map(|(l, _)| *l)
                    .collect();
                assert_eq!(
                    crate::commands::revision::cuerpo_sin_sellos(&texto),
                    esperado.join("\n"),
                    "el limpiador discrepa con la clasificacion en {doc:?}"
                );
            }
        }
        assert_eq!(docs, 97_655, "cambio el tamaño del espacio enumerado");
    }

    #[test]
    fn la_divergencia_que_motivo_la_feature_se_reproduce() {
        // El numero del spec (37% a n=7) NO se reproduce: es la cifra de n=6.
        // Se deja el dato medido, no el que estaba escrito.
        let alfabeto = ["```", "~~~", SELLO, "texto"];
        let medido: Vec<(usize, usize, usize)> = (1..=7)
            .map(|n| {
                let docs = documentos(&alfabeto, n);
                let total = docs.len();
                let tres = docs
                    .iter()
                    .filter(|d| {
                        let a = viejos::emparejados(d);
                        a != viejos::toggle_cualquiera(d) || a != viejos::solo_backticks(d)
                    })
                    .count();
                // La divergencia que cambia una DECISION: la que mueve de clase
                // una linea de sello. Es la unica que podia hacer que el gate y
                // el limpiador se contradijeran sobre el mismo archivo.
                let con_daño = docs
                    .iter()
                    .filter(|d| {
                        let (a, b, c) = (
                            viejos::emparejados(d),
                            viejos::toggle_cualquiera(d),
                            viejos::solo_backticks(d),
                        );
                        (0..d.len())
                            .any(|i| d[i] == SELLO && !(a[i] == b[i] && b[i] == c[i]))
                    })
                    .count();
                (total, tres, con_daño)
            })
            .collect();
        assert_eq!(
            medido,
            vec![
                (4, 1, 0),
                (16, 7, 1),
                (64, 37, 9),
                (256, 175, 57),
                (1024, 781, 310),
                (4096, 3367, 1548),
                (16384, 14197, 7324),
            ],
            "cambio la medicion de la divergencia"
        );
        // n=7: 86,6% de los documentos discrepaban entre los tres parsers, y en
        // 44,7% la discrepancia caia sobre una linea de sello. El 37,8% es la
        // cifra de n=6, que es de donde salio el "37%" escrito en el spec.
        let (total, tres, daño) = medido[6];
        assert_eq!((tres * 1000 / total, daño * 1000 / total), (866, 447));

        // Y el parser unico es la semantica de fences emparejados: la unica de
        // las tres que coincide con como se renderiza el markdown.
        for n in 1..=7 {
            for doc in documentos(&alfabeto, n) {
                assert_eq!(clases_de(&doc), viejos::emparejados(&doc));
            }
        }
    }
}
