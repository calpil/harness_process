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
            println!(
                "  {:<40} {:>4} usos | ultimo: {}",
                l.nombre,
                l.usos(),
                match l.ultimo_uso().as_str() {
                    "" => "nunca".to_string(),
                    f => f.to_string(),
                }
            );
        }
        println!(
            "\n  Siguen siendo consultables con 'buscar'. Vuelven con 'lecciones restaurar <clase>'."
        );
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
    // Ancho por el nombre mas largo en vez del 28 fijo: los nombres de CLASE
    // son descriptivos y varios ya lo pasaban, desalineando toda la tabla
    // (hito #27 del PRD, pagado en la #36). Piso en 28 para que un catalogo de
    // nombres cortos no se vea distinto al de antes.
    let ancho = todas
        .iter()
        .map(|l| l.nombre.chars().count())
        .max()
        .unwrap_or(28)
        .max(28);
    for l in &todas {
        let uso = match l.ultimo_uso().as_str() {
            "" => "nunca".to_string(),
            fecha => fecha.to_string(),
        };
        println!(
            "  {:<ancho$} {:>4} usos | {:<10} | {}",
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
    let mut msg = format!(
        "No existe la leccion '{nombre}' ({}).",
        lecciones::rel_path(nombre)
    );
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
    println!(
        "Todavia no hay biblioteca de lecciones ({}/).",
        lecciones::DIR_NAME
    );
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
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "lecciones": rows,
                "umbrales": {"stale_dias": u.stale, "archivo_dias": u.archivo},
                "hoy": hoy,
            }))?
        );
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
    let a_stale = plan
        .acciones
        .iter()
        .filter(|a| a.transicion == Transicion::AStale)
        .count();
    let a_archivo = plan
        .acciones
        .iter()
        .filter(|a| a.transicion == Transicion::AArchivada)
        .count();
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
    log(
        paths,
        &format!("lecciones {} {nombre}", if valor { "pin" } else { "unpin" }),
    )?;
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
            message: Some(format!(
                "'{nombre}' ya esta archivada ({}).",
                destino.display()
            )),
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
    println!(
        "  No se borro nada: sigue siendo consultable con 'buscar' y vuelve con 'lecciones restaurar'."
    );
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
    println!(
        "'{nombre}' restaurada en {} (estado: activa).",
        destino.display()
    );
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
        println!(
            "\n  Restaura el mas reciente con 'lecciones rollback', o uno puntual con --id <id>."
        );
        return Ok(());
    }
    let stamp = ts();
    let backup = curador::rollback(paths, id, &stamp)?;
    log(paths, &format!("lecciones rollback a {}", backup.id))?;
    println!("Lecciones restauradas desde el backup {}.", backup.id);
    println!("  El estado previo quedo respaldado: este rollback tambien se puede deshacer.");
    Ok(())
}

// ---------------------------------------------------------------------------
// Feature #28: consolidacion asistida por LLM.
//
// Misma simetria que `curar`: sin `--aplicar` INFORMA y no toca nada. La
// diferencia es que la deteccion depende de un backend externo, y por eso la
// mitad que muta toma la fusion de ARGV y no de lo que dijo el modelo: asi se
// verifica sin backend y de forma determinista.
// ---------------------------------------------------------------------------

use crate::consolidacion::{self, Backend};

/// `lecciones consolidar [--aplicar --en <p> --de a,b --motivo "..."]`
pub fn consolidar(
    paths: &HarnessPaths,
    aplicar: bool,
    en: Option<&str>,
    de: Option<&str>,
    motivo: Option<&str>,
) -> anyhow::Result<()> {
    if exigir_biblioteca(paths).is_none() {
        return Ok(());
    }
    if aplicar {
        return aplicar_fusion(paths, en, de, motivo);
    }
    detectar(paths)
}

/// La mitad que necesita modelo. **No escribe nada.**
fn detectar(paths: &HarnessPaths) -> anyhow::Result<()> {
    // Las referencias `relacionadas` son una señal local: se calculan antes
    // de decidir si hay backend. Así una pareja escrita mutuamente sigue siendo
    // revisable aun cuando no haya cuota, red ni CLI de modelo.
    let (activas, _) = lecciones::scan(paths);
    if activas.len() < 2 {
        println!("Hay {} leccion(es): nada que consolidar.", activas.len());
        return Ok(());
    }
    let resumenes: Vec<consolidacion::Resumen> = activas
        .iter()
        .map(|l| consolidacion::Resumen {
            nombre: l.nombre.clone(),
            descripcion: l.descripcion(),
            triggers: l.fm.list("triggers"),
            relacionadas: l.fm.list("relacionadas"),
        })
        .collect();
    let existentes: Vec<String> = activas.iter().map(|l| l.nombre.clone()).collect();
    let pinneadas: Vec<String> = activas
        .iter()
        .filter(|l| l.pinneada())
        .map(|l| l.nombre.clone())
        .collect();
    let senales = consolidacion::por_relacionadas(&resumenes);
    let mut candidatos = senales.candidatos;
    let diagnosticos = senales.diagnosticos;

    let data = crate::features::load_features(paths)?;
    let override_cmd = std::env::var("HARNESS_CONSOLIDAR_CMD").ok();
    let backend = consolidacion::resolver_backend(&data, override_cmd.as_deref(), |n| {
        which::which(n).is_ok()
    });
    if let Some(motivo) = backend.motivo_del_skip() {
        println!("{motivo}");
        if candidatos.is_empty() && diagnosticos.is_empty() {
            return Ok(());
        }
        return informar_candidatos(candidatos, &diagnosticos, &existentes, &pinneadas);
    }
    let Some(argv) = backend.argv() else {
        return Ok(());
    };

    let quien = match &backend {
        Backend::Override(a) => a.join(" "),
        Backend::Cli { nombre, .. } => nombre.clone(),
        _ => String::new(),
    };
    println!(
        "Consultando a `{quien}` por {} lecciones (nombre, descripcion y triggers; NUNCA el cuerpo)...",
        resumenes.len()
    );

    let timeout = std::time::Duration::from_secs(consolidacion::timeout_segundos(&data));
    let salida = match consolidacion::preguntar(
        argv,
        &consolidacion::prompt(&resumenes),
        &paths.repo_root,
        timeout,
    ) {
        Ok(s) => s,
        Err(e) => {
            // Un backend que falla no rompe el flujo: se informa y se sale 0.
            println!("[i] El backend no respondio: {e}");
            if candidatos.is_empty() && diagnosticos.is_empty() {
                return Ok(());
            }
            return informar_candidatos(candidatos, &diagnosticos, &existentes, &pinneadas);
        }
    };

    let Some(json) = consolidacion::extraer_json(&salida) else {
        println!("[i] El backend no devolvio JSON usable. Nada que reportar.");
        if candidatos.is_empty() && diagnosticos.is_empty() {
            return Ok(());
        }
        return informar_candidatos(candidatos, &diagnosticos, &existentes, &pinneadas);
    };
    candidatos.extend(consolidacion::marcar_triggers(
        consolidacion::leer_candidatos(&json),
    ));
    informar_candidatos(candidatos, &diagnosticos, &existentes, &pinneadas)
}

/// Imprime señales locales y candidatas validadas. No recibe `HarnessPaths`,
/// por lo que este tramo de detección sigue sin poder escribir.
fn informar_candidatos(
    candidatos: Vec<consolidacion::Candidato>,
    diagnosticos: &[String],
    existentes: &[String],
    pinneadas: &[String],
) -> anyhow::Result<()> {
    for diagnostico in diagnosticos {
        println!("[i] {diagnostico}");
    }
    let (ok, descartados) = consolidacion::validar(candidatos, existentes, pinneadas);
    for d in &descartados {
        println!("[i] Candidato descartado: {}", d.mensaje());
    }
    let ok = consolidacion::unir_candidatos(ok);
    if ok.is_empty() {
        println!("\nNingun solapamiento: el catalogo esta limpio.");
        return Ok(());
    }
    println!("\n{} candidato(s) a consolidar:", ok.len());
    for c in &ok {
        // La confianza se reporta SIN filtrar (decision del usuario, OBS-3): con
        // 9 lecciones y un solo par real no hay zona gris con que calibrar un
        // umbral, y un umbral no calibrable es un numero inventado.
        println!(
            "\n  {} (confianza {:.2})",
            c.miembros.join(" + "),
            c.confianza
        );
        println!("      {}", c.motivo);
    }
    println!("\nEsto SOLO informa: no se toco ningun archivo.");
    println!("Para fusionar, escribi primero el paraguas y despues:");
    println!(
        "  sh harness_cli lecciones consolidar --aplicar --en <paraguas> --de {} --motivo \"<por que>\"",
        ok[0].miembros.join(",")
    );
    Ok(())
}

/// La mitad que muta. **No necesita modelo**: la fusion viene de argv.
fn aplicar_fusion(
    paths: &HarnessPaths,
    en: Option<&str>,
    de: Option<&str>,
    motivo: Option<&str>,
) -> anyhow::Result<()> {
    let uso =
        "Uso: lecciones consolidar --aplicar --en <paraguas> --de <a,b> --motivo \"<por que>\"";
    let (Some(en), Some(de)) = (en, de) else {
        return Err(Exit {
            code: 2,
            message: Some(format!("Faltan --en y/o --de.\n    {uso}")),
        }
        .into());
    };
    let motivo = motivo.unwrap_or_default().trim().to_string();
    if motivo.is_empty() {
        // Una fusion sin motivo escrito es la que nadie va a poder revisar.
        return Err(Exit {
            code: 2,
            message: Some(format!(
                "Falta --motivo: una fusion sin motivo no se puede revisar despues.\n    {uso}"
            )),
        }
        .into());
    }
    let miembros: Vec<String> = de
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect();
    if miembros.is_empty() {
        return Err(Exit {
            code: 2,
            message: Some(format!("--de no nombra ninguna leccion.\n    {uso}")),
        }
        .into());
    }

    let (activas, _) = lecciones::scan(paths);
    let buscar = |n: &str| activas.iter().find(|l| l.nombre == n);
    let Some(paraguas) = buscar(en) else {
        return Err(Exit {
            code: 2,
            message: Some(format!(
                "El paraguas '{en}' no existe. Escribilo primero: sh harness_cli leccion nueva {en}"
            )),
        }
        .into());
    };
    // El paraguas PUEDE ser una de las miembros: es lo que manda la guia
    // ("patchea el paraguas existente") y es la forma del unico solapamiento
    // real de este repo.
    let a_archivar: Vec<&lecciones::Leccion> = miembros
        .iter()
        .filter(|m| m.as_str() != en)
        .filter_map(|m| buscar(m))
        .collect();
    let faltantes: Vec<&String> = miembros
        .iter()
        .filter(|m| m.as_str() != en && buscar(m).is_none())
        .collect();
    if !faltantes.is_empty() {
        return Err(Exit {
            code: 2,
            message: Some(format!(
                "No existe(n): {}",
                faltantes
                    .iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            )),
        }
        .into());
    }
    if a_archivar.is_empty() {
        return Err(Exit {
            code: 2,
            message: Some(format!(
                "--de no nombra ninguna leccion distinta del paraguas '{en}'."
            )),
        }
        .into());
    }

    // El paraguas tiene que poder REEMPLAZAR a lo que archiva.
    let texto_paraguas = std::fs::read_to_string(&paraguas.file).unwrap_or_default();
    let triggers_paraguas = paraguas.fm.list("triggers");
    let miembros_tri: Vec<(String, Vec<String>)> = a_archivar
        .iter()
        .map(|l| (l.nombre.clone(), l.fm.list("triggers")))
        .collect();
    let faltas =
        consolidacion::revisar_paraguas(&texto_paraguas, &triggers_paraguas, &miembros_tri);
    if !faltas.is_empty() {
        let mut msg =
            format!("El paraguas '{en}' todavia no puede reemplazar a lo que archivaria:\n");
        for f in &faltas {
            msg.push_str(&format!("    - {}\n", f.mensaje()));
        }
        msg.push_str(
            "    Escribilo primero: archivar contra un paraguas incompleto pierde el conocimiento.",
        );
        return Err(Exit {
            code: 2,
            message: Some(msg),
        }
        .into());
    }

    let backup = curador::respaldar(paths, "consolidar", &motivo)?;
    let mut archivadas = Vec::new();
    let nombres: Vec<String> = a_archivar.iter().map(|l| l.nombre.clone()).collect();
    for n in &nombres {
        // Se reusa `archivar`, que ya mueve sin borrar y deja su linea en la
        // bitacora. Consolidar no inventa una segunda forma de archivar.
        archivar(paths, n)?;
        archivadas.push(n.clone());
    }
    log(
        paths,
        &format!(
            "lecciones consolidar --en {en} --de {} motivo={motivo}",
            archivadas.join(",")
        ),
    )?;
    println!("Consolidacion aplicada.");
    println!("  Paraguas:  {en}");
    println!("  Archivadas: {}", archivadas.join(", "));
    println!("  Motivo:    {motivo}");
    println!("  Backup:    {}", backup.display());
    println!("\nNada se borro. Para deshacer: sh harness_cli lecciones rollback");
    Ok(())
}
