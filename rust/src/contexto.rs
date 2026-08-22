//! El paquete de contexto (feature #56): todo lo que hace falta para EMPEZAR a
//! implementar, y —sobre todo— el aviso de cuando no hay nada.
//!
//! Es el gemelo de `revision.rs` (feature #51), del otro lado del flujo. El
//! disparador tambien fue un dato medido: un mapeo de cuatro agentes y
//! **693.6k tokens** sobre el motor de reajuste de un proyecto, para descubrir
//! algo que se podia saber en dos segundos — que el mapa de arquitectura no
//! menciona el tema ni una vez.
//!
//! Lo que este modulo agrega sobre "juntar material":
//!
//! - **Sigue los punteros.** Un `architecture.md` que solo dice "la copia
//!   canonica vive en X" se resuelve, y si X no existe eso es un HUECO con la
//!   ruta que falta — que es un diagnostico distinto de "no hay mapa".
//! - **Dice el vacio.** Si ninguno de los terminos del tema aparece en el mapa,
//!   el paquete lo declara con esas palabras, en vez de devolver vacio y dejar
//!   que el agente lo averigue explorando.
//! - **No escribe nada y nunca bloquea.** Cada fuente es opcional; su ausencia
//!   se anota en `faltantes`.

use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use serde_json::{Value, json};

use crate::paths::HarnessPaths;
use crate::revision::{Recorte, recortar};

/// Presupuesto por default. Mas chico que el de `revision` (400) porque aca el
/// material es prosa: 300 lineas de mapa es bastante mas informacion que 300
/// lineas de diff.
pub const MAX_LINEAS_DEFAULT: usize = 300;

/// A partir de cuantos dias se declara vencido el grafo de graphify. Decision
/// del usuario (OBS-2 del spec #56): 7, no 14.
pub const GRAFO_VENCIDO_DIAS: i64 = 7;

/// Tope de hits de `buscar` que entran al paquete (AC-11). Hoy una consulta de
/// cinco terminos devuelve 12.521 resultados: el volcado ES el problema.
pub const MAX_HITS: usize = 12;

/// Cuantas lineas puede tener un documento para que se lo considere un puntero
/// y no un mapa. Un mapa de verdad no entra en veinte lineas.
const MAX_LINEAS_PUNTERO: usize = 20;

/// Limite para consultar el hub. Sin respuesta no hay error: hay hueco.
const LIMITE_HUB: Duration = Duration::from_secs(5);

/// Terminos que no discriminan nada y solo generan falsos "si cubre".
const VACIAS: &[&str] = &[
    "de", "del", "la", "las", "el", "los", "un", "una", "unos", "unas", "en", "con", "para", "por",
    "que", "como", "sin", "sobre", "al", "lo", "y", "o", "se", "su", "sus", "es", "the", "a",
];

/// El mapa de arquitectura, ya resuelto.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mapa {
    /// Ruta del documento que se leyo de verdad (el destino, si habia puntero).
    pub ruta: String,
    /// Ruta del puntero, cuando el mapa se alcanzo siguiendo uno.
    pub via_puntero: Option<String>,
    pub lineas: usize,
    /// Secciones que mencionan el tema (AC-7). Vacio si no lo cubre.
    pub secciones: Vec<String>,
    pub cubre: bool,
}

/// El grafo de graphify del proyecto.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Grafo {
    pub ruta: String,
    pub dias: i64,
    pub vencido: bool,
}

/// El paquete completo.
#[derive(Debug, Clone, Default)]
pub struct Paquete {
    pub tema: String,
    /// Terminos con los que se busco: van en la salida para que un falso
    /// "no cubre" se pueda diagnosticar de un vistazo.
    pub terminos: Vec<String>,
    pub feature_id: Option<String>,
    pub nombre: Option<String>,
    pub mapa: Option<Mapa>,
    pub impacto: Vec<String>,
    pub grafo: Option<Grafo>,
    pub historia: Vec<String>,
    pub lecciones: Vec<String>,
    pub relacionadas: Vec<String>,
    pub recorte: Option<Recorte>,
    /// Lo que se busco y no estaba, cada uno con el comando que lo consigue.
    pub faltantes: Vec<String>,
}

impl Paquete {
    /// Tamaño del paquete: el costo se ve ANTES de gastarlo, igual que en
    /// `revision` (AC-10).
    pub fn tamano(&self) -> (usize, usize) {
        let texto = self.render_texto();
        (texto.lines().count(), texto.chars().count() / 4)
    }

    /// La linea que aparece cuando el mapa no cubre el tema. Es el corazon de
    /// la feature, asi que vive en una funcion sola y tiene test propio.
    pub fn aviso_de_cobertura(&self) -> Option<String> {
        let mapa = self.mapa.as_ref()?;
        if mapa.cubre {
            return None;
        }
        // Sin terminos utiles el mapa no tiene la culpa: la consulta no
        // pregunta nada. Acusar al mapa aca seria el mismo falso aviso que esta
        // feature existe para evitar.
        if self.terminos.is_empty() {
            return Some(format!(
                "NO SE PUEDE DECIR SI EL MAPA CUBRE ESTE TEMA: '{}' no deja ningun termino\n\
                 con el que buscar (son palabras vacias o de menos de tres letras). Volve a\n\
                 pedirlo con un tema mas especifico: `contexto --tema \"<palabras del dominio>\"`.",
                self.tema
            ));
        }
        Some(format!(
            "EL MAPA NO CUBRE ESTE TEMA: '{}' no menciona ninguno de estos terminos: {}.\n\
             No es que no haya mapa: es que el tema no esta escrito ahi. Antes de explorar el\n\
             repo entero, decidilo con el usuario: mapear primero suele costar menos que\n\
             descubrirlo leyendo.",
            mapa.ruta,
            self.terminos.join(", ")
        ))
    }

    /// Version corta, la que imprime `start` (AC-12). Sin cuerpo: solo que hay,
    /// que falta y como pedir el resto.
    pub fn resumen(&self) -> String {
        let mut out = String::from("== Contexto ==\n");
        match &self.mapa {
            Some(m) if m.cubre => out.push_str(&format!(
                "  mapa: {} ({} lineas), cubre el tema en {} seccion(es)\n",
                m.ruta,
                m.lineas,
                m.secciones.len()
            )),
            Some(m) => out.push_str(&format!(
                "  mapa: {} ({} lineas) -- NO cubre '{}'\n",
                m.ruta, m.lineas, self.tema
            )),
            None => out.push_str("  mapa: no hay\n"),
        }
        if let Some(g) = &self.grafo {
            let estado = if g.vencido { "VENCIDO" } else { "fresco" };
            out.push_str(&format!("  grafo: {} dias ({estado})\n", g.dias));
        }
        out.push_str(&format!(
            "  impacto: {} | lecciones: {} | relacionadas: {} | huecos: {}\n",
            self.impacto.len(),
            self.lecciones.len(),
            self.relacionadas.len(),
            self.faltantes.len()
        ));
        let (lineas, tokens) = self.tamano();
        out.push_str(&format!(
            "  el cuerpo: harness contexto{} (~{lineas} lineas, ~{tokens} tokens)\n",
            match &self.feature_id {
                Some(id) => format!(" --feature {id}"),
                None => String::new(),
            }
        ));
        out
    }

    /// El paquete en texto: lo que lee el agente antes de escribir codigo.
    pub fn render_texto(&self) -> String {
        let mut out = String::new();
        match (&self.feature_id, &self.nombre) {
            (Some(id), Some(n)) => {
                out.push_str(&format!("== Paquete de contexto - Feature #{id}: {n} ==\n\n"))
            }
            _ => out.push_str(&format!("== Paquete de contexto - tema: {} ==\n\n", self.tema)),
        }

        out.push_str("## Mapa\n\n");
        match &self.mapa {
            Some(m) => {
                out.push_str(&format!("{} ({} lineas)\n", m.ruta, m.lineas));
                if let Some(p) = &m.via_puntero {
                    out.push_str(&format!("(alcanzado siguiendo el puntero de {p})\n"));
                }
            }
            None => out.push_str("(no hay mapa de arquitectura)\n"),
        }

        out.push_str("\n## Cobertura del tema\n\n");
        match self.aviso_de_cobertura() {
            Some(aviso) => out.push_str(&format!("{aviso}\n")),
            None if self.mapa.is_some() => {
                out.push_str(&format!(
                    "El mapa cubre el tema. Secciones que lo mencionan ({}):\n\n",
                    self.mapa.as_ref().map_or(0, |m| m.secciones.len())
                ));
                for s in self.mapa.iter().flat_map(|m| &m.secciones) {
                    out.push_str(s);
                    if !s.ends_with('\n') {
                        out.push('\n');
                    }
                    out.push('\n');
                }
            }
            None => out.push_str("(sin mapa no se puede decir si el tema esta cubierto)\n"),
        }
        if let Some(r) = &self.recorte {
            out.push_str(&format!(
                "\n[recortado] se muestran {} de {} lineas del mapa. El resto esta en el archivo.\n",
                r.lineas_mostradas, r.lineas_totales
            ));
        }

        out.push_str("\n## Impacto (hub)\n\n");
        if self.impacto.is_empty() {
            out.push_str("(sin datos de impacto)\n");
        }
        for i in &self.impacto {
            out.push_str(&format!("- {i}\n"));
        }

        out.push_str("\n## Grafo (graphify)\n\n");
        match &self.grafo {
            Some(g) if g.vencido => out.push_str(&format!(
                "{} tiene {} dias (vencido a los {GRAFO_VENCIDO_DIAS}): lo que diga puede estar viejo.\n  refrescar: graphify index\n  consultar: graphify query \"{}\"\n",
                g.ruta, g.dias, self.tema
            )),
            Some(g) => out.push_str(&format!(
                "{} ({} dias).\n  consultar: graphify query \"{}\"\n",
                g.ruta, g.dias, self.tema
            )),
            None => out.push_str("(este proyecto no tiene grafo)\n"),
        }

        out.push_str("\n## Historia (lo que ya se decidio)\n\n");
        if self.historia.is_empty() {
            out.push_str("(sin coincidencias en specs, planes, lecciones ni bitacora)\n");
        }
        for h in &self.historia {
            out.push_str(&format!("- {h}\n"));
        }

        out.push_str("\n## Lecciones que aplican\n\n");
        if self.lecciones.is_empty() {
            out.push_str("(ninguna leccion tiene triggers de este tema)\n");
        }
        for l in &self.lecciones {
            out.push_str(&format!("- {l}\n"));
        }

        out.push_str("\n## Features que tocaron lo mismo\n\n");
        if self.relacionadas.is_empty() {
            out.push_str("(ninguna)\n");
        }
        for r in &self.relacionadas {
            out.push_str(&format!("- {r}\n"));
        }

        if !self.faltantes.is_empty() {
            out.push_str("\n## Falta\n\n");
            for f in &self.faltantes {
                out.push_str(&format!("- {f}\n"));
            }
        }
        out
    }

    /// El mismo contenido en JSON, para que un agente no tenga que parsear.
    pub fn render_json(&self) -> Value {
        let (lineas, tokens) = self.tamano();
        json!({
            "tema": self.tema,
            "terminos": self.terminos,
            "feature": self.feature_id,
            "nombre": self.nombre,
            "mapa": self.mapa.as_ref().map(|m| json!({
                "ruta": m.ruta,
                "via_puntero": m.via_puntero,
                "lineas": m.lineas,
                "cubre": m.cubre,
                "secciones": m.secciones,
            })),
            "aviso_de_cobertura": self.aviso_de_cobertura(),
            "impacto": self.impacto,
            "grafo": self.grafo.as_ref().map(|g| json!({
                "ruta": g.ruta,
                "dias": g.dias,
                "vencido": g.vencido,
            })),
            "historia": self.historia,
            "lecciones": self.lecciones,
            "relacionadas": self.relacionadas,
            "recorte": self.recorte.as_ref().map(|r| json!({
                "lineas_mostradas": r.lineas_mostradas,
                "lineas_totales": r.lineas_totales,
            })),
            "faltantes": self.faltantes,
            "tamano": {"lineas": lineas, "tokens_estimados": tokens},
        })
    }
}

/// Quita los acentos del castellano. Sin esto, un mapa que dice "migracion" y
/// un tema que dice "migración" no se encuentran, y el paquete diria "no cubre"
/// sobre un mapa que si cubre: el falso positivo mas caro que tiene esta
/// feature.
pub fn sin_acentos(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'á' | 'à' | 'ä' | 'â' => 'a',
            'é' | 'è' | 'ë' | 'ê' => 'e',
            'í' | 'ì' | 'ï' | 'î' => 'i',
            'ó' | 'ò' | 'ö' | 'ô' => 'o',
            'ú' | 'ù' | 'ü' | 'û' => 'u',
            'ñ' => 'n',
            otro => otro,
        })
        .collect()
}

/// Terminos con los que vale la pena buscar: sin acentos, en minusculas, sin
/// palabras vacias y sin fragmentos de menos de tres letras (que matchean
/// cualquier cosa y darian un "si cubre" falso).
pub fn terminos_utiles(tema: &str) -> Vec<String> {
    let mut out: Vec<String> = sin_acentos(&tema.to_lowercase())
        .split(|c: char| !c.is_alphanumeric())
        .filter(|t| t.chars().count() >= 3 && !VACIAS.contains(t))
        .map(str::to_string)
        .collect();
    out.dedup();
    out
}

/// Si el documento es un PUNTERO a otro archivo, devuelve la ruta apuntada
/// (AC-4). Un puntero es corto y su unico contenido util es una ruta `.md`
/// entre backticks.
pub fn resolver_puntero(texto: &str) -> Option<String> {
    if texto.lines().count() > MAX_LINEAS_PUNTERO {
        return None;
    }
    let bajo = sin_acentos(&texto.to_lowercase());
    if !(bajo.contains("puntero") || bajo.contains("canonica") || bajo.contains("vive en")) {
        return None;
    }
    texto
        .split('`')
        .skip(1)
        .step_by(2)
        .find(|frag| frag.ends_with(".md"))
        .map(str::to_string)
}

/// ¿El mapa menciona el tema? (AC-6). Basta con UN termino util: exigir todos
/// convertiria cualquier tema de tres palabras en un "no cubre".
pub fn cubre(texto: &str, terminos: &[String]) -> bool {
    if terminos.is_empty() {
        return false;
    }
    let bajo = sin_acentos(&texto.to_lowercase());
    terminos.iter().any(|t| bajo.contains(t.as_str()))
}

/// Las secciones (`## ...`) del mapa que mencionan el tema (AC-7): el mapa
/// entero es justo lo que no queremos entregar.
pub fn secciones_que_mencionan(texto: &str, terminos: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    let mut actual = String::new();
    for linea in texto.lines() {
        if linea.starts_with("## ") && !actual.is_empty() {
            if cubre(&actual, terminos) {
                out.push(actual.trim_end().to_string());
            }
            actual = String::new();
        }
        actual.push_str(linea);
        actual.push('\n');
    }
    if !actual.is_empty() && cubre(&actual, terminos) {
        out.push(actual.trim_end().to_string());
    }
    out
}

/// Edad en dias de un archivo. `None` si el sistema no sabe decirla: un dato
/// que no existe no se inventa.
pub fn edad_en_dias(path: &Path, ahora: SystemTime) -> Option<i64> {
    let modificado = std::fs::metadata(path).ok()?.modified().ok()?;
    let dur = ahora.duration_since(modificado).ok()?;
    Some((dur.as_secs() / 86_400) as i64)
}

/// El impacto segun el hub, con LIMITE: si no contesta, no hay error, hay
/// hueco (AC-15). La consulta corre en un hilo aparte justamente porque
/// conectar a un Postgres caido puede tardar un minuto.
pub fn impacto_con_limite(servicio: &str, limite: Duration) -> Result<Vec<String>, String> {
    let servicio = servicio.to_string();
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let resultado = crate::graph::GraphMemoryManager::new()
            .and_then(|mut m| m.impacto_de(&servicio))
            .map_err(|e| format!("{e:#}"));
        let _ = tx.send(resultado);
    });
    match rx.recv_timeout(limite) {
        Ok(Ok(v)) => Ok(v),
        Ok(Err(e)) => Err(e),
        Err(_) => Err(format!(
            "el hub no contesto en {}s",
            limite.as_secs().max(1)
        )),
    }
}

/// Arma el paquete leyendo lo que exista. Ninguna ausencia es un error: cada
/// una se anota en `faltantes` con el comando que la consigue (AC-15).
pub fn armar(
    paths: &HarnessPaths,
    feature: Option<&serde_json::Map<String, Value>>,
    tema: &str,
    max_lineas: usize,
) -> Paquete {
    use crate::pycompat::py_str;

    let mut p = Paquete {
        tema: tema.to_string(),
        terminos: terminos_utiles(tema),
        ..Default::default()
    };
    if let Some(f) = feature {
        p.feature_id = Some(py_str(f.get("id")));
        p.nombre = Some(py_str(f.get("name")));
    }

    // 1. El mapa, siguiendo punteros (AC-4, AC-5).
    let arch = paths.plans.join(
        crate::documentos::ARCHITECTURE
            .strip_prefix("docs/")
            .unwrap_or(crate::documentos::ARCHITECTURE),
    );
    match std::fs::read_to_string(&arch) {
        Ok(texto) => {
            let (ruta, texto, via) = match resolver_puntero(&texto) {
                Some(destino) => {
                    // Un puntero relativo se resuelve contra el directorio del
                    // documento que lo contiene, NO contra el cwd: si no, el
                    // mismo puntero apuntaria a lugares distintos segun desde
                    // donde se corra el comando.
                    let crudo = PathBuf::from(&destino);
                    let destino_path = if crudo.is_absolute() {
                        crudo
                    } else {
                        arch.parent().unwrap_or(Path::new(".")).join(&crudo)
                    };
                    match std::fs::read_to_string(&destino_path) {
                        Ok(t) => (destino, t, Some(arch.display().to_string())),
                        Err(_) => {
                            // AC-5: puntero roto. Es un diagnostico distinto de
                            // "no hay mapa", y por eso se dice distinto.
                            p.faltantes.push(format!(
                                "el mapa: {} es un puntero a {}, que NO existe. Arreglalo o borralo: un puntero roto se lee como 'aca no hay nada'",
                                arch.display(),
                                destino_path.display()
                            ));
                            (String::new(), String::new(), None)
                        }
                    }
                }
                None => (arch.display().to_string(), texto, None),
            };
            if !ruta.is_empty() {
                let lineas = texto.lines().count();
                let cubre_tema = cubre(&texto, &p.terminos);
                let secciones = if cubre_tema {
                    let crudas = secciones_que_mencionan(&texto, &p.terminos);
                    let (recortado, recorte) = recortar(&crudas.join("\n\n"), max_lineas);
                    p.recorte = recorte;
                    recortado
                        .split("\n\n")
                        .filter(|s| !s.trim().is_empty())
                        .map(str::to_string)
                        .collect()
                } else {
                    Vec::new()
                };
                p.mapa = Some(Mapa {
                    ruta,
                    via_puntero: via,
                    lineas,
                    secciones,
                    cubre: cubre_tema,
                });
            }
        }
        Err(_) => p.faltantes.push(format!(
            "el mapa ({}): sin el, cualquier tema arranca a ciegas",
            arch.display()
        )),
    }

    // 2. Impacto del hub, con limite (AC-15).
    if let Some(f) = feature {
        let servicio = py_str(f.get("service"));
        if !servicio.is_empty() && servicio != "None" {
            match impacto_con_limite(&servicio, LIMITE_HUB) {
                Ok(afectados) if afectados.is_empty() => {
                    p.impacto
                        .push(format!("ningun microservicio registrado depende de {servicio}"));
                }
                Ok(afectados) => {
                    p.impacto
                        .push(format!("si tocas {servicio}, revisa: {}", afectados.join(", ")));
                }
                Err(motivo) => p.faltantes.push(format!(
                    "el impacto del hub ({motivo}): consultalo aparte con `harness graph impacto --microservicio <proyecto>/{servicio}`"
                )),
            }
        }
    }

    // 3. El grafo: edad, no contenido (OBS-1: no se invoca graphify por default).
    let grafo = paths.repo_root.join("graphify-out").join("graph.json");
    match edad_en_dias(&grafo, SystemTime::now()) {
        Some(dias) => {
            p.grafo = Some(Grafo {
                ruta: grafo.display().to_string(),
                dias,
                vencido: dias > GRAFO_VENCIDO_DIAS,
            })
        }
        None => p.faltantes.push(format!(
            "el grafo ({}): generalo con `graphify index` si queres consultarlo",
            grafo.display()
        )),
    }

    // 4. La historia, acotada a los hits mas curados (AC-11).
    let hallazgos = crate::buscar::buscar(paths, tema);
    if hallazgos.hallazgos.is_empty() {
        p.faltantes
            .push("historia: ningun spec, plan, leccion ni linea de bitacora menciona el tema".into());
    }
    p.historia = hallazgos
        .hallazgos
        .iter()
        .take(MAX_HITS)
        .map(|h| {
            format!(
                "{}:{} [{}] {}",
                h.archivo,
                h.linea,
                h.fecha,
                crate::buscar::recorta(&h.texto)
            )
        })
        .collect();
    if hallazgos.hallazgos.len() > MAX_HITS {
        p.historia.push(format!(
            "(+{} coincidencias mas; el resto con `harness buscar \"{tema}\"`)",
            hallazgos.hallazgos.len() - MAX_HITS
        ));
    }

    // 5. Lecciones cuyos triggers pegan con el tema.
    let (lecciones, _) = crate::lecciones::scan(paths);
    p.lecciones = lecciones
        .iter()
        .filter(|l| {
            let triggers = l.fm.list("triggers");
            triggers
                .iter()
                .any(|t| cubre(t, &p.terminos) || cubre(tema, &[sin_acentos(&t.to_lowercase())]))
        })
        .map(|l| format!("{}: {}", l.nombre, l.descripcion()))
        .collect();

    // 6. Features anteriores del mismo servicio.
    if let (Some(f), Ok(data)) = (feature, crate::features::load_features(paths)) {
        let servicio = py_str(f.get("service"));
        let propio = py_str(f.get("id"));
        if let Some(arr) = data.get("features").and_then(Value::as_array) {
            p.relacionadas = arr
                .iter()
                .filter_map(Value::as_object)
                .filter(|o| py_str(o.get("service")) == servicio && py_str(o.get("id")) != propio)
                .filter(|o| py_str(o.get("status")) == "done")
                .rev()
                .take(MAX_HITS)
                .map(|o| {
                    format!(
                        "#{} {} (docs/impl-{}.md)",
                        py_str(o.get("id")),
                        py_str(o.get("name")),
                        py_str(o.get("id"))
                    )
                })
                .collect();
        }
    }

    p
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;

    /// `HarnessPaths` a mano: `from_root` mira el entorno y el CWD, y un test
    /// no puede depender de desde donde se lo corre.
    fn paths_de(root: &Path) -> HarnessPaths {
        HarnessPaths {
            features: root.join("feature_list.json"),
            current: root.join("progress").join("current.md"),
            history: root.join("progress").join("history.md"),
            autocheck_stamp: root.join("progress").join(".last_autocheck"),
            nudge_stamp: root.join("progress").join(".last_nudge"),
            nudge_lecciones: root.join("progress").join(".nudge_lecciones"),
            plans: root.join("docs"),
            progress: root.join("progress"),
            repo_root: root.to_path_buf(),
            root: root.to_path_buf(),
            worktree: None,
        }
    }

    #[test]
    fn contexto_puntero() {
        // AC-4: un architecture.md que solo apunta a otro archivo se resuelve.
        let doc = "# Arquitectura (puntero)\n\nLa copia CANONICA vive en\n`/tmp/x/docs/architecture.md`.\n";
        assert_eq!(
            resolver_puntero(doc).as_deref(),
            Some("/tmp/x/docs/architecture.md")
        );
        // Un mapa de verdad NO es un puntero, aunque nombre un .md.
        let mapa_largo = format!("# Arquitectura\n\n{}\n`otro.md`\n", "linea\n".repeat(40));
        assert_eq!(resolver_puntero(&mapa_largo), None);

        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        std::fs::create_dir_all(root.join("docs")).unwrap();

        // AC-5: puntero ROTO. El paquete no dice "no hay mapa": dice que hay un
        // puntero y que su destino no existe, con la ruta.
        let destino = root.join("no-existe").join("architecture.md");
        std::fs::write(
            root.join("docs").join("architecture.md"),
            format!("# Puntero\n\nLa copia CANONICA vive en `{}`.\n", destino.display()),
        )
        .unwrap();
        let p = armar(&paths_de(root), None, "reajuste", MAX_LINEAS_DEFAULT);
        assert!(p.mapa.is_none(), "un puntero roto no deja mapa utilizable");
        assert!(
            p.faltantes.iter().any(|f| f.contains("NO existe") && f.contains("no-existe")),
            "el hueco tiene que decir la ruta que falta: {:?}",
            p.faltantes
        );

        // AC-4 completo: con el destino creado, el mapa sale de ahi.
        std::fs::create_dir_all(root.join("no-existe")).unwrap();
        std::fs::write(&destino, "# Mapa real\n\n## Motor de reajuste\n\nasi funciona\n").unwrap();
        let p = armar(&paths_de(root), None, "reajuste", MAX_LINEAS_DEFAULT);
        let Some(mapa) = p.mapa else {
            panic!("con el destino creado tiene que haber mapa");
        };
        assert!(mapa.ruta.ends_with("architecture.md"));
        assert!(mapa.via_puntero.is_some(), "tiene que decir que vino por un puntero");
        assert!(mapa.cubre);
    }

    #[test]
    fn puntero_relativo_se_resuelve_contra_el_documento() {
        // Un puntero relativo tiene que apuntar al mismo lugar sin importar
        // desde donde se corra el comando.
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        std::fs::create_dir_all(root.join("docs")).unwrap();
        std::fs::write(
            root.join("docs").join("architecture.md"),
            "# Puntero\n\nLa copia CANONICA vive en `mapa-real.md`.\n",
        )
        .unwrap();
        std::fs::write(
            root.join("docs").join("mapa-real.md"),
            "# Mapa real\n\n## Reajuste\n\nel motor corre mensual\n",
        )
        .unwrap();
        let p = armar(&paths_de(root), None, "reajuste", MAX_LINEAS_DEFAULT);
        let Some(mapa) = p.mapa else {
            panic!("el puntero relativo tiene que resolverse: {:?}", p.faltantes);
        };
        assert!(mapa.ruta.ends_with("mapa-real.md"));
        assert!(mapa.cubre);
    }

    #[test]
    fn contexto_cobertura() {
        // Los acentos no pueden decidir si un mapa cubre un tema.
        let terminos = terminos_utiles("migración de datos");
        assert!(terminos.contains(&"migracion".to_string()));
        assert!(!terminos.contains(&"de".to_string()), "las vacias no cuentan");
        assert!(cubre("Este mapa habla de la migracion del esquema", &terminos));

        // AC-6: el aviso nombra los terminos buscados, para poder diagnosticar
        // un falso "no cubre" de un vistazo.
        let p = Paquete {
            tema: "motor de reajuste".into(),
            terminos: terminos_utiles("motor de reajuste"),
            mapa: Some(Mapa {
                ruta: "docs/architecture.md".into(),
                via_puntero: None,
                lineas: 656,
                secciones: Vec::new(),
                cubre: false,
            }),
            ..Default::default()
        };
        let Some(aviso) = p.aviso_de_cobertura() else {
            panic!("sin cobertura tiene que avisar");
        };
        assert!(aviso.contains("EL MAPA NO CUBRE ESTE TEMA"));
        assert!(aviso.contains("motor"), "tiene que listar los terminos: {aviso}");
        assert!(p.render_texto().contains("EL MAPA NO CUBRE ESTE TEMA"));

        // Y calla cuando SI cubre.
        let mut cubierto = p.clone();
        if let Some(m) = cubierto.mapa.as_mut() {
            m.cubre = true;
        }
        assert_eq!(cubierto.aviso_de_cobertura(), None);

        // AC-7: solo las secciones que mencionan el tema.
        let mapa = "# Mapa\n\n## Auth\n\njwt y sesiones\n\n## Reajuste\n\nel motor de reajuste\n\n## Media\n\nimagenes\n";
        let secciones = secciones_que_mencionan(mapa, &terminos_utiles("reajuste"));
        assert_eq!(secciones.len(), 1, "solo una seccion menciona el tema");
        assert!(secciones[0].starts_with("## Reajuste"));
    }

    #[test]
    fn tema_sin_terminos_no_acusa_al_mapa() {
        // Encontrado intentando romper el AC-6: con `--tema "de la"` el paquete
        // decia "EL MAPA NO CUBRE ESTE TEMA ... terminos: ." — culpando al mapa
        // por una consulta vacia. El aviso tiene que apuntar a la consulta.
        assert!(terminos_utiles("de la").is_empty());
        let p = Paquete {
            tema: "de la".into(),
            terminos: terminos_utiles("de la"),
            mapa: Some(Mapa {
                ruta: "docs/architecture.md".into(),
                via_puntero: None,
                lineas: 656,
                secciones: Vec::new(),
                cubre: false,
            }),
            ..Default::default()
        };
        let Some(aviso) = p.aviso_de_cobertura() else {
            panic!("con un tema vacio tambien hay que decir algo");
        };
        assert!(!aviso.contains("EL MAPA NO CUBRE"), "no se le echa la culpa al mapa: {aviso}");
        assert!(aviso.contains("NO SE PUEDE DECIR"));
        assert!(aviso.contains("mas especifico"), "y se dice como arreglarlo");
    }

    #[test]
    fn contexto_presupuesto() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        std::fs::create_dir_all(root.join("docs")).unwrap();
        let cuerpo = format!("# Mapa\n\n## Reajuste\n\n{}", "detalle de reajuste\n".repeat(200));
        std::fs::write(root.join("docs").join("architecture.md"), cuerpo).unwrap();

        let p = armar(&paths_de(root), None, "reajuste", 20);
        let Some(recorte) = p.recorte.as_ref() else {
            panic!("200 lineas no entran en 20");
        };
        assert_eq!(recorte.lineas_mostradas, 20);
        assert!(recorte.lineas_totales > 20);
        // AC-9: el recorte se DECLARA, nunca es silencioso.
        assert!(p.render_texto().contains("[recortado]"));
        // AC-10: el tamaño se puede ver antes de gastarlo.
        let (lineas, tokens) = p.tamano();
        assert!(lineas > 0 && tokens > 0);
    }

    #[test]
    fn contexto_sin_nada() {
        // AC-15: sin mapa, sin grafo, sin historia y sin hub, el paquete sale
        // igual y declara cada hueco. Ninguna fuente es obligatoria.
        let dir = tempfile::tempdir().unwrap();
        let p = armar(&paths_de(dir.path()), None, "cualquier cosa", MAX_LINEAS_DEFAULT);
        assert!(p.mapa.is_none());
        assert!(p.faltantes.len() >= 3, "tres huecos minimos: {:?}", p.faltantes);
        let texto = p.render_texto();
        assert!(texto.contains("(no hay mapa de arquitectura)"));
        assert!(texto.contains("## Falta"));
        // Y el resumen de `start` tambien sale, que es cuando mas importa.
        assert!(p.resumen().contains("mapa: no hay"));
    }

    #[test]
    fn grafo_vencido_a_los_siete_dias() {
        // OBS-2: la decision del usuario fueron 7 dias, no 14.
        assert_eq!(GRAFO_VENCIDO_DIAS, 7);
        let dir = tempfile::tempdir().unwrap();
        let f = dir.path().join("graph.json");
        std::fs::write(&f, "{}").unwrap();
        let Some(dias) = edad_en_dias(&f, SystemTime::now()) else {
            panic!("un archivo recien escrito tiene edad");
        };
        assert_eq!(dias, 0);
        assert!(edad_en_dias(&dir.path().join("no-esta.json"), SystemTime::now()).is_none());
    }
}
