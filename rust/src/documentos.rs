//! Que los documentos de producto y diseno no queden mintiendo (feature #29).
//!
//! El arnes ya verifica que el codigo cumpla el **spec** (feature #23). El
//! cuerpo del PRD, el SDD y `docs/architecture.md` no los miraba nadie — y no
//! por descuido: la regla "es del USUARIO" esta replicada en cuatro lugares del
//! repo. El problema es que "es del usuario" nunca quiso decir "que quede
//! mintiendo".
//!
//! La mecanica (decision del usuario 2026-08-17, D-1): **el agente PROPONE, el
//! usuario APRUEBA, el binario ESCRIBE**. Mismo ritual que `approve-spec`.
//!
//! Tres decisiones de diseno que salieron de refutar el diseno contra el codigo,
//! y que hay que entender antes de tocar esto:
//!
//! 1. **El anclaje es por TEXTO LITERAL, no por seccion.** `prd::echo_close`
//!    corta secciones con `starts_with("## ")` (`prd.rs:629`), y
//!    `docs/architecture.md` tiene tres encabezados `###` que ese predicado se
//!    tragaria enteros.
//! 2. **La idempotencia sale del CONTENIDO, no de una firma.** El spec es 1:1
//!    con su feature y por eso se puede firmar (`last_spec_sig`); un PRD lo
//!    comparten N features, asi que una firma por feature mentiria desde la
//!    segunda.
//! 3. **El gate NO usa frescura contra el reporte de `verify`.** `verify`
//!    reescribe su reporte en cada corrida y `prd apply` es idempotente: exigir
//!    `mtime(propuesta) >= mtime(reporte)` dejaria la propuesta vieja para
//!    siempre, sin ningun comando capaz de refrescarla.
//!
//! Todo lo de este modulo **lee y decide**; escribir es del comando. El modulo
//! no importa nada que escriba (leccion `promesas-estructurales-vs-disciplina`).

use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::paths::HarnessPaths;

/// Nombre del SDD maestro, hermano del PRD maestro.
pub const SDD: &str = "SDD-master.md";
/// El mapa de lo que YA existe.
pub const ARCHITECTURE: &str = "docs/architecture.md";

/// Un documento del alcance: que archivo es y como se lo nombra.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Documento {
    /// Ruta relativa a la raiz del repo (`docs/prd/PRD-master.md`).
    pub rel: String,
    /// Ruta absoluta.
    pub path: PathBuf,
    /// Que aporta ese documento, para que el bloque diga que se le pregunta.
    pub que_cuenta: &'static str,
}

/// Los documentos que una feature puede haber dejado desactualizados.
///
/// Sale del **arbol real** (`prd::feature_prd_slug` + `prd::segments`), no de una
/// lista escrita a mano: el PRD de origen, todos sus padres hasta el maestro, el
/// SDD y `architecture.md`. Los que no existen se omiten sin fallar.
///
/// Funcion **pura** salvo por comprobar que el archivo exista.
pub fn alcance(paths: &HarnessPaths, feature: &Value) -> Vec<Documento> {
    let mut out: Vec<Documento> = Vec::new();
    let slug = crate::prd::feature_prd_slug(feature);
    let segs: Vec<&str> = crate::prd::segments(&slug);

    // Del mas especifico al maestro: el PRD de origen primero, despues sus
    // padres. `docs/prd/a/b` -> [a,b], [a], [] (el maestro).
    for corte in (0..=segs.len()).rev() {
        let cadena: Vec<&str> = segs[..corte].to_vec();
        let path = crate::prd::file_for(paths, &cadena);
        let rel = crate::prd::rel_path(&cadena.join("/"));
        if path.is_file() && !out.iter().any(|d| d.rel == rel) {
            out.push(Documento {
                rel,
                path,
                que_cuenta: "que se construye y por que",
            });
        }
    }

    let sdd = crate::prd::prd_dir(paths).join(SDD);
    if sdd.is_file() {
        out.push(Documento {
            rel: format!("docs/prd/{SDD}"),
            path: sdd,
            que_cuenta: "como se construye, a nivel proyecto",
        });
    }

    // Feature #49: se resuelve contra el `docs/` de la FEATURE — el mismo del
    // que salen el PRD y el SDD — y no contra la raiz. Con worktree (feature
    // #47) eso hace que el cambio viaje con el merge de su rama; sin worktree,
    // `plans` ES el docs/ de la raiz y el comportamiento no cambia.
    let arch = paths.plans.join(
        ARCHITECTURE
            .strip_prefix("docs/")
            .unwrap_or(ARCHITECTURE),
    );
    if arch.is_file() {
        out.push(Documento {
            rel: ARCHITECTURE.to_string(),
            path: arch,
            que_cuenta: "el mapa de lo que YA existe",
        });
    }
    out
}

/// Como quedo contestado un bloque.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Veredicto {
    /// Todavia sin contestar.
    Pendiente,
    /// Hay que escribir: reemplazar `antes` por `despues`, literal.
    Cambio { antes: String, despues: String },
    /// Ya esta documentado, y aca esta la cita que lo demuestra.
    YaEsta { archivo: String, desde: usize, hasta: usize },
    /// No aplica, y por que.
    NoAplica { razon: String },
}

impl Veredicto {
    pub fn etiqueta(&self) -> &'static str {
        match self {
            Veredicto::Pendiente => "PENDIENTE",
            Veredicto::Cambio { .. } => "cambio",
            Veredicto::YaEsta { .. } => "ya-esta",
            Veredicto::NoAplica { .. } => "no-aplica",
        }
    }

    /// Un bloque sin contestar no deja cerrar.
    pub fn resuelto(&self) -> bool {
        !matches!(self, Veredicto::Pendiente)
    }
}

/// Un bloque de la propuesta: un documento y su veredicto.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bloque {
    pub rel: String,
    pub veredicto: Veredicto,
}

/// Marcador de inicio de bloque en `docs/prd-diff-<id>.md`.
const MARCA: &str = "## Documento: ";
/// Sello que deja `prd apply --yes`.
pub const SELLO: &str = "Aplicado:";

pub fn propuesta_path(paths: &HarnessPaths, fid: &str) -> PathBuf {
    paths.plans.join(format!("prd-diff-{fid}.md"))
}

pub fn propuesta_rel(fid: &str) -> String {
    format!("docs/prd-diff-{fid}.md")
}

/// Lee los bloques de una propuesta. **Pura.**
pub fn parsear(texto: &str) -> Vec<Bloque> {
    let mut out: Vec<Bloque> = Vec::new();
    let lineas: Vec<&str> = texto.lines().collect();
    let mut i = 0usize;
    while i < lineas.len() {
        let Some(rel) = lineas[i].strip_prefix(MARCA) else {
            i += 1;
            continue;
        };
        let rel = rel.trim().to_string();
        // El bloque llega hasta el proximo marcador o el fin.
        let fin = lineas[i + 1..]
            .iter()
            .position(|l| l.starts_with(MARCA))
            .map(|p| i + 1 + p)
            .unwrap_or(lineas.len());
        let cuerpo = &lineas[i + 1..fin];
        out.push(Bloque { rel, veredicto: leer_veredicto(cuerpo) });
        i = fin;
    }
    out
}

fn campo<'a>(cuerpo: &[&'a str], clave: &str) -> Option<&'a str> {
    cuerpo
        .iter()
        .find_map(|l| l.trim().strip_prefix(clave).map(str::trim))
}

/// Bloque literal multi-linea: desde la linea `<clave>` hasta el proximo campo
/// conocido. Permite que `Antes:`/`Despues:` lleven parrafos enteros.
fn bloque_literal(cuerpo: &[&str], clave: &str) -> Option<String> {
    let inicio = cuerpo.iter().position(|l| l.trim().starts_with(clave))?;
    let primera = cuerpo[inicio].trim().strip_prefix(clave)?.trim();
    let mut partes: Vec<String> = Vec::new();
    if !primera.is_empty() {
        partes.push(primera.to_string());
    }
    for l in &cuerpo[inicio + 1..] {
        let t = l.trim();
        if t.starts_with("Veredicto:")
            || t.starts_with("Antes:")
            || t.starts_with("Despues:")
            || t.starts_with("Presente en:")
            || t.starts_with("Ausente en:")
            || t.starts_with(MARCA)
        {
            break;
        }
        partes.push((*l).to_string());
    }
    let texto = partes.join("\n").trim().to_string();
    (!texto.is_empty()).then_some(texto)
}

fn leer_veredicto(cuerpo: &[&str]) -> Veredicto {
    let Some(v) = campo(cuerpo, "Veredicto:") else {
        return Veredicto::Pendiente;
    };
    let v = v.trim();
    if v.is_empty() || v.eq_ignore_ascii_case("PENDIENTE") {
        return Veredicto::Pendiente;
    }
    if let Some(resto) = v.strip_prefix("no-aplica") {
        let razon = resto.trim().to_string();
        return if razon.is_empty() {
            Veredicto::Pendiente // razon vacia no cuenta como contestado
        } else {
            Veredicto::NoAplica { razon }
        };
    }
    if let Some(resto) = v.strip_prefix("ya-esta") {
        if let Some((archivo, rango)) = resto.trim().rsplit_once(':')
            && let Some((d, h)) = rango.split_once('-')
            && let (Ok(desde), Ok(hasta)) = (d.trim().parse(), h.trim().parse())
        {
            return Veredicto::YaEsta {
                archivo: archivo.trim().to_string(),
                desde,
                hasta,
            };
        }
        return Veredicto::Pendiente; // cita mal formada: sin contestar
    }
    if v.starts_with("cambio") {
        let antes = bloque_literal(cuerpo, "Antes:");
        let despues = bloque_literal(cuerpo, "Despues:");
        if let (Some(antes), Some(despues)) = (antes, despues) {
            return Veredicto::Cambio { antes, despues };
        }
        return Veredicto::Pendiente; // `cambio` sin los dos textos no alcanza
    }
    Veredicto::Pendiente
}

/// Por que un bloque no se puede aplicar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Problema {
    SinResolver(String),
    /// El agente agrego, quito o renombro bloques.
    ListaAlterada { esperados: Vec<String>, encontrados: Vec<String> },
    /// La cita no dice lo que el bloque afirma.
    CitaFalsa { rel: String, cita: String, motivo: String },
    /// El `Antes:` no aparece en el documento (o aparece varias veces).
    AnclaNoEncontrada { rel: String, veces: usize },
}

impl Problema {
    pub fn mensaje(&self) -> String {
        match self {
            Problema::SinResolver(rel) => {
                format!("{rel}: sin resolver (Veredicto: PENDIENTE)")
            }
            Problema::ListaAlterada { esperados, encontrados } => format!(
                "la lista de documentos no coincide con el alcance real.\n      esperados: {}\n      encontrados: {}",
                esperados.join(", "),
                encontrados.join(", ")
            ),
            Problema::CitaFalsa { rel, cita, motivo } => {
                format!("{rel}: la cita `{cita}` no se sostiene ({motivo})")
            }
            Problema::AnclaNoEncontrada { rel, veces } => format!(
                "{rel}: el texto de `Antes:` aparece {veces} vez/veces (tiene que aparecer exactamente 1)"
            ),
        }
    }
}

/// Una escritura pendiente sobre un documento del usuario.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Escritura {
    pub rel: String,
    pub path: PathBuf,
    pub contenido: String,
}

/// Que se puede hacer con la propuesta tal como esta.
#[derive(Debug, Clone)]
pub struct Plan {
    pub escrituras: Vec<Escritura>,
    pub problemas: Vec<Problema>,
    /// Bloques ya aplicados (idempotencia por contenido).
    pub ya_aplicados: Vec<String>,
}

impl Plan {
    pub fn aplicable(&self) -> bool {
        self.problemas.is_empty()
    }
}

/// Valida la propuesta contra el alcance real y el disco, y devuelve el plan de
/// escritura. **No escribe nada**: quien escribe es el comando.
pub fn planificar(alcance: &[Documento], bloques: &[Bloque], repo_root: &Path) -> Plan {
    let esperados: Vec<String> = alcance.iter().map(|d| d.rel.clone()).collect();
    let encontrados: Vec<String> = bloques.iter().map(|b| b.rel.clone()).collect();
    let mut problemas: Vec<Problema> = Vec::new();
    let mut escrituras: Vec<Escritura> = Vec::new();
    let mut ya_aplicados: Vec<String> = Vec::new();

    // El agente no puede agregar, quitar ni renombrar bloques: si pudiera,
    // colapsaria N preguntas en una respuesta.
    if esperados != encontrados {
        problemas.push(Problema::ListaAlterada {
            esperados: esperados.clone(),
            encontrados,
        });
        return Plan { escrituras, problemas, ya_aplicados };
    }

    for (doc, bloque) in alcance.iter().zip(bloques.iter()) {
        match &bloque.veredicto {
            Veredicto::Pendiente => problemas.push(Problema::SinResolver(doc.rel.clone())),
            Veredicto::NoAplica { .. } => {}
            Veredicto::YaEsta { archivo, desde, hasta } => {
                if let Some(p) = verificar_cita(repo_root, &doc.rel, archivo, *desde, *hasta) {
                    problemas.push(p);
                }
            }
            Veredicto::Cambio { antes, despues } => {
                let Ok(texto) = std::fs::read_to_string(&doc.path) else {
                    problemas.push(Problema::AnclaNoEncontrada {
                        rel: doc.rel.clone(),
                        veces: 0,
                    });
                    continue;
                };
                // Idempotencia POR CONTENIDO: si el `despues` ya esta, el
                // bloque ya se aplico.
                //
                // La condicion NO puede pedir ademas que el `antes` haya
                // desaparecido, y esto se aprendio rompiendolo: el patron mas
                // comun es "insertar antes de esta linea", donde el `despues`
                // CONTIENE al `antes`. Con la version anterior
                // (`!contains(antes) && contains(despues)`) el `antes` seguia
                // presente despues de aplicar, el bloque no se reconocia como
                // aplicado, y la segunda corrida DUPLICABA el texto en el
                // documento del usuario. Paso en docs/architecture.md.
                if texto.contains(despues.as_str()) {
                    ya_aplicados.push(doc.rel.clone());
                    continue;
                }
                let veces = texto.matches(antes.as_str()).count();
                if veces != 1 {
                    problemas.push(Problema::AnclaNoEncontrada {
                        rel: doc.rel.clone(),
                        veces,
                    });
                    continue;
                }
                escrituras.push(Escritura {
                    rel: doc.rel.clone(),
                    path: doc.path.clone(),
                    contenido: texto.replacen(antes.as_str(), despues.as_str(), 1),
                });
            }
        }
    }
    Plan { escrituras, problemas, ya_aplicados }
}

/// La cita tiene que sostenerse contra el disco: el rango citado debe existir y
/// no estar vacio. Es lo que convierte "eso ya esta documentado" —la mentira mas
/// probable del agente— en algo refutable por maquina, sin heuristica.
fn verificar_cita(
    repo_root: &Path,
    rel_bloque: &str,
    archivo: &str,
    desde: usize,
    hasta: usize,
) -> Option<Problema> {
    let cita = format!("{archivo}:{desde}-{hasta}");
    let falla = |motivo: &str| {
        Some(Problema::CitaFalsa {
            rel: rel_bloque.to_string(),
            cita: cita.clone(),
            motivo: motivo.to_string(),
        })
    };
    if desde == 0 || hasta < desde {
        return falla("rango invalido");
    }
    let Ok(texto) = std::fs::read_to_string(repo_root.join(archivo)) else {
        return falla("el archivo citado no existe");
    };
    let lineas: Vec<&str> = texto.lines().collect();
    if hasta > lineas.len() {
        return falla(&format!("el archivo tiene {} lineas", lineas.len()));
    }
    let tramo = lineas[desde - 1..hasta].join("\n");
    if tramo.trim().is_empty() {
        return falla("el rango citado esta vacio");
    }
    None
}

/// Lee `rules.require_docs_al_dia` (default false: la regla nace apagada, como
/// las otras tres, para no romper instalaciones existentes).
pub fn require_docs_al_dia(data: &Value) -> bool {
    data.get("rules")
        .and_then(|r| r.get("require_docs_al_dia"))
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

/// `true` si la propuesta ya fue aplicada por el usuario.
pub fn aplicada(texto: &str) -> bool {
    texto.lines().any(|l| l.trim_start().starts_with(SELLO))
}

/// Gate de cierre (AC-17, AC-18). **Solo lee.**
///
/// NO compara frescura contra `docs/verify-<id>.md`: `verify` reescribe su
/// reporte en cada corrida y `prd apply` es idempotente, asi que esa regla
/// dejaria la propuesta vieja para siempre sin ningun comando capaz de
/// refrescarla. El deadlock se encontro refutando el diseno antes de escribirlo.
pub fn gate(
    paths: &HarnessPaths,
    data: &Value,
    status: &str,
    feature: &Value,
    fid: &str,
) -> Result<(), crate::exit::Exit> {
    use crate::exit::Exit;
    if status != "done" || !require_docs_al_dia(data) {
        return Ok(());
    }
    let rel = propuesta_rel(fid);
    let Ok(texto) = std::fs::read_to_string(propuesta_path(paths, fid)) else {
        return Err(Exit {
            code: 2,
            message: Some(format!(
                "[GATE] Falta la propuesta de documentos: {rel}.\n    \
                 La regla require_docs_al_dia esta activa: al cerrar, el PRD, el SDD y\n    \
                 architecture.md tienen que reflejar lo implementado.\n    \
                 Sembrala con: sh harness_cli prd propose --feature {fid}"
            )),
        });
    };
    let bloques = parsear(&texto);
    let sin_resolver: Vec<&Bloque> = bloques.iter().filter(|b| !b.veredicto.resuelto()).collect();
    if !sin_resolver.is_empty() {
        return Err(Exit {
            code: 2,
            message: Some(format!(
                "[GATE] La propuesta {rel} tiene {} bloque(s) sin resolver:\n    {}\n    \
                 Contesta cada uno con `cambio`, `ya-esta <archivo>:<L1>-<L2>` o\n    \
                 `no-aplica <razon>`, y despues: sh harness_cli prd apply --feature {fid}",
                sin_resolver.len(),
                sin_resolver
                    .iter()
                    .map(|b| b.rel.as_str())
                    .collect::<Vec<_>>()
                    .join("\n    ")
            )),
        });
    }
    // Decision del usuario 2026-08-18 (OBS-1): el gate exige la propuesta
    // APLICADA, o sea con el SI del usuario. Que este contestada no alcanza.
    if !aplicada(&texto) {
        let _ = feature; // reservado: el alcance ya se valido al aplicar
        return Err(Exit {
            code: 2,
            message: Some(format!(
                "[GATE] La propuesta {rel} esta contestada pero el USUARIO todavia no la aprobo.\n    \
                 Mostrasela y, con su SI: sh harness_cli prd apply --feature {fid} --yes"
            )),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;
    use serde_json::json;

    fn paths_en(dir: &Path) -> HarnessPaths {
        let harness = dir.join("hp");
        std::fs::create_dir_all(&harness).unwrap();
        std::fs::write(harness.join(".harness_layout"), "subdir").unwrap();
        HarnessPaths::from_root(harness)
    }

    fn sembrar_docs(paths: &HarnessPaths, con_sdd: bool, con_arch: bool) {
        let prd = crate::prd::prd_dir(paths);
        std::fs::create_dir_all(&prd).unwrap();
        std::fs::write(prd.join("PRD-master.md"), "# PRD\n\ncuerpo maestro\n").unwrap();
        if con_sdd {
            std::fs::write(prd.join(SDD), "# SDD\n\ncuerpo del sdd\n").unwrap();
        }
        if con_arch {
            // Feature #49: architecture.md vive en el mismo `docs/` que el PRD
            // y el SDD (el de la feature). Sin worktree, ese docs/ es el de la
            // raiz, asi que esto siembra donde siempre.
            std::fs::create_dir_all(&paths.plans).unwrap();
            std::fs::write(
                paths.plans.join("architecture.md"),
                "# Arquitectura\n\nlinea dos\nlinea tres\n",
            )
            .unwrap();
        }
    }

    #[test]
    fn architecture_should_come_from_the_feature_docs_not_the_repo_root() {
        // Feature #49 / AC-1 + AC-4: cuando la feature trabaja en un worktree,
        // `plans` apunta al docs/ DE ESA FEATURE. architecture.md tiene que
        // salir de ahi — como el PRD y el SDD — para viajar con el merge de su
        // rama. Si alguien vuelve a armar la ruta contra `repo_root`, este test
        // falla.
        let dir = tempfile::tempdir().unwrap();
        let mut paths = paths_en(dir.path());
        // Simula el worktree: su docs/ es otro directorio.
        let worktree_docs = dir.path().join("wt-49/docs");
        std::fs::create_dir_all(&worktree_docs).unwrap();
        paths.plans = worktree_docs.clone();
        sembrar_docs(&paths, true, true);

        // Y en la raiz hay un architecture.md VIEJO que no debe ganar.
        std::fs::create_dir_all(paths.repo_root.join("docs")).unwrap();
        std::fs::write(
            paths.repo_root.join(ARCHITECTURE),
            "# Arquitectura vieja\n",
        )
        .unwrap();

        let Some(arch) = alcance(&paths, &json!({"id": 49, "prd": "master"}))
            .into_iter()
            .find(|d| d.rel == ARCHITECTURE)
        else {
            panic!("architecture.md tiene que entrar al alcance");
        };
        assert_eq!(
            arch.path,
            worktree_docs.join("architecture.md"),
            "sale del docs/ de la feature, no de la raiz"
        );
        // La etiqueta relativa no cambia: es como se nombra el documento.
        assert_eq!(arch.rel, "docs/architecture.md");
    }

    #[test]
    fn architecture_should_fall_back_to_the_root_without_worktree() {
        // AC-3: sin worktree, `plans` ES el docs/ de la raiz: cero regresion.
        let dir = tempfile::tempdir().unwrap();
        let paths = paths_en(dir.path());
        sembrar_docs(&paths, true, true);
        let Some(arch) = alcance(&paths, &json!({"id": 1, "prd": "master"}))
            .into_iter()
            .find(|d| d.rel == ARCHITECTURE)
        else {
            panic!("architecture.md tiene que entrar al alcance");
        };
        assert_eq!(arch.path, paths.repo_root.join(ARCHITECTURE));
    }

    #[test]
    fn documentos_alcance_should_include_the_prd_chain_sdd_and_architecture() {
        let dir = tempfile::tempdir().unwrap();
        let paths = paths_en(dir.path());
        sembrar_docs(&paths, true, true);
        let rels: Vec<String> = alcance(&paths, &json!({"id": 1, "prd": "master"}))
            .into_iter()
            .map(|d| d.rel)
            .collect();
        assert_eq!(
            rels,
            vec![
                "docs/prd/PRD-master.md".to_string(),
                format!("docs/prd/{SDD}"),
                ARCHITECTURE.to_string(),
            ]
        );
    }

    #[test]
    fn documentos_alcance_should_walk_nested_prds_without_repeating() {
        // Un PRD anidado tiene que traer al hijo Y al maestro, en ese orden y
        // sin repetir: si el alcance perdiera al padre, el hito quedaria
        // marcado en un documento que nadie reviso.
        let dir = tempfile::tempdir().unwrap();
        let paths = paths_en(dir.path());
        sembrar_docs(&paths, false, false);
        let hijo = crate::prd::dir_for(&paths, &["aprendizaje"]);
        std::fs::create_dir_all(&hijo).unwrap();
        std::fs::write(hijo.join("PRD-aprendizaje.md"), "# hijo\n").unwrap();
        let rels: Vec<String> = alcance(&paths, &json!({"id": 17, "prd": "aprendizaje"}))
            .into_iter()
            .map(|d| d.rel)
            .collect();
        assert_eq!(
            rels,
            vec![
                "docs/prd/aprendizaje/PRD-aprendizaje.md".to_string(),
                "docs/prd/PRD-master.md".to_string(),
            ],
            "del mas especifico al maestro, sin repetir"
        );
    }

    #[test]
    fn documentos_alcance_should_skip_missing_documents() {
        // Una instalacion sin SDD ni architecture.md sigue funcionando.
        let dir = tempfile::tempdir().unwrap();
        let paths = paths_en(dir.path());
        sembrar_docs(&paths, false, false);
        let rels: Vec<String> = alcance(&paths, &json!({"id": 1, "prd": "master"}))
            .into_iter()
            .map(|d| d.rel)
            .collect();
        assert_eq!(rels, vec!["docs/prd/PRD-master.md".to_string()]);
    }

    #[test]
    fn parsear_should_read_the_three_verdicts() {
        let texto = format!(
            "# Propuesta\n\n\
             {MARCA}docs/prd/PRD-master.md\nVeredicto: no-aplica la feature no cambia el producto\n\n\
             {MARCA}docs/prd/{SDD}\nVeredicto: ya-esta docs/prd/{SDD}:3-5\n\n\
             {MARCA}{ARCHITECTURE}\nVeredicto: cambio\nAntes:\nlinea dos\nDespues:\nlinea dos y media\n"
        );
        let b = parsear(&texto);
        assert_eq!(b.len(), 3);
        assert!(matches!(b[0].veredicto, Veredicto::NoAplica { .. }));
        assert!(matches!(b[1].veredicto, Veredicto::YaEsta { desde: 3, hasta: 5, .. }));
        match &b[2].veredicto {
            Veredicto::Cambio { antes, despues } => {
                assert_eq!(antes, "linea dos");
                assert_eq!(despues, "linea dos y media");
            }
            otro => panic!("esperaba cambio, vino {otro:?}"),
        }
    }

    #[test]
    fn parsear_should_treat_half_answers_as_pending() {
        // `cambio` sin los dos textos, `no-aplica` sin razon y una cita mal
        // formada NO cuentan como contestados: si contaran, el gate se pasaria
        // con media respuesta.
        for cuerpo in [
            "Veredicto: cambio\nAntes:\nsolo el antes\n",
            "Veredicto: no-aplica\n",
            "Veredicto: ya-esta archivo-sin-rango\n",
            "Veredicto: PENDIENTE\n",
            "Veredicto:\n",
            "sin veredicto\n",
        ] {
            let texto = format!("{MARCA}docs/x.md\n{cuerpo}");
            let b = parsear(&texto);
            assert_eq!(b.len(), 1, "{cuerpo}");
            assert!(!b[0].veredicto.resuelto(), "deberia quedar pendiente: {cuerpo}");
        }
    }

    fn doc(rel: &str, path: PathBuf) -> Documento {
        Documento { rel: rel.to_string(), path, que_cuenta: "x" }
    }

    #[test]
    fn prd_apply_should_reject_a_tampered_block_list() {
        let dir = tempfile::tempdir().unwrap();
        let raiz = dir.path();
        let alc = vec![doc("docs/a.md", raiz.join("docs/a.md")), doc("docs/b.md", raiz.join("docs/b.md"))];
        let bloques = vec![Bloque {
            rel: "docs/a.md".to_string(),
            veredicto: Veredicto::NoAplica { razon: "x".into() },
        }];
        let plan = planificar(&alc, &bloques, raiz);
        assert!(!plan.aplicable());
        assert!(matches!(plan.problemas[0], Problema::ListaAlterada { .. }));
        assert!(plan.problemas[0].mensaje().contains("docs/b.md"));
    }

    #[test]
    fn prd_apply_should_replace_the_literal_anchor_not_the_section() {
        // El anclaje es por TEXTO: anclar por `## ` se tragaria los `###`.
        let dir = tempfile::tempdir().unwrap();
        let raiz = dir.path();
        std::fs::create_dir_all(raiz.join("docs")).unwrap();
        let f = raiz.join("docs/arch.md");
        std::fs::write(&f, "## Modulos\n\n- `viejo.rs`: hace algo\n\n### Sub\n\ncontenido de la subseccion\n").unwrap();
        let alc = vec![doc("docs/arch.md", f.clone())];
        let bloques = vec![Bloque {
            rel: "docs/arch.md".to_string(),
            veredicto: Veredicto::Cambio {
                antes: "- `viejo.rs`: hace algo".into(),
                despues: "- `viejo.rs`: hace algo\n- `nuevo.rs`: hace algo mas".into(),
            },
        }];
        let plan = planificar(&alc, &bloques, raiz);
        assert!(plan.aplicable(), "{:?}", plan.problemas);
        assert_eq!(plan.escrituras.len(), 1);
        let nuevo = &plan.escrituras[0].contenido;
        assert!(nuevo.contains("nuevo.rs"), "{nuevo}");
        assert!(
            nuevo.contains("### Sub") && nuevo.contains("contenido de la subseccion"),
            "se trago la subseccion ###: {nuevo}"
        );
    }

    #[test]
    fn prd_apply_should_be_idempotent_by_content() {
        // Sin firma `last_prd_sig`: el spec es 1:1 con su feature, pero un PRD
        // lo comparten N features y una firma por feature mentiria.
        let dir = tempfile::tempdir().unwrap();
        let raiz = dir.path();
        std::fs::create_dir_all(raiz.join("docs")).unwrap();
        let f = raiz.join("docs/arch.md");
        std::fs::write(&f, "# A\n\nya escrito\n").unwrap();
        let alc = vec![doc("docs/arch.md", f)];
        let bloques = vec![Bloque {
            rel: "docs/arch.md".to_string(),
            veredicto: Veredicto::Cambio {
                antes: "pendiente".into(),
                despues: "ya escrito".into(),
            },
        }];
        let plan = planificar(&alc, &bloques, raiz);
        assert!(plan.aplicable(), "{:?}", plan.problemas);
        assert!(plan.escrituras.is_empty(), "no deberia reescribir");
        assert_eq!(plan.ya_aplicados, ["docs/arch.md"]);
    }

    #[test]
    fn idempotence_should_hold_when_despues_contains_antes() {
        // El caso que rompio en produccion: "insertar antes de esta linea".
        // El `despues` contiene al `antes`, asi que despues de aplicar el
        // `antes` SIGUE presente. Si la idempotencia exigiera que desapareciera,
        // la segunda corrida duplicaria el texto en el documento del usuario.
        let dir = tempfile::tempdir().unwrap();
        let raiz = dir.path();
        std::fs::create_dir_all(raiz.join("docs")).unwrap();
        let f = raiz.join("docs/arch.md");
        let ancla = "- `progress.rs`: estado vivo";
        let nuevo = format!("- `doctor.rs`: diagnostico\n{ancla}");
        std::fs::write(&f, format!("# A\n\n{ancla}\n")).unwrap();

        let alc = vec![doc("docs/arch.md", f.clone())];
        let bloques = vec![Bloque {
            rel: "docs/arch.md".to_string(),
            veredicto: Veredicto::Cambio { antes: ancla.into(), despues: nuevo.clone() },
        }];

        // Primera corrida: escribe.
        let plan = planificar(&alc, &bloques, raiz);
        assert_eq!(plan.escrituras.len(), 1, "{:?}", plan.problemas);
        std::fs::write(&f, &plan.escrituras[0].contenido).unwrap();
        assert_eq!(
            std::fs::read_to_string(&f).unwrap().matches("doctor.rs").count(),
            1
        );

        // Segunda corrida: NO escribe, y no duplica.
        let plan = planificar(&alc, &bloques, raiz);
        assert!(
            plan.escrituras.is_empty(),
            "reaplico y habria duplicado: {:?}",
            plan.escrituras
        );
        assert_eq!(plan.ya_aplicados, ["docs/arch.md"]);
    }

    #[test]
    fn prd_apply_should_refuse_a_citation_that_does_not_hold() {
        let dir = tempfile::tempdir().unwrap();
        let raiz = dir.path();
        std::fs::create_dir_all(raiz.join("docs")).unwrap();
        std::fs::write(raiz.join("docs/a.md"), "uno\ndos\ntres\n").unwrap();
        let alc = vec![doc("docs/a.md", raiz.join("docs/a.md"))];
        // Rango fuera del archivo.
        let plan = planificar(
            &alc,
            &[Bloque {
                rel: "docs/a.md".into(),
                veredicto: Veredicto::YaEsta { archivo: "docs/a.md".into(), desde: 90, hasta: 99 },
            }],
            raiz,
        );
        assert!(!plan.aplicable());
        assert!(plan.problemas[0].mensaje().contains("3 lineas"), "{}", plan.problemas[0].mensaje());
        // Archivo inexistente.
        let plan = planificar(
            &alc,
            &[Bloque {
                rel: "docs/a.md".into(),
                veredicto: Veredicto::YaEsta { archivo: "docs/no-existe.md".into(), desde: 1, hasta: 1 },
            }],
            raiz,
        );
        assert!(plan.problemas[0].mensaje().contains("no existe"));
        // Rango valido: pasa.
        let plan = planificar(
            &alc,
            &[Bloque {
                rel: "docs/a.md".into(),
                veredicto: Veredicto::YaEsta { archivo: "docs/a.md".into(), desde: 1, hasta: 2 },
            }],
            raiz,
        );
        assert!(plan.aplicable(), "{:?}", plan.problemas);
    }

    #[test]
    fn prd_apply_should_accept_no_aplica_with_a_reason() {
        let dir = tempfile::tempdir().unwrap();
        let raiz = dir.path();
        let alc = vec![doc("docs/a.md", raiz.join("docs/a.md"))];
        let plan = planificar(
            &alc,
            &[Bloque {
                rel: "docs/a.md".into(),
                veredicto: Veredicto::NoAplica { razon: "la feature no toca el producto".into() },
            }],
            raiz,
        );
        assert!(plan.aplicable(), "{:?}", plan.problemas);
        assert!(plan.escrituras.is_empty());
    }

    #[test]
    fn prd_apply_should_name_the_unresolved_block() {
        let dir = tempfile::tempdir().unwrap();
        let raiz = dir.path();
        let alc = vec![doc("docs/a.md", raiz.join("docs/a.md"))];
        let plan = planificar(
            &alc,
            &[Bloque { rel: "docs/a.md".into(), veredicto: Veredicto::Pendiente }],
            raiz,
        );
        assert!(!plan.aplicable());
        assert!(plan.problemas[0].mensaje().contains("docs/a.md"));
        assert!(plan.problemas[0].mensaje().contains("sin resolver"));
    }

    #[test]
    fn anchor_that_appears_twice_should_be_refused() {
        // Escribir en el lugar equivocado de un documento del usuario es peor
        // que no escribir: ante ambiguedad, falla ruidoso.
        let dir = tempfile::tempdir().unwrap();
        let raiz = dir.path();
        std::fs::create_dir_all(raiz.join("docs")).unwrap();
        let f = raiz.join("docs/a.md");
        std::fs::write(&f, "repetido\notra cosa\nrepetido\n").unwrap();
        let plan = planificar(
            &[doc("docs/a.md", f)],
            &[Bloque {
                rel: "docs/a.md".into(),
                veredicto: Veredicto::Cambio { antes: "repetido".into(), despues: "unico".into() },
            }],
            raiz,
        );
        assert!(!plan.aplicable());
        assert!(plan.problemas[0].mensaje().contains("2 vez/veces"));
    }

    #[test]
    fn docs_gate_should_not_depend_on_verify_report_freshness() {
        // El deadlock que la refutacion encontro: `verify` reescribe su reporte
        // en cada corrida y `prd apply` es idempotente, asi que una regla de
        // frescura dejaria la propuesta vieja para siempre. El gate NO puede
        // mirar docs/verify-<id>.md.
        let dir = tempfile::tempdir().unwrap();
        let paths = paths_en(dir.path());
        std::fs::create_dir_all(&paths.plans).unwrap();
        let propuesta = propuesta_path(&paths, "1");
        std::fs::write(
            &propuesta,
            format!("{SELLO} 2026-08-18T00:00:00Z por USUARIO (confirmacion explicita)\n\n{MARCA}docs/a.md\nVeredicto: no-aplica x\n"),
        )
        .unwrap();
        // Un reporte de verify MAS NUEVO que la propuesta no puede bloquear.
        let reporte = paths.plans.join("verify-1.md");
        std::fs::write(&reporte, "# reporte\n").unwrap();
        let nuevo = filetime::FileTime::from_unix_time(2_000_000_000, 0);
        filetime::set_file_mtime(&reporte, nuevo).unwrap();
        let viejo = filetime::FileTime::from_unix_time(1_000_000_000, 0);
        filetime::set_file_mtime(&propuesta, viejo).unwrap();
        let data = json!({"rules": {"require_docs_al_dia": true}});
        assert!(
            gate(&paths, &data, "done", &json!({"id": 1}), "1").is_ok(),
            "el gate se deadlockeo por frescura contra el reporte de verify"
        );
    }

    #[test]
    fn gate_should_be_off_by_default_and_demand_the_seal_when_on() {
        let dir = tempfile::tempdir().unwrap();
        let paths = paths_en(dir.path());
        std::fs::create_dir_all(&paths.plans).unwrap();
        let f = json!({"id": 1});
        // Regla apagada: cerrar es lo de siempre aunque no haya propuesta.
        assert!(gate(&paths, &json!({}), "done", &f, "1").is_ok());
        let data = json!({"rules": {"require_docs_al_dia": true}});
        // Sin propuesta: bloquea nombrando el comando que la siembra.
        let err = gate(&paths, &data, "done", &f, "1").unwrap_err();
        assert_eq!(err.code, 2);
        assert!(err.message.unwrap().contains("prd propose --feature 1"));
        // Contestada pero SIN el si del usuario: bloquea (OBS-1).
        std::fs::write(
            propuesta_path(&paths, "1"),
            format!("{MARCA}docs/a.md\nVeredicto: no-aplica x\n"),
        )
        .unwrap();
        let err = gate(&paths, &data, "done", &f, "1").unwrap_err();
        assert!(err.message.unwrap().contains("todavia no la aprobo"));
        // blocked/pending no gatean: la valvula de escape sigue.
        assert!(gate(&paths, &data, "blocked", &f, "1").is_ok());
    }
}
