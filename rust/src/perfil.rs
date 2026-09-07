//! Perfil de usuario: el tercer almacen de memoria del arnes (feature #19).
//!
//! El hub guarda **eventos**, `docs/lecciones/` guarda **procedimiento** y este
//! archivo guarda **preferencias** — como quiere trabajar el usuario. Es el unico
//! de los tres que viaja hasta la superficie que cada backend lee al arrancar.
//!
//! Cuatro invariantes que el codigo tiene que sostener:
//!
//! - **Es del USUARIO**: ninguna escritura ocurre sin su `--yes` explicito.
//! - **Limite duro**: al pasarse se FALLA con las entradas a la vista; nunca se
//!   recorta ni se descarta nada en silencio.
//! - **Se versiona Y se inyecta**: por eso el escaneo de secretos bloquea antes
//!   de escribir (un secreto aca queda en git para siempre y ademas viaja al
//!   prompt de cada agente).
//! - **Cero dependencia del hub**: es un archivo.

use std::path::PathBuf;

use serde_json::Value;

use crate::exit::Exit;
use crate::paths::HarnessPaths;
use crate::pycompat::py_str;

/// Nombre del documento dentro del `docs/` de la RAIZ.
pub const FILE_NAME: &str = "perfil-usuario.md";
/// Tope de las entradas, en caracteres (decision del usuario 2026-08-16, OBS-2 de
/// la #19; el `USER.md` de Hermes usa 1375). No es tacaneria: cada caracter se
/// paga en TODAS las sesiones de TODOS los backends, para siempre.
pub const LIMITE: usize = 1500;

/// Marcadores del bloque que el instalador inyecta en las superficies. Viven aca
/// para que el binario y los dos instaladores no puedan divergir.
pub const MARCA_INICIO: &str = "<!-- harness:perfil:inicio -->";
pub const MARCA_FIN: &str = "<!-- harness:perfil:fin -->";

pub fn file_for(paths: &HarnessPaths) -> PathBuf {
    paths.plans.join(FILE_NAME)
}

pub fn rel_path() -> String {
    format!("docs/{FILE_NAME}")
}

// ---------------------------------------------------------------------------
// Documento
// ---------------------------------------------------------------------------

/// El perfil: encabezado verbatim + entradas. Se preserva todo lo que no sea una
/// entrada, asi que el usuario puede escribir prosa alrededor sin perderla.
#[derive(Debug, Clone, Default)]
pub struct Perfil {
    /// Lineas del documento, tal cual estan en disco.
    lineas: Vec<String>,
}

/// Resultado de buscar una entrada por subcadena.
///
/// Es un enum y no un `Option` a proposito: el caso "matchea mas de una" es un
/// estado real del dominio con su propio mensaje, y modelarlo como `None` lo
/// perderia. (Patron "model states as enums" — skill `rust-patterns`.)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Coincidencia {
    Ninguna,
    Unica(usize),
    Ambigua(Vec<String>),
}

impl Perfil {
    pub fn parse(texto: &str) -> Perfil {
        Perfil {
            lineas: texto.lines().map(str::to_string).collect(),
        }
    }

    /// Carga el perfil. Si el archivo no esta (instalacion anterior a esta
    /// feature), arranca desde la plantilla: asi un `perfil add` no produce un
    /// documento sin encabezado.
    pub fn load(paths: &HarnessPaths) -> Perfil {
        match std::fs::read_to_string(file_for(paths)) {
            Ok(t) => Perfil::parse(&t),
            Err(_) => Perfil::parse(&plantilla()),
        }
    }

    /// El bloque que el instalador inyecta en cada superficie. `None` cuando no
    /// hay entradas: sin perfil, las superficies quedan como siempre (AC-12).
    pub fn bloque(&self) -> Option<String> {
        if self.is_empty() {
            return None;
        }
        let entradas = self.entradas();
        let mut out = format!(
            "{MARCA_INICIO}\n\n## Perfil de usuario\n\n\
             Como quiere trabajar el usuario de este repositorio. Respetalo en el plan,\n\
             en la implementacion y en el veredicto. Fuente: `{}`.\n\n",
            rel_path()
        );
        for e in entradas {
            out.push_str(&format!("- {e}\n"));
        }
        out.push_str(&format!("\n{MARCA_FIN}\n"));
        Some(out)
    }

    /// Indices de las lineas que son entradas (`- <texto>`).
    fn indices(&self) -> Vec<usize> {
        self.lineas
            .iter()
            .enumerate()
            .filter(|(_, l)| es_entrada(l))
            .map(|(i, _)| i)
            .collect()
    }

    /// Las entradas, sin el `- ` inicial.
    pub fn entradas(&self) -> Vec<String> {
        self.indices()
            .into_iter()
            .filter_map(|i| texto_de(&self.lineas[i]))
            .collect()
    }

    /// Caracteres usados: SOLO las entradas, nunca el encabezado (AC-2).
    pub fn usados(&self) -> usize {
        self.entradas().iter().map(|e| e.chars().count()).sum()
    }

    pub fn porcentaje(&self) -> usize {
        self.usados() * 100 / LIMITE.max(1)
    }

    pub fn is_empty(&self) -> bool {
        self.entradas().is_empty()
    }

    pub fn render(&self) -> String {
        let mut out = self.lineas.join("\n");
        out.push('\n');
        out
    }

    pub fn save(&self, paths: &HarnessPaths) -> anyhow::Result<()> {
        let file = file_for(paths);
        if let Some(dir) = file.parent() {
            std::fs::create_dir_all(dir)?;
        }
        crate::features::write_text_atomic(&file, &self.render())
    }

    pub fn buscar(&self, fragmento: &str) -> Coincidencia {
        let fragmento = fragmento.trim();
        let matches: Vec<usize> = self
            .indices()
            .into_iter()
            .filter(|&i| {
                texto_de(&self.lineas[i]).is_some_and(|t| t.contains(fragmento))
            })
            .collect();
        match matches.as_slice() {
            [] => Coincidencia::Ninguna,
            [uno] => Coincidencia::Unica(*uno),
            varios => Coincidencia::Ambigua(
                varios
                    .iter()
                    .filter_map(|&i| texto_de(&self.lineas[i]))
                    .collect(),
            ),
        }
    }

    /// Inserta una entrada al final de las existentes (o al final del documento
    /// si todavia no hay ninguna).
    pub fn insertar(&mut self, texto: &str) {
        let entrada = format!("- {texto}");
        match self.indices().last() {
            Some(&ultima) => self.lineas.insert(ultima + 1, entrada),
            None => {
                while self.lineas.last().is_some_and(|l| l.trim().is_empty()) {
                    self.lineas.pop();
                }
                self.lineas.push(String::new());
                self.lineas.push(entrada);
            }
        }
    }

    pub fn reemplazar(&mut self, idx: usize, texto: &str) {
        self.lineas[idx] = format!("- {texto}");
    }

    pub fn quitar(&mut self, idx: usize) -> String {
        let quitada = texto_de(&self.lineas[idx]).unwrap_or_default();
        self.lineas.remove(idx);
        quitada
    }

    /// Cuanto ocuparia el perfil si la entrada de `idx` pasara a ser `texto`
    /// (con `idx = None`, si se agregara una entrada nueva).
    pub fn usados_con(&self, idx: Option<usize>, texto: &str) -> usize {
        let nuevo = texto.chars().count();
        match idx {
            None => self.usados() + nuevo,
            Some(i) => {
                let viejo = texto_de(&self.lineas[i])
                    .map(|t| t.chars().count())
                    .unwrap_or(0);
                self.usados() - viejo + nuevo
            }
        }
    }

    /// Error de limite: dice cuanto ocupa, cuanto ocuparia, LISTA las entradas
    /// actuales y pide consolidar en el mismo turno. Nunca recorta (AC-3).
    pub fn error_de_limite(&self, quedaria: usize) -> Exit {
        let mut msg = format!(
            "El perfil quedaria en {quedaria}/{LIMITE} caracteres y el limite es duro: no se recorta nada.\n    \
             Hoy ocupa {}/{LIMITE} ({}%). Consolida AHORA, en este mismo turno, y reintenta:\n      \
             perfil replace --old \"<fragmento>\" --texto \"<version mas corta>\" --yes\n      \
             perfil remove  --old \"<fragmento>\" --yes\n    \
             Entradas actuales:",
            self.usados(),
            self.porcentaje()
        );
        for (n, e) in self.entradas().iter().enumerate() {
            msg.push_str(&format!("\n      {}. {e}", n + 1));
        }
        Exit {
            code: 2,
            message: Some(msg),
        }
    }
}

fn es_entrada(linea: &str) -> bool {
    let t = linea.trim_start();
    (t.starts_with("- ") || t.starts_with("* ")) && t.len() > 2
}

fn texto_de(linea: &str) -> Option<String> {
    let t = linea.trim_start();
    t.strip_prefix("- ")
        .or_else(|| t.strip_prefix("* "))
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

// ---------------------------------------------------------------------------
// Escaneo de seguridad (AC-10)
// ---------------------------------------------------------------------------

/// El perfil se versiona (queda en el historial de git para siempre) Y se
/// inyecta en el prompt de cada agente. Un secreto aca es irreversible: rotarlo
/// es la unica salida. Por eso esto BLOQUEA y no avisa (decision del usuario
/// 2026-08-16, OBS-4 de la #19).
///
/// Devuelve el nombre del patron que disparo, para que el mensaje pueda decir
/// QUE reescribir en vez de un "rechazado" opaco.
pub fn motivo_inseguro(texto: &str) -> Option<String> {
    static CREDENCIAL: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
        #[expect(
            clippy::unwrap_used,
            reason = "patron constante validado por los tests de este modulo"
        )]
        regex::Regex::new(
            r"(?i)(password|passwd|secret|api[_\-]?key|access[_\-]?token|bearer|authorization)\s*[:=]",
        )
        .unwrap()
    });
    static CLAVE_PRIVADA: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
        #[expect(
            clippy::unwrap_used,
            reason = "patron constante validado por los tests de este modulo"
        )]
        regex::Regex::new(r"-----BEGIN [A-Z ]*PRIVATE KEY-----").unwrap()
    });
    static TOKEN_LARGO: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
        #[expect(
            clippy::unwrap_used,
            reason = "patron constante validado por los tests de este modulo"
        )]
        // Prefijos de token conocidos (GitHub, Slack, OpenAI/Anthropic, AWS).
        regex::Regex::new(r"(gh[pousr]_[A-Za-z0-9]{16,}|xox[baprs]-[A-Za-z0-9-]{10,}|sk-[A-Za-z0-9_\-]{16,}|AKIA[0-9A-Z]{16})").unwrap()
    });

    if CLAVE_PRIVADA.is_match(texto) {
        return Some("una clave privada (bloque BEGIN ... PRIVATE KEY)".to_string());
    }
    if let Some(m) = TOKEN_LARGO.find(texto) {
        let prefijo: String = m.as_str().chars().take(4).collect();
        return Some(format!("un token con prefijo conocido ('{prefijo}...')"));
    }
    if let Some(m) = CREDENCIAL.captures(texto).and_then(|c| c.get(1)) {
        return Some(format!(
            "algo que parece una credencial ('{}' seguido de : o =)",
            m.as_str()
        ));
    }
    if let Some(c) = invisible(texto) {
        return Some(format!(
            "un caracter Unicode invisible (U+{:04X}), que puede esconder instrucciones",
            c as u32
        ));
    }
    None
}

/// Zero-width y controles bidi: invisibles al leer, pero llegan al prompt.
fn invisible(texto: &str) -> Option<char> {
    texto.chars().find(|c| {
        matches!(c,
            '\u{200B}'..='\u{200F}'   // zero-width space/joiner + marcas LTR/RTL
            | '\u{202A}'..='\u{202E}' // embeddings y overrides bidi
            | '\u{2060}'..='\u{2064}' // word joiner e invisibles matematicos
            | '\u{FEFF}'              // BOM en medio del texto
        )
    })
}

// ---------------------------------------------------------------------------
// Evidencia para `perfil sugerir` (AC-14, AC-15)
// ---------------------------------------------------------------------------

/// Un registro de decision ya escrito en el repo. El arnes los junta; **no**
/// destila la entrada: eso lo propone el agente y lo decide el usuario (NO1 del
/// PRD de aprendizaje).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Registro {
    /// Id de feature (`14`), o vacio si no se pudo determinar.
    pub feature: String,
    pub fecha: String,
    /// Cuando se registro, timestamp completo (`2026-09-05T00:00:00Z`): el de
    /// la linea de bitacora, o el `started_at` de la feature para un plan o un
    /// spec (que no tienen fecha propia). Vacio si no se pudo fechar, y sin
    /// fecha se cuenta como nueva (feature #82).
    pub momento: String,
    /// De donde salio: `history`, `plan` o `spec`.
    pub fuente: &'static str,
    pub texto: String,
    /// Ya citado por alguna entrada del perfil (por su `#<id>`).
    pub ya_incorporado: bool,
}

/// Palabras que marcan una decision del usuario. Deliberadamente pocas y
/// especificas: `sugerir` tiene que traer senal, no volcar `history.md` entero.
const SENALES: [&str; 6] = ["decision", "decidio", "eligio", "decidido", "obs-", "aprobo"];

/// Anti-senales: frases que MENCIONAN decisiones para decir que **no** hay
/// ninguna. Sin esto, cada plan aporta su "(ninguna observacion pendiente sin
/// decision)" como si fuera evidencia. Encontrado corriendo `sugerir` sobre este
/// mismo repo.
const ANTI_SENALES: [&str; 6] = [
    "ninguna observacion",
    "sin decisiones",
    "sin observaciones",
    "no queda ninguna",
    "ninguna abierta",
    "ninguna pendiente",
];

fn tiene_senal(texto: &str) -> bool {
    let bajo = texto.to_lowercase();
    if ANTI_SENALES.iter().any(|a| bajo.contains(a)) {
        return false;
    }
    SENALES.iter().any(|s| bajo.contains(s))
}

/// `#<id>` de una linea de bitacora (`... feature #14 ...`).
fn feature_de(linea: &str) -> String {
    linea
        .split_once("feature #")
        .map(|(_, resto)| {
            resto
                .chars()
                .take_while(char::is_ascii_digit)
                .collect::<String>()
        })
        .unwrap_or_default()
}

/// `<id>` de un nombre de archivo `plan-feature-14-...md` / `spec-feature-14-...`.
fn feature_de_archivo(nombre: &str) -> String {
    nombre
        .split_once("-feature-")
        .map(|(_, resto)| {
            resto
                .chars()
                .take_while(char::is_ascii_digit)
                .collect::<String>()
        })
        .unwrap_or_default()
}

fn recorta(texto: &str, max: usize) -> String {
    let limpio = texto.trim();
    if limpio.chars().count() <= max {
        return limpio.to_string();
    }
    let corto: String = limpio.chars().take(max).collect();
    format!("{corto}...")
}

/// Junta los registros de decision de `progress/history.md`, de los planes y de
/// las `## Observaciones` de los specs (OBS-5). No escribe nada.
pub fn recolectar(paths: &HarnessPaths) -> Vec<Registro> {
    recolectar_con(paths, &cargar_backlog(paths))
}

/// El backlog para fechar planes y specs; un backlog ilegible es un backlog
/// vacio (los registros quedan sin momento y se cuentan), nunca un error.
fn cargar_backlog(paths: &HarnessPaths) -> Value {
    crate::features::load_features(paths).unwrap_or_else(|_| serde_json::json!({}))
}

fn recolectar_con(paths: &HarnessPaths, backlog: &Value) -> Vec<Registro> {
    let perfil = Perfil::load(paths);
    let citadas: Vec<String> = perfil.entradas();
    let ya = |fid: &str| -> bool {
        !fid.is_empty() && citadas.iter().any(|e| e.contains(&format!("#{fid}")))
    };
    let mut out = Vec::new();

    // (a) La bitacora: una linea por transicion, con su fecha.
    if let Ok(texto) = std::fs::read_to_string(&paths.history) {
        for linea in texto.lines().filter(|l| tiene_senal(l)) {
            let momento = linea
                .split_whitespace()
                .find(|t| es_timestamp(t))
                .unwrap_or_default()
                .to_string();
            let fecha = momento.chars().take(10).collect::<String>();
            let fid = feature_de(linea);
            out.push(Registro {
                ya_incorporado: ya(&fid),
                feature: fid,
                fecha,
                momento,
                fuente: "history",
                texto: recorta(linea.trim_start_matches("- "), 240),
            });
        }
    }

    // (b) Planes y specs: las lineas con senal de su seccion de observaciones.
    let Ok(entries) = std::fs::read_dir(&paths.plans) else {
        return out;
    };
    let mut archivos: Vec<_> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .map(|n| n.to_string_lossy())
                .is_some_and(|n| {
                    (n.starts_with("plan-feature-") || n.starts_with("spec-feature-"))
                        && n.ends_with(".md")
                })
        })
        .collect();
    archivos.sort();
    for path in archivos {
        let nombre = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let fuente = if nombre.starts_with("spec-") { "spec" } else { "plan" };
        let Ok(texto) = std::fs::read_to_string(&path) else {
            continue;
        };
        let fid = feature_de_archivo(&nombre);
        // Un plan o un spec no tienen fecha propia: se fechan por el inicio de
        // su feature, la cota inferior (OBS-2 de la #82).
        let momento = inicio_de_feature(backlog, &fid);
        for linea in texto.lines() {
            let l = linea.trim();
            // Solo items de lista con senal: el cuerpo en prosa no es un registro.
            if !l.starts_with("- ") || !tiene_senal(l) {
                continue;
            }
            out.push(Registro {
                feature: fid.clone(),
                fecha: String::new(),
                momento: momento.clone(),
                fuente,
                texto: recorta(l.trim_start_matches("- "), 240),
                ya_incorporado: ya(&fid),
            });
        }
    }
    out
}

/// Un token con la forma de `now_stamp` (`2026-09-05T00:00:00Z`) o al menos su
/// fecha. Todos los timestamps del arnes salen del mismo formato, asi que se
/// comparan como texto.
fn es_timestamp(token: &str) -> bool {
    token.len() >= 10 && token.starts_with("20") && token.as_bytes().get(4) == Some(&b'-')
}

/// `started_at` de la feature `fid` en el backlog; vacio si no esta o no arranco.
fn inicio_de_feature(backlog: &Value, fid: &str) -> String {
    if fid.is_empty() {
        return String::new();
    }
    crate::features::features_slice(backlog)
        .iter()
        .find(|f| py_str(f.get("id")) == fid)
        .and_then(|f| f.get("started_at"))
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

// ---------------------------------------------------------------------------
// El corte: la ultima entrada del perfil (feature #82)
// ---------------------------------------------------------------------------

/// Desde cuando cuenta el aviso de perfil. Tres estados porque son tres
/// conductas: sin corte se cuenta todo (como antes de la #82), y con corte el
/// texto dice de donde salio para que el usuario pueda discutir el numero.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Corte {
    /// El perfil no tiene entradas fechables: ni linea `perfil add|replace` en
    /// la bitacora ni cita a una feature iniciada.
    Ninguno,
    /// La ultima linea `- <ts> perfil add|replace ...` de `history.md`.
    Bitacora { momento: String },
    /// El `started_at` de la feature mas alta que cita el perfil (`#<id>`): lo
    /// que queda cuando la bitacora se perdio o el perfil se edito a mano.
    Backlog { feature: String, momento: String },
}

impl Corte {
    pub fn momento(&self) -> Option<&str> {
        match self {
            Corte::Ninguno => None,
            Corte::Bitacora { momento } | Corte::Backlog { momento, .. } => Some(momento),
        }
    }

    /// `bitacora` o `backlog`; `None` sin corte. Es lo que sale en el JSON.
    pub fn origen(&self) -> Option<&'static str> {
        match self {
            Corte::Ninguno => None,
            Corte::Bitacora { .. } => Some("bitacora"),
            Corte::Backlog { .. } => Some("backlog"),
        }
    }

    /// Para el texto: `2026-09-05, bitacora` / `2026-09-06, backlog: inicio de
    /// la #80` / `sin corte`.
    pub fn describir(&self) -> String {
        let fecha = |m: &str| m.chars().take(10).collect::<String>();
        match self {
            Corte::Ninguno => "sin corte".to_string(),
            Corte::Bitacora { momento } => format!("{}, bitacora", fecha(momento)),
            Corte::Backlog { feature, momento } => {
                format!("{}, backlog: inicio de la #{feature}", fecha(momento))
            }
        }
    }
}

/// Las dos cuentas del aviso: las nuevas desde el corte y el total historico.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pendientes {
    pub nuevas: usize,
    pub total: usize,
    pub corte: Corte,
}

impl Pendientes {
    /// El numero que se compara con `rules.perfil_pendientes_max`: las nuevas,
    /// o el total cuando no hay corte (nunca menos que antes de la #82).
    pub fn para_el_umbral(&self) -> usize {
        match self.corte {
            Corte::Ninguno => self.total,
            Corte::Bitacora { .. } | Corte::Backlog { .. } => self.nuevas,
        }
    }
}

/// Ids de feature que citan las entradas (`(#14, #16)`), de mayor a menor.
fn features_citadas(perfil: &Perfil) -> Vec<u64> {
    let mut ids: Vec<u64> = perfil
        .entradas()
        .iter()
        .flat_map(|e| {
            e.split('#')
                .skip(1)
                .filter_map(|resto| {
                    resto
                        .chars()
                        .take_while(char::is_ascii_digit)
                        .collect::<String>()
                        .parse::<u64>()
                        .ok()
                })
                .collect::<Vec<_>>()
        })
        .collect();
    ids.sort_unstable_by(|a, b| b.cmp(a));
    ids.dedup();
    ids
}

/// El corte, sin filesystem: `history` es el texto de la bitacora y `backlog`
/// el JSON. Con las dos senales gana la mas reciente (OBS-1 de la #82);
/// `perfil remove` no es una entrada (OBS-3).
pub fn ultima_entrada(perfil: &Perfil, history: &str, backlog: &Value) -> Corte {
    let de_bitacora = history
        .lines()
        .filter_map(|l| {
            let (ts, mensaje) = l.strip_prefix("- ")?.split_once(' ')?;
            let escribe =
                mensaje.starts_with("perfil add ") || mensaje.starts_with("perfil replace ");
            (escribe && es_timestamp(ts)).then(|| ts.to_string())
        })
        .next_back();
    let de_backlog = features_citadas(perfil).into_iter().find_map(|id| {
        let fid = id.to_string();
        let momento = inicio_de_feature(backlog, &fid);
        (!momento.is_empty()).then_some((fid, momento))
    });
    match (de_bitacora, de_backlog) {
        (None, None) => Corte::Ninguno,
        (Some(momento), None) => Corte::Bitacora { momento },
        (None, Some((feature, momento))) => Corte::Backlog { feature, momento },
        (Some(bitacora), Some((feature, momento))) => {
            if momento > bitacora {
                Corte::Backlog { feature, momento }
            } else {
                Corte::Bitacora { momento: bitacora }
            }
        }
    }
}

/// Las dos cuentas, sin filesystem. Un registro sin momento se cuenta como
/// nuevo: ante la duda el aviso sale de mas, nunca de menos.
pub fn contar(registros: &[Registro], corte: Corte) -> Pendientes {
    let sin_incorporar = || registros.iter().filter(|r| !r.ya_incorporado);
    let total = sin_incorporar().count();
    let nuevas = match corte.momento() {
        None => total,
        Some(desde) => sin_incorporar()
            .filter(|r| r.momento.is_empty() || r.momento.as_str() > desde)
            .count(),
    };
    Pendientes {
        nuevas,
        total,
        corte,
    }
}

/// Las cuentas sobre registros ya juntados (para `sugerir`, que los lista).
pub fn pendientes_de(paths: &HarnessPaths, registros: &[Registro]) -> Pendientes {
    let history = std::fs::read_to_string(&paths.history).unwrap_or_default();
    let corte = ultima_entrada(&Perfil::load(paths), &history, &cargar_backlog(paths));
    contar(registros, corte)
}

/// Lo que `close` y `lecciones status` preguntan: junta y cuenta.
pub fn pendientes(paths: &HarnessPaths) -> Pendientes {
    let backlog = cargar_backlog(paths);
    let history = std::fs::read_to_string(&paths.history).unwrap_or_default();
    let corte = ultima_entrada(&Perfil::load(paths), &history, &backlog);
    contar(&recolectar_con(paths, &backlog), corte)
}

/// El contrato que cierra `perfil sugerir`: como se destila una entrada durable.
/// El arnes lo emite; el agente propone; el usuario decide.
pub fn contrato_de_sugerencia() -> String {
    format!(
        "\nCOMO DESTILAR UNA ENTRADA (el arnes no lo hace por vos):\n\
         \n\
         - Una entrada dice COMO trabajar, en presente y en general.\n\
         \x20   Bien: \"Ante un fork de consistencia, elige la opcion segura aunque cueste mas.\"\n\
         \x20   Mal:  \"En la #14 eligio escribir solo el delta.\" (es un hecho, no una preferencia)\n\
         - Una preferencia entra cuando se REPITE. Un caso aislado no es un patron.\n\
         - Cita las features de origen al final: '(#14, #16)'. Cuestan 5 caracteres y\n\
         \x20   hacen que 'perfil sugerir' no vuelva a proponerte lo mismo.\n\
         - NO pongas: hechos de una feature puntual (eso es docs/lecciones/), datos\n\
         \x20   personales, ni jamas un secreto: este archivo se versiona.\n\
         - El limite es {LIMITE} caracteres y es duro. Cada caracter se paga en TODAS\n\
         \x20   las sesiones de TODOS los backends: si no cambia una decision, no va.\n\
         \n\
         Despues: MOSTRALE la entrada al usuario, PREGUNTALE, y solo con su si:\n\
         \x20   sh harness_cli perfil add --texto \"<entrada>\" --yes\n"
    )
}

// ---------------------------------------------------------------------------
// Plantilla
// ---------------------------------------------------------------------------

/// Documento inicial: encabezado y **ninguna entrada** (AC-1).
pub fn plantilla() -> String {
    format!(
        "# Perfil de usuario\n\
         \n\
         Como quiere trabajar el usuario de este repositorio. Lo escribe el arnes\n\
         **solo con su si explicito** (`harness_cli perfil add --yes`) y el instalador\n\
         lo inyecta en las superficies que lee cada agente al arrancar.\n\
         \n\
         Que va aca: preferencias durables sobre COMO trabajar (que elegir ante un\n\
         fork, que exigir antes de cerrar, que estilo de trabajo espera).\n\
         Que NO va: hechos de una feature puntual (eso es `docs/lecciones/`),\n\
         datos personales, y jamas un secreto — este archivo se versiona.\n\
         \n\
         Limite duro: {LIMITE} caracteres contando solo las entradas. Al pasarse, el\n\
         comando falla y hay que consolidar: nunca se recorta nada en silencio.\n\
         \n\
         Entradas (una por linea, empezando con `- `):\n"
    )
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;

    /// Perfil de prueba con las entradas dadas, sobre el encabezado real.
    fn perfil_con(entradas: &[&str]) -> Perfil {
        let mut p = Perfil::parse(&plantilla());
        for e in entradas {
            p.insertar(e);
        }
        p
    }

    #[test]
    fn usados_should_count_only_entries() {
        let vacio = Perfil::parse(&plantilla());
        assert_eq!(vacio.usados(), 0, "el encabezado no cuenta");
        let uno = perfil_con(&["abc"]);
        assert_eq!(uno.usados(), 3);
    }

    #[test]
    fn entradas_should_survive_a_round_trip() {
        let p = perfil_con(&["primera", "segunda"]);
        let releido = Perfil::parse(&p.render());
        assert_eq!(releido.entradas(), ["primera", "segunda"]);
    }

    #[test]
    fn render_should_preserve_the_user_header() {
        let p = perfil_con(&["una"]);
        assert!(p.render().starts_with("# Perfil de usuario"));
    }

    #[test]
    fn buscar_should_report_no_match() {
        let p = perfil_con(&["la opcion segura", "features amplias"]);
        assert_eq!(p.buscar("inexistente"), Coincidencia::Ninguna);
    }

    #[test]
    fn buscar_should_report_a_unique_match() {
        let p = perfil_con(&["la opcion segura", "features amplias"]);
        let Coincidencia::Unica(idx) = p.buscar("amplias") else {
            panic!("deberia matchear una sola");
        };
        assert_eq!(texto_de(&p.lineas[idx]).unwrap(), "features amplias");
    }

    #[test]
    fn buscar_should_report_every_candidate_when_ambiguous() {
        let p = perfil_con(&["la opcion segura", "la opcion rapida"]);
        let Coincidencia::Ambigua(candidatas) = p.buscar("la opcion") else {
            panic!("deberia ser ambigua");
        };
        assert_eq!(candidatas, ["la opcion segura", "la opcion rapida"]);
    }

    #[test]
    fn usados_con_should_account_for_a_replacement() {
        let p = perfil_con(&["abcde"]);
        let Coincidencia::Unica(idx) = p.buscar("abc") else {
            panic!("una sola");
        };
        // Reemplazar cuenta el nuevo, no la suma de los dos.
        assert_eq!(p.usados_con(Some(idx), "xy"), 2);
        // Agregar si suma.
        assert_eq!(p.usados_con(None, "xy"), 7);
    }

    #[test]
    fn error_de_limite_should_list_current_entries_and_ask_to_consolidate() {
        let p = perfil_con(&["una entrada", "otra entrada"]);
        let err = p.error_de_limite(LIMITE + 10);
        assert_eq!(err.code, 2);
        let msg = err.message.unwrap();
        assert!(msg.contains("no se recorta nada"), "{msg}");
        assert!(msg.contains("en este mismo turno"), "{msg}");
        assert!(msg.contains("1. una entrada"), "{msg}");
        assert!(msg.contains("2. otra entrada"), "{msg}");
    }

    #[test]
    fn quitar_should_remove_only_that_entry() {
        let mut p = perfil_con(&["una", "dos", "tres"]);
        let Coincidencia::Unica(idx) = p.buscar("dos") else {
            panic!("una sola");
        };
        assert_eq!(p.quitar(idx), "dos");
        assert_eq!(p.entradas(), ["una", "tres"]);
    }

    #[test]
    fn motivo_inseguro_should_accept_ordinary_preferences() {
        for bueno in [
            "Ante un fork de consistencia, elige la opcion segura aunque cueste mas.",
            "Prefiere features amplias y completas antes que incrementales.",
            "Exige sincronia total con sistemas externos, incluido el backfill.",
        ] {
            assert_eq!(motivo_inseguro(bueno), None, "rechazo una entrada valida: {bueno}");
        }
    }

    #[test]
    fn motivo_inseguro_should_reject_credentials() {
        for (malo, esperado) in [
            ("el password= hunter2 del staging", "credencial"),
            ("usar api_key: abc123 para el hub", "credencial"),
            ("-----BEGIN RSA PRIVATE KEY-----", "clave privada"),
            ("el token ghp_abcdefghijklmnop1234", "prefijo conocido"),
            ("la clave AKIAIOSFODNN7EXAMPLE de AWS", "prefijo conocido"),
        ] {
            let motivo = motivo_inseguro(malo)
                .unwrap_or_else(|| panic!("deberia rechazar: {malo}"));
            assert!(motivo.contains(esperado), "{malo} -> {motivo}");
        }
    }

    #[test]
    fn motivo_inseguro_should_reject_invisible_unicode() {
        let motivo = motivo_inseguro("texto\u{200B}oculto").unwrap();
        assert!(motivo.contains("invisible"), "{motivo}");
        assert!(motivo.contains("U+200B"), "{motivo}");
    }

    #[test]
    fn bloque_should_be_absent_without_entries() {
        // Sin entradas no hay bloque, y por eso las superficies quedan intactas.
        assert_eq!(Perfil::parse(&plantilla()).bloque(), None);
    }

    #[test]
    fn bloque_should_carry_the_entries_between_markers() {
        let p = perfil_con(&["la opcion segura", "features amplias"]);
        let b = p.bloque().unwrap();
        assert!(b.starts_with(MARCA_INICIO), "{b}");
        assert!(b.trim_end().ends_with(MARCA_FIN), "{b}");
        assert!(b.contains("- la opcion segura"), "{b}");
        assert!(b.contains("- features amplias"), "{b}");
        assert!(b.contains(&rel_path()), "el bloque cita su fuente: {b}");
    }

    #[test]
    fn recolectar_should_be_empty_without_material() {
        let dir = tempfile::tempdir().unwrap();
        let paths = HarnessPaths::from_root(dir.path().to_path_buf());
        assert!(recolectar(&paths).is_empty());
    }

    #[test]
    fn recolectar_should_pick_decisions_and_skip_noise() {
        let dir = tempfile::tempdir().unwrap();
        let paths = HarnessPaths::from_root(dir.path().to_path_buf());
        std::fs::create_dir_all(&paths.progress).unwrap();
        std::fs::write(
            &paths.history,
            "- 2026-08-14T03:43:37Z approve-spec feature #14 nota=Alan eligio la opcion segura\n\
             - 2026-08-14T04:10:09Z close feature #14 status=done note=hub por lotes\n",
        )
        .unwrap();
        let registros = recolectar(&paths);
        assert_eq!(registros.len(), 1, "solo la linea con senal de decision");
        assert_eq!(registros[0].feature, "14");
        assert_eq!(registros[0].fecha, "2026-08-14");
        assert_eq!(registros[0].fuente, "history");
        assert!(!registros[0].ya_incorporado);
    }

    #[test]
    fn recolectar_should_skip_lines_that_say_there_are_no_decisions() {
        // Hallazgo de correr `sugerir` sobre el repo real: cada plan aportaba su
        // "(ninguna observacion pendiente sin decision)" como si fuera evidencia.
        let dir = tempfile::tempdir().unwrap();
        let paths = HarnessPaths::from_root(dir.path().to_path_buf());
        std::fs::create_dir_all(&paths.plans).unwrap();
        std::fs::write(
            paths.plans.join("plan-feature-9-x.md"),
            "- (ninguna observacion pendiente sin decision)\n\
             - Sin decisiones pendientes abiertas: la implementacion avanzo sola.\n\
             - Ninguna abierta. Las cinco observaciones fueron decididas.\n\
             - Decision usuario: la opcion segura.\n",
        )
        .unwrap();
        let registros = recolectar(&paths);
        assert_eq!(registros.len(), 1, "solo la decision real: {registros:?}");
        assert!(registros[0].texto.contains("la opcion segura"));
    }

    #[test]
    fn recolectar_should_mark_what_the_profile_already_cites() {
        let dir = tempfile::tempdir().unwrap();
        let paths = HarnessPaths::from_root(dir.path().to_path_buf());
        std::fs::create_dir_all(&paths.progress).unwrap();
        std::fs::create_dir_all(&paths.plans).unwrap();
        std::fs::write(
            &paths.history,
            "- 2026-08-14T03:43:37Z approve-spec feature #14 nota=Alan eligio la opcion segura\n",
        )
        .unwrap();
        // Una entrada que cita la #14: ese registro ya no se propone.
        perfil_con(&["Ante un fork, elige la opcion segura. (#14)"])
            .save(&paths)
            .unwrap();
        let registros = recolectar(&paths);
        assert_eq!(registros.len(), 1);
        assert!(registros[0].ya_incorporado);
    }

    #[test]
    fn recolectar_should_read_plans_and_specs() {
        let dir = tempfile::tempdir().unwrap();
        let paths = HarnessPaths::from_root(dir.path().to_path_buf());
        std::fs::create_dir_all(&paths.plans).unwrap();
        std::fs::write(
            paths.plans.join("spec-feature-16-x.md"),
            "## Observaciones\n\n- OBS-1: algo — DECIDIDO: sincronia total.\n\nProsa que no es un item.\n",
        )
        .unwrap();
        std::fs::write(
            paths.plans.join("plan-feature-16-x.md"),
            "- Decision usuario: worker detached.\n- Item sin senal.\n",
        )
        .unwrap();
        let registros = recolectar(&paths);
        assert_eq!(registros.len(), 2);
        assert!(registros.iter().all(|r| r.feature == "16"));
        let fuentes: Vec<_> = registros.iter().map(|r| r.fuente).collect();
        assert!(fuentes.contains(&"spec"));
        assert!(fuentes.contains(&"plan"));
    }

    #[test]
    fn plantilla_should_parse_with_no_entries() {
        let p = Perfil::parse(&plantilla());
        assert!(p.is_empty());
        assert_eq!(p.porcentaje(), 0);
    }

    // -- feature #82: el corte y las dos cuentas ---------------------------

    fn backlog_con(features: &[(u64, &str)]) -> serde_json::Value {
        let lista: Vec<_> = features
            .iter()
            .map(|(id, inicio)| {
                let mut f = serde_json::json!({"id": id, "name": "x", "status": "in_progress"});
                if !inicio.is_empty() {
                    f["started_at"] = serde_json::json!(inicio);
                }
                f
            })
            .collect();
        serde_json::json!({ "features": lista })
    }

    fn registro(momento: &str, ya_incorporado: bool) -> Registro {
        Registro {
            feature: String::new(),
            fecha: momento.chars().take(10).collect(),
            momento: momento.to_string(),
            fuente: "history",
            texto: String::new(),
            ya_incorporado,
        }
    }

    #[test]
    fn momento_de_plan_should_be_the_started_at_of_its_feature() {
        // AC-2: un spec no tiene fecha propia; se fecha por su feature, y sin
        // feature iniciada queda sin momento (y se cuenta como nuevo).
        let dir = tempfile::tempdir().unwrap();
        let paths = HarnessPaths::from_root(dir.path().to_path_buf());
        std::fs::create_dir_all(&paths.plans).unwrap();
        for n in ["spec-feature-2-x.md", "spec-feature-3-y.md"] {
            std::fs::write(paths.plans.join(n), "- OBS-1: algo. DECIDIDO: si.\n").unwrap();
        }
        std::fs::write(
            &paths.features,
            backlog_con(&[(2, "2026-09-03T00:00:00Z"), (3, "")]).to_string(),
        )
        .unwrap();
        let registros = recolectar(&paths);
        assert_eq!(registros.len(), 2, "{registros:?}");
        let de2 = registros.iter().find(|r| r.feature == "2").unwrap();
        assert_eq!(de2.momento, "2026-09-03T00:00:00Z");
        let de3 = registros.iter().find(|r| r.feature == "3").unwrap();
        assert!(de3.momento.is_empty(), "{de3:?}");
        let cuentas = contar(
            &registros,
            Corte::Bitacora {
                momento: "2026-09-04T00:00:00Z".into(),
            },
        );
        assert_eq!((cuentas.nuevas, cuentas.total), (1, 2), "{cuentas:?}");
    }

    #[test]
    fn momento_de_plan_should_keep_the_full_timestamp_of_a_history_line() {
        let dir = tempfile::tempdir().unwrap();
        let paths = HarnessPaths::from_root(dir.path().to_path_buf());
        std::fs::create_dir_all(&paths.progress).unwrap();
        std::fs::write(
            &paths.history,
            "- 2026-08-14T03:43:37Z approve-spec feature #14 nota=Alan eligio la opcion segura\n",
        )
        .unwrap();
        let registros = recolectar(&paths);
        assert_eq!(registros[0].momento, "2026-08-14T03:43:37Z");
        assert_eq!(registros[0].fecha, "2026-08-14");
    }

    #[test]
    fn corte_desde_el_backlog_should_use_the_highest_cited_feature_without_a_bitacora_entry() {
        // AC-3: gana la feature mas ALTA, no la que arranco mas tarde.
        let perfil = perfil_con(&["Una. (#14, #16)", "Otra. (#15)"]);
        let backlog = backlog_con(&[
            (14, "2026-09-01T00:00:00Z"),
            (15, "2026-09-05T00:00:00Z"),
            (16, "2026-09-02T00:00:00Z"),
        ]);
        assert_eq!(
            ultima_entrada(&perfil, "", &backlog),
            Corte::Backlog {
                feature: "16".into(),
                momento: "2026-09-02T00:00:00Z".into()
            }
        );
    }

    #[test]
    fn corte_desde_el_backlog_should_skip_a_cited_feature_that_never_started() {
        let perfil = perfil_con(&["Una. (#20, #14)"]);
        let backlog = backlog_con(&[(14, "2026-09-01T00:00:00Z"), (20, "")]);
        assert_eq!(
            ultima_entrada(&perfil, "", &backlog),
            Corte::Backlog {
                feature: "14".into(),
                momento: "2026-09-01T00:00:00Z".into()
            }
        );
    }

    #[test]
    fn corte_desde_el_backlog_should_lose_to_a_newer_bitacora_entry_and_ignore_remove() {
        let perfil = perfil_con(&["Una. (#14)"]);
        let backlog = backlog_con(&[(14, "2026-09-02T00:00:00Z")]);
        let history = "- 2026-09-01T00:00:00Z perfil add Una. (#14)\n\
                       - 2026-09-03T00:00:00Z perfil replace Una -> Una. (#14)\n\
                       - 2026-09-09T00:00:00Z perfil remove Otra.\n";
        assert_eq!(
            ultima_entrada(&perfil, history, &backlog),
            Corte::Bitacora {
                momento: "2026-09-03T00:00:00Z".into()
            }
        );
        // Y una bitacora mas vieja que el inicio de la feature citada pierde.
        let vieja = "- 2026-09-01T00:00:00Z perfil add Una. (#14)\n";
        assert_eq!(
            ultima_entrada(&perfil, vieja, &backlog),
            Corte::Backlog {
                feature: "14".into(),
                momento: "2026-09-02T00:00:00Z".into()
            }
        );
    }

    #[test]
    fn perfil_sin_corte_should_count_everything() {
        // AC-4: sin cita y sin linea `perfil add`, no hay corte y se cuenta todo.
        let perfil = perfil_con(&["Sin cita."]);
        let backlog = backlog_con(&[(1, "2026-09-01T00:00:00Z")]);
        let history = "- 2026-09-01T00:00:00Z start feature #1 x\n\
                       - 2026-09-02T00:00:00Z perfil remove Sin cita.\n";
        assert_eq!(ultima_entrada(&perfil, history, &backlog), Corte::Ninguno);
        let registros = [
            registro("2026-09-01T00:00:00Z", false),
            registro("2026-09-05T00:00:00Z", false),
            registro("2026-09-06T00:00:00Z", true),
        ];
        let cuentas = contar(&registros, Corte::Ninguno);
        assert_eq!(
            (cuentas.nuevas, cuentas.total, cuentas.para_el_umbral()),
            (2, 2, 2)
        );
        assert_eq!(cuentas.corte.describir(), "sin corte");
        assert_eq!(cuentas.corte.origen(), None);
    }

    #[test]
    fn dos_cuentas_should_split_nuevas_from_total_by_the_corte() {
        // AC-5: el mismo momento que el corte NO es posterior; lo incorporado
        // no entra en ninguna de las dos.
        let registros = [
            registro("2026-09-01T00:00:00Z", false),
            registro("2026-09-05T00:00:00Z", false),
            registro("2026-09-06T00:00:00Z", false),
            registro("2026-09-07T00:00:00Z", true),
        ];
        let corte = Corte::Bitacora {
            momento: "2026-09-05T00:00:00Z".into(),
        };
        let cuentas = contar(&registros, corte);
        assert_eq!(
            (cuentas.nuevas, cuentas.total, cuentas.para_el_umbral()),
            (1, 3, 1)
        );
        assert_eq!(cuentas.corte.describir(), "2026-09-05, bitacora");
        assert_eq!(cuentas.corte.origen(), Some("bitacora"));
        let backlog = Corte::Backlog {
            feature: "80".into(),
            momento: "2026-09-06T23:25:01Z".into(),
        };
        assert_eq!(backlog.describir(), "2026-09-06, backlog: inicio de la #80");
        assert_eq!(backlog.momento(), Some("2026-09-06T23:25:01Z"));
    }
}
