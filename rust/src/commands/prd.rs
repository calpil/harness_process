//! `harness prd add` / `harness prd tree`: el arbol de PRDs anidados.

use crate::exit::Exit;
use crate::features::load_features;
use crate::paths::HarnessPaths;
use crate::prd;
use crate::progress::log;

/// `prd add --name <slug> [--parent <ruta>]`: crea el PRD hijo desde plantilla
/// y lo engancha en su padre. No pisa nada: si el destino existe, falla.
pub fn add(paths: &HarnessPaths, name: &str, parent: Option<&str>) -> anyhow::Result<()> {
    let segment = prd::normalize_segment(name)?;
    let parent_ref = parent.unwrap_or(prd::MASTER);
    let parent_prd = prd::resolve(paths, parent_ref)?;

    let mut segments: Vec<String> = parent_prd
        .segments()
        .iter()
        .map(|s| (*s).to_string())
        .collect();
    segments.push(segment);
    let chain: Vec<&str> = segments.iter().map(String::as_str).collect();
    let slug = chain.join("/");
    let file = prd::file_for(paths, &chain);
    if file.exists() {
        return Err(Exit::msg(format!(
            "Ese PRD ya existe: {} (los PRDs son documentos del USUARIO; el arnes no los pisa).",
            prd::rel_path(&slug)
        ))
        .into());
    }

    let dir = prd::dir_for(paths, &chain);
    std::fs::create_dir_all(&dir)?;
    crate::features::write_text_atomic(&file, &prd::child_template(&slug, &parent_prd.slug))?;
    let linked = prd::link_child(&parent_prd.file, &parent_prd.slug, &slug)?;

    let rel_child = prd::rel_path(&slug);
    let rel_parent = prd::rel_path(&parent_prd.slug);
    // Las dos escrituras las hizo el ARNES, no el agente, y las dos caen bajo
    // `docs/prd/**` (feature #26): se registran para que la red de seguridad no
    // las reporte como violacion.
    crate::commands::rutas::registrar_escritura_del_arnes(paths, &rel_child);
    crate::commands::rutas::registrar_escritura_del_arnes(paths, &rel_parent);
    log(
        paths,
        &format!("prd add {slug} (padre: {})", parent_prd.reference()),
    )?;
    // Feature #16 (AC-3): el PRD nuevo nace como epic sin esperar a que se le
    // cargue la primera feature, y el worker detached lo empuja solo.
    crate::atlassian::emit::on_prd_add(paths, &slug);
    crate::atlassian::push::push_bg(paths);
    println!("PRD anidado creado: {rel_child}");
    if linked {
        println!(
            "Enlazado en {rel_parent} (seccion \"{}\")",
            prd::CHILDREN_SECTION.trim_start_matches("## ")
        );
    } else {
        println!("Ya estaba enlazado en {rel_parent}.");
    }
    println!("  Contale su historia (antes/despues) y cargale hitos; despues, por cada hito:");
    println!(
        "  sh harness_cli add --name <slug> --service <servicio> --acceptance \"<criterio>\" --prd {slug}"
    );
    Ok(())
}

/// `prd tree [--prd <ref>]`: dibuja el arbol con hitos y estado de features.
pub fn tree(paths: &HarnessPaths, reference: Option<&str>) -> anyhow::Result<()> {
    let all = prd::scan(paths);
    if all.is_empty() {
        println!(
            "No hay PRDs todavia en {}.",
            prd::rel_path("").trim_end_matches("PRD-master.md")
        );
        println!(
            "  Empeza por el maestro (docs/prd/PRD-master.md) y despues partilo: sh harness_cli prd add --name <parte>"
        );
        return Ok(());
    }
    let root = prd::resolve(paths, reference.unwrap_or(prd::MASTER))?;
    let data = load_features(paths)?;
    print!("{}", prd::render_tree(paths, &data, &root));
    println!();
    println!("  hitos: filas de la tabla \"10. Hitos -> features\" de cada PRD.");
    println!(
        "  features: las que declaran ese PRD con --prd (las que no lo declaran cuentan para el maestro)."
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// Feature #29: que el PRD, el SDD y architecture.md no queden mintiendo.
//
// El agente PROPONE, el usuario APRUEBA, el binario ESCRIBE (D-1). El ritual es
// el mismo de `approve-spec`, y el `[GATE]` se calca a mano porque no hay
// funcion compartida: son diez lineas de println! + Exit::code(2).
// ---------------------------------------------------------------------------

use crate::documentos::{self, Bloque, Documento, Plan, Veredicto};

/// `prd propose --feature <id>`: siembra una pregunta cerrada por documento.
///
/// El ALCANCE lo calcula el binario desde el arbol real: si lo eligiera el
/// agente, "el SDD ya lo refleja" seria una afirmacion sin contraparte. Y las
/// senales `Presente en:` / `Ausente en:` las precomputa el binario para que el
/// agente no parta de cero.
pub fn propose(paths: &HarnessPaths, fid: &str) -> anyhow::Result<()> {
    let data = load_features(paths)?;
    let idx = crate::features::find_feature_index(&data, fid)?;
    let feature = crate::features::feature_at(&data, idx).clone();
    let paths = rutas_documentales(paths, &feature, fid)?;
    let alcance = documentos::alcance(&paths, &feature);
    if alcance.is_empty() {
        println!("No hay documentos que revisar: ni PRD, ni SDD, ni architecture.md.");
        return Ok(());
    }
    let destino = documentos::propuesta_path(&paths, fid);
    let mut previo = std::fs::read_to_string(&destino).unwrap_or_default();
    let ya = documentos::parsear(&previo);
    if documentos::sellada(&previo) && !documentos::aplicada(&alcance, &ya, &previo) {
        previo = documentos::invalidar_sello(&previo);
        println!("[i] Sello Aplicado invalidado: el texto aplicado ya no esta en los documentos.");
    }

    let nombre = feature
        .get("name")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default();
    // El candidato se calcula una vez por propuesta: todos los documentos
    // reciben la misma evidencia del cambio real, pero el usuario decide para
    // cada uno si y como ese cambio amerita documentarse.
    let candidato = candidato_del_diff(&paths, &feature);
    let mut texto = if previo.is_empty() {
        format!(
            "# Documentos al dia - Feature #{fid}: {nombre}\n\n\
             Contesta CADA bloque con uno de los tres veredictos y despues corre\n\
             `sh harness_cli prd apply --feature {fid}`:\n\n\
             - `Veredicto: cambio` + `Antes:` y `Despues:` (texto LITERAL del documento)\n\
             - `Veredicto: ya-esta <archivo>:<L1>-<L2>` (el binario verifica la cita)\n\
             - `Veredicto: no-aplica <razon>` (la razon no puede estar vacia)\n\n"
        )
    } else {
        // No se pisa lo ya contestado: solo se agregan los bloques que falten.
        previo.trim_end().to_string() + "\n\n"
    };

    let mut agregados = 0usize;
    for doc in &alcance {
        if ya.iter().any(|b| b.rel == doc.rel) {
            continue;
        }
        texto.push_str(&bloque_sembrado(
            doc,
            &paths,
            &feature,
            nombre,
            candidato.as_deref(),
        ));
        agregados += 1;
    }

    if let Some(padre) = destino.parent() {
        std::fs::create_dir_all(padre)?;
    }
    std::fs::write(&destino, &texto)?;

    let rel = documentos::propuesta_rel(fid);
    let pendientes = documentos::parsear(&texto)
        .into_iter()
        .filter(|b| !b.veredicto.resuelto())
        .count();
    if agregados > 0 {
        println!(
            "Propuesta {rel}: {agregados} bloque(s) sembrado(s) de {} documento(s).",
            alcance.len()
        );
    } else {
        println!("Propuesta {rel}: ya tenia los {} bloques.", alcance.len());
    }
    if pendientes == 0 {
        println!("Todos contestados. Seguí con: sh harness_cli prd apply --feature {fid}");
        return Ok(());
    }
    println!("Quedan {pendientes} sin contestar. Abrí {rel} y resolvelos.");
    Err(Exit {
        code: 2,
        message: None,
    }
    .into())
}

/// El bloque que el binario siembra, con las senales ya calculadas.
fn bloque_sembrado(
    doc: &Documento,
    paths: &HarnessPaths,
    feature: &serde_json::Value,
    nombre_feature: &str,
    candidato: Option<&str>,
) -> String {
    let (presente, ausente) = senales(doc, paths, feature, nombre_feature);
    let candidato = candidato
        .unwrap_or("(sin candidato: el diff de la feature no contiene cambios atribuibles)");
    format!(
        "## Documento: {}\n\n\
         Que cuenta: {}\n\
         Presente en: {}\n\
         Ausente en: {}\n\
         Candidato despues:\n{}\n\n\
         Veredicto: PENDIENTE\n\n",
        doc.rel, doc.que_cuenta, presente, ausente, candidato
    )
}

/// Resumen determinista y editable del cambio de la feature.
///
/// No lee el cuerpo del diff: un archivo de propuesta no debe convertirse en
/// un segundo paquete de revisión ni exponer salida arbitraria. Las rutas ya
/// bastan para que la persona sepa qué cambió y decida si el documento necesita
/// una actualización. Incluimos tanto commits de la rama como trabajo todavía
/// sin commitear, porque `prd propose` suele correrse antes del cierre.
fn candidato_del_diff(paths: &HarnessPaths, feature: &serde_json::Value) -> Option<String> {
    candidato_desde_rutas(&rutas_del_diff(paths, feature))
}

/// Rutas de la rama más sus cambios sin commitear. Es la fuente compartida de
/// candidatos y señales: ambas ayudas tienen que hablar del mismo cambio.
fn rutas_del_diff(paths: &HarnessPaths, feature: &serde_json::Value) -> Vec<String> {
    let dir = feature
        .get("worktree")
        .and_then(serde_json::Value::as_str)
        .map(std::path::PathBuf::from)
        .filter(|p| p.is_dir())
        .or_else(|| paths.worktree.clone())
        .unwrap_or_else(|| paths.repo_root.clone());
    let mut rutas: Vec<String> = Vec::new();
    if let Some(base) = crate::git::rama_base(&dir, None) {
        rutas.extend(git_lineas(
            &dir,
            &["diff", "--name-only", &format!("{base}...HEAD")],
        ));
    }
    rutas.extend(git_lineas(&dir, &["diff", "--name-only"]));
    rutas.extend(git_lineas(
        &dir,
        &["ls-files", "--others", "--exclude-standard"],
    ));
    rutas.sort();
    rutas.dedup();
    rutas.retain(|ruta| !es_artefacto_del_arnes(ruta));
    rutas
}

fn git_lineas(dir: &std::path::Path, args: &[&str]) -> Vec<String> {
    std::process::Command::new("git")
        .args(args)
        .current_dir(dir)
        .stdin(std::process::Stdio::null())
        .output()
        .ok()
        .filter(|out| out.status.success())
        .map(|out| String::from_utf8_lossy(&out.stdout).into_owned())
        .unwrap_or_default()
        .lines()
        .map(str::trim)
        .filter(|ruta| !ruta.is_empty())
        .map(str::to_string)
        .collect()
}

fn es_artefacto_del_arnes(ruta: &str) -> bool {
    matches!(
        ruta,
        ruta if ruta.starts_with("docs/plan-feature-")
            || ruta.starts_with("docs/spec-feature-")
            || ruta.starts_with("docs/impl-")
            || ruta.starts_with("docs/review-")
            || ruta.starts_with("docs/verify-")
            || ruta.starts_with("docs/prd-diff-")
            || ruta.starts_with("progress/")
    )
}

fn candidato_desde_rutas(rutas: &[String]) -> Option<String> {
    if rutas.is_empty() {
        return None;
    }
    const MAX_RUTAS: usize = 4;
    let mut out = String::from("- Cambio de la feature en: ");
    out.push_str(
        &rutas
            .iter()
            .take(MAX_RUTAS)
            .map(|ruta| format!("`{ruta}`"))
            .collect::<Vec<_>>()
            .join(", "),
    );
    if rutas.len() > MAX_RUTAS {
        out.push_str(&format!(" y {} ruta(s) más", rutas.len() - MAX_RUTAS));
    }
    out.push_str(". Revisa si este documento debe reflejarlo.");
    Some(out)
}

/// Busca evidencia textual del nombre, términos específicos del spec y módulos
/// del diff. Sigue siendo una ayuda: encontrar una palabra no equivale a que el
/// documento esté correcto ni permite resolver el veredicto.
fn senales(
    doc: &Documento,
    paths: &HarnessPaths,
    feature: &serde_json::Value,
    nombre_feature: &str,
) -> (String, String) {
    let Ok(texto) = std::fs::read_to_string(&doc.path) else {
        return ("(no se pudo leer)".to_string(), String::new());
    };
    let mut terminos: Vec<(String, String)> = Vec::new();
    let nombre = nombre_feature.trim().to_ascii_lowercase();
    if !nombre.is_empty() {
        terminos.push(("nombre".to_string(), nombre));
    }
    terminos.extend(
        terminos_del_spec(paths, feature)
            .into_iter()
            .map(|termino| ("spec".to_string(), termino)),
    );
    terminos.extend(
        modulos_de_rutas(&rutas_del_diff(paths, feature))
            .into_iter()
            .map(|termino| ("módulo".to_string(), termino)),
    );
    terminos.sort();
    terminos.dedup();

    let mut hallazgos = Vec::new();
    for (linea, texto_linea) in texto.lines().enumerate() {
        let minuscula = texto_linea.to_ascii_lowercase();
        for (origen, termino) in &terminos {
            if minuscula.contains(termino) {
                hallazgos.push(format!("{}:{} ({origen} `{termino}`)", doc.rel, linea + 1));
            }
        }
    }
    hallazgos.sort();
    hallazgos.dedup();
    const MAX_SENALES: usize = 3;
    if hallazgos.is_empty() {
        (
            "-".to_string(),
            format!("{} (sin señales de nombre, spec o módulo)", doc.rel),
        )
    } else {
        let mut presente = hallazgos
            .iter()
            .take(MAX_SENALES)
            .cloned()
            .collect::<Vec<_>>()
            .join(", ");
        if hallazgos.len() > MAX_SENALES {
            presente.push_str(&format!(" y {} más", hallazgos.len() - MAX_SENALES));
        }
        (presente, "-".to_string())
    }
}

fn terminos_del_spec(paths: &HarnessPaths, feature: &serde_json::Value) -> Vec<String> {
    let Some(feature) = feature.as_object() else {
        return Vec::new();
    };
    let Ok(texto) = std::fs::read_to_string(crate::spec::spec_path(paths, feature)) else {
        return Vec::new();
    };
    palabras_significativas(&texto)
}

fn modulos_de_rutas(rutas: &[String]) -> Vec<String> {
    rutas
        .iter()
        .flat_map(|ruta| palabras_significativas(ruta))
        .collect()
}

fn palabras_significativas(texto: &str) -> Vec<String> {
    const RUIDO: &[&str] = &[
        "acuerdo",
        "agente",
        "alcance",
        "arquitectura",
        "archivo",
        "cambio",
        "cambios",
        "candado",
        "codigo",
        "comando",
        "como",
        "constitucion",
        "criterios",
        "despues",
        "documento",
        "documentos",
        "donde",
        "entonces",
        "escribe",
        "estado",
        "feature",
        "harness",
        "historia",
        "implementacion",
        "metodo",
        "nunca",
        "objetivo",
        "observabilidad",
        "plan",
        "propuesta",
        "pruebas",
        "resultado",
        "seguridad",
        "siempre",
        "usuario",
        "veredicto",
    ];
    let mut palabras = texto
        .split(|c: char| !c.is_alphanumeric())
        .map(|palabra| palabra.to_ascii_lowercase())
        .filter(|palabra| palabra.len() >= 6 && !RUIDO.contains(&palabra.as_str()))
        .collect::<Vec<_>>();
    palabras.sort();
    palabras.dedup();
    palabras
}

/// `prd apply --feature <id> [--yes]`: valida, muestra y —solo con el SI del
/// usuario— escribe.
pub fn apply(paths: &HarnessPaths, fid: &str, yes: bool) -> anyhow::Result<()> {
    let data = load_features(paths)?;
    let idx = crate::features::find_feature_index(&data, fid)?;
    let feature = crate::features::feature_at(&data, idx).clone();
    let paths = rutas_documentales(paths, &feature, fid)?;
    let alcance = documentos::alcance(&paths, &feature);
    let destino = documentos::propuesta_path(&paths, fid);
    let rel = documentos::propuesta_rel(fid);

    let Ok(mut texto) = std::fs::read_to_string(&destino) else {
        println!("[GATE] No existe la propuesta: {rel}");
        println!("    Sembrala con: sh harness_cli prd propose --feature {fid}");
        return Err(Exit {
            code: 2,
            message: None,
        }
        .into());
    };

    let bloques = documentos::parsear(&texto);
    if documentos::sellada(&texto) && !documentos::aplicada(&alcance, &bloques, &texto) {
        texto = documentos::invalidar_sello(&texto);
        std::fs::write(&destino, &texto)?;
        println!("[i] Sello Aplicado invalidado: el texto aplicado ya no esta en los documentos.");
    }
    let plan = documentos::planificar(&alcance, &bloques, raiz_documental(&paths));
    if !plan.aplicable() {
        println!("[GATE] La propuesta {rel} todavia no se puede aplicar:");
        for p in &plan.problemas {
            println!("    {}", p.mensaje());
        }
        return Err(Exit {
            code: 2,
            message: None,
        }
        .into());
    }

    // Ya aplicada e idempotente: nada que hacer, y se dice.
    if plan.escrituras.is_empty() && documentos::aplicada(&alcance, &bloques, &texto) {
        println!("La propuesta {rel} ya estaba aplicada. Nada que escribir.");
        return Ok(());
    }

    mostrar(&plan, &bloques, &rel);

    if !yes {
        println!();
        println!("[GATE] prd apply exige la confirmacion explicita del USUARIO.");
        println!("    Son SUS documentos: el arnes propone, el usuario decide.");
        println!("      1) Mostrale lo de arriba en el chat y abrile {rel} en su editor.");
        println!("      2) Preguntale si lo aprueba.");
        println!("      3) Solo con su SI: sh harness_cli prd apply --feature {fid} --yes");
        return Err(Exit {
            code: 2,
            message: None,
        }
        .into());
    }

    for e in &plan.escrituras {
        crate::features::write_text_atomic(&e.path, &e.contenido)?;
        // El binario escribe en docs/prd/**, que la feature #26 protege de las
        // herramientas del AGENTE. Se registra para no dispararse a si mismo la
        // red de seguridad, igual que `close` y `prd add`.
        crate::commands::rutas::registrar_escritura_del_arnes(&paths, &e.rel);
        println!("Escrito: {}", e.rel);
    }

    let stamp = crate::progress::now_stamp();
    let sello = format!(
        "{} {stamp} por USUARIO (confirmacion explicita)",
        documentos::SELLO
    );
    let sellado = if documentos::sellada(&texto) {
        texto
    } else {
        format!("{sello}\n\n{texto}")
    };
    std::fs::write(&destino, sellado)?;
    log(
        &paths,
        &format!(
            "prd apply feature #{fid} documentos={} escritos={}",
            alcance.len(),
            plan.escrituras.len()
        ),
    )?;
    println!("\n{sello}");
    println!(
        "Propuesta aplicada: {} documento(s) escrito(s).",
        plan.escrituras.len()
    );
    Ok(())
}

/// Todos los documentos de una propuesta pertenecen a un solo arbol. La
/// feature ya guarda su worktree al iniciar; resolverlo aca —antes del alcance
/// y del destino— evita leer el principal y escribir la rama (o al reves).
///
/// Sin un worktree usable se conserva el modo clasico y se lo hace visible:
/// nunca se busca un arbol alternativo ni se cambia el CWD silenciosamente.
fn rutas_documentales(
    paths: &HarnessPaths,
    feature: &serde_json::Value,
    fid: &str,
) -> anyhow::Result<HarnessPaths> {
    let Some(meta) = feature.as_object() else {
        anyhow::bail!("feature_list.json: feature invalida");
    };
    let worktree_valido = meta
        .get("worktree")
        .and_then(serde_json::Value::as_str)
        .is_some_and(|worktree| std::path::Path::new(worktree).is_dir());
    if !worktree_valido {
        println!("[i] Feature #{fid} sin worktree valido: uso el docs de la raiz efectiva.");
    }
    Ok(paths.para_feature(meta))
}

/// `plans` siempre es el `docs/` del arbol seleccionado; las citas usan rutas
/// relativas al repositorio (`docs/...`), por lo que su raiz es el padre de ese
/// directorio y no necesariamente `repo_root` (que conserva el principal para
/// el estado global).
fn raiz_documental(paths: &HarnessPaths) -> &std::path::Path {
    paths.plans.parent().unwrap_or(&paths.repo_root)
}

/// Lo que el usuario tiene que poder leer en 30 segundos.
fn mostrar(plan: &Plan, bloques: &[Bloque], rel: &str) {
    println!("== Documentos al dia: {rel} ==\n");
    for b in bloques {
        let detalle = match &b.veredicto {
            Veredicto::Cambio { .. } if plan.ya_aplicados.contains(&b.rel) => {
                "ya aplicado".to_string()
            }
            Veredicto::Cambio { despues, .. } => {
                let primera = despues.lines().next().unwrap_or("").trim();
                format!("escribe: {}", recortar(primera, 70))
            }
            Veredicto::YaEsta {
                archivo,
                desde,
                hasta,
            } => {
                format!("ya documentado en {archivo}:{desde}-{hasta}")
            }
            Veredicto::NoAplica { razon } => format!("no aplica: {}", recortar(razon, 70)),
            Veredicto::Pendiente => "SIN CONTESTAR".to_string(),
        };
        println!(
            "  [{:<9}] {:<42} {}",
            b.veredicto.etiqueta(),
            b.rel,
            detalle
        );
    }
    if plan.escrituras.is_empty() {
        println!("\nNingun documento cambia.");
    } else {
        println!(
            "\nSe va a escribir en {} documento(s):",
            plan.escrituras.len()
        );
        for e in &plan.escrituras {
            println!("  {}", e.rel);
        }
    }
}

fn recortar(s: &str, n: usize) -> String {
    if s.chars().count() <= n {
        return s.to_string();
    }
    let corto: String = s.chars().take(n.saturating_sub(3)).collect();
    format!("{corto}...")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn candidato_de_rutas_es_acotado_y_muestra_el_excedente() {
        let rutas = (0..5)
            .map(|i| format!("rust/src/modulo-{i}.rs"))
            .collect::<Vec<_>>();
        let candidato = candidato_desde_rutas(&rutas);
        assert!(candidato.is_some(), "hay candidato");
        let candidato = candidato.unwrap_or_default();
        assert!(candidato.contains("`rust/src/modulo-0.rs`"), "{candidato}");
        assert!(candidato.contains("1 ruta(s) más"), "{candidato}");
        assert!(!candidato.contains("modulo-4.rs`"), "{candidato}");
    }

    #[test]
    fn artefactos_del_arnes_no_entran_en_el_candidato() {
        for ruta in [
            "docs/plan-feature-38-x.md",
            "docs/spec-feature-38-x.md",
            "docs/prd-diff-38.md",
            "progress/current-38.md",
        ] {
            assert!(es_artefacto_del_arnes(ruta), "{ruta}");
        }
        assert!(!es_artefacto_del_arnes("rust/src/commands/prd.rs"));
        assert!(candidato_desde_rutas(&[]).is_none());
    }
}
