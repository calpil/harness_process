//! Feature #85: Copilot CLI como backend. Lo que el instalador necesita escribir
//! en `.github/` vive aca y no en los dos instaladores, para que el formato
//! exista en UN solo lugar (mismo patron que `perfil bloque`, feature #19).
//!
//! Dos archivos, los dos del USUARIO (cualquier repo con Copilot puede tenerlos
//! antes del arnes): `.github/copilot.json`, donde el arnes MEZCLA sus tres
//! hooks sin pisar claves ni hooks ajenos, y `.github/copilot-instructions.md`,
//! donde deja un bloque entre marcadores y no toca el texto de afuera. Las dos
//! escrituras son idempotentes y `quitar` las deshace.
//!
//! Lo que NO se toca, con aviso: un archivo que no se puede leer (no UTF-8,
//! JSON invalido, BOM + comentarios), un enlace simbolico, un marcador de
//! inicio sin su fin, un `hooks` vacio o no-objeto del usuario en `quitar`.
//! Ante la duda con un archivo ajeno, dejarlo como esta es lo correcto
//! (hallazgos de la revision adversarial de esta feature).
//!
//! Hechos medidos con Copilot CLI 1.0.83: los hooks son un comando por evento
//! (`command` + `shell`, `timeout` en milisegundos como en el ejemplo de la
//! documentacion, `5000`), `agentStop` responde `{"block","reason"}` y Copilot
//! lee `AGENTS.md` nativamente (por eso el bloque apunta ahi y no lo copia).

use std::path::Path;

use anyhow::Context;
use serde_json::{json, Map, Value};

pub const CONFIG: &str = ".github/copilot.json";
pub const INSTRUCCIONES: &str = ".github/copilot-instructions.md";
pub const MARCA_INICIO: &str = "<!-- harness:copilot:inicio -->";
pub const MARCA_FIN: &str = "<!-- harness:copilot:fin -->";
/// Los tres eventos que el arnes engancha. `agentStop` es el Stop: bloquea.
pub const EVENTOS: [&str; 3] = ["sessionStart", "agentStop", "sessionEnd"];
/// El modo con que se invoca `bin/harness-hook` (y `harness-hook.ps1`).
pub const MODO: &str = "copilot-json";
/// Milisegundos (el ejemplo de la documentacion de Copilot usa `5000`); el
/// mismo margen que los hooks de Codex y Gemini.
const TIMEOUT_MS: u64 = 120_000;
const BOM: char = '\u{feff}';

/// Un hook de `copilot.json` es del arnes si su comando invoca el modo
/// `copilot-json` del runtime: es el token que solo el arnes escribe, y no
/// depende de donde este instalado el runtime (`doctor` mira ademas que
/// apunte a `bin/harness-hook`). Un comando ajeno ENCADENADO con el del arnes
/// cuenta como del arnes (decision del spec: el pseudo-codigo lo define asi).
pub fn es_del_arnes(hook: &Value) -> bool {
    hook.get("command")
        .and_then(Value::as_str)
        .is_some_and(|c| c.contains(MODO))
}

fn comando_de(hook: &Value) -> String {
    match hook.get("command").and_then(Value::as_str) {
        Some(c) => c.to_string(),
        None => hook.to_string(),
    }
}

/// Mezcla los hooks del arnes en el JSON del usuario, EN EL LUGAR: el orden
/// de las claves se conserva (`preserve_order`), `hooks` no se mueve. Devuelve
/// el JSON nuevo y los avisos: un hook AJENO en uno de los tres eventos no se
/// pisa.
pub fn mezclar_config(json: Value, prefijo: &str, shell: &str) -> (Value, Vec<String>) {
    let mut avisos = Vec::new();
    let mut raiz = match json {
        Value::Object(m) => m,
        otro => {
            avisos.push(format!(
                "el archivo no era un objeto JSON ({otro}): se reemplaza"
            ));
            Map::new()
        }
    };
    if !matches!(raiz.get("hooks"), Some(Value::Object(_))) {
        if let Some(otro) = raiz.get("hooks") {
            avisos.push(format!("`hooks` no era un objeto ({otro}): se reemplaza"));
        }
        // `insert` sobre una clave existente conserva su posicion.
        raiz.insert("hooks".to_string(), Value::Object(Map::new()));
    }
    if let Some(Value::Object(hooks)) = raiz.get_mut("hooks") {
        for ev in EVENTOS {
            match hooks.get(ev) {
                Some(existente) if !es_del_arnes(existente) => {
                    avisos.push(format!(
                        "hook ajeno en {ev} (se deja): {}",
                        comando_de(existente)
                    ));
                }
                _ => {
                    hooks.insert(
                        ev.to_string(),
                        json!({
                            "command": format!("{prefijo} {MODO} {ev}"),
                            "shell": shell,
                            "timeout": TIMEOUT_MS,
                        }),
                    );
                }
            }
        }
    }
    (Value::Object(raiz), avisos)
}

/// Saca los hooks del arnes en el lugar y devuelve cuantos saco. `hooks` se
/// va solo si quedo vacio POR ESO: un `hooks` vacio o no-objeto del usuario
/// no se toca (no es del arnes).
pub fn sin_hooks_del_arnes(json: &mut Value) -> usize {
    let Some(raiz) = json.as_object_mut() else {
        return 0;
    };
    let Some(Value::Object(hooks)) = raiz.get_mut("hooks") else {
        return 0;
    };
    let antes = hooks.len();
    hooks.retain(|_, h| !es_del_arnes(h));
    let quitados = antes - hooks.len();
    if quitados > 0 && hooks.is_empty() {
        raiz.remove("hooks");
    }
    quitados
}

pub fn esta_vacio(json: &Value) -> bool {
    json.as_object().is_some_and(Map::is_empty)
}

/// JSON con sangria de dos y salto final (lo que un editor deja), y el BOM
/// si el archivo lo tenia. El formato original no se conserva: es lo unico
/// que cambia de un archivo ajeno, y se documenta.
pub fn render_json(v: &Value, bom: bool) -> String {
    let mut s = String::new();
    if bom {
        s.push(BOM);
    }
    s.push_str(&serde_json::to_string_pretty(v).unwrap_or_else(|_| v.to_string()));
    s.push('\n');
    s
}

/// El bloque que va a `copilot-instructions.md`. `arnes` es la carpeta del
/// arnes relativa a la raiz (vacia en layout root): las rutas de los comandos
/// tienen que ser las que existen.
pub fn bloque_de_instrucciones(arnes: &str) -> String {
    let a = arnes.trim().trim_end_matches('/');
    let a = if a.is_empty() {
        String::new()
    } else {
        format!("{a}/")
    };
    format!(
        "{MARCA_INICIO}\n\
         ## Harness Process (Copilot CLI)\n\
         \n\
         Este repositorio usa Harness Process. El flujo, los roles (leader, implementer,\n\
         reviewer), las reglas y el perfil del usuario estan en `AGENTS.md`, que Copilot\n\
         CLI carga solo: leelo antes de tocar nada. Los roles se aplican como fases\n\
         secuenciales en una sola sesion (Copilot no tiene subagentes del arnes).\n\
         \n\
         Hooks del arnes en `.github/copilot.json`: `sessionStart` muestra el estado,\n\
         `agentStop` corre el commit guard y BLOQUEA con el motivo si el repo queda\n\
         sucio o un gate falla (resolvelo o documenta el bloqueo antes de cerrar),\n\
         `sessionEnd` informa. Comandos: `sh {a}harness_cli status`,\n\
         `sh {a}harness_cli next`, `bash {a}harness_check.sh`.\n\
         {MARCA_FIN}\n"
    )
}

/// Rango (inicio, fin exclusivo) del primer bloque COMPLETO: el ultimo
/// marcador de inicio antes del primer marcador de fin. Un inicio sin fin (o
/// un fin sin inicio) no es un bloque y no se toca.
fn rango_del_bloque(lineas: &[&str]) -> Option<(usize, usize)> {
    let fin = lineas.iter().position(|l| l.trim_end() == MARCA_FIN)?;
    let inicio = lineas[..fin]
        .iter()
        .rposition(|l| l.trim_end() == MARCA_INICIO)?;
    Some((inicio, fin + 1))
}

/// `true` si hay un marcador del arnes sin su pareja: se avisa y no se toca.
pub fn marcadores_sueltos(texto: &str) -> bool {
    let lineas: Vec<&str> = texto.split('\n').collect();
    let inicios = lineas
        .iter()
        .filter(|l| l.trim_end() == MARCA_INICIO)
        .count();
    let fines = lineas.iter().filter(|l| l.trim_end() == MARCA_FIN).count();
    inicios != fines
}

/// El texto sin NINGUN bloque completo del arnes. La linea en blanco previa
/// se quita solo cuando el bloque estaba al final del archivo (la puso el
/// arnes); en el medio, el blanco es del usuario. Lo ajeno queda byte a byte
/// (un archivo sin salto final gana uno: no se puede distinguir). Si no queda
/// nada, devuelve vacio.
pub fn sin_bloque(texto: &str) -> String {
    let mut lineas: Vec<String> = texto.split('\n').map(str::to_string).collect();
    let mut hubo = false;
    loop {
        let prestadas: Vec<&str> = lineas.iter().map(String::as_str).collect();
        let Some((inicio, fin)) = rango_del_bloque(&prestadas) else {
            break;
        };
        hubo = true;
        let al_final = lineas[fin..].iter().all(|l| l.trim().is_empty());
        let desde = if al_final && inicio > 0 && lineas[inicio - 1].trim().is_empty() {
            inicio - 1
        } else {
            inicio
        };
        lineas.drain(desde..fin);
    }
    if !hubo {
        return texto.to_string();
    }
    let salida = lineas.join("\n");
    if salida.trim().is_empty() {
        String::new()
    } else {
        salida
    }
}

/// El texto con el bloque al final, una sola vez (reemplaza los anteriores).
pub fn con_bloque(texto: &str, bloque: &str) -> String {
    let mut base = sin_bloque(texto);
    if !base.is_empty() {
        if !base.ends_with('\n') {
            base.push('\n');
        }
        base.push('\n');
    }
    base.push_str(bloque);
    base
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Accion {
    Escrito,
    SinCambios,
    Quitado,
    Borrado,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cambio {
    /// Relativa a la raiz, con `/`.
    pub ruta: String,
    pub accion: Accion,
    pub detalle: String,
}

impl Cambio {
    fn de(ruta: &str, accion: Accion, detalle: impl Into<String>) -> Cambio {
        Cambio {
            ruta: ruta.to_string(),
            accion,
            detalle: detalle.into(),
        }
    }

    pub fn linea(&self) -> String {
        let verbo = match self.accion {
            Accion::Escrito => "Escrito",
            Accion::SinCambios => "Sin cambios",
            Accion::Quitado => "Quitado",
            Accion::Borrado => "Borrado",
        };
        if self.detalle.is_empty() {
            format!("{verbo}: {}", self.ruta)
        } else {
            format!("{verbo}: {} ({})", self.ruta, self.detalle)
        }
    }
}

fn es_enlace(ruta: &Path) -> bool {
    std::fs::symlink_metadata(ruta).is_ok_and(|m| m.file_type().is_symlink())
}

/// El texto del archivo; ausente = vacio. Cualquier OTRO fallo (no UTF-8,
/// permisos) es un error: un archivo que no se puede leer no se reescribe.
fn leer_texto(ruta: &Path) -> anyhow::Result<String> {
    match std::fs::read_to_string(ruta) {
        Ok(t) => Ok(t),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(String::new()),
        Err(e) => Err(anyhow::anyhow!("no se pudo leer {} ({e})", ruta.display())),
    }
}

/// El JSON del archivo y si tenia BOM; ausente o vacio = `{}`.
fn leer_json(ruta: &Path) -> anyhow::Result<(Value, bool)> {
    let texto = leer_texto(ruta)?;
    let bom = texto.starts_with(BOM);
    let cuerpo = texto.trim_start_matches(BOM);
    if cuerpo.trim().is_empty() {
        return Ok((json!({}), bom));
    }
    let v = serde_json::from_str(cuerpo)
        .with_context(|| format!("JSON invalido en {}", ruta.display()))?;
    Ok((v, bom))
}

fn escribir_si_cambio(ruta: &Path, nuevo: &str) -> anyhow::Result<bool> {
    let viejo = std::fs::read_to_string(ruta).ok();
    if viejo.as_deref() == Some(nuevo) {
        return Ok(false);
    }
    if let Some(dir) = ruta.parent() {
        std::fs::create_dir_all(dir)
            .with_context(|| format!("no se pudo crear {}", dir.display()))?;
    }
    crate::features::write_text_atomic(ruta, nuevo)
        .with_context(|| format!("no se pudo escribir {}", ruta.display()))?;
    Ok(true)
}

/// Escribe (mezclando) los dos archivos. Devuelve que paso con cada uno y los
/// avisos. Un archivo que no se puede leer, que es un enlace simbolico o que
/// tiene un marcador suelto NO se toca, y el aviso lo dice.
pub fn instalar(
    raiz: &Path,
    prefijo: &str,
    shell: &str,
    arnes: &str,
) -> anyhow::Result<(Vec<Cambio>, Vec<String>)> {
    let mut cambios = Vec::new();
    let mut avisos = Vec::new();
    let cfg = raiz.join(CONFIG);
    if es_enlace(&cfg) {
        avisos.push(format!("{CONFIG} es un enlace simbolico: no se toca"));
        cambios.push(Cambio::de(CONFIG, Accion::SinCambios, "enlace simbolico"));
    } else {
        match leer_json(&cfg) {
            Ok((json, bom)) => {
                let (nuevo, mas) = mezclar_config(json, prefijo, shell);
                avisos.extend(mas);
                let escrito = escribir_si_cambio(&cfg, &render_json(&nuevo, bom))?;
                cambios.push(Cambio::de(
                    CONFIG,
                    if escrito {
                        Accion::Escrito
                    } else {
                        Accion::SinCambios
                    },
                    format!("hooks: {}", EVENTOS.join(", ")),
                ));
            }
            Err(e) => {
                avisos.push(format!(
                    "{CONFIG}: {e:#}; no se toca. Los hooks del arnes van a mano: \"<evento>\": {{\"command\": \"{prefijo} {MODO} <evento>\", \"shell\": \"{shell}\"}} para {}",
                    EVENTOS.join(", ")
                ));
                cambios.push(Cambio::de(
                    CONFIG,
                    Accion::SinCambios,
                    "ilegible, no se toca",
                ));
            }
        }
    }
    let md = raiz.join(INSTRUCCIONES);
    if es_enlace(&md) {
        avisos.push(format!(
            "{INSTRUCCIONES} es un enlace simbolico: no se toca"
        ));
        cambios.push(Cambio::de(
            INSTRUCCIONES,
            Accion::SinCambios,
            "enlace simbolico",
        ));
    } else {
        match leer_texto(&md) {
            Ok(viejo) => {
                if marcadores_sueltos(&viejo) {
                    avisos.push(format!(
                        "{INSTRUCCIONES}: hay un marcador del arnes sin su pareja; se deja como esta y se agrega el bloque al final"
                    ));
                }
                let nuevo = con_bloque(&viejo, &bloque_de_instrucciones(arnes));
                let escrito = escribir_si_cambio(&md, &nuevo)?;
                cambios.push(Cambio::de(
                    INSTRUCCIONES,
                    if escrito {
                        Accion::Escrito
                    } else {
                        Accion::SinCambios
                    },
                    "bloque del arnes entre marcadores",
                ));
            }
            Err(e) => {
                avisos.push(format!("{INSTRUCCIONES}: {e:#}; no se toca"));
                cambios.push(Cambio::de(
                    INSTRUCCIONES,
                    Accion::SinCambios,
                    "ilegible, no se toca",
                ));
            }
        }
    }
    Ok((cambios, avisos))
}

/// Deshace `instalar`: quita los hooks del arnes y el bloque, y borra el
/// archivo que quede sin nada POR ESO. Un archivo ilegible o enlazado no se
/// toca (con aviso) y el otro se procesa igual. Sin archivos, devuelve vacio.
pub fn quitar(raiz: &Path) -> anyhow::Result<(Vec<Cambio>, Vec<String>)> {
    let mut cambios = Vec::new();
    let mut avisos = Vec::new();
    let cfg = raiz.join(CONFIG);
    if es_enlace(&cfg) {
        avisos.push(format!("{CONFIG} es un enlace simbolico: no se toca"));
        cambios.push(Cambio::de(CONFIG, Accion::SinCambios, "enlace simbolico"));
    } else if cfg.is_file() {
        match leer_json(&cfg) {
            Ok((mut json, bom)) => {
                let quitados = sin_hooks_del_arnes(&mut json);
                if quitados == 0 {
                    cambios.push(Cambio::de(
                        CONFIG,
                        Accion::SinCambios,
                        "no tenia hooks del arnes",
                    ));
                } else if esta_vacio(&json) {
                    std::fs::remove_file(&cfg)
                        .with_context(|| format!("no se pudo borrar {}", cfg.display()))?;
                    cambios.push(Cambio::de(
                        CONFIG,
                        Accion::Borrado,
                        "solo tenia lo del arnes",
                    ));
                } else {
                    escribir_si_cambio(&cfg, &render_json(&json, bom))?;
                    cambios.push(Cambio::de(
                        CONFIG,
                        Accion::Quitado,
                        "hooks del arnes; quedan las claves ajenas",
                    ));
                }
            }
            Err(e) => {
                avisos.push(format!("{CONFIG}: {e:#}; no se toca. Quita a mano los hooks cuyo comando contenga `{MODO}`"));
                cambios.push(Cambio::de(
                    CONFIG,
                    Accion::SinCambios,
                    "ilegible, no se toca",
                ));
            }
        }
    }
    let md = raiz.join(INSTRUCCIONES);
    if es_enlace(&md) {
        avisos.push(format!(
            "{INSTRUCCIONES} es un enlace simbolico: no se toca"
        ));
        cambios.push(Cambio::de(
            INSTRUCCIONES,
            Accion::SinCambios,
            "enlace simbolico",
        ));
    } else if md.is_file() {
        match leer_texto(&md) {
            Ok(viejo) => {
                if marcadores_sueltos(&viejo) {
                    avisos.push(format!("{INSTRUCCIONES}: hay un marcador del arnes sin su pareja; se deja como esta"));
                }
                let nuevo = sin_bloque(&viejo);
                if nuevo == viejo {
                    cambios.push(Cambio::de(
                        INSTRUCCIONES,
                        Accion::SinCambios,
                        "no tenia el bloque del arnes",
                    ));
                } else if nuevo.is_empty() {
                    std::fs::remove_file(&md)
                        .with_context(|| format!("no se pudo borrar {}", md.display()))?;
                    cambios.push(Cambio::de(
                        INSTRUCCIONES,
                        Accion::Borrado,
                        "solo tenia el bloque del arnes",
                    ));
                } else {
                    escribir_si_cambio(&md, &nuevo)?;
                    cambios.push(Cambio::de(
                        INSTRUCCIONES,
                        Accion::Quitado,
                        "bloque del arnes",
                    ));
                }
            }
            Err(e) => {
                avisos.push(format!("{INSTRUCCIONES}: {e:#}; no se toca"));
                cambios.push(Cambio::de(
                    INSTRUCCIONES,
                    Accion::SinCambios,
                    "ilegible, no se toca",
                ));
            }
        }
    }
    Ok((cambios, avisos))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;

    #[test]
    fn copilot_instalar_should_merge_hooks_without_touching_foreign_keys() {
        let (j, avisos) = mezclar_config(
            json!({"model": "gpt-5", "hooks": {"toolCall": {"command": "echo x", "shell": "bash"}}}),
            "bash \"/r/bin/harness-hook\"",
            "bash",
        );
        assert!(avisos.is_empty(), "{avisos:?}");
        assert_eq!(j["model"], "gpt-5");
        assert_eq!(j["hooks"]["toolCall"]["command"], "echo x");
        assert_eq!(
            j["hooks"]["agentStop"]["command"],
            "bash \"/r/bin/harness-hook\" copilot-json agentStop"
        );
        assert_eq!(j["hooks"]["agentStop"]["shell"], "bash");
        assert_eq!(j["hooks"]["agentStop"]["timeout"], 120_000);
        assert!(j["hooks"]["sessionStart"].is_object() && j["hooks"]["sessionEnd"].is_object());
    }

    #[test]
    fn copilot_instalar_should_keep_the_users_key_order() {
        // Hallazgo de la revision: sacar y volver a poner `hooks` lo mandaba al final.
        let (j, _) = mezclar_config(
            json!({"hooks": {"toolCall": {"command": "echo"}}, "model": "gpt-5"}),
            "bash hook",
            "bash",
        );
        let claves: Vec<&String> = j.as_object().unwrap().keys().collect();
        assert_eq!(claves, vec!["hooks", "model"], "{j}");
        let mut sin = j.clone();
        assert_eq!(sin_hooks_del_arnes(&mut sin), 3);
        let claves: Vec<&String> = sin.as_object().unwrap().keys().collect();
        assert_eq!(claves, vec!["hooks", "model"], "{sin}");
    }

    #[test]
    fn copilot_instalar_should_leave_a_foreign_hook_of_the_same_event_and_say_so() {
        let (j, avisos) = mezclar_config(
            json!({"hooks": {"sessionStart": {"command": "./mio.sh", "shell": "bash"}}}),
            "bash hook",
            "bash",
        );
        assert_eq!(j["hooks"]["sessionStart"]["command"], "./mio.sh");
        assert_eq!(
            avisos,
            vec!["hook ajeno en sessionStart (se deja): ./mio.sh".to_string()]
        );
        // Y el propio se reemplaza (por si cambio el prefijo); el ajeno sigue
        // avisando, que es lo que hay que ver en cada reinstalacion.
        let (j2, avisos2) = mezclar_config(j, "bash otro/bin/harness-hook", "bash");
        assert_eq!(avisos2.len(), 1, "{avisos2:?}");
        assert!(avisos2[0].contains("sessionStart"), "{avisos2:?}");
        assert_eq!(
            j2["hooks"]["agentStop"]["command"],
            "bash otro/bin/harness-hook copilot-json agentStop"
        );
        assert_eq!(j2["hooks"]["sessionStart"]["command"], "./mio.sh");
    }

    #[test]
    fn copilot_instalar_should_replace_a_non_object_with_a_warning() {
        let (j, avisos) = mezclar_config(json!([1, 2]), "bash hook", "bash");
        assert!(j["hooks"]["agentStop"].is_object());
        assert_eq!(avisos.len(), 1, "{avisos:?}");
        let (j, avisos) = mezclar_config(json!({"hooks": "nada"}), "bash hook", "bash");
        assert!(j["hooks"]["agentStop"].is_object());
        assert!(avisos[0].contains("`hooks` no era un objeto"), "{avisos:?}");
    }

    #[test]
    fn copilot_quitar_should_remove_only_our_hooks_and_report_emptiness() {
        let (mut j, _) = mezclar_config(
            json!({"model": "x", "hooks": {"toolCall": {"command": "echo"}}}),
            "bash hook",
            "bash",
        );
        assert_eq!(sin_hooks_del_arnes(&mut j), 3);
        assert_eq!(
            j,
            json!({"model": "x", "hooks": {"toolCall": {"command": "echo"}}})
        );
        let (mut solo, _) = mezclar_config(json!({}), "bash hook", "bash");
        assert_eq!(sin_hooks_del_arnes(&mut solo), 3);
        assert!(esta_vacio(&solo), "{solo}");
    }

    #[test]
    fn copilot_quitar_should_leave_an_empty_or_non_object_hooks_of_the_user_alone() {
        // Hallazgo de la revision: `{"hooks":{}}` se borraba como si fuera del arnes.
        for original in [
            json!({"hooks": {}}),
            json!({"model": "x", "hooks": {}}),
            json!({"hooks": [{"command": "echo"}]}),
            json!({"hooks": "nada"}),
        ] {
            let mut j = original.clone();
            assert_eq!(sin_hooks_del_arnes(&mut j), 0, "{original}");
            assert_eq!(j, original);
        }
    }

    #[test]
    fn copilot_instrucciones_block_should_round_trip_around_foreign_text() {
        let bloque = bloque_de_instrucciones("harness_process");
        assert!(bloque.contains("sh harness_process/harness_cli status"));
        assert!(bloque_de_instrucciones("").contains("sh harness_cli status"));
        assert!(bloque_de_instrucciones("arnes/").contains("bash arnes/harness_check.sh"));
        let ajeno = "# Mis reglas\n\nUsar tabs.\n";
        let con = con_bloque(ajeno, &bloque);
        assert!(con.starts_with(ajeno), "{con}");
        assert_eq!(con.matches(MARCA_INICIO).count(), 1);
        assert_eq!(con_bloque(&con, &bloque), con, "no es idempotente");
        assert_eq!(sin_bloque(&con), ajeno, "no vuelve byte a byte");
        // Sin texto ajeno: el bloque solo, y al quitarlo no queda nada.
        let solo = con_bloque("", &bloque);
        assert_eq!(solo, bloque);
        assert_eq!(sin_bloque(&solo), "");
        // Un bloque viejo con otro texto se reemplaza por el nuevo.
        let viejo = format!("{ajeno}\n{MARCA_INICIO}\nviejo\n{MARCA_FIN}\n");
        let nuevo = con_bloque(&viejo, &bloque);
        assert!(!nuevo.contains("\nviejo\n"), "{nuevo}");
        assert_eq!(sin_bloque(&nuevo), ajeno);
    }

    #[test]
    fn copilot_instrucciones_should_merge_two_blocks_keep_user_blanks_and_leave_orphan_markers() {
        // Hallazgos de la revision: dos bloques no convergian, un bloque en el
        // medio se comia una linea en blanco del usuario, y un inicio sin fin
        // borraba todo hasta el final.
        let bloque = bloque_de_instrucciones("");
        let dos = format!("{bloque}{bloque}");
        let uno = con_bloque(&dos, &bloque);
        assert_eq!(uno.matches(MARCA_INICIO).count(), 1, "{uno}");
        assert_eq!(sin_bloque(&dos), "");
        let medio = format!("A\n\n{bloque}\nC\n");
        assert_eq!(sin_bloque(&medio), "A\n\n\nC\n");
        let huerfano = format!("{MARCA_INICIO}\n# Mis notas\n");
        assert!(marcadores_sueltos(&huerfano));
        assert_eq!(
            sin_bloque(&huerfano),
            huerfano,
            "un inicio sin fin no es un bloque"
        );
        let con = con_bloque(&huerfano, &bloque);
        assert!(con.starts_with(&huerfano), "{con}");
        assert_eq!(con.matches(MARCA_FIN).count(), 1);
        // Y un fin suelto delante de un bloque completo tampoco arrastra nada.
        let fin_suelto = format!("{MARCA_FIN}\nmio\n\n{bloque}");
        assert_eq!(sin_bloque(&fin_suelto), fin_suelto);
    }

    #[test]
    fn copilot_instalar_and_quitar_should_touch_the_disk_idempotently() {
        let tmp = tempfile::tempdir().unwrap();
        let raiz = tmp.path();
        let (cambios, avisos) = instalar(raiz, "bash hook", "bash", "").unwrap();
        assert!(avisos.is_empty());
        assert!(
            cambios.iter().all(|c| c.accion == Accion::Escrito),
            "{cambios:?}"
        );
        let (cambios2, _) = instalar(raiz, "bash hook", "bash", "").unwrap();
        assert!(
            cambios2.iter().all(|c| c.accion == Accion::SinCambios),
            "{cambios2:?}"
        );
        assert_eq!(
            cambios2[0].linea(),
            "Sin cambios: .github/copilot.json (hooks: sessionStart, agentStop, sessionEnd)"
        );
        let (quitados, _) = quitar(raiz).unwrap();
        assert!(
            quitados.iter().all(|c| c.accion == Accion::Borrado),
            "{quitados:?}"
        );
        assert!(!raiz.join(CONFIG).exists() && !raiz.join(INSTRUCCIONES).exists());
        assert!(quitar(raiz).unwrap().0.is_empty());
    }

    #[test]
    fn copilot_instalar_should_keep_a_bom_and_skip_an_unreadable_json_or_md() {
        let tmp = tempfile::tempdir().unwrap();
        let raiz = tmp.path();
        std::fs::create_dir_all(raiz.join(".github")).unwrap();
        std::fs::write(raiz.join(CONFIG), "\u{feff}{\"model\": \"x\"}").unwrap();
        let (_, avisos) = instalar(raiz, "bash hook", "bash", "").unwrap();
        assert!(avisos.is_empty(), "{avisos:?}");
        let t = std::fs::read_to_string(raiz.join(CONFIG)).unwrap();
        assert!(t.starts_with('\u{feff}'), "se perdio el BOM");
        assert!(t.contains("copilot-json agentStop"));
        // JSON con comentario: no se toca, se avisa, y el .md se escribe igual.
        std::fs::write(raiz.join(CONFIG), "// mio\n{\"model\": \"x\"}\n").unwrap();
        std::fs::remove_file(raiz.join(INSTRUCCIONES)).unwrap();
        let (cambios, avisos) = instalar(raiz, "bash hook", "bash", "").unwrap();
        assert_eq!(
            std::fs::read_to_string(raiz.join(CONFIG)).unwrap(),
            "// mio\n{\"model\": \"x\"}\n"
        );
        assert!(
            avisos
                .iter()
                .any(|a| a.contains("JSON invalido") && a.contains("no se toca")),
            "{avisos:?}"
        );
        assert_eq!(cambios[1].accion, Accion::Escrito, "{cambios:?}");
        // .md que no es UTF-8 (Latin-1): no se reemplaza (era el hallazgo bloqueante).
        let latin1 = b"Reglas: configuraci\xf3n del equipo\n".to_vec();
        std::fs::write(raiz.join(INSTRUCCIONES), &latin1).unwrap();
        let (cambios, avisos) = instalar(raiz, "bash hook", "bash", "").unwrap();
        assert_eq!(
            std::fs::read(raiz.join(INSTRUCCIONES)).unwrap(),
            latin1,
            "se piso el archivo no UTF-8"
        );
        assert!(
            avisos
                .iter()
                .any(|a| a.contains(INSTRUCCIONES) && a.contains("no se toca")),
            "{avisos:?}"
        );
        assert_eq!(cambios[1].accion, Accion::SinCambios);
        // Y quitar sobre el JSON ilegible sigue con el .md.
        std::fs::write(raiz.join(INSTRUCCIONES), bloque_de_instrucciones("")).unwrap();
        let (cambios, avisos) = quitar(raiz).unwrap();
        assert!(avisos.iter().any(|a| a.contains(CONFIG)), "{avisos:?}");
        assert_eq!(cambios[1].accion, Accion::Borrado, "{cambios:?}");
    }

    #[cfg(unix)]
    #[test]
    fn copilot_instalar_should_not_replace_a_symlink() {
        let tmp = tempfile::tempdir().unwrap();
        let raiz = tmp.path();
        std::fs::create_dir_all(raiz.join(".github")).unwrap();
        std::fs::write(raiz.join("AGENTS.md"), "# agentes\n").unwrap();
        std::os::unix::fs::symlink("../AGENTS.md", raiz.join(INSTRUCCIONES)).unwrap();
        let (cambios, avisos) = instalar(raiz, "bash hook", "bash", "").unwrap();
        assert!(
            es_enlace(&raiz.join(INSTRUCCIONES)),
            "el enlace se reemplazo por un archivo"
        );
        assert_eq!(
            std::fs::read_to_string(raiz.join("AGENTS.md")).unwrap(),
            "# agentes\n"
        );
        assert!(
            avisos.iter().any(|a| a.contains("enlace simbolico")),
            "{avisos:?}"
        );
        assert_eq!(cambios[1].accion, Accion::SinCambios);
        let (cambios, _) = quitar(raiz).unwrap();
        assert!(es_enlace(&raiz.join(INSTRUCCIONES)));
        assert_eq!(cambios[1].accion, Accion::SinCambios);
    }
}
