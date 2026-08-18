//! `harness perfil <show|add|replace|remove|sugerir>` (feature #19).
//!
//! `docs/perfil-usuario.md` es el documento del USUARIO: el arnes junta la
//! evidencia (`sugerir`) y verifica (limite, duplicados, seguridad), el agente
//! propone, y **solo el usuario decide**. Los tres comandos de escritura exigen
//! `--yes`, igual que `approve-spec` (Articulo 2 y 5 de la constitution).
//!
//! Exit codes: 0 = ok o no-op; 2 = sin `--yes`, limite excedido, subcadena que no
//! matchea o matchea varias, o entrada rechazada por el escaneo de seguridad.

use crate::exit::Exit;
use crate::paths::HarnessPaths;
use crate::perfil::{self, Coincidencia, Perfil};
use crate::progress::log;

/// Gate del `--yes`, comun a los tres comandos de escritura (AC-6).
fn exigir_si_del_usuario(yes: bool, accion: &str, texto: &str) -> Result<(), Exit> {
    if yes {
        return Ok(());
    }
    println!("[GATE] perfil {accion} exige la confirmacion explicita del USUARIO.");
    println!("    El perfil es SU documento y viaja al prompt de cada agente.");
    println!("    1) Mostrale la entrada en el chat:");
    println!("       {texto}");
    println!("    2) Preguntale si la aprueba.");
    println!("    3) Solo con su SI: sh harness_cli perfil {accion} ... --yes");
    Err(Exit::code(2))
}

/// Escaneo previo a cualquier escritura (AC-10). BLOQUEA: un secreto en este
/// archivo queda en el historial de git para siempre y ademas se inyecta en cada
/// prompt (decision del usuario 2026-08-16, OBS-4).
fn exigir_texto_seguro(texto: &str) -> Result<(), Exit> {
    match perfil::motivo_inseguro(texto) {
        None => Ok(()),
        Some(motivo) => Err(Exit {
            code: 2,
            message: Some(format!(
                "La entrada trae {motivo}: no entra al perfil.\n    \
                 Este archivo se versiona (queda en git para siempre) y se inyecta en el\n    \
                 prompt de cada agente. Reescribi la frase sin ese dato; si necesitas nombrar\n    \
                 un secreto, nombra la VARIABLE de entorno, nunca su valor."
            )),
        }),
    }
}

/// Aviso comun tras escribir: las superficies son un snapshot congelado (AC-13).
fn aviso_snapshot() {
    println!("  Las superficies (CLAUDE.md, AGENTS.md, GEMINI.md, LLM.md) se refrescan");
    println!("  al reinstalar: este cambio recien llega a los agentes en la proxima sesion.");
}

pub fn show(paths: &HarnessPaths) -> anyhow::Result<()> {
    let p = Perfil::load(paths);
    let entradas = p.entradas();
    if entradas.is_empty() {
        println!("Perfil vacio ({}).", perfil::rel_path());
        println!("  Junta evidencia con 'sh harness_cli perfil sugerir', proponele una");
        println!("  entrada al usuario y, con su si: perfil add --texto \"...\" --yes");
        return Ok(());
    }
    println!(
        "Perfil de usuario [{}% - {}/{} chars]",
        p.porcentaje(),
        p.usados(),
        perfil::LIMITE
    );
    for (n, e) in entradas.iter().enumerate() {
        println!("  {}. {e}", n + 1);
    }
    Ok(())
}

pub fn add(paths: &HarnessPaths, texto: &str, yes: bool) -> anyhow::Result<()> {
    let texto = texto.trim();
    if texto.is_empty() {
        return Err(Exit {
            code: 2,
            message: Some("La entrada no puede estar vacia.".to_string()),
        }
        .into());
    }
    exigir_si_del_usuario(yes, "add", texto)?;
    exigir_texto_seguro(texto)?;
    let mut p = Perfil::load(paths);
    // Duplicado exacto: no-op explicito, no error (AC-7).
    if p.entradas().iter().any(|e| e == texto) {
        println!("El perfil ya tenia esa entrada; no se duplico.");
        return Ok(());
    }
    let quedaria = p.usados_con(None, texto);
    if quedaria > perfil::LIMITE {
        return Err(p.error_de_limite(quedaria).into());
    }
    p.insertar(texto);
    p.save(paths)?;
    log(paths, &format!("perfil add {texto}"))?;
    println!(
        "Entrada agregada a {} [{}% - {}/{} chars].",
        perfil::rel_path(),
        p.porcentaje(),
        p.usados(),
        perfil::LIMITE
    );
    aviso_snapshot();
    Ok(())
}

pub fn replace(paths: &HarnessPaths, old: &str, texto: &str, yes: bool) -> anyhow::Result<()> {
    let texto = texto.trim();
    exigir_si_del_usuario(yes, "replace", texto)?;
    exigir_texto_seguro(texto)?;
    let mut p = Perfil::load(paths);
    let idx = resolver(&p, old)?;
    // El limite aplica a TODA escritura, no solo a add (AC-5).
    let quedaria = p.usados_con(Some(idx), texto);
    if quedaria > perfil::LIMITE {
        return Err(p.error_de_limite(quedaria).into());
    }
    let anterior = p.entradas();
    p.reemplazar(idx, texto);
    p.save(paths)?;
    log(paths, &format!("perfil replace {old} -> {texto}"))?;
    println!(
        "Entrada reemplazada en {} [{}% - {}/{} chars].",
        perfil::rel_path(),
        p.porcentaje(),
        p.usados(),
        perfil::LIMITE
    );
    if let Some(vieja) = anterior.iter().find(|e| e.contains(old.trim())) {
        println!("  Antes: {vieja}");
    }
    println!("  Ahora: {texto}");
    aviso_snapshot();
    Ok(())
}

pub fn remove(paths: &HarnessPaths, old: &str, yes: bool) -> anyhow::Result<()> {
    exigir_si_del_usuario(yes, "remove", old)?;
    let mut p = Perfil::load(paths);
    let idx = resolver(&p, old)?;
    let quitada = p.quitar(idx);
    p.save(paths)?;
    log(paths, &format!("perfil remove {quitada}"))?;
    println!(
        "Entrada quitada de {} [{}% - {}/{} chars]: {quitada}",
        perfil::rel_path(),
        p.porcentaje(),
        p.usados(),
        perfil::LIMITE
    );
    aviso_snapshot();
    Ok(())
}

/// Resuelve la subcadena a UNA entrada. Los tres casos del enum se manejan
/// explicitamente: "matchea varias" es un estado real con su propio remedio, no
/// un "no se encontro" (AC-8).
fn resolver(p: &Perfil, old: &str) -> Result<usize, Exit> {
    match p.buscar(old) {
        Coincidencia::Unica(idx) => Ok(idx),
        Coincidencia::Ninguna => Err(Exit {
            code: 2,
            message: Some(format!(
                "Ninguna entrada del perfil contiene '{old}'.\n    \
                 Vela con 'sh harness_cli perfil show'."
            )),
        }),
        Coincidencia::Ambigua(candidatas) => {
            let mut msg = format!(
                "'{old}' matchea {} entradas; usa un fragmento mas especifico:",
                candidatas.len()
            );
            for c in &candidatas {
                msg.push_str(&format!("\n      - {c}"));
            }
            Err(Exit {
                code: 2,
                message: Some(msg),
            })
        }
    }
}

/// `perfil sugerir`: junta la evidencia que ya esta escrita en el repo y emite el
/// contrato. **No escribe nada** (AC-14, AC-15, AC-16).
pub fn sugerir(paths: &HarnessPaths) -> anyhow::Result<()> {
    let registros = perfil::recolectar(paths);
    if registros.is_empty() {
        println!("Sin material todavia: no hay decisiones registradas en la bitacora,");
        println!("los planes ni los specs. Las decisiones se registran con");
        println!("  sh harness_cli advance --nota \"Decision usuario: ...\"");
        println!("y en la seccion Observaciones de cada spec.");
        return Ok(());
    }
    let nuevos: Vec<_> = registros.iter().filter(|r| !r.ya_incorporado).collect();
    let ya = registros.len() - nuevos.len();
    println!(
        "Evidencia encontrada: {} registro(s) de decision, {} sin incorporar al perfil.",
        registros.len(),
        nuevos.len()
    );
    if ya > 0 {
        println!("  ({ya} ya citado(s) por una entrada del perfil: se omiten.)");
    }
    // Agrupado por feature: las preferencias se ven cuando se REPITEN, y agrupar
    // por origen es lo que deja ver la repeticion.
    let mut features: Vec<String> = nuevos.iter().map(|r| r.feature.clone()).collect();
    features.sort_by_key(|f| f.parse::<u64>().unwrap_or(u64::MAX));
    features.dedup();
    for fid in features {
        let etiqueta = if fid.is_empty() {
            "(sin feature)".to_string()
        } else {
            format!("feature #{fid}")
        };
        println!("\n== {etiqueta} ==");
        for r in nuevos.iter().filter(|r| r.feature == fid) {
            let fecha = if r.fecha.is_empty() {
                String::new()
            } else {
                format!("{} ", r.fecha)
            };
            println!("  [{}] {fecha}{}", r.fuente, r.texto);
        }
    }
    print!("{}", perfil::contrato_de_sugerencia());
    Ok(())
}

/// `perfil bloque` (interno): imprime el bloque para que el instalador lo
/// inyecte. Vive en el binario y no en los dos instaladores para que el formato
/// y el parseo del perfil existan en UN solo lugar. Sin entradas no imprime nada,
/// y el instalador interpreta "vacio" como "no inyectar" (AC-12).
pub fn bloque(paths: &HarnessPaths) -> anyhow::Result<()> {
    if let Some(b) = Perfil::load(paths).bloque() {
        print!("{b}");
    }
    Ok(())
}

/// `perfil check`: gate de integridad que consume `harness_check.sh` (AC-18).
///
/// Vive en el binario y no en bash a proposito: contar caracteres UTF-8 en awk es
/// poco confiable, y el limite tiene que ser EL MISMO que aplica al escribir.
/// Sin el archivo no dice nada (exit 0): un proyecto sin perfil no ve el gate.
pub fn check(paths: &HarnessPaths) -> anyhow::Result<()> {
    if !perfil::file_for(paths).is_file() {
        return Ok(());
    }
    let p = Perfil::load(paths);
    let usados = p.usados();
    if usados > perfil::LIMITE {
        return Err(Exit {
            code: 2,
            message: Some(format!(
                "[GATE] {} supera el limite: {usados}/{} caracteres.\n    \
                 Es lo que se inyecta en el prompt de CADA agente, asi que no puede crecer\n    \
                 sin control. Consolida con:\n      \
                 sh harness_cli perfil replace --old \"<fragmento>\" --texto \"<mas corto>\" --yes\n      \
                 sh harness_cli perfil remove  --old \"<fragmento>\" --yes",
                perfil::rel_path(),
                perfil::LIMITE
            )),
        }
        .into());
    }
    // Formato: el encabezado es del usuario, pero si lo borro entero conviene
    // avisar (el archivo pierde el contexto que explica que es). Solo avisa.
    if !p.render().starts_with("# Perfil de usuario") {
        println!(
            "[i] {} no empieza con su encabezado '# Perfil de usuario'.",
            perfil::rel_path()
        );
    }
    Ok(())
}
