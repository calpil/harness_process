//! Consolidacion de lecciones asistida por LLM (feature #28).
//!
//! Ultimo hito del PRD de aprendizaje, y **la unica parte del arnes que necesita
//! un modelo**. Salio de la OBS-1 de la #21, que la aparto por no ser
//! verificable entonces.
//!
//! El riesgo es de otra clase que en el resto del arnes: es la primera vez que
//! contenido de `docs/` sale del repo hacia un proceso externo, y la primera vez
//! que la salida de un modelo influye sobre archivos del usuario. Por eso las
//! defensas son **estructurales** y no de prosa:
//!
//! - **`detectar()` no recibe `&HarnessPaths`.** No puede escribir aunque
//!   quiera: no tiene con que.
//! - **Al modelo se le manda solo `nombre`, `descripcion` y `triggers`.** El
//!   CUERPO —los procedimientos y los pitfalls, que son la parte cara— nunca
//!   sale de `docs/`.
//! - **El prompt viaja como un item de argv, jamas por `sh -c`.** Una
//!   descripcion con backticks o `$(...)` no puede ejecutar nada. Por eso este
//!   modulo NO reusa `verificacion::ejecutar`, que corre con `sh -c`.
//! - **Lo que muta se toma de argv, no de la respuesta del modelo.** La mitad
//!   que escribe se verifica sin backend y de forma determinista.
//!
//! Y una medicion que condiciona el diseno: sobre las 9 lecciones reales de este
//! repo hay **un** solapamiento (Jaccard 0.400 sobre triggers) y el siguiente
//! esta en 0.050. No hay zona gris, asi que **no se puede calibrar un umbral**:
//! la confianza se reporta sin filtrar y decide quien lee (OBS-3).

use std::collections::BTreeMap;
use std::path::Path;
use std::time::Duration;

use serde_json::Value;

/// Backends conocidos, en orden. Ninguno esta pinneado: el primero que exista
/// gana, y `HARNESS_CONSOLIDAR_CMD` los sobreescribe a todos.
pub const CLIS: [(&str, &[&str]); 2] = [("claude", &["-p"]), ("kimi", &["-p"])];

/// Segundos antes de cortar al backend. Un modelo colgado no cuelga el comando.
pub const TIMEOUT_DEFAULT: u64 = 120;

/// Umbral desde `rules.consolidar_timeout_segundos`.
pub fn timeout_segundos(data: &Value) -> u64 {
    data.get("rules")
        .and_then(|r| r.get("consolidar_timeout_segundos"))
        .and_then(Value::as_u64)
        .filter(|v| *v > 0)
        .unwrap_or(TIMEOUT_DEFAULT)
}

/// De donde salio el backend elegido. Es un enum y no un `Option<String>`
/// porque el mensaje de skip tiene que poder decir POR QUE no hay backend.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Backend {
    /// `HARNESS_CONSOLIDAR_CMD`: el usuario eligio cual.
    Override(Vec<String>),
    /// Un CLI de la tabla, detectado en el PATH.
    Cli { nombre: String, argv: Vec<String> },
    /// La regla no esta: la feature esta APAGADA. Ni se mira el entorno.
    Apagada,
    /// La regla esta pero no hay con que hablar.
    SinNinguno { hay_api_key: bool },
}

impl Backend {
    pub fn argv(&self) -> Option<&[String]> {
        match self {
            Backend::Override(a) => Some(a),
            Backend::Cli { argv, .. } => Some(argv),
            _ => None,
        }
    }

    /// El mensaje que ve el usuario cuando no se puede consolidar. Dice que
    /// falto, no solo que no se pudo.
    pub fn motivo_del_skip(&self) -> Option<String> {
        match self {
            Backend::Apagada => Some(
                "Consolidacion APAGADA: no hay `rules.consolidar_backend` en feature_list.json.\n    \
                 Encendela con: \"rules\": { \"consolidar_backend\": \"auto\" }"
                    .to_string(),
            ),
            Backend::SinNinguno { hay_api_key: true } => Some(format!(
                "Sin backend: hay una API key en el entorno, pero este arnes NO habla HTTP.\n    \
                 Declara un CLI: HARNESS_CONSOLIDAR_CMD=\"<comando>\"\n    \
                 CLIs que detecta solo: {}",
                CLIS.iter().map(|(n, _)| *n).collect::<Vec<_>>().join(", ")
            )),
            Backend::SinNinguno { hay_api_key: false } => Some(format!(
                "Sin backend: no se encontro ninguno de {} en el PATH.\n    \
                 Declara uno con HARNESS_CONSOLIDAR_CMD=\"<comando>\".",
                CLIS.iter().map(|(n, _)| *n).collect::<Vec<_>>().join(", ")
            )),
            _ => None,
        }
    }
}

/// Variables de API key que se reconocen SOLO para poder decir en el skip que el
/// arnes no habla HTTP (OBS-1: ese camino quedo fuera de alcance).
const API_KEYS: [&str; 3] = ["ANTHROPIC_API_KEY", "OPENAI_API_KEY", "MOONSHOT_API_KEY"];

/// La cadena: **override -> CLI -> skip limpio**.
///
/// `override_cmd` es el valor de `HARNESS_CONSOLIDAR_CMD`, que lee el comando y
/// pasa aca: este modulo no toca el entorno.
///
/// El tramo de API key NO se implementa (decision del usuario 2026-08-18,
/// OBS-1): serian tres formatos de request/respuesta/error escritos a ciegas,
/// sin forma de verificarlos aca. Se nombra en el skip en vez de disimularse.
pub fn resolver_backend(
    data: &Value,
    override_cmd: Option<&str>,
    existe: impl Fn(&str) -> bool,
) -> Backend {
    // Interruptor primero: sin la regla no se mira ni el entorno.
    let encendida = data
        .get("rules")
        .and_then(|r| r.get("consolidar_backend"))
        .and_then(Value::as_str)
        .is_some_and(|v| !v.trim().is_empty());
    if !encendida {
        return Backend::Apagada;
    }
    // El override llega por parametro y no se lee aca: asi la funcion es pura
    // y los tests no dependen del entorno del proceso, que en Rust es
    // COMPARTIDO entre los tests que corren en paralelo (se descubrio con dos
    // tests pisandose entre si).
    if let Some(cmd) = override_cmd.filter(|v| !v.trim().is_empty()) {
        let argv: Vec<String> = cmd.split_whitespace().map(str::to_string).collect();
        if !argv.is_empty() {
            return Backend::Override(argv);
        }
    }
    for (nombre, flags) in CLIS {
        if existe(nombre) {
            let mut argv = vec![nombre.to_string()];
            argv.extend(flags.iter().map(|f| (*f).to_string()));
            return Backend::Cli {
                nombre: nombre.to_string(),
                argv,
            };
        }
    }
    let hay_api_key = API_KEYS
        .iter()
        .any(|k| std::env::var(k).is_ok_and(|v| !v.trim().is_empty()));
    Backend::SinNinguno { hay_api_key }
}

/// Lo unico que el modelo ve de una leccion. **Nunca el cuerpo.**
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resumen {
    pub nombre: String,
    pub descripcion: String,
    pub triggers: Vec<String>,
    /// Referencias locales para detectar parejas candidatas. No forman parte
    /// del prompt: el backend sigue viendo solo nombre, descripción y triggers.
    pub relacionadas: Vec<String>,
}

/// Resultado local de interpretar `relacionadas`: los avisos explican por qué
/// una referencia escrita no se elevó a candidato.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SenalesRelacionadas {
    pub candidatos: Vec<Candidato>,
    pub diagnosticos: Vec<String>,
}

/// Arma el prompt. Funcion **pura**: se puede inspeccionar en un test para
/// confirmar que el cuerpo no esta (AC-7).
pub fn prompt(resumenes: &[Resumen]) -> String {
    let catalogo: Vec<Value> = resumenes
        .iter()
        .map(|r| {
            serde_json::json!({
                "nombre": r.nombre,
                "descripcion": r.descripcion,
                "triggers": r.triggers,
            })
        })
        .collect();
    let json = serde_json::to_string(&serde_json::json!({ "lecciones": catalogo }))
        .unwrap_or_else(|_| "{\"lecciones\":[]}".to_string());
    format!(
        "Sos un asistente que detecta lecciones SOLAPADAS en un catalogo de memoria \
         procedural de un proyecto de software.\n\n\
         Te doy el nombre, una descripcion de una oracion y los triggers de cada una. \
         NO tenes el cuerpo, y no lo necesitas.\n\n\
         Devolve UN SOLO objeto JSON, sin markdown, sin texto alrededor, con esta forma:\n\
         {{\"candidatos\":[{{\"miembros\":[\"a\",\"b\"],\"motivo\":\"<una oracion, max 200 chars>\",\"confianza\":0.0}}]}}\n\n\
         Reglas:\n\
         - Un candidato agrupa lecciones que ensenan LO MISMO, no solo que hablan de temas vecinos.\n\
         - Si no hay solapamiento real, devolve {{\"candidatos\":[]}}. NO inventes grupos.\n\
         - `miembros` usa los nombres exactos del catalogo.\n\
         - `confianza` va de 0.0 a 1.0.\n\n\
         Catalogo:\n{json}"
    )
}

/// Un candidato tal como lo devolvio el modelo, antes de validarse.
#[derive(Debug, Clone, PartialEq)]
pub struct Candidato {
    pub miembros: Vec<String>,
    pub motivo: String,
    pub confianza: f64,
}

/// Detecta parejas que se señalan mutuamente en `relacionadas`.
///
/// Es deliberadamente local y puro: no lee cuerpos, no interpreta una
/// referencia como ruta y no depende de un backend. Una referencia unilateral
/// o a algo que no pertenece al catálogo elegible deja un diagnóstico, pero no
/// inventa una candidata.
pub fn por_relacionadas(resumenes: &[Resumen]) -> SenalesRelacionadas {
    let por_nombre: BTreeMap<&str, &Resumen> =
        resumenes.iter().map(|r| (r.nombre.as_str(), r)).collect();
    let mut pares: BTreeMap<(String, String), Candidato> = BTreeMap::new();
    let mut diagnosticos = Vec::new();

    for origen in resumenes {
        for destino in &origen.relacionadas {
            let destino = destino.trim();
            if destino.is_empty() || destino == origen.nombre {
                continue;
            }
            let Some(otra) = por_nombre.get(destino) else {
                diagnosticos.push(format!(
                    "relacionadas: '{}' declara '{}' pero no es una leccion activa y valida; se ignora",
                    origen.nombre, destino
                ));
                continue;
            };
            if !otra.relacionadas.iter().any(|r| r.trim() == origen.nombre) {
                diagnosticos.push(format!(
                    "relacionadas: '{}' declara '{}' pero la referencia no es mutua; se ignora",
                    origen.nombre, destino
                ));
                continue;
            }
            let (a, b) = if origen.nombre < otra.nombre {
                (origen.nombre.clone(), otra.nombre.clone())
            } else {
                (otra.nombre.clone(), origen.nombre.clone())
            };
            pares.entry((a.clone(), b.clone())).or_insert(Candidato {
                miembros: vec![a.clone(), b.clone()],
                motivo: format!("relacionadas mutuas: '{a}' declara '{b}' y '{b}' declara '{a}'"),
                confianza: 1.0,
            });
        }
    }
    diagnosticos.sort();
    diagnosticos.dedup();
    SenalesRelacionadas {
        candidatos: pares.into_values().collect(),
        diagnosticos,
    }
}

/// El backend propone desde nombre/descripción/triggers. La razón se etiqueta
/// antes de mezclarse con señales locales para que la salida mantenga su
/// procedencia revisable.
pub fn marcar_triggers(mut candidatos: Vec<Candidato>) -> Vec<Candidato> {
    for candidato in &mut candidatos {
        candidato.motivo = format!("triggers/LLM: {}", candidato.motivo);
    }
    candidatos
}

/// Une candidatas equivalentes por el conjunto canónico de miembros. Si una
/// pareja llegó tanto por triggers/LLM como por `relacionadas`, conserva ambas
/// razones en vez de listar A-B dos veces.
pub fn unir_candidatos(candidatos: Vec<Candidato>) -> Vec<Candidato> {
    let mut unidos: BTreeMap<Vec<String>, Candidato> = BTreeMap::new();
    for candidato in candidatos {
        // La llave es canónica para que A-B y B-A coincidan, pero la salida
        // conserva el orden que propuso la primera fuente (compatibilidad de
        // CLI y una instrucción de fusión más legible).
        let mut llave = candidato.miembros.clone();
        llave.sort();
        llave.dedup();
        match unidos.get_mut(&llave) {
            Some(anterior) => {
                if !anterior.motivo.contains(&candidato.motivo) {
                    anterior.motivo = format!("{}; {}", anterior.motivo, candidato.motivo);
                }
                anterior.confianza = anterior.confianza.max(candidato.confianza);
            }
            None => {
                unidos.insert(llave, candidato);
            }
        }
    }
    unidos.into_values().collect()
}

/// Estructura local que un paraguas debe heredar antes de que una persona
/// escriba su explicación. No crea archivos ni toma decisiones sobre prosa.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BorradorParaguas {
    pub miembros: Vec<String>,
    pub triggers: Vec<String>,
    pub cuerpo: String,
}

/// Une los datos mecánicos de una selección aceptada. Los triggers se
/// normalizan a minúscula y se ordenan para que capitalización u orden de los
/// miembros no cambien el borrador. Los punteros se originan exclusivamente en
/// los nombres ya validados de las lecciones seleccionadas.
pub fn preparar_paraguas(miembros: &[(String, Vec<String>)]) -> BorradorParaguas {
    let mut nombres: Vec<String> = miembros.iter().map(|(nombre, _)| nombre.clone()).collect();
    nombres.sort();
    nombres.dedup();

    let mut triggers: Vec<String> = miembros
        .iter()
        .flat_map(|(_, triggers)| triggers)
        .map(|trigger| trigger.trim().to_lowercase())
        .filter(|trigger| !trigger.is_empty())
        .collect();
    triggers.sort();
    triggers.dedup();

    let punteros = nombres
        .iter()
        .map(|nombre| format!("- [[{nombre}]]"))
        .collect::<Vec<_>>()
        .join("\n");
    let cuerpo = format!(
        "## Cuando aplica\n\nPendiente de redaccion humana.\n\n\
         ## Miembros a consolidar\n\n{punteros}\n\n\
         ## Procedimiento\n\nPendiente de redaccion humana.\n\n\
         ## Pitfalls\n\nPendiente de redaccion humana.\n\n\
         ## Verificacion\n\nPendiente de redaccion humana.\n"
    );
    BorradorParaguas {
        miembros: nombres,
        triggers,
        cuerpo,
    }
}

/// Por que se descarto un candidato. Se imprime: una alucinacion descartada en
/// silencio es indistinguible de un modelo que no encontro nada.
#[derive(Debug, Clone, PartialEq)]
pub enum Descarte {
    NoExiste {
        candidato: Candidato,
        nombre: String,
    },
    Pinneada {
        candidato: Candidato,
        nombre: String,
    },
    MuyChico {
        candidato: Candidato,
    },
}

impl Descarte {
    pub fn mensaje(&self) -> String {
        match self {
            Descarte::NoExiste { nombre, .. } => {
                format!("nombra una leccion que no existe: '{nombre}'")
            }
            Descarte::Pinneada { nombre, .. } => {
                format!("toca la leccion pinneada '{nombre}'")
            }
            Descarte::MuyChico { .. } => "tiene menos de dos miembros".to_string(),
        }
    }
}

/// Extrae el primer objeto JSON de llaves balanceadas del stdout.
///
/// Agnostico de backend a proposito: `claude -p` devuelve JSON pelado y
/// `kimi -p` lo envuelve (`• {...}` y una linea de sesion al final). Buscar el
/// primer objeto balanceado sirve para los dos sin conocer a ninguno.
pub fn extraer_json(salida: &str) -> Option<Value> {
    let bytes: Vec<char> = salida.chars().collect();
    let inicio = bytes.iter().position(|c| *c == '{')?;
    let mut nivel = 0usize;
    let mut en_string = false;
    let mut escapado = false;
    for (i, c) in bytes.iter().enumerate().skip(inicio) {
        if escapado {
            escapado = false;
            continue;
        }
        match c {
            '\\' if en_string => escapado = true,
            '"' => en_string = !en_string,
            '{' if !en_string => nivel += 1,
            '}' if !en_string => {
                nivel -= 1;
                if nivel == 0 {
                    let texto: String = bytes[inicio..=i].iter().collect();
                    return serde_json::from_str(&texto).ok();
                }
            }
            _ => {}
        }
    }
    None
}

/// Lee los candidatos de la respuesta ya parseada. Tolerante: lo que no tenga
/// forma se ignora en vez de romper.
pub fn leer_candidatos(v: &Value) -> Vec<Candidato> {
    let Some(lista) = v.get("candidatos").and_then(Value::as_array) else {
        return Vec::new();
    };
    lista
        .iter()
        .filter_map(|c| {
            let miembros: Vec<String> = c
                .get("miembros")?
                .as_array()?
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect();
            Some(Candidato {
                miembros,
                motivo: c
                    .get("motivo")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .trim()
                    .to_string(),
                confianza: c.get("confianza").and_then(Value::as_f64).unwrap_or(0.0),
            })
        })
        .collect()
}

/// Separa lo aceptable de lo descartado. **Funcion pura** y sin filesystem: por
/// eso `detectar` no puede escribir aunque quiera.
pub fn validar(
    candidatos: Vec<Candidato>,
    existentes: &[String],
    pinneadas: &[String],
) -> (Vec<Candidato>, Vec<Descarte>) {
    let mut ok = Vec::new();
    let mut fuera = Vec::new();
    for c in candidatos {
        if c.miembros.len() < 2 {
            fuera.push(Descarte::MuyChico { candidato: c });
            continue;
        }
        if let Some(n) = c.miembros.iter().find(|m| !existentes.contains(m)) {
            let nombre = n.clone();
            fuera.push(Descarte::NoExiste {
                candidato: c,
                nombre,
            });
            continue;
        }
        if let Some(n) = c.miembros.iter().find(|m| pinneadas.contains(m)) {
            let nombre = n.clone();
            fuera.push(Descarte::Pinneada {
                candidato: c,
                nombre,
            });
            continue;
        }
        ok.push(c);
    }
    (ok, fuera)
}

/// Corre el backend con el prompt como UN item de argv.
///
/// Deliberadamente NO reusa `verificacion::ejecutar`: esa corre con `sh -c`, y
/// aca el texto de las lecciones no puede pasar por un shell.
pub fn preguntar(
    argv: &[String],
    prompt: &str,
    cwd: &Path,
    timeout: Duration,
) -> Result<String, String> {
    use std::io::Read;
    use std::process::{Command, Stdio};
    use wait_timeout::ChildExt;

    let Some((exe, resto)) = argv.split_first() else {
        return Err("comando vacio".to_string());
    };
    let mut hijo = Command::new(exe)
        .args(resto)
        .arg(prompt) // <- un item de argv, nunca `sh -c`
        .current_dir(cwd)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("no se pudo ejecutar '{exe}': {e}"))?;
    let estado = match hijo.wait_timeout(timeout) {
        Ok(Some(s)) => s,
        Ok(None) => {
            let _ = hijo.kill();
            let _ = hijo.wait();
            return Err(format!("el backend no respondio en {}s", timeout.as_secs()));
        }
        Err(e) => return Err(format!("fallo esperando al backend: {e}")),
    };
    let mut salida = String::new();
    if let Some(mut out) = hijo.stdout.take() {
        let _ = out.read_to_string(&mut salida);
    }
    if !estado.success() && salida.trim().is_empty() {
        let mut err = String::new();
        if let Some(mut e) = hijo.stderr.take() {
            let _ = e.read_to_string(&mut err);
        }
        return Err(format!("el backend fallo: {}", err.trim()));
    }
    Ok(salida)
}

/// Por que un paraguas no puede recibir todavia lo que las miembros ensenaban.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Falta {
    /// Sigue teniendo los placeholders de la plantilla.
    Esqueleto,
    /// No hereda un trigger de una miembro: `buscar` puntua una leccion activa
    /// 100 y una archivada 30, asi que el conocimiento dejaria de encontrarse.
    Trigger { miembro: String, trigger: String },
    /// No deja el puntero de recuperacion `[[miembro]]`.
    Puntero { miembro: String },
}

impl Falta {
    pub fn mensaje(&self) -> String {
        match self {
            Falta::Esqueleto => {
                "el paraguas todavia tiene los placeholders de la plantilla: archivar contra un esqueleto perderia el conocimiento".to_string()
            }
            Falta::Trigger { miembro, trigger } => format!(
                "no hereda el trigger '{trigger}' de '{miembro}' (buscar puntua activa=100 vs archivada=30: se dejaria de encontrar)"
            ),
            Falta::Puntero { miembro } => {
                format!("no cita [[{miembro}]]: quedaria sin puntero de recuperacion")
            }
        }
    }
}

/// Marcas de la plantilla que delatan un paraguas sin escribir.
const PLACEHOLDERS: [&str; 3] = [
    "<una sola oracion",
    "<cuando aplica",
    "<que hacer, paso a paso",
];

/// El paraguas tiene que poder reemplazar a lo que archiva. **Funcion pura.**
pub fn revisar_paraguas(
    paraguas_texto: &str,
    paraguas_triggers: &[String],
    miembros: &[(String, Vec<String>)],
) -> Vec<Falta> {
    let mut faltas = Vec::new();
    let bajo = paraguas_texto.to_lowercase();
    if PLACEHOLDERS
        .iter()
        .any(|p| bajo.contains(&p.to_lowercase()))
    {
        faltas.push(Falta::Esqueleto);
    }
    for (nombre, triggers) in miembros {
        for t in triggers {
            if !paraguas_triggers.iter().any(|p| p.eq_ignore_ascii_case(t)) {
                faltas.push(Falta::Trigger {
                    miembro: nombre.clone(),
                    trigger: t.clone(),
                });
            }
        }
        if !paraguas_texto.contains(&format!("[[{nombre}]]")) {
            faltas.push(Falta::Puntero {
                miembro: nombre.clone(),
            });
        }
    }
    faltas
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;
    use serde_json::json;

    /// Salida REAL de `claude -p` (capturada el 2026-08-18): JSON pelado.
    const SALIDA_CLAUDE: &str =
        r#"{"candidatos":[{"miembros":["a","b"],"motivo":"prueba","confianza":0.9}]}"#;

    /// Salida REAL de `kimi -p` (capturada el 2026-08-18): banner de version,
    /// una vinneta de razonamiento, el JSON con vinneta, y la linea de sesion.
    const SALIDA_KIMI: &str = "kimi version 0.36.0\n\
        • The user asks to return only this JSON, without markdown. Simple request.\n\n\
        • {\"candidatos\":[{\"miembros\":[\"a\",\"b\"],\"motivo\":\"prueba\",\"confianza\":0.9}]}\n\n\
        To resume this session: kimi -r session_dd6bc69a-1e57-4dbe-96c9-78e256b07dc3";

    fn encendida() -> Value {
        json!({"rules": {"consolidar_backend": "auto"}})
    }

    #[test]
    fn consolidar_should_be_off_without_the_rule() {
        // Apagada de forma ESTRUCTURAL: ni siquiera se mira el entorno.
        assert_eq!(
            resolver_backend(&json!({}), None, |_| true),
            Backend::Apagada
        );
        assert_eq!(
            resolver_backend(&json!({"rules": {}}), None, |_| true),
            Backend::Apagada
        );
        assert_eq!(
            resolver_backend(
                &json!({"rules": {"consolidar_backend": "  "}}),
                None,
                |_| true
            ),
            Backend::Apagada
        );
        let apagada = resolver_backend(&json!({}), None, |_| true);
        assert!(apagada.argv().is_none(), "apagada no puede dar un comando");
        assert!(
            apagada
                .motivo_del_skip()
                .unwrap()
                .contains("consolidar_backend")
        );
    }

    #[test]
    fn consolidar_should_detect_the_first_available_cli() {
        // El primero de la tabla que exista, sin pinnear ninguno.
        let b = resolver_backend(&encendida(), None, |n| n == "kimi");
        assert_eq!(
            b,
            Backend::Cli {
                nombre: "kimi".into(),
                argv: vec!["kimi".into(), "-p".into()]
            }
        );
        let b = resolver_backend(&encendida(), None, |_| true);
        assert!(
            matches!(&b, Backend::Cli { nombre, .. } if nombre == "claude"),
            "con los dos disponibles gana el primero de la tabla: {b:?}"
        );
    }

    #[test]
    fn consolidar_should_skip_cleanly_without_a_backend() {
        let b = resolver_backend(&encendida(), None, |_| false);
        assert!(matches!(b, Backend::SinNinguno { .. }));
        assert!(b.argv().is_none());
        let msg = b.motivo_del_skip().unwrap();
        assert!(msg.contains("claude") && msg.contains("kimi"), "{msg}");
    }

    #[test]
    fn consolidar_should_name_the_api_key_limitation() {
        // OBS-1: el camino HTTP quedo fuera de alcance, y el skip lo DICE.
        let b = Backend::SinNinguno { hay_api_key: true };
        let msg = b.motivo_del_skip().unwrap();
        assert!(msg.contains("NO habla HTTP"), "{msg}");
        assert!(msg.contains("HARNESS_CONSOLIDAR_CMD"), "{msg}");
    }

    #[test]
    fn consolidar_should_never_send_the_lesson_body() {
        // La frontera de la feature: el modelo ve una oracion y keywords, jamas
        // los procedimientos ni los pitfalls.
        let r = vec![Resumen {
            nombre: "una-leccion".into(),
            descripcion: "Una sola oracion.".into(),
            triggers: vec!["uno".into(), "dos".into()],
            relacionadas: vec!["solo-local".into()],
        }];
        let p = prompt(&r);
        assert!(p.contains("una-leccion") && p.contains("Una sola oracion."));
        assert!(p.contains("uno") && p.contains("dos"));
        assert!(
            !p.contains("solo-local"),
            "el prompt no lleva relacionadas: {p}"
        );
        for prohibido in [
            "## Cuando aplica",
            "## Procedimiento",
            "## Pitfalls",
            "## Verificacion",
        ] {
            assert!(
                !p.contains(prohibido),
                "el prompt lleva el cuerpo: {prohibido}"
            );
        }
        // Y le dice al modelo que la respuesta vacia es valida.
        assert!(p.contains("{\"candidatos\":[]}"), "{p}");
    }

    #[test]
    fn consolidar_should_parse_the_output_of_both_backends() {
        // Fixtures REALES, no inventados: claude devuelve JSON pelado y kimi lo
        // envuelve en vinnetas con banner y linea de sesion.
        for (quien, salida) in [("claude", SALIDA_CLAUDE), ("kimi", SALIDA_KIMI)] {
            let v = extraer_json(salida).unwrap_or_else(|| panic!("{quien}: no parseo"));
            let c = leer_candidatos(&v);
            assert_eq!(c.len(), 1, "{quien}");
            assert_eq!(c[0].miembros, ["a", "b"], "{quien}");
            assert_eq!(c[0].motivo, "prueba", "{quien}");
            assert!((c[0].confianza - 0.9).abs() < 1e-9, "{quien}");
        }
    }

    #[test]
    fn consolidar_should_survive_a_garbage_answer() {
        // Un modelo que balbucea no puede romper el flujo.
        for basura in [
            "",
            "no se, la verdad",
            "```json\n{roto",
            "{\"otra_cosa\": 1}",
            "{\"candidatos\": \"no es una lista\"}",
        ] {
            let v = extraer_json(basura);
            let c = v.map(|v| leer_candidatos(&v)).unwrap_or_default();
            assert!(c.is_empty(), "deberia quedar vacio: {basura}");
        }
        // Y la respuesta vacia legitima tambien es vacia, sin error.
        let v = extraer_json("{\"candidatos\":[]}").unwrap();
        assert!(leer_candidatos(&v).is_empty());
    }

    #[test]
    fn extraer_json_should_ignore_braces_inside_strings() {
        let salida = r#"habla de {llaves} y despues: {"candidatos":[{"miembros":["a","b"],"motivo":"con } adentro","confianza":0.1}]}"#;
        // El primer objeto balanceado es `{llaves}`, que no es JSON valido ->
        // el parser devuelve None en vez de inventar. Es el limite conocido.
        assert!(
            extraer_json(salida).is_none(),
            "no puede adivinar cual objeto es"
        );
    }

    fn cand(miembros: &[&str]) -> Candidato {
        Candidato {
            miembros: miembros.iter().map(|s| (*s).to_string()).collect(),
            motivo: "porque si".into(),
            confianza: 0.5,
        }
    }

    fn resumen(nombre: &str, triggers: &[&str], relacionadas: &[&str]) -> Resumen {
        Resumen {
            nombre: nombre.to_string(),
            descripcion: format!("Leccion {nombre}."),
            triggers: triggers.iter().map(|s| (*s).to_string()).collect(),
            relacionadas: relacionadas.iter().map(|s| (*s).to_string()).collect(),
        }
    }

    #[test]
    fn relacionadas_mutuas_should_produce_a_local_candidate_without_shared_triggers() {
        // AC-1/AC-5: no depende del modelo ni de coincidir por keywords.
        let senales = por_relacionadas(&[
            resumen("a", &["solo-a"], &["b"]),
            resumen("b", &["solo-b"], &["a"]),
        ]);
        assert_eq!(senales.candidatos.len(), 1, "{senales:?}");
        let candidato = &senales.candidatos[0];
        assert_eq!(candidato.miembros, ["a", "b"]);
        assert!(candidato.motivo.contains("relacionadas mutuas"));
        assert!(candidato.motivo.contains("'a' declara 'b'"));
        assert!(senales.diagnosticos.is_empty(), "{senales:?}");
    }

    #[test]
    fn relacionadas_unilaterales_or_unknown_should_not_invent_candidates() {
        // AC-2/AC-4: el catálogo elegible es la frontera; ni una referencia
        // rota ni una unilateral se convierten en acción implícita.
        let senales = por_relacionadas(&[
            resumen("a", &["x"], &["b", "archivada"]),
            resumen("b", &["y"], &[]),
        ]);
        assert!(senales.candidatos.is_empty(), "{senales:?}");
        assert!(
            senales
                .diagnosticos
                .iter()
                .any(|d| d.contains("no es mutua")),
            "{senales:?}"
        );
        assert!(
            senales
                .diagnosticos
                .iter()
                .any(|d| d.contains("no es una leccion activa y valida")),
            "{senales:?}"
        );
    }

    #[test]
    fn related_and_trigger_candidates_should_merge_and_keep_both_reasons() {
        // AC-3: A-B y B-A son el mismo par, incluso si el LLM lo propuso con
        // el orden opuesto a la señal local.
        let relacionadas = por_relacionadas(&[
            resumen("a", &["comun"], &["b"]),
            resumen("b", &["comun"], &["a"]),
        ]);
        let mut candidatos = relacionadas.candidatos;
        candidatos.extend(marcar_triggers(vec![Candidato {
            miembros: vec!["b".into(), "a".into()],
            motivo: "comparten comun".into(),
            confianza: 0.8,
        }]));
        let unidos = unir_candidatos(candidatos);
        assert_eq!(unidos.len(), 1, "{unidos:?}");
        assert_eq!(unidos[0].miembros, ["a", "b"]);
        assert!(unidos[0].motivo.contains("triggers/LLM"), "{unidos:?}");
        assert!(
            unidos[0].motivo.contains("relacionadas mutuas"),
            "{unidos:?}"
        );
    }

    #[test]
    fn preparar_paraguas_should_union_triggers_and_satisfy_structural_review() {
        // AC-1/AC-2/AC-3/AC-5: el orden y la capitalización de la selección no
        // cambian el borrador; los únicos huecos que quedan son humanos.
        let miembros = vec![
            (
                "b".to_string(),
                vec!["Beta".to_string(), "alfa".to_string()],
            ),
            (
                "a".to_string(),
                vec!["ALFA".to_string(), "zeta".to_string()],
            ),
        ];
        let borrador = preparar_paraguas(&miembros);
        assert_eq!(borrador.miembros, ["a", "b"]);
        assert_eq!(borrador.triggers, ["alfa", "beta", "zeta"]);
        assert_eq!(borrador.cuerpo.matches("[[a]]").count(), 1);
        assert_eq!(borrador.cuerpo.matches("[[b]]").count(), 1);
        assert!(
            revisar_paraguas(&borrador.cuerpo, &borrador.triggers, &miembros).is_empty(),
            "el borrador no cumple la estructura"
        );
    }

    #[test]
    fn consolidar_should_drop_hallucinated_members() {
        let existentes = vec!["a".to_string(), "b".to_string()];
        let (ok, fuera) = validar(vec![cand(&["a", "no-existe"])], &existentes, &[]);
        assert!(ok.is_empty());
        assert_eq!(fuera.len(), 1);
        assert!(
            fuera[0].mensaje().contains("no existe"),
            "{}",
            fuera[0].mensaje()
        );
        assert!(fuera[0].mensaje().contains("no-existe"));
    }

    #[test]
    fn consolidar_should_respect_the_pin() {
        let existentes = vec!["a".to_string(), "b".to_string()];
        let pin = vec!["b".to_string()];
        let (ok, fuera) = validar(vec![cand(&["a", "b"])], &existentes, &pin);
        assert!(ok.is_empty());
        assert!(
            fuera[0].mensaje().contains("pinneada"),
            "{}",
            fuera[0].mensaje()
        );
    }

    #[test]
    fn validar_should_drop_groups_smaller_than_two() {
        let existentes = vec!["a".to_string()];
        let (ok, fuera) = validar(vec![cand(&["a"])], &existentes, &[]);
        assert!(ok.is_empty());
        assert!(fuera[0].mensaje().contains("menos de dos"));
    }

    #[test]
    fn validar_should_keep_a_legitimate_candidate() {
        let existentes = vec!["a".to_string(), "b".to_string()];
        let (ok, fuera) = validar(vec![cand(&["a", "b"])], &existentes, &[]);
        assert_eq!(ok.len(), 1);
        assert!(fuera.is_empty());
    }

    fn miembro(n: &str, t: &[&str]) -> (String, Vec<String>) {
        (n.to_string(), t.iter().map(|s| (*s).to_string()).collect())
    }

    #[test]
    fn consolidar_should_refuse_a_skeleton_umbrella() {
        // Archivar contra un esqueleto perderia el conocimiento de forma
        // estructural: quedaria solo en archivo/, que `buscar` puntua 30.
        let esqueleto = "---\nnombre: p\ndescripcion: <una sola oracion, max 80 caracteres.>\n---\n\n## Cuando aplica\n\n<cuando aplica esta clase de tarea>\n";
        let faltas = revisar_paraguas(esqueleto, &["x".into()], &[miembro("a", &["x"])]);
        assert!(faltas.contains(&Falta::Esqueleto), "{faltas:?}");
        assert!(faltas[0].mensaje().contains("placeholders"));
    }

    #[test]
    fn consolidar_should_demand_the_union_of_triggers() {
        // Sin heredar los triggers, el conocimiento deja de encontrarse.
        let texto = "cuerpo escrito de verdad, con [[a]] citada.";
        let faltas = revisar_paraguas(
            texto,
            &["comun".into()],
            &[miembro("a", &["comun", "propio"])],
        );
        let t: Vec<&Falta> = faltas
            .iter()
            .filter(|f| matches!(f, Falta::Trigger { .. }))
            .collect();
        assert_eq!(t.len(), 1, "{faltas:?}");
        assert!(t[0].mensaje().contains("'propio'"), "{}", t[0].mensaje());
        assert!(t[0].mensaje().contains("100"), "explica por que importa");
    }

    #[test]
    fn consolidar_should_demand_a_pointer_to_each_member() {
        let texto = "cuerpo escrito, sin citar a nadie.";
        let faltas = revisar_paraguas(texto, &["x".into()], &[miembro("a", &["x"])]);
        assert!(
            faltas.iter().any(|f| matches!(f, Falta::Puntero { .. })),
            "{faltas:?}"
        );
    }

    #[test]
    fn consolidar_should_accept_a_complete_umbrella() {
        // Y el caso feliz: paraguas escrito, con todos los triggers y los
        // punteros. Sin este test, los tres de arriba no prueban que se pueda
        // pasar.
        let texto = "cuerpo escrito de verdad. Ver [[a]] y [[b]].";
        let triggers: Vec<String> = ["x", "y", "z"].iter().map(|s| s.to_string()).collect();
        let faltas = revisar_paraguas(
            texto,
            &triggers,
            &[miembro("a", &["x", "y"]), miembro("b", &["z"])],
        );
        assert!(faltas.is_empty(), "{faltas:?}");
    }

    #[test]
    fn consolidar_override_should_win_over_detection() {
        // El override elige CUAL backend; nunca ENCIENDE la feature.
        let b = resolver_backend(&encendida(), Some("mi-cli --flag"), |_| true);
        assert_eq!(
            b,
            Backend::Override(vec!["mi-cli".into(), "--flag".into()]),
            "el override tiene que ganarle a la deteccion"
        );
        // Sin la regla, el override NO alcanza para encenderla.
        assert_eq!(
            resolver_backend(&json!({}), Some("mi-cli"), |_| true),
            Backend::Apagada,
            "el override elige cual backend, nunca enciende la feature"
        );
    }

    #[test]
    fn consolidar_should_not_pass_the_prompt_through_a_shell() {
        // Contrato de comportamiento: un prompt con metacaracteres de shell
        // llega LITERAL al proceso, porque va como item de argv.
        let dir = tempfile::tempdir().unwrap();
        let veneno = "$(echo EJECUTADO) `echo OTRO` && echo PEOR";
        let salida = preguntar(
            &["/bin/echo".to_string()],
            veneno,
            dir.path(),
            Duration::from_secs(10),
        )
        .unwrap();
        // La assertion correcta es la IGUALDAD, no la ausencia de palabras: el
        // texto literal del veneno CONTIENE la palabra "EJECUTADO", asi que
        // `!salida.contains("EJECUTADO")` era tautologicamente falso y el test
        // fallaba con el codigo bien. Si pasara por un shell, la salida seria
        // "EJECUTADO OTRO" y no el literal.
        assert_eq!(salida.trim(), veneno, "el prompt no llego literal");
    }

    #[test]
    fn preguntar_should_time_out_a_hung_backend() {
        // Un script propio, porque `sleep 30 <prompt>` en macOS falla al
        // instante ("invalid time interval") y el test media otra cosa.
        let dir = tempfile::tempdir().unwrap();
        let script = dir.path().join("colgado.sh");
        std::fs::write(&script, "#!/bin/sh\nsleep 30\n").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
        }
        let err = preguntar(
            &[script.to_string_lossy().into_owned()],
            "x",
            dir.path(),
            Duration::from_millis(200),
        )
        .unwrap_err();
        assert!(err.contains("no respondio"), "{err}");
    }

    #[test]
    fn preguntar_should_report_a_missing_backend() {
        let dir = tempfile::tempdir().unwrap();
        let err = preguntar(
            &["comando-que-no-existe-en-ningun-lado".to_string()],
            "x",
            dir.path(),
            Duration::from_secs(5),
        )
        .unwrap_err();
        assert!(err.contains("no se pudo ejecutar"), "{err}");
    }
}
