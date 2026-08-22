//! `buscar`: hacer consultable la memoria del proyecto (feature #20).
//!
//! Las features #17-#19 le dieron memoria al arnes (lecciones, nudge, perfil).
//! Esto la hace **preguntable**: una memoria que no se puede consultar no es
//! memoria.
//!
//! Tres decisiones que el codigo tiene que respetar:
//!
//! - **Sin indice.** El corpus de un proyecto son ~1 MB de texto; escanearlo
//!   entero es del orden de milisegundos y un indice desactualizado MIENTE, que
//!   es peor que escanear (AC-12).
//! - **Solo lectura.** No escribe un byte en ningun lado.
//! - **Sin hub y sin modelo.** Decision del usuario 2026-08-17 (OBS-1): el hub
//!   guarda eventos, no la prosa donde estan las decisiones.

use std::path::{Path, PathBuf};

use crate::paths::HarnessPaths;

/// Cuantos resultados se imprimen sin `--todos`.
pub const TOPE: usize = 20;
/// Ancho al que se recorta el texto de cada linea en la salida humana.
pub const ANCHO: usize = 120;

/// De donde salio una linea. El ORDEN de las variantes es el orden de
/// relevancia, de mas curado a mas crudo: es la decision central del ranking y
/// por eso vive en el tipo, no en un `match` suelto.
///
/// (Patron "model states as enums" — skill `rust-patterns`.)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Fuente {
    /// Conocimiento curado: alguien lo escribio para que se reuse.
    Leccion,
    Perfil,
    /// Decisiones: lo que se acordo antes de hacer.
    Spec,
    Plan,
    Adr,
    Prd,
    /// Evidencia: lo que efectivamente paso.
    Impl,
    Review,
    Estado,
    /// Documentacion de proceso del proyecto.
    Doc,
    /// Leccion archivada (feature #21): sigue siendo consultable, pero no
    /// compite con lo vigente.
    LeccionArchivada,
    /// Bitacora cruda.
    Historia,
}

impl Fuente {
    /// Peso de la fuente. Los saltos son grandes a proposito: una leccion
    /// siempre gana a una linea de bitacora, sin importar los demas bonus.
    pub fn peso(self) -> i64 {
        match self {
            Fuente::Leccion => 100,
            Fuente::Perfil => 95,
            Fuente::Spec => 80,
            Fuente::Plan => 75,
            // Un ADR es una decision tecnica con nombre propio y sin fecha de
            // vencimiento: pesa como un spec, no como un doc cualquiera.
            Fuente::Adr => 78,
            Fuente::Prd => 70,
            Fuente::Impl => 55,
            Fuente::Review => 50,
            Fuente::Estado => 45,
            Fuente::Doc => 40,
            // Por debajo de CUALQUIER fuente activa y por encima de la bitacora:
            // el conocimiento archivado no desaparece, solo deja de ganar
            // (decision del usuario 2026-08-17, OBS-4 de la #21).
            Fuente::LeccionArchivada => 30,
            Fuente::Historia => 20,
        }
    }

    pub fn etiqueta(self) -> &'static str {
        match self {
            Fuente::Leccion => "leccion",
            Fuente::Perfil => "perfil",
            Fuente::Spec => "spec",
            Fuente::Plan => "plan",
            Fuente::Adr => "adr",
            Fuente::Prd => "prd",
            Fuente::Impl => "impl",
            Fuente::Review => "review",
            Fuente::Estado => "estado",
            Fuente::Doc => "doc",
            Fuente::LeccionArchivada => "leccion-archivada",
            Fuente::Historia => "historia",
        }
    }

    /// Clasifica por la ruta relativa del archivo.
    pub fn de_ruta(rel: &str) -> Fuente {
        let base = rel.rsplit('/').next().unwrap_or(rel);
        // La GUIA no es una leccion: es plantilla del arnes, y darle el peso del
        // conocimiento curado hace que sus EJEMPLOS (nombres malos, casos de
        // demostracion) le ganen a la decision real. Misma exclusion que ya
        // aplica `lecciones::scan`.
        if rel.contains("docs/lecciones/archivo/") {
            return Fuente::LeccionArchivada;
        }
        if rel.contains("docs/lecciones/") && base != crate::lecciones::GUIA {
            return Fuente::Leccion;
        }
        if rel.contains("docs/adr/") || base.starts_with("ADR-") {
            return Fuente::Adr;
        }
        if base == "perfil-usuario.md" {
            return Fuente::Perfil;
        }
        if rel.contains("docs/prd/") {
            return Fuente::Prd;
        }
        if base.starts_with("spec-feature-") {
            return Fuente::Spec;
        }
        if base.starts_with("plan-feature-") {
            return Fuente::Plan;
        }
        if base.starts_with("estado-feature-") {
            return Fuente::Estado;
        }
        if base.starts_with("impl-") {
            return Fuente::Impl;
        }
        if base.starts_with("review-") {
            return Fuente::Review;
        }
        if base == "history.md" {
            return Fuente::Historia;
        }
        Fuente::Doc
    }
}

/// Una linea que matcheo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hallazgo {
    pub archivo: String,
    pub linea: usize,
    pub texto: String,
    pub fuente: Fuente,
    /// Id de feature cuando se puede determinar (del nombre del archivo o de la
    /// linea de bitacora).
    pub feature: String,
    pub fecha: String,
    pub score: i64,
    /// Cuantas VECES MAS aparece esta misma linea en otros archivos
    /// (feature #39). 0 = no se repite. Se cuenta en vez de descartarse en
    /// silencio: que un encabezado este en tres documentos es informacion.
    pub repetido: usize,
}

/// Resultado de una busqueda. `parcial` marca que ninguna linea tenia TODOS los
/// terminos y se cayo a "alguno" (AC-2): el comando tiene que avisarlo, porque
/// si no el usuario cree que encontro una coincidencia exacta.
#[derive(Debug, Clone, Default)]
pub struct Resultado {
    pub hallazgos: Vec<Hallazgo>,
    pub parcial: bool,
    /// Archivos que se pudieron leer (para poder decir que el corpus existe).
    pub archivos: usize,
}

/// Normaliza la consulta a terminos comparables. Sin regex: la consulta del
/// usuario NUNCA se compila, asi que no hay ReDoS ni inyeccion (no funcional de
/// seguridad del spec).
pub fn terminos(consulta: &str) -> Vec<String> {
    consulta
        .split_whitespace()
        .map(|t| t.to_lowercase())
        .filter(|t| !t.is_empty())
        .collect()
}

/// Archivos del corpus: `docs/**/*.md` + `progress/history.md`. Excluye `bkp/`
/// y cualquier directorio de respaldo (AC-1).
pub fn corpus(paths: &HarnessPaths) -> Vec<PathBuf> {
    let mut out = Vec::new();
    recorrer(&paths.plans, &mut out);
    out.sort();
    if paths.history.is_file() {
        out.push(paths.history.clone());
    }
    out
}

/// Directorios que nunca se recorren: son respaldo o estado interno, y sus
/// copias viejas contaminarian los resultados con texto ya superado.
const EXCLUIDOS: [&str; 4] = ["bkp", ".git", "node_modules", "target"];

fn recorrer(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let nombre = entry.file_name().to_string_lossy().into_owned();
        if path.is_dir() {
            if !EXCLUIDOS.contains(&nombre.as_str()) && !nombre.starts_with('.') {
                recorrer(&path, out);
            }
        } else if nombre.ends_with(".md") {
            out.push(path);
        }
    }
}

/// `<id>` de un nombre `spec-feature-14-...md` / `plan-feature-14-...`.
fn feature_de_archivo(base: &str) -> String {
    base.split_once("-feature-")
        .map(|(_, resto)| {
            resto
                .chars()
                .take_while(char::is_ascii_digit)
                .collect::<String>()
        })
        .unwrap_or_default()
}

/// `<id>` y fecha de una linea de bitacora (`- 2026-08-14T... close feature #14 ...`).
fn de_linea_historia(linea: &str) -> (String, String) {
    let fecha = linea
        .split_whitespace()
        .find(|t| t.len() >= 10 && t.starts_with("20"))
        .map(|t| t.chars().take(10).collect::<String>())
        .unwrap_or_default();
    let feature = linea
        .split_once("feature #")
        .map(|(_, resto)| {
            resto
                .chars()
                .take_while(char::is_ascii_digit)
                .collect::<String>()
        })
        .unwrap_or_default();
    (feature, fecha)
}

fn fecha_de_archivo(path: &Path) -> String {
    let Ok(meta) = std::fs::metadata(path) else {
        return String::new();
    };
    let Ok(modificado) = meta.modified() else {
        return String::new();
    };
    let dur = modificado
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    chrono::DateTime::from_timestamp(dur.as_secs() as i64, 0)
        .map(|d| d.format("%Y-%m-%d").to_string())
        .unwrap_or_default()
}

/// Una linea es "titulo" si es encabezado markdown o un campo de frontmatter
/// que nombra el tema del documento (AC-5).
fn es_titulo(texto: &str) -> bool {
    let t = texto.trim_start();
    t.starts_with('#')
        || t.starts_with("nombre:")
        || t.starts_with("descripcion:")
        || t.starts_with("triggers:")
}

/// Palabras comparables de un texto: minusculas, sin puntuacion, y sin las de
/// una o dos letras, que no distinguen nada (`de`, `el`, `md`, `56`).
fn palabras(texto: &str) -> Vec<String> {
    texto
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { ' ' })
        .collect::<String>()
        .split_whitespace()
        .filter(|w| w.chars().count() > 2)
        .map(str::to_string)
        .collect()
}

/// Cuanto puede sobrar y todavia considerarse "el nombre del archivo otra vez".
const RESIDUO_MAXIMO: usize = 1;
/// Cuantas palabras del nombre tiene que repetir para que sea el nombre y no
/// una coincidencia de una palabra suelta.
const COINCIDENCIAS_MINIMAS: usize = 2;

/// Una linea que no dice nada que su propio nombre de archivo no dijera ya
/// (feature #39).
///
/// `# Spec - Feature #56: paquete_de_contexto_para_implementar` encabeza TODA
/// busqueda sobre esa feature y no contesta ninguna: repite el nombre del
/// archivo, que el resultado ya muestra en la ruta. Con tres documentos por
/// feature (spec, plan, estado) son tres lugares del tope gastados antes de la
/// primera linea que dice algo.
///
/// La prueba NO es "matcheo por una palabra del nombre" —el cuerpo de un spec
/// habla del tema del spec, y esas son justamente las lineas buenas— sino "no
/// queda casi nada cuando se le sacan las palabras del nombre".
fn solo_repite_el_nombre(texto: &str, archivo: &str) -> bool {
    let del_archivo = palabras(archivo);
    if del_archivo.is_empty() {
        return false;
    }
    let del_texto = palabras(texto);
    let coincidentes = del_texto.iter().filter(|w| del_archivo.contains(w)).count();
    let residuo = del_texto.len() - coincidentes;
    coincidentes >= COINCIDENCIAS_MINIMAS && residuo <= RESIDUO_MAXIMO
}

/// Campo de metadata cuyo valor ENTERO es una ruta: `Plan: docs/plan-...md`.
/// Es un puntero al documento, no una linea sobre el tema; el que busca ya
/// tiene la ruta en el resultado.
fn es_puntero(texto: &str) -> bool {
    let Some((campo, valor)) = texto.trim().split_once(':') else {
        return false;
    };
    let valor = valor.trim();
    !campo.is_empty()
        && !campo.contains(char::is_whitespace)
        && !valor.is_empty()
        && !valor.contains(char::is_whitespace)
        && valor.ends_with(".md")
}

/// Las dos formas de ocupar un lugar sin decir nada (feature #39): un puntero a
/// otro archivo, y un encabezado que solo repite el nombre del suyo.
///
/// La regla se probo primero mas amplia —"todo H1 es el titulo del documento y
/// no dice nada"— y estaba mal: en `docs/adr/ADR-0001-cliente-http.md`, el H1
/// `# ADR-0001: cliente HTTP para el espejo` es la UNICA linea que nombra el
/// tema, y el nombre del archivo no lo nombra. Un titulo que agrega algo al
/// nombre de su archivo es contenido; el descuento es para el que no.
fn identifica_en_vez_de_decir(texto: &str, archivo: &str) -> bool {
    es_puntero(texto) || solo_repite_el_nombre(texto, archivo)
}

/// Lo que se le descuenta a una linea que no aporta contenido propio. Con el
/// bonus de titulo ya negado, deja al titulo de un spec en 50 contra los 110
/// del cuerpo de ese mismo spec, y todavia por encima de una linea de bitacora
/// (20): no es basura, es un puntero, y como puntero se ordena.
const SIN_CONTENIDO_PROPIO: i64 = 60;

/// Score de una linea que ya matcheo. Funcion pura: todo el ranking se testea
/// sin tocar el filesystem.
pub fn score(
    fuente: Fuente,
    texto: &str,
    terminos: &[String],
    feature: &str,
    todos: bool,
    archivo: &str,
) -> i64 {
    let mut s = fuente.peso();
    let bajo = texto.to_lowercase();
    // Feature #39: el bonus de titulo y el descuento por no decir nada se
    // estaban peleando. El bonus premia al encabezado que NOMBRA el tema
    // (`## ureq como cliente`); darselo tambien al que solo se nombra a si
    // mismo y despues quitarselo con el descuento dejaba a los titulos de
    // documento empatando con lineas de cuerpo. Un titulo que identifica no
    // cobra el bonus Y ademas paga el descuento.
    let identifica = identifica_en_vez_de_decir(texto, archivo);
    if es_titulo(texto) && !identifica {
        s += 30;
    }
    // Frase contigua: los terminos, en orden, separados por un solo espacio.
    if terminos.len() > 1 && bajo.contains(&terminos.join(" ")) {
        s += 25;
    }
    // Frescura: una decision reciente vale mas que una de hace quince features.
    // Acotada para que nunca de vuelta el orden entre fuentes.
    s += feature.parse::<i64>().unwrap_or(0).min(30);
    // Los resultados parciales (solo algunos terminos) valen menos que los
    // completos, aunque en la practica no se mezclan.
    if !todos {
        s -= 40;
    }
    // Feature #39: lo ultimo, y despues del bonus de titulo a proposito. El
    // bonus premia al encabezado que NOMBRA el tema; esto le saca el premio al
    // que solo se nombra a si mismo. Sin este descuento, en una busqueda real
    // sobre este repo los primeros doce resultados eran titulos de archivo y
    // punteros: el tope se llenaba antes de la primera linea con contenido.
    if identifica {
        s -= SIN_CONTENIDO_PROPIO;
    }
    s
}

/// Busca `consulta` en el corpus. No escribe nada, no abre red, no toca el hub.
pub fn buscar(paths: &HarnessPaths, consulta: &str) -> Resultado {
    let terms = terminos(consulta);
    let mut res = Resultado::default();
    if terms.is_empty() {
        return res;
    }
    let mut completos: Vec<Hallazgo> = Vec::new();
    let mut parciales: Vec<Hallazgo> = Vec::new();
    for path in corpus(paths) {
        // Un archivo ilegible o con bytes invalidos se saltea (AC-15).
        let Ok(contenido) = std::fs::read_to_string(&path) else {
            continue;
        };
        res.archivos += 1;
        let rel = relativo(&path, &paths.repo_root);
        let base = rel.rsplit('/').next().unwrap_or(&rel).to_string();
        let fuente = Fuente::de_ruta(&rel);
        let fecha_archivo = fecha_de_archivo(&path);
        let feature_archivo = feature_de_archivo(&base);
        for (i, linea) in contenido.lines().enumerate() {
            let texto = linea.trim();
            if texto.is_empty() {
                continue;
            }
            let bajo = texto.to_lowercase();
            let cuantos = terms.iter().filter(|t| bajo.contains(t.as_str())).count();
            if cuantos == 0 {
                continue;
            }
            let todos = cuantos == terms.len();
            let (feature, fecha) = if fuente == Fuente::Historia {
                de_linea_historia(texto)
            } else {
                (feature_archivo.clone(), fecha_archivo.clone())
            };
            let hallazgo = Hallazgo {
                archivo: rel.clone(),
                linea: i + 1,
                score: score(fuente, texto, &terms, &feature, todos, &base),
                texto: texto.to_string(),
                fuente,
                feature,
                fecha,
                repetido: 0,
            };
            if todos {
                completos.push(hallazgo);
            } else {
                parciales.push(hallazgo);
            }
        }
    }
    // Caida a "algun termino" SOLO si no hubo ninguna coincidencia completa
    // (decision del usuario 2026-08-17, OBS-3).
    let mut elegidos = if completos.is_empty() {
        res.parcial = !parciales.is_empty();
        parciales
    } else {
        completos
    };
    elegidos.sort_by(|a, b| {
        b.score
            .cmp(&a.score)
            .then_with(|| a.archivo.cmp(&b.archivo))
            .then_with(|| a.linea.cmp(&b.linea))
    });
    // Dedup por TEXTO (feature #39). En un repo donde el mismo encabezado vive
    // en el spec, en el prd-diff y en architecture.md, doce resultados podian
    // ser doce copias de la misma linea: el tope se llenaba sin agregar nada.
    // Se queda la mejor —la lista ya viene ordenada— y se dice cuantas mas
    // habia. Descartarlas calladas escondria que el tema esta en varios
    // documentos, que es justo lo que uno quiere saber.
    let mut donde: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let mut unicos: Vec<Hallazgo> = Vec::with_capacity(elegidos.len());
    for hallazgo in elegidos {
        let clave = hallazgo.texto.to_lowercase();
        match donde.get(&clave) {
            Some(&i) => unicos[i].repetido += 1,
            None => {
                donde.insert(clave, unicos.len());
                unicos.push(hallazgo);
            }
        }
    }
    res.hallazgos = unicos;
    res
}

/// Ruta relativa a la raiz, con separadores `/` para que sea clickeable igual en
/// Windows.
pub fn relativo(path: &Path, root: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

/// Recorta a `ANCHO` respetando caracteres (no bytes).
pub fn recorta(texto: &str) -> String {
    if texto.chars().count() <= ANCHO {
        return texto.to_string();
    }
    let corto: String = texto.chars().take(ANCHO).collect();
    format!("{corto}...")
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;

    /// Sandbox con archivos en `docs/` y una bitacora opcional.
    fn sandbox(archivos: &[(&str, &str)]) -> (tempfile::TempDir, HarnessPaths) {
        let dir = tempfile::tempdir().unwrap();
        let paths = HarnessPaths::from_root(dir.path().to_path_buf());
        for (rel, contenido) in archivos {
            let path = paths.repo_root.join(rel);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(&path, contenido).unwrap();
        }
        (dir, paths)
    }

    #[test]
    fn terminos_should_split_and_lowercase() {
        assert_eq!(terminos("  Ureq   ADR "), ["ureq", "adr"]);
        assert!(terminos("   ").is_empty());
    }

    #[test]
    fn fuente_should_be_derived_from_the_path() {
        let casos = [
            ("docs/lecciones/espejo.md", Fuente::Leccion),
            // La guia es plantilla del arnes, no conocimiento curado.
            ("docs/lecciones/COMO-ESCRIBIR-UNA-LECCION.md", Fuente::Doc),
            ("docs/adr/ADR-0001-cliente-http-ureq.md", Fuente::Adr),
            ("docs/lecciones/archivo/vieja.md", Fuente::LeccionArchivada),
            ("docs/perfil-usuario.md", Fuente::Perfil),
            ("docs/prd/PRD-master.md", Fuente::Prd),
            ("docs/spec-feature-14-x.md", Fuente::Spec),
            ("docs/plan-feature-14-x.md", Fuente::Plan),
            ("docs/estado-feature-14-x.md", Fuente::Estado),
            ("docs/impl-14.md", Fuente::Impl),
            ("docs/review-14.md", Fuente::Review),
            ("progress/history.md", Fuente::Historia),
            ("docs/architecture.md", Fuente::Doc),
        ];
        for (rel, esperada) in casos {
            assert_eq!(Fuente::de_ruta(rel), esperada, "{rel}");
        }
    }

    #[test]
    fn fuente_order_should_put_curated_knowledge_first() {
        // El orden del enum ES el orden de relevancia.
        assert!(Fuente::Leccion.peso() > Fuente::Spec.peso());
        assert!(Fuente::Spec.peso() > Fuente::Impl.peso());
        assert!(Fuente::Impl.peso() > Fuente::Historia.peso());
    }

    #[test]
    fn an_archived_lesson_should_rank_below_any_active_source_but_above_the_log() {
        // AC-18 de la #21: archivar no puede hacer desaparecer el conocimiento,
        // solo dejar de competir con lo vigente.
        assert!(Fuente::LeccionArchivada.peso() < Fuente::Doc.peso());
        assert!(Fuente::LeccionArchivada.peso() < Fuente::Leccion.peso());
        assert!(Fuente::LeccionArchivada.peso() > Fuente::Historia.peso());
    }

    #[test]
    fn adr_should_rank_as_a_decision_not_as_a_generic_doc() {
        // Hallazgo de correr `buscar ureq` sobre el repo real: el ADR-0001 salia
        // en el puesto 10, debajo de ejemplos de la guia de lecciones.
        assert!(Fuente::Adr.peso() > Fuente::Doc.peso());
        assert!(Fuente::Adr.peso() > Fuente::Impl.peso());
    }

    #[test]
    fn score_should_reward_headings() {
        let t = terminos("ureq");
        let cuerpo = score(Fuente::Spec, "usamos ureq aca", &t, "", true, "");
        let titulo = score(Fuente::Spec, "## ureq como cliente", &t, "", true, "");
        assert!(titulo > cuerpo);
    }

    #[test]
    fn score_should_reward_frontmatter_fields() {
        let t = terminos("espejo");
        let cuerpo = score(Fuente::Leccion, "hablamos del espejo", &t, "", true, "");
        let campo = score(Fuente::Leccion, "triggers: [espejo, roles]", &t, "", true, "");
        assert!(campo > cuerpo);
    }

    #[test]
    fn score_should_reward_a_contiguous_phrase() {
        let t = terminos("opcion segura");
        let disperso = score(Fuente::Plan, "la opcion mas segura", &t, "", true, "");
        let contiguo = score(Fuente::Plan, "elige la opcion segura", &t, "", true, "");
        assert!(contiguo > disperso);
    }

    // Feature #39: relevancia. Los tres casos salieron de una busqueda real
    // sobre este repo, donde los primeros doce resultados no contestaban nada.

    #[test]
    fn score_should_sink_the_title_of_the_document() {
        // `# Spec - Feature #56: paquete_de_contexto_para_implementar` era el
        // primer resultado de toda busqueda sobre esa feature. No dice nada que
        // la ruta del resultado no diga ya.
        let t = terminos("paquete contexto");
        let archivo = "spec-feature-56-paquete-de-contexto-para-implementar.md";
        let titulo = score(Fuente::Spec, "# Spec - Feature #56: paquete_de_contexto_para_implementar", &t, "56", true, archivo);
        let cuerpo = score(Fuente::Spec, "el paquete de contexto sigue los punteros del mapa", &t, "56", true, archivo);
        assert!(cuerpo > titulo, "cuerpo {cuerpo} deberia ganarle al titulo {titulo}");
    }

    #[test]
    fn score_should_sink_a_pointer_to_another_file() {
        // `Plan: docs/plan-feature-56-....md` es metadata, no una linea sobre
        // el tema.
        let t = terminos("contexto");
        let puntero = score(Fuente::Spec, "Plan: docs/plan-feature-56-paquete-de-contexto.md", &t, "56", true, "spec-feature-56-x.md");
        let cuerpo = score(Fuente::Spec, "decidimos armar el contexto antes de explorar", &t, "56", true, "spec-feature-56-x.md");
        assert!(cuerpo > puntero, "cuerpo {cuerpo} deberia ganarle al puntero {puntero}");
    }

    #[test]
    fn score_should_keep_rewarding_a_section_heading_that_says_something() {
        // La contracara, y el limite del arreglo: un `##` que NOMBRA el tema
        // sigue valiendo mas que el cuerpo. Sin este test, la forma facil de
        // hacer pasar a los dos de arriba seria matar el bonus de titulo.
        let t = terminos("ureq");
        let cuerpo = score(Fuente::Spec, "usamos ureq aca", &t, "", true, "spec-feature-3-http.md");
        let seccion = score(Fuente::Spec, "## ureq como cliente", &t, "", true, "spec-feature-3-http.md");
        assert!(seccion > cuerpo, "seccion {seccion} deberia ganarle al cuerpo {cuerpo}");
    }

    #[test]
    fn buscar_should_collapse_the_same_line_repeated_across_files() {
        // El mismo encabezado vive en tres documentos: doce resultados podian
        // ser doce copias. Se muestra una y se dice cuantas mas habia.
        let (_d, paths) = sandbox(&[
            ("docs/architecture.md", "## Paquete de contexto (feature #56)
"),
            ("docs/prd-diff-56.md", "## Paquete de contexto (feature #56)
"),
            ("docs/prd-diff-58.md", "## Paquete de contexto (feature #56)
"),
        ]);
        let res = buscar(&paths, "paquete contexto");
        assert_eq!(res.hallazgos.len(), 1, "no se colapsaron las copias: {:?}", res.hallazgos);
        assert_eq!(res.hallazgos[0].repetido, 2, "no dice cuantas copias mas habia");
    }

    #[test]
    fn buscar_should_put_content_above_titles_and_pointers() {
        // El caso completo, de punta a punta: el documento entero como esta en
        // el repo, y lo que tiene que salir primero.
        let (_d, paths) = sandbox(&[(
            "docs/spec-feature-56-paquete-de-contexto.md",
            "# Spec - Feature #56: paquete_de_contexto
             Plan: docs/plan-feature-56-paquete-de-contexto.md
             El paquete de contexto se arma antes de explorar el repo.
",
        )]);
        let res = buscar(&paths, "paquete contexto");
        assert_eq!(res.hallazgos.len(), 3);
        assert_eq!(
            res.hallazgos[0].linea, 3,
            "primero tiene que ir la linea con contenido, no el titulo ni el puntero: {:?}",
            res.hallazgos.iter().map(|h| (h.linea, h.score)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn score_should_prefer_recent_features() {
        let t = terminos("gate");
        let vieja = score(Fuente::Spec, "el gate", &t, "3", true, "");
        let nueva = score(Fuente::Spec, "el gate", &t, "19", true, "");
        assert!(nueva > vieja);
    }

    #[test]
    fn score_freshness_should_never_beat_the_source_weight() {
        // Una leccion vieja tiene que seguir ganandole a una bitacora nueva.
        let t = terminos("x");
        let leccion_vieja = score(Fuente::Leccion, "x", &t, "1", true, "");
        let historia_nueva = score(Fuente::Historia, "x", &t, "99", true, "");
        assert!(leccion_vieja > historia_nueva);
    }

    #[test]
    fn buscar_should_require_all_terms() {
        let (_d, paths) = sandbox(&[
            ("docs/spec-feature-1-x.md", "linea con ureq y adr\notra con ureq solo\n"),
        ]);
        let res = buscar(&paths, "ureq adr");
        assert!(!res.parcial);
        assert_eq!(res.hallazgos.len(), 1);
        assert_eq!(res.hallazgos[0].linea, 1);
    }

    #[test]
    fn buscar_should_fall_back_to_any_term_and_flag_it() {
        let (_d, paths) = sandbox(&[("docs/impl-1.md", "solo menciona ureq\n")]);
        let res = buscar(&paths, "ureq inexistente");
        assert!(res.parcial, "deberia marcarse como parcial");
        assert_eq!(res.hallazgos.len(), 1);
    }

    #[test]
    fn buscar_should_return_nothing_for_an_empty_query() {
        let (_d, paths) = sandbox(&[("docs/impl-1.md", "texto\n")]);
        assert!(buscar(&paths, "   ").hallazgos.is_empty());
    }

    #[test]
    fn buscar_should_rank_a_lesson_above_the_log() {
        let (_d, paths) = sandbox(&[
            ("docs/lecciones/espejo.md", "el espejo de roles\n"),
            ("progress/history.md", "- 2026-08-14T00:00:00Z close feature #14 espejo\n"),
        ]);
        let res = buscar(&paths, "espejo");
        assert_eq!(res.hallazgos.len(), 2);
        assert_eq!(res.hallazgos[0].fuente, Fuente::Leccion);
        assert_eq!(res.hallazgos[1].fuente, Fuente::Historia);
    }

    #[test]
    fn buscar_should_read_feature_and_date_from_the_log_line() {
        let (_d, paths) = sandbox(&[(
            "progress/history.md",
            "- 2026-08-14T03:43:37Z approve-spec feature #14 nota=ureq\n",
        )]);
        let res = buscar(&paths, "ureq");
        assert_eq!(res.hallazgos[0].feature, "14");
        assert_eq!(res.hallazgos[0].fecha, "2026-08-14");
    }

    #[test]
    fn buscar_should_read_the_feature_from_the_file_name() {
        let (_d, paths) = sandbox(&[("docs/spec-feature-16-x.md", "sobre ureq\n")]);
        assert_eq!(buscar(&paths, "ureq").hallazgos[0].feature, "16");
    }

    #[test]
    fn corpus_should_skip_backups_and_hidden_dirs() {
        let (_d, paths) = sandbox(&[
            ("docs/impl-1.md", "vivo\n"),
            ("docs/prd/PRD-master.md", "vivo\n"),
            ("bkp/impl-1.md.bak.20260101", "viejo\n"),
            ("bkp/viejo.md", "viejo\n"),
        ]);
        let archivos = corpus(&paths);
        assert!(archivos.iter().all(|p| !p.to_string_lossy().contains("/bkp/")));
        assert_eq!(archivos.len(), 2);
    }

    #[test]
    fn buscar_should_be_empty_without_a_docs_dir() {
        let dir = tempfile::tempdir().unwrap();
        let paths = HarnessPaths::from_root(dir.path().to_path_buf());
        let res = buscar(&paths, "lo que sea");
        assert!(res.hallazgos.is_empty());
        assert_eq!(res.archivos, 0);
    }

    #[test]
    fn recorta_should_respect_characters_not_bytes() {
        let largo = "á".repeat(ANCHO + 10);
        let corto = recorta(&largo);
        assert!(corto.ends_with("..."));
        assert_eq!(corto.chars().count(), ANCHO + 3);
        assert_eq!(recorta("corto"), "corto");
    }
}
