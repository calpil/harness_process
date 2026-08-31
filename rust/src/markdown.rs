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
}
