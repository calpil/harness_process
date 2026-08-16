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
