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
    log(paths, &format!("prd add {slug} (padre: {})", parent_prd.reference()))?;
    // Feature #16 (AC-3): el PRD nuevo nace como epic sin esperar a que se le
    // cargue la primera feature, y el worker detached lo empuja solo.
    crate::atlassian::emit::on_prd_add(paths, &slug);
    crate::atlassian::push::push_bg(paths);
    println!("PRD anidado creado: {rel_child}");
    if linked {
        println!("Enlazado en {rel_parent} (seccion \"{}\")", prd::CHILDREN_SECTION.trim_start_matches("## "));
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
        println!("No hay PRDs todavia en {}.", prd::rel_path("").trim_end_matches("PRD-master.md"));
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
    println!("  features: las que declaran ese PRD con --prd (las que no lo declaran cuentan para el maestro).");
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
    let previo = std::fs::read_to_string(&destino).unwrap_or_default();
    let ya = documentos::parsear(&previo);

    let nombre = feature.get("name").and_then(serde_json::Value::as_str).unwrap_or_default();
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
        texto.push_str(&bloque_sembrado(doc, nombre));
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
        println!("Propuesta {rel}: {agregados} bloque(s) sembrado(s) de {} documento(s).", alcance.len());
    } else {
        println!("Propuesta {rel}: ya tenia los {} bloques.", alcance.len());
    }
    if pendientes == 0 {
        println!("Todos contestados. Seguí con: sh harness_cli prd apply --feature {fid}");
        return Ok(());
    }
    println!("Quedan {pendientes} sin contestar. Abrí {rel} y resolvelos.");
    Err(Exit { code: 2, message: None }.into())
}

/// El bloque que el binario siembra, con las senales ya calculadas.
fn bloque_sembrado(doc: &Documento, nombre_feature: &str) -> String {
    let (presente, ausente) = senales(doc, nombre_feature);
    format!(
        "## Documento: {}\n\n\
         Que cuenta: {}\n\
         Presente en: {}\n\
         Ausente en: {}\n\
         Veredicto: PENDIENTE\n\n",
        doc.rel, doc.que_cuenta, presente, ausente
    )
}

/// Busca las senales de la feature en el documento. Es una ayuda, no un
/// veredicto: dice si el nombre de la feature aparece, no si el documento esta
/// bien.
fn senales(doc: &Documento, nombre_feature: &str) -> (String, String) {
    let Ok(texto) = std::fs::read_to_string(&doc.path) else {
        return ("(no se pudo leer)".to_string(), String::new());
    };
    let aguja = nombre_feature.trim();
    if aguja.is_empty() {
        return ("(sin nombre de feature)".to_string(), String::new());
    }
    let linea = texto
        .lines()
        .enumerate()
        .find(|(_, l)| l.contains(aguja))
        .map(|(i, _)| i + 1);
    match linea {
        Some(n) => (format!("{}:{n}", doc.rel), "-".to_string()),
        None => ("-".to_string(), format!("{} (no menciona '{aguja}')", doc.rel)),
    }
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

    let Ok(texto) = std::fs::read_to_string(&destino) else {
        println!("[GATE] No existe la propuesta: {rel}");
        println!("    Sembrala con: sh harness_cli prd propose --feature {fid}");
        return Err(Exit { code: 2, message: None }.into());
    };

    let bloques = documentos::parsear(&texto);
    let plan = documentos::planificar(&alcance, &bloques, raiz_documental(&paths));
    if !plan.aplicable() {
        println!("[GATE] La propuesta {rel} todavia no se puede aplicar:");
        for p in &plan.problemas {
            println!("    {}", p.mensaje());
        }
        return Err(Exit { code: 2, message: None }.into());
    }

    // Ya aplicada e idempotente: nada que hacer, y se dice.
    if plan.escrituras.is_empty() && documentos::aplicada(&texto) {
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
        return Err(Exit { code: 2, message: None }.into());
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
    let sellado = if documentos::aplicada(&texto) {
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
    println!("Propuesta aplicada: {} documento(s) escrito(s).", plan.escrituras.len());
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
            Veredicto::YaEsta { archivo, desde, hasta } => {
                format!("ya documentado en {archivo}:{desde}-{hasta}")
            }
            Veredicto::NoAplica { razon } => format!("no aplica: {}", recortar(razon, 70)),
            Veredicto::Pendiente => "SIN CONTESTAR".to_string(),
        };
        println!("  [{:<9}] {:<42} {}", b.veredicto.etiqueta(), b.rel, detalle);
    }
    if plan.escrituras.is_empty() {
        println!("\nNingun documento cambia.");
    } else {
        println!("\nSe va a escribir en {} documento(s):", plan.escrituras.len());
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
