//! El mapa de aprendizaje: los tres almacenes juntos, con sus enlaces y sus
//! huecos (feature #22, ultimo hito del PRD de aprendizaje).
//!
//! Dos decisiones que el codigo tiene que respetar:
//!
//! - **Solo lectura.** `construir()` LEE y devuelve el mapa; no existe ninguna
//!   funcion que escriba. La promesa es estructural, no una regla que recordar
//!   (leccion `promesas-estructurales-vs-disciplina`).
//! - **Sin hub.** Todo sale de archivos: `feature_list.json`, el frontmatter de
//!   las lecciones y el perfil (decision del usuario 2026-08-17, OBS-1).
//!
//! Lo que hace util al mapa no es el dibujo: son los **huecos**. Los tres
//! almacenes pueden estar sanos por separado y ser incoherentes entre si.

use serde_json::Value;

use crate::features::{features_slice, load_features};
use crate::lecciones::{self, Leccion};
use crate::paths::HarnessPaths;
use crate::perfil::Perfil;
use crate::pycompat::py_str;

/// Que clase de cosa es un nodo del mapa.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tipo {
    Feature,
    Leccion,
    LeccionArchivada,
    Perfil,
}

impl Tipo {
    pub fn etiqueta(self) -> &'static str {
        match self {
            Tipo::Feature => "feature",
            Tipo::Leccion => "leccion",
            Tipo::LeccionArchivada => "leccion-archivada",
            Tipo::Perfil => "perfil",
        }
    }
}

/// Por que dos nodos estan unidos. Son cuatro relaciones DISTINTAS: una feature
/// puede declarar una leccion al cerrar y ademas haber parido otra por el camino
/// (paso en la #17), y mostrar solo la declarada perderia la mitad.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Clase {
    /// La feature la declaro al cerrar (`close --leccion`).
    Declarada,
    /// La leccion nacio en esa feature (campo `origen`).
    Origen,
    /// La entrada del perfil cita esa feature (`(#n)`).
    Cita,
    /// Leccion relacionada con otra (`relacionadas`).
    Relacionada,
}

impl Clase {
    /// Cuando dos enlaces unen los MISMOS nodos, gana el de mayor prioridad: una
    /// leccion declarada al cerrar tambien tiene esa feature como `origen`, y
    /// mostrar las dos duplica la misma cosa. (Hallazgo de correr el mapa sobre
    /// el repo real.)
    pub fn prioridad(self) -> u8 {
        match self {
            Clase::Declarada => 3,
            Clase::Origen => 2,
            Clase::Cita => 1,
            Clase::Relacionada => 0,
        }
    }

    pub fn etiqueta(self) -> &'static str {
        match self {
            Clase::Declarada => "declarada",
            Clase::Origen => "origen",
            Clase::Cita => "cita",
            Clase::Relacionada => "relacionada",
        }
    }
}

/// Por que algo es un hueco. Cada variante obliga (por matcheo exhaustivo) a
/// tener su mensaje y su comando de correccion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Motivo {
    /// Cita a una feature que no existe en el backlog.
    EnlaceRoto,
    /// Feature cerrada como done sin declarar leccion ni `ninguna`.
    CierreSinLeccion,
    /// Leccion sin `origen`: no se sabe de donde salio.
    LeccionHuerfana,
    /// Archivo de leccion ilegible o con frontmatter roto.
    ArchivoIlegible,
}

impl Motivo {
    pub fn etiqueta(self) -> &'static str {
        match self {
            Motivo::EnlaceRoto => "enlace-roto",
            Motivo::CierreSinLeccion => "cierre-sin-leccion",
            Motivo::LeccionHuerfana => "leccion-huerfana",
            Motivo::ArchivoIlegible => "archivo-ilegible",
        }
    }

    /// Que comando corrige este hueco. Se imprime como TEXTO: `journey` no
    /// ejecuta nada ni tiene puerta propia para podar (OBS-2).
    pub fn remedio(self, sujeto: &str) -> String {
        match self {
            Motivo::EnlaceRoto => {
                format!("sh harness_cli leccion show {sujeto}   # corregi 'origen' a mano")
            }
            Motivo::CierreSinLeccion => {
                "sh harness_cli leccion list   # decidi si esa feature dejo algo".to_string()
            }
            Motivo::LeccionHuerfana => {
                format!("sh harness_cli leccion show {sujeto}   # agregale 'origen: [<id>]'")
            }
            Motivo::ArchivoIlegible => {
                format!("corregi {sujeto} a mano; el formato esta en la guia de lecciones")
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Nodo {
    pub tipo: Tipo,
    /// Identificador estable: `#17`, el nombre de la clase, o `perfil:<n>`.
    pub id: String,
    pub fecha: String,
    pub titulo: String,
    /// Linea de detalle (usos, citas, estado).
    pub detalle: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Enlace {
    pub desde: String,
    pub hacia: String,
    pub clase: Clase,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hueco {
    pub motivo: Motivo,
    pub sujeto: String,
    pub detalle: String,
}

#[derive(Debug, Clone, Default)]
pub struct Mapa {
    pub nodos: Vec<Nodo>,
    pub enlaces: Vec<Enlace>,
    pub huecos: Vec<Hueco>,
}

impl Mapa {
    /// Hijos que se RENDERIZAN bajo un nodo. Dos reglas, las dos salidas de
    /// correr el mapa sobre datos reales:
    ///
    /// - Si dos enlaces unen los mismos nodos, gana el de mayor prioridad
    ///   (declarada > origen): si no, la misma leccion sale dos veces.
    /// - Una entrada de perfil cuelga SOLO de la feature mas reciente que cita
    ///   (OBS-5), no de todas: si no, se repite en cada una.
    pub fn hijos(&self, nodo: &Nodo) -> Vec<(&Nodo, Clase)> {
        let mut out: Vec<(&Nodo, Clase)> = Vec::new();
        for enlace in self.enlaces.iter().filter(|e| e.desde == nodo.id) {
            let Some(hijo) = self.nodos.iter().find(|n| n.id == enlace.hacia) else {
                continue;
            };
            if hijo.tipo == Tipo::Perfil && hijo.fecha != nodo.fecha {
                continue;
            }
            match out.iter_mut().find(|(h, _)| h.id == hijo.id) {
                Some(existente) => {
                    if enlace.clase.prioridad() > existente.1.prioridad() {
                        existente.1 = enlace.clase;
                    }
                }
                None => out.push((hijo, enlace.clase)),
            }
        }
        out.sort_by_key(|(h, _)| h.id.clone());
        out
    }

    pub fn vacio(&self) -> bool {
        self.nodos.is_empty()
    }

    /// Nodos ordenados cronologicamente; los sin fecha van al final.
    pub fn cronologico(&self) -> Vec<&Nodo> {
        let mut out: Vec<&Nodo> = self.nodos.iter().collect();
        out.sort_by(|a, b| {
            let orden = |n: &Nodo| {
                if n.fecha.is_empty() {
                    "9999-99-99".to_string()
                } else {
                    n.fecha.clone()
                }
            };
            orden(a)
                .cmp(&orden(b))
                .then_with(|| a.tipo.etiqueta().cmp(b.tipo.etiqueta()))
                .then_with(|| a.id.cmp(&b.id))
        });
        out
    }
}

/// Ids de feature citados en un texto: `(#14, #16)` -> `["14", "16"]`.
/// Parseo de texto plano, sin regex sobre entrada del usuario.
pub fn citas(texto: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut resto = texto;
    while let Some(pos) = resto.find('#') {
        let despues = &resto[pos + 1..];
        let id: String = despues.chars().take_while(char::is_ascii_digit).collect();
        let largo = id.len();
        if !id.is_empty() && !out.contains(&id) {
            out.push(id);
        }
        resto = &despues[largo..];
    }
    out
}

/// Arma el mapa leyendo las tres fuentes. **No escribe nada.**
pub fn construir(paths: &HarnessPaths) -> Mapa {
    let mut mapa = Mapa::default();
    let data = load_features(paths).unwrap_or(Value::Null);

    // Desde cuando el proyecto USA lecciones: la fecha de cierre mas temprana
    // entre las features que declararon una. Antes de eso el concepto no existia,
    // asi que "cerro sin declarar" no es un hueco: es prehistoria. Sin esto, un
    // repo con historia previa reporta decenas de huecos que nadie puede
    // corregir, y un mapa que grita por cosas que estan bien se ignora.
    let usa_lecciones_desde = features_slice(&data)
        .iter()
        .filter(|f| {
            f.get("leccion")
                .and_then(Value::as_str)
                .is_some_and(|l| !l.is_empty())
        })
        // Timestamp COMPLETO, no la fecha truncada: la #15 y la #16 cerraron el
        // mismo dia que la #17 pero varias horas ANTES de que existiera la
        // maquinaria. Con granularidad de dia se reportaban como huecos.
        .filter_map(|f| f.get("closed_at").and_then(Value::as_str))
        .map(str::to_string)
        .min();

    // (a) Features cerradas.
    let mut ids_existentes: Vec<String> = Vec::new();
    for f in features_slice(&data) {
        let id = py_str(f.get("id"));
        if !id.is_empty() {
            ids_existentes.push(id.clone());
        }
        if f.get("status").and_then(Value::as_str) != Some("done") {
            continue;
        }
        let fecha = f
            .get("closed_at")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .chars()
            .take(10)
            .collect::<String>();
        let nombre = f.get("name").and_then(Value::as_str).unwrap_or_default();
        let declarada = f.get("leccion").and_then(Value::as_str).unwrap_or_default();
        let cerrada_en = f
            .get("closed_at")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        let posterior_a_la_maquinaria = usa_lecciones_desde
            .as_ref()
            .is_some_and(|desde| !cerrada_en.is_empty() && cerrada_en >= *desde);
        mapa.nodos.push(Nodo {
            tipo: Tipo::Feature,
            id: format!("#{id}"),
            fecha,
            titulo: nombre.to_string(),
            detalle: if declarada.is_empty() {
                "sin leccion declarada".to_string()
            } else {
                format!("leccion declarada: {declarada}")
            },
        });
        if declarada.is_empty() {
            if posterior_a_la_maquinaria {
                mapa.huecos.push(Hueco {
                    motivo: Motivo::CierreSinLeccion,
                    sujeto: format!("#{id}"),
                    detalle: format!("'{nombre}' cerro como done sin declarar leccion ni 'ninguna'"),
                });
            }
        } else if declarada != lecciones::NINGUNA {
            mapa.enlaces.push(Enlace {
                desde: format!("#{id}"),
                hacia: declarada.to_string(),
                clase: Clase::Declarada,
            });
        }
    }

    // (b) Lecciones activas y archivadas.
    let (activas, rotas) = lecciones::scan(paths);
    for (path, motivo) in rotas {
        mapa.huecos.push(Hueco {
            motivo: Motivo::ArchivoIlegible,
            sujeto: path.to_string_lossy().into_owned(),
            detalle: motivo,
        });
    }
    let archivadas = lecciones::scan_archivadas(paths);
    for (l, tipo) in activas
        .iter()
        .map(|l| (l, Tipo::Leccion))
        .chain(archivadas.iter().map(|l| (l, Tipo::LeccionArchivada)))
    {
        agregar_leccion(&mut mapa, l, tipo, &ids_existentes);
    }

    // (c) Entradas del perfil, ubicadas en la feature mas reciente que citan.
    for (i, entrada) in Perfil::load(paths).entradas().iter().enumerate() {
        let citadas = citas(entrada);
        let id = format!("perfil:{}", i + 1);
        let fecha = citadas
            .iter()
            .filter_map(|c| fecha_de_feature(&data, c))
            .max()
            .unwrap_or_default();
        mapa.nodos.push(Nodo {
            tipo: Tipo::Perfil,
            id: id.clone(),
            fecha,
            titulo: entrada.clone(),
            detalle: if citadas.is_empty() {
                "sin citas".to_string()
            } else {
                format!("cita: #{}", citadas.join(", #"))
            },
        });
        for c in &citadas {
            if ids_existentes.contains(c) {
                mapa.enlaces.push(Enlace {
                    desde: format!("#{c}"),
                    hacia: id.clone(),
                    clase: Clase::Cita,
                });
            } else {
                mapa.huecos.push(Hueco {
                    motivo: Motivo::EnlaceRoto,
                    sujeto: id.clone(),
                    detalle: format!("la entrada del perfil cita la feature #{c}, que no existe"),
                });
            }
        }
    }
    mapa
}

fn agregar_leccion(mapa: &mut Mapa, l: &Leccion, tipo: Tipo, ids: &[String]) {
    let origen = l.fm.list("origen");
    let uso = match l.ultimo_uso().as_str() {
        "" => "nunca usada".to_string(),
        fecha => format!("{} uso(s), ultimo {fecha}", l.usos()),
    };
    mapa.nodos.push(Nodo {
        tipo,
        id: l.nombre.clone(),
        // La leccion se ubica cuando se escribio.
        fecha: l.fm.get("ultima_actualizacion").unwrap_or_default(),
        titulo: l.descripcion(),
        detalle: uso,
    });
    if origen.is_empty() {
        mapa.huecos.push(Hueco {
            motivo: Motivo::LeccionHuerfana,
            sujeto: l.nombre.clone(),
            detalle: "no declara 'origen': no se sabe de que feature salio".to_string(),
        });
    }
    for o in &origen {
        if ids.contains(o) {
            mapa.enlaces.push(Enlace {
                desde: format!("#{o}"),
                hacia: l.nombre.clone(),
                clase: Clase::Origen,
            });
        } else {
            mapa.huecos.push(Hueco {
                motivo: Motivo::EnlaceRoto,
                sujeto: l.nombre.clone(),
                detalle: format!("declara 'origen: #{o}' y esa feature no existe"),
            });
        }
    }
    for r in l.fm.list("relacionadas") {
        mapa.enlaces.push(Enlace {
            desde: l.nombre.clone(),
            hacia: r,
            clase: Clase::Relacionada,
        });
    }
}

fn fecha_de_feature(data: &Value, id: &str) -> Option<String> {
    features_slice(data)
        .iter()
        .find(|f| py_str(f.get("id")) == id)
        .and_then(|f| f.get("closed_at").and_then(Value::as_str))
        .map(|s| s.chars().take(10).collect())
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;
    use serde_json::json;

    /// Sandbox con los tres almacenes poblados a medida.
    fn sandbox(
        features: Value,
        lecciones_: &[(&str, &str, &str)],
        perfil_: &[&str],
    ) -> (tempfile::TempDir, HarnessPaths) {
        let dir = tempfile::tempdir().unwrap();
        let paths = HarnessPaths::from_root(dir.path().to_path_buf());
        std::fs::create_dir_all(&paths.plans).unwrap();
        std::fs::write(
            &paths.features,
            serde_json::to_string_pretty(&features).unwrap(),
        )
        .unwrap();
        if !lecciones_.is_empty() {
            std::fs::create_dir_all(lecciones::dir(&paths)).unwrap();
        }
        for (nombre, origen, fecha) in lecciones_ {
            std::fs::write(
                lecciones::file_for(&paths, nombre),
                format!(
                    "---\nnombre: {nombre}\ndescripcion: Sobre {nombre}.\norigen: [{origen}]\n\
                     usos: 0\nultimo_uso:\nultima_actualizacion: {fecha}\nestado: activa\n---\n\ncuerpo\n"
                ),
            )
            .unwrap();
        }
        if !perfil_.is_empty() {
            let mut p = Perfil::parse(&crate::perfil::plantilla());
            for e in perfil_ {
                p.insertar(e);
            }
            p.save(&paths).unwrap();
        }
        (dir, paths)
    }

    fn features_cerradas() -> Value {
        json!({"features": [
            {"id": 17, "name": "lecciones", "status": "done",
             "closed_at": "2026-08-16T00:00:00Z", "leccion": "docs-generados"},
            {"id": 19, "name": "perfil", "status": "done",
             "closed_at": "2026-08-17T00:00:00Z", "leccion": "documentos-del-usuario"}
        ]})
    }

    #[test]
    fn citas_should_extract_feature_ids() {
        assert_eq!(citas("algo (#14, #16)"), ["14", "16"]);
        assert_eq!(citas("sin citas"), Vec::<String>::new());
        // No duplica.
        assert_eq!(citas("(#14) y otra vez (#14)"), ["14"]);
    }

    #[test]
    fn construir_should_link_the_declared_lesson() {
        let (_d, paths) = sandbox(features_cerradas(), &[("docs-generados", "17", "2026-08-16")], &[]);
        let m = construir(&paths);
        assert!(m.enlaces.contains(&Enlace {
            desde: "#17".to_string(),
            hacia: "docs-generados".to_string(),
            clase: Clase::Declarada,
        }));
    }

    #[test]
    fn construir_should_link_an_origin_lesson_that_was_not_declared() {
        // El caso real de la #17: declaro una y pario otra. Mostrar solo la
        // declarada perderia la mitad de lo aprendido.
        let (_d, paths) = sandbox(
            features_cerradas(),
            &[("docs-generados", "17", "2026-08-16"), ("hitos-del-prd", "17", "2026-08-16")],
            &[],
        );
        let m = construir(&paths);
        let de_17: Vec<&Enlace> = m.enlaces.iter().filter(|e| e.desde == "#17").collect();
        assert_eq!(de_17.len(), 3, "declarada + 2 de origen: {de_17:?}");
        assert!(de_17.iter().any(|e| e.clase == Clase::Declarada));
        assert!(de_17.iter().any(|e| e.clase == Clase::Origen && e.hacia == "hitos-del-prd"));
    }

    #[test]
    fn construir_should_place_a_profile_entry_on_its_most_recent_citation() {
        let (_d, paths) = sandbox(features_cerradas(), &[], &["Algo. (#17, #19)"]);
        let m = construir(&paths);
        let nodo = m.nodos.iter().find(|n| n.tipo == Tipo::Perfil).unwrap();
        // #19 cerro despues que #17.
        assert_eq!(nodo.fecha, "2026-08-17");
        assert!(nodo.detalle.contains("#17, #19"));
        assert_eq!(m.enlaces.iter().filter(|e| e.clase == Clase::Cita).count(), 2);
    }

    #[test]
    fn construir_should_report_a_broken_link_from_a_lesson() {
        let (_d, paths) = sandbox(features_cerradas(), &[("huerfana", "99", "2026-08-17")], &[]);
        let m = construir(&paths);
        let h = m.huecos.iter().find(|h| h.motivo == Motivo::EnlaceRoto).unwrap();
        assert_eq!(h.sujeto, "huerfana");
        assert!(h.detalle.contains("#99"));
    }

    #[test]
    fn construir_should_report_a_broken_link_from_the_profile() {
        let (_d, paths) = sandbox(features_cerradas(), &[], &["Algo viejo. (#99)"]);
        let m = construir(&paths);
        assert!(m.huecos.iter().any(|h| h.motivo == Motivo::EnlaceRoto
            && h.detalle.contains("#99")));
    }

    #[test]
    fn construir_should_report_a_close_without_a_lesson() {
        // La maquinaria tiene que estar EN USO para que "sin leccion" sea hueco:
        // por eso el fixture incluye una feature que si declaro.
        let features = json!({"features": [
            {"id": 17, "name": "primera", "status": "done",
             "closed_at": "2026-08-16T00:00:00Z", "leccion": "docs-generados"},
            {"id": 18, "name": "olvidadiza", "status": "done", "closed_at": "2026-08-17T00:00:00Z"}
        ]});
        let (_d, paths) = sandbox(features, &[("docs-generados", "17", "2026-08-16")], &[]);
        let m = construir(&paths);
        let h = m
            .huecos
            .iter()
            .find(|h| h.motivo == Motivo::CierreSinLeccion)
            .unwrap();
        assert_eq!(h.sujeto, "#18");
    }

    #[test]
    fn construir_should_not_report_closes_from_before_the_machinery_existed() {
        // Hallazgo real: el repo reportaba 16 huecos por las features #1..#16,
        // que cerraron ANTES de que existieran las lecciones. Ninguna se puede
        // corregir, y un mapa que grita por cosas que estan bien se ignora.
        let features = json!({"features": [
            {"id": 5, "name": "vieja", "status": "done", "closed_at": "2026-07-24T00:00:00Z"},
            {"id": 17, "name": "primera-con-leccion", "status": "done",
             "closed_at": "2026-08-16T00:00:00Z", "leccion": "docs-generados"},
            {"id": 18, "name": "posterior-sin-leccion", "status": "done",
             "closed_at": "2026-08-17T00:00:00Z"}
        ]});
        let (_d, paths) = sandbox(features, &[("docs-generados", "17", "2026-08-16")], &[]);
        let m = construir(&paths);
        let sin_leccion: Vec<&Hueco> = m
            .huecos
            .iter()
            .filter(|h| h.motivo == Motivo::CierreSinLeccion)
            .collect();
        assert_eq!(sin_leccion.len(), 1, "solo la posterior: {sin_leccion:?}");
        assert_eq!(sin_leccion[0].sujeto, "#18");
    }

    #[test]
    fn construir_should_use_full_timestamps_not_just_dates() {
        // Hallazgo real: la #15 y la #16 cerraron el MISMO DIA que la #17 pero
        // horas antes. Con granularidad de dia salian como huecos.
        let features = json!({"features": [
            {"id": 16, "name": "misma-fecha-antes", "status": "done",
             "closed_at": "2026-08-16T05:36:00Z"},
            {"id": 17, "name": "la-que-creo-la-maquinaria", "status": "done",
             "closed_at": "2026-08-16T20:00:00Z", "leccion": "docs-generados"}
        ]});
        let (_d, paths) = sandbox(features, &[("docs-generados", "17", "2026-08-16")], &[]);
        let m = construir(&paths);
        assert!(
            !m.huecos.iter().any(|h| h.motivo == Motivo::CierreSinLeccion),
            "la #16 cerro antes: no es hueco. {:?}",
            m.huecos
        );
    }

    #[test]
    fn construir_should_report_nothing_when_the_project_never_used_lessons() {
        // Un repo que no usa lecciones no tiene huecos de este tipo.
        let features = json!({"features": [
            {"id": 1, "name": "a", "status": "done", "closed_at": "2026-01-01T00:00:00Z"},
            {"id": 2, "name": "b", "status": "done", "closed_at": "2026-01-02T00:00:00Z"}
        ]});
        let (_d, paths) = sandbox(features, &[], &[]);
        let m = construir(&paths);
        assert!(m.huecos.is_empty(), "{:?}", m.huecos);
    }

    #[test]
    fn construir_should_report_an_orphan_lesson() {
        let (_d, paths) = sandbox(features_cerradas(), &[("suelta", "", "2026-08-17")], &[]);
        let m = construir(&paths);
        assert!(m.huecos.iter().any(|h| h.motivo == Motivo::LeccionHuerfana
            && h.sujeto == "suelta"));
    }

    #[test]
    fn construir_should_report_an_unreadable_lesson() {
        let (_d, paths) = sandbox(features_cerradas(), &[("ok", "17", "2026-08-17")], &[]);
        std::fs::write(lecciones::dir(&paths).join("rota.md"), "sin frontmatter\n").unwrap();
        let m = construir(&paths);
        assert!(m.huecos.iter().any(|h| h.motivo == Motivo::ArchivoIlegible));
    }

    #[test]
    fn construir_should_find_no_gaps_in_a_coherent_repo() {
        let (_d, paths) = sandbox(
            features_cerradas(),
            &[("docs-generados", "17", "2026-08-16"), ("documentos-del-usuario", "19", "2026-08-17")],
            &["Algo. (#17, #19)"],
        );
        let m = construir(&paths);
        assert!(m.huecos.is_empty(), "no deberia haber huecos: {:?}", m.huecos);
    }

    #[test]
    fn hijos_should_not_show_the_same_lesson_twice() {
        // Hallazgo real: la #17 declaro `docs-generados` Y esa leccion la cita
        // como origen. Sin dedup, salia dos veces bajo la misma feature.
        let (_d, paths) = sandbox(features_cerradas(), &[("docs-generados", "17", "2026-08-16")], &[]);
        let m = construir(&paths);
        let f17 = m.nodos.iter().find(|n| n.id == "#17").unwrap();
        let hijos = m.hijos(f17);
        assert_eq!(hijos.len(), 1, "duplicada: {hijos:?}");
        assert_eq!(hijos[0].1, Clase::Declarada, "gana la relacion mas fuerte");
    }

    #[test]
    fn hijos_should_anchor_a_profile_entry_to_its_most_recent_feature() {
        // Hallazgo real: una entrada que cita (#17, #19) colgaba de las DOS.
        let (_d, paths) = sandbox(features_cerradas(), &[], &["Algo. (#17, #19)"]);
        let m = construir(&paths);
        let f17 = m.nodos.iter().find(|n| n.id == "#17").unwrap();
        let f19 = m.nodos.iter().find(|n| n.id == "#19").unwrap();
        assert!(m.hijos(f17).is_empty(), "no deberia colgar de la mas vieja");
        assert_eq!(m.hijos(f19).len(), 1, "cuelga de la mas reciente");
    }

    #[test]
    fn cronologico_should_order_by_date() {
        let (_d, paths) = sandbox(
            features_cerradas(),
            &[("tardia", "19", "2026-08-17"), ("temprana", "17", "2026-08-16")],
            &[],
        );
        let m = construir(&paths);
        let fechas: Vec<&str> = m.cronologico().iter().map(|n| n.fecha.as_str()).collect();
        let mut ordenadas = fechas.clone();
        ordenadas.sort_unstable();
        assert_eq!(fechas, ordenadas);
    }

    #[test]
    fn construir_should_be_empty_on_a_fresh_repo() {
        let (_d, paths) = sandbox(json!({"features": []}), &[], &[]);
        let m = construir(&paths);
        assert!(m.vacio());
        assert!(m.huecos.is_empty());
    }

    #[test]
    fn motivo_should_offer_a_command_for_every_gap() {
        // Matcheo exhaustivo: agregar un motivo obliga a darle su remedio.
        for motivo in [
            Motivo::EnlaceRoto,
            Motivo::CierreSinLeccion,
            Motivo::LeccionHuerfana,
            Motivo::ArchivoIlegible,
        ] {
            assert!(!motivo.remedio("x").is_empty(), "{motivo:?} sin remedio");
        }
    }
}
