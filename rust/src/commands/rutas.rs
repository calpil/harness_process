//! `harness rutas [--check <ruta>...] [--json]` (feature #26).
//!
//! La consulta que hacen los hooks. Es el mismo matcher que usa la red de
//! seguridad de `harness_check.sh`: una sola implementacion, para que la
//! prevencion y la deteccion nunca puedan discrepar sobre que esta protegido
//! (la duplicacion de logica de rutas ya costo la feature #10).
//!
//! **Solo lee.** El modulo no importa nada que escriba.

use std::path::{Path, PathBuf};

use serde_json::json;

use crate::exit::Exit;
use crate::features::load_features;
use crate::paths::HarnessPaths;
use crate::rutas;

/// Registro de las escrituras que hizo el PROPIO arnes sobre rutas protegidas.
/// Vive en `progress/`, que es donde vive el resto del estado local del arnes.
pub fn registro_path(paths: &HarnessPaths) -> PathBuf {
    paths.progress.join(".rutas_arnes")
}

fn mtime_nanos(ruta: &Path) -> Option<u128> {
    std::fs::metadata(ruta)
        .and_then(|m| m.modified())
        .ok()?
        .duration_since(std::time::UNIX_EPOCH)
        .ok()
        .map(|d| d.as_nanos())
}

/// Anota que el arnes acaba de escribir `rel` (relativa a la raiz). Best-effort:
/// si no se puede escribir el registro, la unica consecuencia es un aviso de mas.
pub fn registrar_escritura_del_arnes(paths: &HarnessPaths, rel: &str) {
    let absoluta = paths.repo_root.join(rel);
    let Some(stamp) = mtime_nanos(&absoluta) else {
        return;
    };
    let previo = std::fs::read_to_string(registro_path(paths)).unwrap_or_default();
    let mut lineas: Vec<String> = previo
        .lines()
        .filter(|l| !l.starts_with(&format!("{rel}\t")))
        .map(str::to_string)
        .collect();
    lineas.push(rutas::linea_registro(rel, stamp));
    let _ = std::fs::create_dir_all(&paths.progress);
    let _ = std::fs::write(registro_path(paths), format!("{}\n", lineas.join("\n")));
}

/// Las rutas protegidas modificadas y sin commitear, ya descontadas las que
/// escribio el arnes. Es lo que consulta la red de seguridad.
pub fn violaciones_actuales(paths: &HarnessPaths, patrones: &[String]) -> Vec<rutas::Violacion> {
    let salida = std::process::Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(&paths.repo_root)
        .output();
    let Ok(salida) = salida else {
        return Vec::new(); // sin git no hay nada que comparar
    };
    let porcelain = String::from_utf8_lossy(&salida.stdout);
    let registro = std::fs::read_to_string(registro_path(paths)).unwrap_or_default();
    let exentas = rutas::exentas(&registro, |r| mtime_nanos(&paths.repo_root.join(r)));
    // Poda en cada consulta (decision del usuario 2026-08-18, OBS-2 de la #36):
    // este camino ya leyo el archivo y ya corrio `git status`, asi que sacar las
    // entradas muertas no cuesta nada y el registro no crece nunca. Consultar es
    // muchisimo mas frecuente que escribir, asi que podar al escribir dejaria
    // entradas muertas por mucho tiempo.
    podar_registro_en_disco(paths, &registro, &porcelain);
    rutas::violaciones(&porcelain, &paths.repo_root, patrones, &exentas)
}

/// Best-effort: si no se puede escribir, la unica consecuencia es que el
/// registro queda con una linea de mas.
fn podar_registro_en_disco(paths: &HarnessPaths, registro: &str, porcelain: &str) {
    if registro.is_empty() {
        return;
    }
    let sucias: Vec<String> = porcelain
        .lines()
        .filter(|l| l.len() >= 4)
        .map(|l| {
            let ruta = l[3..].trim();
            ruta.rsplit(" -> ").next().unwrap_or(ruta).trim_matches('"').to_string()
        })
        .collect();
    let podado = rutas::podar_registro(registro, &sucias);
    if podado != registro {
        let _ = std::fs::write(registro_path(paths), podado);
    }
}

pub fn run(
    paths: &HarnessPaths,
    check: &[String],
    violaciones: bool,
    aceptar: bool,
    as_json: bool,
) -> anyhow::Result<()> {
    let data = load_features(paths)?;
    let patrones = rutas::patrones(&data);

    if aceptar {
        return aceptar_estado_actual(paths, &patrones);
    }

    if violaciones {
        return emitir_violaciones(paths, &patrones, as_json);
    }

    if check.is_empty() {
        // Sin rutas: se informa la configuracion vigente. Es lo que permite
        // saber si la proteccion esta activa sin tener que adivinar.
        if as_json {
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "activa": !patrones.is_empty(),
                    "patrones": patrones,
                }))?
            );
        } else if patrones.is_empty() {
            println!("Rutas protegidas: APAGADO (rules.rutas_protegidas es una lista vacia).");
        } else {
            println!("Rutas protegidas ({}):", patrones.len());
            for p in &patrones {
                println!("  {p}");
            }
        }
        return Ok(());
    }

    let protegidas: Vec<&String> = check
        .iter()
        .filter(|r| rutas::esta_protegida(r, &paths.repo_root, &patrones))
        .collect();

    if as_json {
        let filas: Vec<_> = protegidas
            .iter()
            .map(|r| json!({"ruta": r}))
            .collect();
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "protegidas": filas,
                "total": protegidas.len(),
            }))?
        );
    } else {
        for r in &protegidas {
            println!("{r}");
        }
    }

    if protegidas.is_empty() {
        return Ok(());
    }
    // Exit 2 = "al menos una esta protegida". Es lo que el hook mira; el mensaje
    // accionable lo arma quien llama, que sabe si puede prevenir o solo avisar.
    Err(Exit { code: 2, message: None }.into())
}

/// La red de seguridad: rutas protegidas tocadas y sin commitear. Exit 2 cuando
/// hay alguna, con el comando de reversion por cada una.
fn emitir_violaciones(
    paths: &HarnessPaths,
    patrones: &[String],
    as_json: bool,
) -> anyhow::Result<()> {
    if patrones.is_empty() {
        // Proteccion apagada a pedido del usuario: silencio total.
        if as_json {
            println!("{}", serde_json::to_string_pretty(&json!({"violaciones": [], "activa": false}))?);
        }
        return Ok(());
    }
    let encontradas = violaciones_actuales(paths, patrones);
    if as_json {
        let filas: Vec<_> = encontradas
            .iter()
            .map(|v| json!({"ruta": v.ruta, "trackeada": v.trackeada, "remedio": rutas::remedio(v)}))
            .collect();
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({"violaciones": filas, "activa": true}))?
        );
    } else {
        for v in &encontradas {
            println!("{}\t{}", v.ruta, rutas::remedio(v));
        }
    }
    if encontradas.is_empty() {
        return Ok(());
    }
    Err(Exit { code: 2, message: None }.into())
}

/// Toma el estado actual de las rutas protegidas como **linea de base**.
///
/// Existe por el AC-14: una instalacion que adopta la proteccion con trabajo ya
/// en curso tiene cambios legitimos sin commitear que no hizo ningun agente. Sin
/// esto, la red de seguridad bloquearia desde el primer minuto por algo que
/// nadie hizo mal, y un gate que arranca en rojo se apaga en dos dias.
///
/// Es EXPLICITO y lo corre una persona, como `--aplicar` del curador (#21): el
/// chequeo por si solo nunca escribe.
fn aceptar_estado_actual(paths: &HarnessPaths, patrones: &[String]) -> anyhow::Result<()> {
    if patrones.is_empty() {
        println!("Rutas protegidas apagadas: no hay estado que aceptar.");
        return Ok(());
    }
    // Se lee el estado SIN exenciones: la linea de base incluye todo lo que hoy
    // esta tocado bajo una ruta protegida.
    let salida = std::process::Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(&paths.repo_root)
        .output();
    let Ok(salida) = salida else {
        println!("[i] Sin git: no hay estado que aceptar.");
        return Ok(());
    };
    let porcelain = String::from_utf8_lossy(&salida.stdout);
    let actuales = rutas::violaciones(&porcelain, &paths.repo_root, patrones, &[]);
    if actuales.is_empty() {
        println!("Nada que aceptar: ninguna ruta protegida esta modificada.");
        return Ok(());
    }
    for v in &actuales {
        registrar_escritura_del_arnes(paths, &v.ruta);
        println!("Aceptada como linea de base: {}", v.ruta);
    }
    println!(
        "\n{} ruta(s) aceptadas. A partir de aca, cualquier cambio NUEVO sobre ellas",
        actuales.len()
    );
    println!("se reporta: la exencion caduca en cuanto el archivo se vuelve a tocar.");
    Ok(())
}
