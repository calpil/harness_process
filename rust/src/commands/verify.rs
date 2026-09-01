//! `harness verify --feature <id>` (feature #23).
//!
//! Ejecuta los comandos que el spec declara en sus AC. Es el UNICO comando del
//! arnes que ejecuta shell, y por eso es el unico que exige spec aprobado:
//! aprobar el spec es el acto en el que el usuario leyo los comandos.
//!
//! No lo llama ningun hook ni ningun otro comando (AC-7). Se corre a mano.

use std::time::Duration;

use serde_json::{Map, Value, json};

use crate::exit::Exit;
use crate::features::{feature_at, find_feature_index, load_features};
use crate::paths::HarnessPaths;
use crate::progress::now_stamp;
use crate::pycompat::relpath;
use crate::spec::{SpecState, spec_path, spec_state};
use crate::verificacion::{
    self, Estado, Resultado, ejecutar, parsear, render_reporte_desde, reporte_path, reporte_rel,
};

pub fn run(paths: &HarnessPaths, fid: &str, as_json: bool, solo: Option<&str>) -> anyhow::Result<()> {
    let data = load_features(paths)?;
    let idx = find_feature_index(&data, fid)?;
    let Some(feature) = feature_at(&data, idx).as_object() else {
        anyhow::bail!("feature_list.json: feature invalida");
    };
    // Los documentos y el código de la feature viven juntos en su worktree.
    // `--feature` manda sobre el CWD desde el que se invoca este binario.
    let paths = paths.para_feature(feature);
    let spec = spec_path(&paths, feature);
    let rel_spec = relpath(&spec, &paths.repo_root).unwrap_or_else(|| spec.clone());

    // BARRERA (AC-5): sin spec aprobado no se ejecuta ni un comando. Se valida
    // antes de leer el archivo entero, y desde luego antes de lanzar nada.
    exigir_aprobado(&paths, feature, &rel_spec.display().to_string())?;

    let texto = std::fs::read_to_string(&spec)
        .map_err(|e| Exit::msg(format!("No se pudo leer {}: {e}", rel_spec.display())))?;
    let verificaciones = filtrar(parsear(&texto), solo)?;

    let con_comando = verificaciones.iter().filter(|v| v.comando.is_some()).count();
    if con_comando == 0 {
        // Compatibilidad (AC-2): un spec sin `Comando:` no es un error. Los 310
        // AC ya escritos caen aca y salen con 0.
        return sin_nada_que_verificar(&rel_spec.display().to_string(), &verificaciones, as_json);
    }

    let timeout = Duration::from_secs(verificacion::timeout_segundos(&data));
    let raiz = raiz_de_ejecucion(&paths);
    let worktree_valido = feature
        .get("worktree")
        .and_then(Value::as_str)
        .is_some_and(|worktree| std::path::Path::new(worktree).is_dir());
    if !as_json {
        if !worktree_valido {
            println!(
                "[i] Feature #{fid} sin worktree valido: se verifica desde la raiz documental efectiva."
            );
        }
        println!(
            "Verificando {} AC con comando declarado ({} en total) de {}",
            con_comando,
            verificaciones.len(),
            rel_spec.display()
        );
        // Feature #69: una linea que dice ser un AC y no se puede leer se
        // descartaba SIN UNA PALABRA. El criterio desaparecia por un caracter y
        // el autor se enteraba —si se enteraba— en el review. Se avisa antes de
        // correr nada, que es donde el error todavia es barato, y se sigue
        // verificando el resto: cortar aca le quitaria al autor el resultado de
        // los AC que si estan bien.
        for linea in crate::verificacion::lineas_ac_ilegibles(&texto) {
            println!("[!] Linea que dice ser un AC y no se pudo leer: {linea}");
            println!("    Forma esperada: `- AC-<n>[letra] [(anotacion)]: ...`. Ese AC NO se verifica.");
        }
        println!("Raiz de ejecucion: {}", raiz.display());
        println!("Timeout por comando: {}s\n", timeout.as_secs());
    }

    let mut resultados: Vec<Resultado> = Vec::with_capacity(verificaciones.len());
    for v in &verificaciones {
        let Some(comando) = v.comando.as_deref() else {
            resultados.push(Resultado {
                ac: v.ac.clone(),
                comando: None,
                estado: Estado::Manual,
                exit: None,
                duracion_ms: 0,
                salida: String::new(),
            });
            continue;
        };
        // AC-4: el comando se IMPRIME antes de correr. Nada a ciegas.
        if !as_json {
            println!("{}  $ {comando}", v.ac);
        }
        let (estado, exit, ms, salida) = ejecutar(comando, raiz, timeout);
        if !as_json {
            println!("       {} {} ({ms} ms)", estado.simbolo(), estado.etiqueta());
            if estado.bloquea() && !salida.is_empty() {
                for linea in salida.lines() {
                    println!("       | {linea}");
                }
            }
        }
        // AC-6: uno que falla o se cuelga no corta la corrida; se sigue con los
        // demas, porque el valor del reporte es ver TODO lo que esta roto.
        resultados.push(Resultado {
            ac: v.ac.clone(),
            comando: Some(comando.to_string()),
            estado,
            exit,
            duracion_ms: ms,
            salida,
        });
    }

    let stamp = now_stamp();
    let reporte = render_reporte_desde(fid, &stamp, Some(raiz), &resultados);
    let destino = reporte_path(&paths, fid);
    if let Some(padre) = destino.parent() {
        std::fs::create_dir_all(padre)?;
    }
    std::fs::write(&destino, &reporte)?;

    let bloqueantes: Vec<&Resultado> = resultados.iter().filter(|r| r.estado.bloquea()).collect();
    if as_json {
        emitir_json(
            fid,
            &rel_spec.display().to_string(),
            &stamp,
            raiz,
            &resultados,
        );
    } else {
        let cuenta = |e: Estado| resultados.iter().filter(|r| r.estado == e).count();
        let vacios = cuenta(Estado::Vacio);
        let verdes = cuenta(Estado::Verde);
        let manuales = cuenta(Estado::Manual);
        let sin_casos = if vacios > 0 {
            format!(", {vacios} sin casos")
        } else {
            String::new()
        };
        println!(
            "\n{verdes} verde(s), {} en rojo, {manuales} manual(es){sin_casos}.",
            bloqueantes.len() - vacios
        );
        println!("Reporte: {}", reporte_rel(fid));
        if manuales > 0 {
            println!("Los AC sin comando los verifica el reviewer: no cuentan como fallo.");
        }
        if vacios > 0 {
            println!(
                "Un AC `sin casos` corrio y salio 0, pero no ejecuto ningun test:\n\
                 revisa que el nombre del filtro exista de verdad."
            );
        }
    }
    if bloqueantes.is_empty() {
        return Ok(());
    }
    // Exit 1 con la lista: el que corre esto quiere saber QUE fallo sin abrir
    // el reporte.
    Err(Exit::msg(format!(
        "AC en rojo: {}",
        bloqueantes
            .iter()
            .map(|r| r.ac.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    ))
    .into())
}

fn exigir_aprobado(
    paths: &HarnessPaths,
    feature: &Map<String, Value>,
    rel: &str,
) -> Result<(), Exit> {
    match spec_state(paths, feature) {
        SpecState::Approved => Ok(()),
        SpecState::Missing => Err(Exit {
            code: 2,
            message: Some(format!("No hay spec para esta feature: {rel}")),
        }),
        estado => Err(Exit {
            code: 2,
            message: Some(format!(
                "[BARRERA] Spec sin aprobar: {rel} (estado: {}).\n    \
                 verify NO ejecuta comandos de un spec que el usuario no aprobo:\n    \
                 aprobar el spec es el acto en el que alguien leyo esos comandos.\n    \
                 Flujo: mostrale el spec al USUARIO, preguntale, y con su SI:\n      \
                 sh harness_cli approve-spec --feature <id> --yes",
                estado.label()
            )),
        }),
    }
}

/// `--solo AC-3` o `--solo AC-3,AC-7`. La normalizacion vive aca y en un solo
/// lugar, asi que el resto del comando no tiene que saber que ahora hay varios.
fn filtrar(
    todas: Vec<verificacion::Verificacion>,
    solo: Option<&str>,
) -> Result<Vec<verificacion::Verificacion>, Exit> {
    let Some(objetivo) = solo else {
        return Ok(todas);
    };
    let pedidos: Vec<String> = objetivo
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(normalizar_ac)
        .collect();
    if pedidos.is_empty() {
        return Err(Exit {
            code: 2,
            message: Some("--solo no nombra ningun AC.".to_string()),
        });
    }
    // Se nombra CUAL falta, no "alguno": con varios pedidos, "no existe" a secas
    // obliga a probar de a uno.
    let faltantes: Vec<&String> = pedidos
        .iter()
        .filter(|p| !todas.iter().any(|v| &v.ac == *p))
        .collect();
    if !faltantes.is_empty() {
        return Err(Exit {
            code: 2,
            message: Some(format!(
                "El spec no declara {}.",
                faltantes
                    .iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            )),
        });
    }
    Ok(todas
        .into_iter()
        .filter(|v| pedidos.iter().any(|p| p == &v.ac))
        .collect())
}

fn normalizar_ac(pedido: &str) -> String {
    let arriba = pedido.to_ascii_uppercase();
    if arriba.starts_with("AC-") {
        arriba
    } else {
        format!("AC-{arriba}")
    }
}

fn sin_nada_que_verificar(
    rel: &str,
    verificaciones: &[verificacion::Verificacion],
    as_json: bool,
) -> anyhow::Result<()> {
    if as_json {
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "spec": rel,
                "total_ac": verificaciones.len(),
                "con_comando": 0,
                "resultados": [],
            }))?
        );
    } else {
        println!(
            "{rel}: {} AC, ninguno declara `Comando:`. Nada que ejecutar.",
            verificaciones.len()
        );
        println!(
            "Para que un AC se verifique solo, agregale debajo una linea:\n    Comando: `<como se prueba>`"
        );
    }
    Ok(())
}

fn raiz_de_ejecucion(paths: &HarnessPaths) -> &std::path::Path {
    paths.plans.parent().unwrap_or(&paths.repo_root)
}

fn emitir_json(
    fid: &str,
    rel: &str,
    stamp: &str,
    raiz: &std::path::Path,
    resultados: &[Resultado],
) {
    let rows: Vec<Value> = resultados
        .iter()
        .map(|r| {
            json!({
                "ac": r.ac,
                "comando": r.comando,
                "estado": r.estado.etiqueta(),
                "exit": r.exit,
                "duracion_ms": r.duracion_ms,
                "salida": r.salida,
            })
        })
        .collect();
    let rojos = resultados.iter().filter(|r| r.estado.bloquea()).count();
    let salida = json!({
        "feature": fid,
        "spec": rel,
        "raiz_ejecucion": raiz.display().to_string(),
        "corrida": stamp,
        "reporte": reporte_rel(fid),
        "verde": rojos == 0,
        "total_ac": resultados.len(),
        "con_comando": resultados.iter().filter(|r| r.comando.is_some()).count(),
        "resultados": rows,
    });
    if let Ok(texto) = serde_json::to_string_pretty(&salida) {
        println!("{texto}");
    }
}
