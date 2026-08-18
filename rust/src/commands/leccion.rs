//! `harness leccion <list|show|nueva|usar>` (feature #17).
//!
//! Ningun subcomando abre conexion al hub (AC-9): las lecciones son archivos del
//! repositorio y el aprendizaje tiene que funcionar con el hub caido.

use serde_json::json;

use crate::exit::Exit;
use crate::features::{active_indices, feature_at, load_features};
use crate::lecciones::{self, Leccion};
use crate::paths::HarnessPaths;
use crate::pycompat::py_str;

/// Feature activa (si hay exactamente una) para sembrar `origen` al crear.
fn origen_activo(paths: &HarnessPaths) -> Option<String> {
    let data = load_features(paths).ok()?;
    let activas = active_indices(&data);
    let [idx] = activas.as_slice() else {
        return None;
    };
    let id = py_str(feature_at(&data, *idx).get("id"));
    (!id.is_empty()).then_some(id)
}

pub fn nueva(paths: &HarnessPaths, nombre: &str) -> anyhow::Result<()> {
    // El nombre se valida ANTES de tocar el filesystem: un nombre malo no deja
    // un archivo a medio crear.
    let slug = lecciones::validar_nombre_de_clase(nombre)?;
    let file = lecciones::file_for(paths, &slug);
    let rel = lecciones::rel_path(&slug);
    if file.exists() {
        // Crear es el ULTIMO recurso del orden de preferencia: si la clase ya
        // existe, lo correcto es patchearla.
        return Err(Exit {
            code: 2,
            message: Some(format!(
                "La leccion '{slug}' ya existe: {rel}\n    \
                 Patchea esa (mirala con 'sh harness_cli leccion show {slug}') en vez de crear otra:\n    \
                 la biblioteca busca POCAS lecciones de clase, ricas, no una lista plana."
            )),
        }
        .into());
    }
    std::fs::create_dir_all(lecciones::dir(paths))?;
    let origen = origen_activo(paths);
    crate::features::write_text_atomic(&file, &lecciones::plantilla(&slug, origen.as_deref()))?;
    println!("Leccion creada: {rel}");
    println!(
        "  Completa descripcion (una oracion, max {} caracteres) y triggers: son los",
        lecciones::DESCRIPCION_MAX
    );
    println!("  campos que deciden si alguien la encuentra dentro de seis meses.");
    println!("  Metodo: docs/{}/{}", lecciones::DIR_NAME, lecciones::GUIA);
    Ok(())
}

pub fn list(paths: &HarnessPaths, as_json: bool, archivadas: bool) -> anyhow::Result<()> {
    // Las archivadas no ensucian el catalogo por defecto (AC-19): se piden.
    if archivadas {
        let arch = lecciones::scan_archivadas(paths);
        if arch.is_empty() {
            println!("No hay lecciones archivadas.");
            return Ok(());
        }
        println!("Lecciones archivadas: {}", arch.len());
        for l in &arch {
            println!("  {:<40} {:>4} usos | ultimo: {}", l.nombre, l.usos(), match l.ultimo_uso().as_str() { "" => "nunca".to_string(), f => f.to_string() });
        }
        println!("\n  Siguen siendo consultables con 'buscar'. Vuelven con 'lecciones restaurar <clase>'.");
        return Ok(());
    }
    let (todas, rotas) = lecciones::scan(paths);
    if as_json {
        let rows: Vec<_> = todas
            .iter()
            .map(|l| {
                json!({
                    "nombre": l.nombre,
                    "descripcion": l.descripcion(),
                    "triggers": l.fm.list("triggers"),
                    "relacionadas": l.fm.list("relacionadas"),
                    "origen": l.fm.list("origen"),
                    "usos": l.usos(),
                    "ultimo_uso": l.ultimo_uso(),
                    "estado": l.estado(),
                })
            })
            .collect();
        let rotas_json: Vec<_> = rotas
            .iter()
            .map(|(p, motivo)| json!({"archivo": p.to_string_lossy(), "motivo": motivo}))
            .collect();
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({"lecciones": rows, "rotas": rotas_json}))?
        );
        return Ok(());
    }
    if todas.is_empty() && rotas.is_empty() {
        println!("Sin lecciones todavia.");
        println!("  Crea la primera cuando cierres algo que valga la pena repetir:");
        println!("    sh harness_cli leccion nueva <clase>");
        println!("  El nombre va a nivel de CLASE ('espejo-de-roles'), nunca por feature.");
        return Ok(());
    }
    println!("Lecciones: {} (por uso)", todas.len());
    for l in &todas {
        let uso = match l.ultimo_uso().as_str() {
            "" => "nunca".to_string(),
            fecha => fecha.to_string(),
        };
        println!(
            "  {:<28} {:>4} usos | {:<10} | {}",
            l.nombre,
            l.usos(),
            uso,
            l.estado()
        );
        let desc = l.descripcion();
        if !desc.is_empty() {
            println!("      {desc}");
        }
    }
    for (path, motivo) in &rotas {
        println!("  [!] {} ilegible: {motivo}", path.display());
    }
    Ok(())
}

pub fn show(paths: &HarnessPaths, nombre: &str) -> anyhow::Result<()> {
    let file = lecciones::file_for(paths, nombre);
    match Leccion::load(&file) {
        Ok(l) => {
            println!("{}", l.render().trim_end());
            Ok(())
        }
        Err(motivo) if file.exists() => Err(Exit {
            code: 2,
            message: Some(format!(
                "La leccion '{nombre}' esta ilegible: {motivo}.\n    \
                 Corregi {} a mano; el formato esta en docs/{}/{}.",
                lecciones::rel_path(nombre),
                lecciones::DIR_NAME,
                lecciones::GUIA
            )),
        }
        .into()),
        Err(_) => Err(no_existe(paths, nombre).into()),
    }
}

pub fn usar(paths: &HarnessPaths, nombre: &str) -> anyhow::Result<()> {
    let file = lecciones::file_for(paths, nombre);
    let mut leccion = match Leccion::load(&file) {
        Ok(l) => l,
        Err(motivo) if file.exists() => {
            return Err(Exit {
                code: 2,
                message: Some(format!("La leccion '{nombre}' esta ilegible: {motivo}.")),
            }
            .into());
        }
        Err(_) => return Err(no_existe(paths, nombre).into()),
    };
    leccion.registrar_uso();
    leccion.save()?;
    println!(
        "Uso registrado en {}: {} usos (ultimo: {}).",
        lecciones::rel_path(nombre),
        leccion.usos(),
        leccion.ultimo_uso()
    );
    Ok(())
}

/// Error de "no existe" con las clases mas parecidas: un typo tiene que sugerir
/// la leccion buena, no empujar a crear una duplicada.
pub fn no_existe(paths: &HarnessPaths, nombre: &str) -> Exit {
    let (todas, _) = lecciones::scan(paths);
    let mut msg = format!("No existe la leccion '{nombre}' ({}).", lecciones::rel_path(nombre));
    let cercanas = lecciones::parecidas(&todas, nombre);
    if !cercanas.is_empty() {
        msg.push_str(&format!("\n    ¿Quisiste decir? {}", cercanas.join(", ")));
    } else if !todas.is_empty() {
        let nombres: Vec<&str> = todas.iter().take(5).map(|l| l.nombre.as_str()).collect();
        msg.push_str(&format!("\n    Disponibles: {}", nombres.join(", ")));
    }
    msg.push_str("\n    Vela con 'sh harness_cli leccion list' o creala con 'leccion nueva'.");
    Exit {
        code: 2,
        message: Some(msg),
    }
}

// ---------------------------------------------------------------------------
// Curador (feature #21): `harness lecciones <status|curar|pin|...>`
// ---------------------------------------------------------------------------

use crate::curador::{self, Plan};
use crate::lecciones::{Transicion, Umbrales};
use crate::progress::{log, now_stamp};

/// Sin biblioteca no hay nada que curar: se informa y se sale con 0 (AC-2).
fn exigir_biblioteca(paths: &HarnessPaths) -> Option<()> {
    if lecciones::dir(paths).is_dir() {
        return Some(());
    }
    println!("Todavia no hay biblioteca de lecciones ({}/).", lecciones::DIR_NAME);
    println!("  Se crea sola con 'sh harness_cli leccion nueva <clase>'.");
    None
}

fn umbrales(paths: &HarnessPaths) -> Umbrales {
    load_features(paths)
        .map(|d| Umbrales::from_rules(&d))
        .unwrap_or_default()
}

/// Timestamp compacto para backups y reportes: `20260817-041530`.
fn ts() -> String {
    now_stamp()
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == 'T')
        .collect::<String>()
        .replace('T', "-")
}

pub fn status(paths: &HarnessPaths, as_json: bool) -> anyhow::Result<()> {
    if exigir_biblioteca(paths).is_none() {
        return Ok(());
    }
    let hoy = lecciones::hoy();
    let u = umbrales(paths);
    let (activas, _) = lecciones::scan(paths);
    let archivadas = lecciones::scan_archivadas(paths);

    // Proxima transicion de cada una: cuantos dias faltan y hacia donde.
    let fila = |l: &Leccion| {
        let dias = l.dias_inactiva(&hoy).unwrap_or(0);
        let (proxima, faltan) = if l.pinneada() {
            ("ninguna (pinneada)".to_string(), -1)
        } else if l.estado() == lecciones::ESTADO_STALE && u.archivo > 0 {
            ("archivada".to_string(), (u.archivo - dias).max(0))
        } else if u.stale > 0 {
            ("stale".to_string(), (u.stale - dias).max(0))
        } else {
            ("ninguna (umbral apagado)".to_string(), -1)
        };
        (dias, proxima, faltan)
    };

    if as_json {
        let rows: Vec<_> = activas
            .iter()
            .chain(archivadas.iter())
            .map(|l| {
                let (dias, proxima, faltan) = fila(l);
                serde_json::json!({
                    "nombre": l.nombre,
                    "estado": l.estado(),
                    "usos": l.usos(),
                    "ultimo_uso": l.ultimo_uso(),
                    "dias_inactiva": dias,
                    "pinneada": l.pinneada(),
                    "proxima_transicion": proxima,
                    "dias_para_transicion": faltan,
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&serde_json::json!({
            "lecciones": rows,
            "umbrales": {"stale_dias": u.stale, "archivo_dias": u.archivo},
            "hoy": hoy,
        }))?);
        return Ok(());
    }

    let plan = curador::planificar(paths, &hoy, u);
    println!(
        "Lecciones: {} activa(s), {} archivada(s). Umbrales: stale >= {}d, archivo >= {}d.",
        activas.len(),
        archivadas.len(),
        u.stale,
        u.archivo
    );
    for l in &activas {
        let (dias, proxima, faltan) = fila(l);
        let marca = if l.pinneada() { " [pin]" } else { "" };
        let cuando = if faltan < 0 {
            proxima.clone()
        } else {
            format!("{proxima} en {faltan}d")
        };
        println!(
            "  {:<40}{marca} {:>3} usos | {:>4}d inactiva | {} | -> {cuando}",
            l.nombre,
            l.usos(),
            dias,
            l.estado()
        );
    }
    for l in &archivadas {
        println!("  {:<40} [archivada]", l.nombre);
    }
    // El resumen que responde "¿tengo que hacer algo hoy?".
    let a_stale = plan.acciones.iter().filter(|a| a.transicion == Transicion::AStale).count();
    let a_archivo = plan.acciones.iter().filter(|a| a.transicion == Transicion::AArchivada).count();
    println!("\nCandidatas HOY: {a_stale} a stale, {a_archivo} a archivar.");
    if !plan.vacio() {
        println!("  Vealas con 'sh harness_cli lecciones curar' (solo informa).");
    }
    Ok(())
}

pub fn curar(paths: &HarnessPaths, aplicar: bool) -> anyhow::Result<()> {
    if exigir_biblioteca(paths).is_none() {
        return Ok(());
    }
    let hoy = lecciones::hoy();
    let u = umbrales(paths);
    let plan = curador::planificar(paths, &hoy, u);
    imprimir_plan(&plan);
    if plan.vacio() {
        println!("\nNada que hacer: ninguna leccion cambia de estado hoy.");
        return Ok(());
    }
    if !aplicar {
        // Modo informe: NO se toco un solo archivo (AC-9, OBS-3).
        println!("\nEsto es solo un informe: no se toco ningun archivo.");
        println!("  Para aplicarlo: sh harness_cli lecciones curar --aplicar");
        return Ok(());
    }
    let stamp = ts();
    let Some(hecho) = curador::aplicar(paths, &plan, &hoy, &stamp, u)? else {
        return Ok(());
    };
    log(
        paths,
        &format!(
            "lecciones curar --aplicar: {} transicion(es), backup {}",
            hecho.aplicadas, stamp
        ),
    )?;
    println!("\n{} transicion(es) aplicada(s).", hecho.aplicadas);
    println!("  Backup previo: {}", hecho.backup.display());
    println!("  Reporte: {}", hecho.reporte.display());
    println!("  Para deshacer: sh harness_cli lecciones rollback");
    Ok(())
}

fn imprimir_plan(plan: &Plan) {
    println!("Evaluadas: {} leccion(es).", plan.evaluadas);
    if !plan.pinneadas.is_empty() {
        println!("  Salteadas por pin: {}", plan.pinneadas.join(", "));
    }
    for a in &plan.acciones {
        let que = match a.transicion {
            Transicion::AStale => "-> stale",
            Transicion::AArchivada => "-> ARCHIVAR (mover a archivo/)",
            Transicion::AActiva => "-> activa (volvio a usarse)",
            Transicion::Ninguna => "sin cambio",
        };
        println!("  {:<40} {:>4}d inactiva  {que}", a.nombre, a.dias);
    }
}

/// Carga una leccion activa por nombre, con el error de "no existe" compartido.
fn cargar_activa(paths: &HarnessPaths, nombre: &str) -> Result<Leccion, Exit> {
    let file = lecciones::file_for(paths, nombre);
    Leccion::load(&file).map_err(|_| no_existe(paths, nombre))
}

pub fn pin(paths: &HarnessPaths, nombre: &str, valor: bool) -> anyhow::Result<()> {
    if exigir_biblioteca(paths).is_none() {
        return Ok(());
    }
    let mut l = cargar_activa(paths, nombre)?;
    l.set_pin(valor);
    l.save()?;
    log(paths, &format!("lecciones {} {nombre}", if valor { "pin" } else { "unpin" }))?;
    if valor {
        println!("'{nombre}' quedo pinneada: ninguna transicion automatica la va a tocar.");
    } else {
        println!("'{nombre}' ya no esta pinneada: vuelve al ciclo de vida normal.");
    }
    Ok(())
}

pub fn archivar(paths: &HarnessPaths, nombre: &str) -> anyhow::Result<()> {
    if exigir_biblioteca(paths).is_none() {
        return Ok(());
    }
    let destino = lecciones::archivo_dir(paths).join(format!("{nombre}.md"));
    if destino.exists() {
        return Err(Exit {
            code: 2,
            message: Some(format!("'{nombre}' ya esta archivada ({}).", destino.display())),
        }
        .into());
    }
    let mut l = cargar_activa(paths, nombre)?;
    let origen = lecciones::file_for(paths, nombre);
    l.set_estado(lecciones::ESTADO_ARCHIVADA);
    std::fs::create_dir_all(lecciones::archivo_dir(paths))?;
    crate::features::write_text_atomic(&destino, &l.render())?;
    std::fs::remove_file(&origen)?;
    log(paths, &format!("lecciones archivar {nombre}"))?;
    println!("'{nombre}' archivada en {}.", destino.display());
    println!("  No se borro nada: sigue siendo consultable con 'buscar' y vuelve con 'lecciones restaurar'.");
    Ok(())
}

pub fn restaurar(paths: &HarnessPaths, nombre: &str) -> anyhow::Result<()> {
    if exigir_biblioteca(paths).is_none() {
        return Ok(());
    }
    let origen = lecciones::archivo_dir(paths).join(format!("{nombre}.md"));
    let Ok(mut l) = Leccion::load(&origen) else {
        return Err(Exit {
            code: 2,
            message: Some(format!(
                "'{nombre}' no esta archivada.\n    Vea las archivadas con 'sh harness_cli lecciones status'."
            )),
        }
        .into());
    };
    let destino = lecciones::file_for(paths, nombre);
    l.set_estado(lecciones::ESTADO_ACTIVA);
    crate::features::write_text_atomic(&destino, &l.render())?;
    std::fs::remove_file(&origen)?;
    log(paths, &format!("lecciones restaurar {nombre}"))?;
    println!("'{nombre}' restaurada en {} (estado: activa).", destino.display());
    Ok(())
}

pub fn rollback(paths: &HarnessPaths, id: Option<&str>, list: bool) -> anyhow::Result<()> {
    let backups = curador::listar_backups(paths);
    if list {
        if backups.is_empty() {
            println!("No hay backups de lecciones todavia.");
            return Ok(());
        }
        println!("Backups de lecciones ({}):", backups.len());
        for b in &backups {
            println!("  {}  {}", b.id, b.motivo);
        }
        println!("\n  Restaura el mas reciente con 'lecciones rollback', o uno puntual con --id <id>.");
        return Ok(());
    }
    let stamp = ts();
    let backup = curador::rollback(paths, id, &stamp)?;
    log(paths, &format!("lecciones rollback a {}", backup.id))?;
    println!("Lecciones restauradas desde el backup {}.", backup.id);
    println!("  El estado previo quedo respaldado: este rollback tambien se puede deshacer.");
    Ok(())
}
