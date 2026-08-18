//! Lecciones: la memoria procedural del proyecto (feature #17).
//!
//! Una leccion vive en `docs/lecciones/<clase>.md` y esta ordenada por CLASE de
//! trabajo, no por id de feature: los artefactos `impl-*`/`review-*` cuentan que
//! paso en la feature N, y una leccion cuenta como se hace esta clase de tarea.
//!
//! Tres decisiones de diseno que el codigo tiene que respetar:
//!
//! - **El nombre es de clase, sin escape hatch** (OBS-1): si el nombre solo tiene
//!   sentido para la tarea de hoy, se rechaza. No hay flag que lo evite.
//! - **El cuerpo no se toca nunca desde el binario**: el arnes crea el esqueleto y
//!   actualiza telemetria en el frontmatter; el contenido lo escribe el agente.
//! - **Cero dependencia del hub** (AC-9): son archivos, y ningun camino de este
//!   modulo abre una conexion.

use std::path::{Path, PathBuf};

use crate::exit::Exit;
use crate::paths::HarnessPaths;
use crate::plan::slugify;
use crate::progress::now_stamp;

/// Carpeta de lecciones dentro del `docs/` de la RAIZ.
pub const DIR_NAME: &str = "lecciones";
/// Guia del arnes que convive con las lecciones (plantilla, no es una leccion).
pub const GUIA: &str = "COMO-ESCRIBIR-UNA-LECCION.md";
/// Declaracion de cierre para "no hubo nada que aprender".
pub const NINGUNA: &str = "ninguna";
/// Tope de la descripcion, en caracteres (una sola oracion que entre en el list).
pub const DESCRIPCION_MAX: usize = 80;

/// Fecha de hoy en `YYYY-MM-DD` (los sellos del arnes son UTC).
pub fn hoy() -> String {
    now_stamp().chars().take(10).collect()
}

pub fn dir(paths: &HarnessPaths) -> PathBuf {
    paths.plans.join(DIR_NAME)
}

pub fn file_for(paths: &HarnessPaths, nombre: &str) -> PathBuf {
    dir(paths).join(format!("{nombre}.md"))
}

pub fn rel_path(nombre: &str) -> String {
    format!("docs/{DIR_NAME}/{nombre}.md")
}

// ---------------------------------------------------------------------------
// Nombre de clase
// ---------------------------------------------------------------------------

/// Motivos de rechazo del nombre (AC-4). Cada uno explica QUE regla se violo,
/// porque el mensaje de error es la unica documentacion que el agente lee en el
/// momento en que se equivoca.
fn motivo_de_rechazo(bruto: &str) -> Option<String> {
    static FECHA: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
        #[allow(clippy::unwrap_used)] // patron constante, cubierto por los tests
        regex::Regex::new(r"\d{4}-\d{2}-\d{2}").unwrap()
    });
    static DIGITOS: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
        #[allow(clippy::unwrap_used)] // patron constante, cubierto por los tests
        regex::Regex::new(r"\d{3,}").unwrap()
    });
    let bajo = bruto.trim().to_lowercase();
    if bajo.contains("feature") {
        return Some("nombra una feature ('feature')".to_string());
    }
    if bajo.contains('#') {
        return Some("referencia un id ('#')".to_string());
    }
    for prefijo in ["fix-", "debug-", "audit-", "hotfix-"] {
        if bajo.starts_with(prefijo) {
            return Some(format!("empieza con '{prefijo}' (describe un arreglo puntual, no una clase)"));
        }
    }
    if FECHA.is_match(&bajo) {
        return Some("contiene una fecha".to_string());
    }
    if DIGITOS.is_match(&bajo) {
        return Some("contiene un numero de tres o mas digitos".to_string());
    }
    None
}

/// Valida y normaliza el nombre de una leccion. Devuelve el slug definitivo.
///
/// **Sin escape hatch** (decision del usuario 2026-08-16, OBS-1 de la #17): si el
/// nombre solo tiene sentido para la tarea de hoy, el remedio es elegir otro, no
/// saltear la regla. Es lo unico que impide que la biblioteca degenere en una
/// lista plana de una-leccion-por-feature.
pub fn validar_nombre_de_clase(nombre: &str) -> Result<String, Exit> {
    let bruto = nombre.trim();
    if bruto.is_empty() {
        return Err(Exit {
            code: 2,
            message: Some("El nombre de la leccion no puede estar vacio.".to_string()),
        });
    }
    if let Some(motivo) = motivo_de_rechazo(bruto) {
        return Err(Exit {
            code: 2,
            message: Some(format!(
                "'{bruto}' no es un nombre de CLASE: {motivo}.\n    \
                 Si el nombre solo tiene sentido para la tarea de hoy, esta mal: patchea una\n    \
                 leccion existente ('sh harness_cli leccion list') en vez de crear otra.\n    \
                 Validos: 'espejo-de-roles', 'instalador-idempotente'."
            )),
        });
    }
    let slug = slugify(bruto);
    if slug == "feature" {
        return Err(Exit {
            code: 2,
            message: Some(format!(
                "'{bruto}' no deja ninguna letra ni numero utilizable como nombre."
            )),
        });
    }
    Ok(slug)
}

// ---------------------------------------------------------------------------
// Frontmatter
// ---------------------------------------------------------------------------

/// Frontmatter como lineas crudas: se preservan el orden y las claves que este
/// binario no conoce, asi una leccion editada a mano no pierde nada al pasar por
/// `leccion usar`.
#[derive(Debug, Clone)]
pub struct Frontmatter {
    lines: Vec<String>,
    /// Fin de linea del archivo original. `.gitattributes` no normaliza `*.md`,
    /// asi que un checkout Windows puede traer CRLF: re-renderizar con `\n` a
    /// secas dejaria el frontmatter en LF y el cuerpo en CRLF.
    eol: &'static str,
}

impl Default for Frontmatter {
    fn default() -> Self {
        Frontmatter {
            lines: Vec::new(),
            eol: "\n",
        }
    }
}

impl Frontmatter {
    fn split_line(line: &str) -> Option<(&str, &str)> {
        let (k, v) = line.split_once(':')?;
        if k.trim().is_empty() || k.starts_with([' ', '\t', '-']) {
            return None;
        }
        Some((k.trim(), v.trim()))
    }

    pub fn get(&self, key: &str) -> Option<String> {
        self.lines.iter().find_map(|l| {
            Self::split_line(l).and_then(|(k, v)| (k == key).then(|| v.to_string()))
        })
    }

    /// Valor de lista `clave: [a, b, c]`. Un valor sin corchetes se lee como un
    /// unico elemento; vacio devuelve vacio.
    pub fn list(&self, key: &str) -> Vec<String> {
        let raw = self.get(key).unwrap_or_default();
        let inner = raw
            .strip_prefix('[')
            .and_then(|s| s.strip_suffix(']'))
            .unwrap_or(&raw);
        inner
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .collect()
    }

    pub fn usos(&self) -> u64 {
        self.get("usos")
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(0)
    }

    /// Reemplaza el valor de una clave existente; si no existe, la agrega al
    /// final (nunca reordena lo que ya estaba).
    pub fn set(&mut self, key: &str, value: &str) {
        for line in &mut self.lines {
            if let Some((k, _)) = Self::split_line(line)
                && k == key
            {
                *line = format!("{key}: {value}");
                return;
            }
        }
        self.lines.push(format!("{key}: {value}"));
    }
}

// ---------------------------------------------------------------------------
// Leccion
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct Leccion {
    pub nombre: String,
    pub file: PathBuf,
    pub fm: Frontmatter,
    /// Cuerpo verbatim (todo lo que sigue al cierre del frontmatter).
    pub body: String,
}

impl Leccion {
    /// Parsea una leccion. El `Err` es el motivo legible que consume el gate de
    /// `harness_check.sh`: un frontmatter ilegible BLOQUEA (OBS-4).
    pub fn parse(file: &Path, text: &str) -> Result<Leccion, String> {
        let nombre = file
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        let sin_bom = text.strip_prefix('\u{feff}').unwrap_or(text);
        let resto = sin_bom
            .strip_prefix("---\n")
            .or_else(|| sin_bom.strip_prefix("---\r\n"))
            .ok_or_else(|| "no empieza con el frontmatter '---'".to_string())?;
        let cierre = resto
            .find("\n---")
            .ok_or_else(|| "el frontmatter no cierra con '---'".to_string())?;
        let cabecera = &resto[..cierre];
        // La ULTIMA linea del frontmatter llega con su `\r` pegado (el `\n` que
        // lo cerraba quedo del lado del separador), y `str::lines` solo saca el
        // `\r` que precede a un `\n`: hay que limpiarlo a mano.
        let lines: Vec<String> = cabecera
            .lines()
            .map(|l| l.trim_end_matches('\r').to_string())
            .collect();
        // `resto[cierre + 1..]` arranca en el `---` de cierre; el cuerpo es todo
        // lo que sigue a esa linea, verbatim.
        let body = resto[cierre + 1..]
            .split_once('\n')
            .map(|(_, b)| b)
            .unwrap_or("")
            .to_string();
        let eol = if cabecera.contains("\r\n") || cabecera.ends_with('\r') {
            "\r\n"
        } else {
            "\n"
        };
        let fm = Frontmatter { lines, eol };
        let declarado = fm.get("nombre").unwrap_or_default();
        if declarado.is_empty() {
            return Err("el frontmatter no declara 'nombre'".to_string());
        }
        if declarado != nombre {
            return Err(format!(
                "declara 'nombre: {declarado}' pero el archivo se llama '{nombre}.md'"
            ));
        }
        Ok(Leccion {
            nombre,
            file: file.to_path_buf(),
            fm,
            body,
        })
    }

    pub fn load(file: &Path) -> Result<Leccion, String> {
        let text = std::fs::read_to_string(file).map_err(|e| format!("no se pudo leer: {e}"))?;
        Leccion::parse(file, &text)
    }

    pub fn render(&self) -> String {
        let eol = self.fm.eol;
        format!(
            "---{eol}{}{eol}---{eol}{}",
            self.fm.lines.join(eol),
            self.body
        )
    }

    pub fn save(&self) -> anyhow::Result<()> {
        crate::features::write_text_atomic(&self.file, &self.render())
    }

    pub fn descripcion(&self) -> String {
        self.fm.get("descripcion").unwrap_or_default()
    }

    pub fn estado(&self) -> String {
        let e = self.fm.get("estado").unwrap_or_default();
        if e.is_empty() { "activa".to_string() } else { e }
    }

    pub fn usos(&self) -> u64 {
        self.fm.usos()
    }

    pub fn ultimo_uso(&self) -> String {
        self.fm.get("ultimo_uso").unwrap_or_default()
    }

    /// `leccion usar`: +1 uso y sello de fecha. NO toca el cuerpo ni
    /// `ultima_actualizacion`, que refleja cambios de CONTENIDO (AC-8).
    pub fn registrar_uso(&mut self) {
        let usos = self.usos() + 1;
        self.fm.set("usos", &usos.to_string());
        self.fm.set("ultimo_uso", &hoy());
    }
}

/// Recorre `docs/lecciones/`. Devuelve `(lecciones validas, rotas)`, donde cada
/// rota lleva su motivo. Sin la carpeta devuelve ambas vacias: no tener lecciones
/// no es un error.
pub fn scan(paths: &HarnessPaths) -> (Vec<Leccion>, Vec<(PathBuf, String)>) {
    let root = dir(paths);
    let mut ok = Vec::new();
    let mut rotas = Vec::new();
    let Ok(entries) = std::fs::read_dir(&root) else {
        return (ok, rotas);
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() || path.extension().is_none_or(|e| e != "md") {
            continue;
        }
        if path.file_name().is_some_and(|n| n == GUIA) {
            continue; // la guia es plantilla del arnes, no una leccion
        }
        match Leccion::load(&path) {
            Ok(l) => ok.push(l),
            Err(motivo) => rotas.push((path, motivo)),
        }
    }
    ok.sort_by(|a, b| b.usos().cmp(&a.usos()).then_with(|| a.nombre.cmp(&b.nombre)));
    rotas.sort_by(|a, b| a.0.cmp(&b.0));
    (ok, rotas)
}

/// Nombres mas parecidos a `buscado`, para el mensaje de "no existe" (AC-7).
/// Ranking barato y sin dependencias: prefijo comun mas largo, y a igualdad el
/// orden alfabetico.
pub fn parecidas(candidatas: &[Leccion], buscado: &str) -> Vec<String> {
    let mut por_puntaje: Vec<(usize, &str)> = candidatas
        .iter()
        .map(|l| (prefijo_comun(&l.nombre, buscado), l.nombre.as_str()))
        .filter(|(p, _)| *p > 0)
        .collect();
    por_puntaje.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(b.1)));
    por_puntaje
        .into_iter()
        .take(3)
        .map(|(_, n)| n.to_string())
        .collect()
}

fn prefijo_comun(a: &str, b: &str) -> usize {
    a.chars().zip(b.chars()).take_while(|(x, y)| x == y).count()
}

/// Esqueleto de una leccion nueva. El cuerpo son encabezados vacios a proposito:
/// el contenido lo escribe el agente, nunca el binario.
pub fn plantilla(nombre: &str, origen: Option<&str>) -> String {
    let origen_txt = origen.unwrap_or_default();
    let fecha = hoy();
    format!(
        "---\n\
         nombre: {nombre}\n\
         descripcion: <una sola oracion, max {DESCRIPCION_MAX} caracteres, terminada en punto.>\n\
         triggers: []\n\
         relacionadas: []\n\
         origen: [{origen_txt}]\n\
         usos: 0\n\
         ultimo_uso:\n\
         ultima_actualizacion: {fecha}\n\
         estado: activa\n\
         ---\n\
         \n\
         ## Cuando aplica\n\
         \n\
         <En que situacion alguien deberia leer esto: el sintoma o la tarea.>\n\
         \n\
         ## Procedimiento\n\
         \n\
         <Los pasos, en orden.>\n\
         \n\
         ## Pitfalls\n\
         \n\
         <Lo que sale mal, cada uno con su sintoma.>\n\
         \n\
         ## Verificacion\n\
         \n\
         <Como se sabe que quedo bien: el comando y la salida esperada.>\n"
    )
}

// ---------------------------------------------------------------------------
// Ciclo de vida (feature #21)
// ---------------------------------------------------------------------------

/// Carpeta de lecciones archivadas. **Visible** a proposito (decision del usuario
/// 2026-08-17, OBS-4 de la #21): `buscar` saltea los directorios ocultos, asi que
/// un `.archivo/` haria desaparecer el conocimiento archivado de las busquedas.
pub const ARCHIVO_DIR: &str = "archivo";
/// Dias de inactividad para pasar a `stale` (configurable por `rules`).
pub const STALE_DIAS: i64 = 30;
/// Dias de inactividad para archivar (configurable por `rules`).
pub const ARCHIVO_DIAS: i64 = 90;

pub const ESTADO_ACTIVA: &str = "activa";
pub const ESTADO_STALE: &str = "stale";
pub const ESTADO_ARCHIVADA: &str = "archivada";

/// Umbrales del ciclo de vida. Con `0` o negativo, ese tramo queda **apagado**.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Umbrales {
    pub stale: i64,
    pub archivo: i64,
}

impl Default for Umbrales {
    fn default() -> Self {
        Umbrales {
            stale: STALE_DIAS,
            archivo: ARCHIVO_DIAS,
        }
    }
}

impl Umbrales {
    /// Lee `rules.leccion_stale_dias` / `rules.leccion_archivo_dias`.
    pub fn from_rules(data: &serde_json::Value) -> Umbrales {
        let leer = |clave: &str, default: i64| {
            data.get("rules")
                .and_then(|r| r.get(clave))
                .and_then(serde_json::Value::as_i64)
                .unwrap_or(default)
        };
        Umbrales {
            stale: leer("leccion_stale_dias", STALE_DIAS),
            archivo: leer("leccion_archivo_dias", ARCHIVO_DIAS),
        }
    }
}

/// Que le corresponde a una leccion en esta pasada.
///
/// Es un enum y no un `Option<&str>` porque cada caso tiene su regla, su motivo y
/// su efecto en el filesystem (archivar MUEVE el archivo, las demas solo marcan).
/// (Patron "model states as enums" — skill `rust-patterns`.)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Transicion {
    /// Nada que hacer.
    Ninguna,
    /// Se enfrio: `activa` -> `stale`.
    AStale,
    /// Se vencio: `stale` -> `archivada` (se mueve a `archivo/`).
    AArchivada,
    /// Volvio a usarse: el uso resucita.
    AActiva,
}

impl Transicion {
    pub fn muta(self) -> bool {
        !matches!(self, Transicion::Ninguna)
    }

    pub fn estado_destino(self) -> Option<&'static str> {
        match self {
            Transicion::Ninguna => None,
            Transicion::AStale => Some(ESTADO_STALE),
            Transicion::AArchivada => Some(ESTADO_ARCHIVADA),
            Transicion::AActiva => Some(ESTADO_ACTIVA),
        }
    }
}

/// Dias entre dos fechas `YYYY-MM-DD`. `None` si alguna no parsea.
pub fn dias_entre(desde: &str, hasta: &str) -> Option<i64> {
    let d = chrono::NaiveDate::parse_from_str(desde.trim(), "%Y-%m-%d").ok()?;
    let h = chrono::NaiveDate::parse_from_str(hasta.trim(), "%Y-%m-%d").ok()?;
    Some((h - d).num_days())
}

impl Leccion {
    pub fn pinneada(&self) -> bool {
        self.fm
            .get("pinneada")
            .is_some_and(|v| matches!(v.trim(), "true" | "si" | "yes"))
    }

    pub fn set_pin(&mut self, valor: bool) {
        self.fm.set("pinneada", if valor { "true" } else { "false" });
    }

    pub fn set_estado(&mut self, estado: &str) {
        self.fm.set("estado", estado);
    }

    /// Dias de inactividad: desde `ultimo_uso`; si nunca se uso, desde
    /// `ultima_actualizacion`. Cero usos es ausencia de evidencia, no prueba de
    /// que sobra (AC-6): por eso una leccion nueva y sin usar no envejece antes
    /// que una usada.
    pub fn dias_inactiva(&self, hoy: &str) -> Option<i64> {
        let referencia = match self.ultimo_uso() {
            u if !u.is_empty() => u,
            _ => self.fm.get("ultima_actualizacion").unwrap_or_default(),
        };
        if referencia.is_empty() {
            return None;
        }
        dias_entre(&referencia, hoy).map(|d| d.max(0))
    }

    /// Que transicion le toca hoy. Funcion **pura**: no toca el filesystem, asi
    /// que el ciclo entero se testea sin esperar 90 dias.
    pub fn transicion(&self, hoy: &str, umbrales: Umbrales) -> Transicion {
        // El pin congela TODA transicion automatica, sin importar la antiguedad.
        if self.pinneada() {
            return Transicion::Ninguna;
        }
        let estado = self.estado();
        // Una archivada no vuelve sola: restaurar es manual.
        if estado == ESTADO_ARCHIVADA {
            return Transicion::Ninguna;
        }
        let Some(dias) = self.dias_inactiva(hoy) else {
            return Transicion::Ninguna;
        };
        if umbrales.archivo > 0 && dias >= umbrales.archivo {
            return Transicion::AArchivada;
        }
        if umbrales.stale > 0 && dias >= umbrales.stale {
            if estado == ESTADO_STALE {
                return Transicion::Ninguna;
            }
            return Transicion::AStale;
        }
        // Por debajo del umbral: si estaba stale, el uso la resucita.
        if estado == ESTADO_STALE {
            return Transicion::AActiva;
        }
        Transicion::Ninguna
    }
}

/// Carpeta de archivadas.
pub fn archivo_dir(paths: &HarnessPaths) -> PathBuf {
    dir(paths).join(ARCHIVO_DIR)
}

/// Recorre las lecciones **archivadas**.
pub fn scan_archivadas(paths: &HarnessPaths) -> Vec<Leccion> {
    let root = archivo_dir(paths);
    let Ok(entries) = std::fs::read_dir(&root) else {
        return Vec::new();
    };
    let mut out: Vec<Leccion> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_file() && p.extension().is_some_and(|e| e == "md"))
        .filter_map(|p| Leccion::load(&p).ok())
        .collect();
    out.sort_by(|a, b| a.nombre.cmp(&b.nombre));
    out
}

// ---------------------------------------------------------------------------
// El contrato (feature #18)
// ---------------------------------------------------------------------------

/// Las dos secciones de la guia que SON el contrato: el orden de preferencia y
/// la lista de que no capturar. El texto vive en la guia y no aca (decision del
/// usuario 2026-08-16, OBS-6 de la #18): una sola fuente de verdad, editable sin
/// recompilar y sin copia que pueda divergir.
const CONTRATO_SECCIONES: [&str; 2] = [
    "## La regla que ordena todo: primero patchear, crear al final",
    "## Que NO capturar",
];

pub fn guia_path(paths: &HarnessPaths) -> PathBuf {
    dir(paths).join(GUIA)
}

pub fn guia_rel() -> String {
    format!("docs/{DIR_NAME}/{GUIA}")
}

/// Extrae una seccion `## ...` completa: desde su encabezado hasta el proximo
/// encabezado de nivel 2 (o el final).
fn seccion(texto: &str, encabezado: &str) -> Option<String> {
    let inicio = texto.find(encabezado)?;
    let resto = &texto[inicio + encabezado.len()..];
    let fin = resto.find("\n## ").map_or(resto.len(), |i| i + 1);
    let cuerpo = resto[..fin].trim_end();
    Some(format!("{encabezado}{cuerpo}"))
}

/// El contrato leido de la guia. `None` cuando la guia falta, esta vacia o le
/// falta alguna de las dos secciones: ahi el llamador degrada a un puntero
/// (AC-21). Leer la guia NUNCA puede romper un cierre.
pub fn contrato(paths: &HarnessPaths) -> Option<String> {
    let texto = std::fs::read_to_string(guia_path(paths)).ok()?;
    let partes: Option<Vec<String>> = CONTRATO_SECCIONES
        .iter()
        .map(|h| seccion(&texto, h))
        .collect();
    let partes = partes?;
    if partes.iter().any(|p| p.lines().count() < 2) {
        return None; // encabezado sin cuerpo: no es un contrato utilizable
    }
    Some(partes.join("\n\n"))
}

/// Contrato completo que se emite al cerrar sin declarar leccion. Degrada a un
/// puntero de dos lineas si la guia no da (AC-21).
pub fn texto_contrato_de_cierre(paths: &HarnessPaths) -> String {
    let guia = guia_rel();
    let comandos = format!(
        "  sh harness_cli leccion list            # el catalogo, por uso\n\
           sh harness_cli leccion show <clase>    # leerla antes de patchearla\n\
           sh harness_cli leccion nueva <clase>   # el ULTIMO recurso\n\
         Metodo completo: {guia}"
    );
    match contrato(paths) {
        Some(cuerpo) => format!(
            "\n[harness] Esta feature cerro SIN declarar que se aprendio.\n\
             Antes de seguir, revisa si dejo una leccion. El contrato:\n\n\
             {cuerpo}\n\n\
             {comandos}\n\
             'ninguna' es una salida real (--leccion ninguna --leccion-motivo \"...\"),\n\
             pero no deberia ser la respuesta por default.\n"
        ),
        None => format!(
            "\n[harness] Esta feature cerro SIN declarar que se aprendio.\n\
             El metodo para decidirlo esta en {guia}.\n"
        ),
    }
}

/// Recordatorio corto del nudge periodico (AC-1): <= 5 lineas, para que no se
/// vuelva ruido de fondo.
///
/// Dice "acciones" y no "escrituras" a proposito: el matcher del hook depende del
/// backend (Claude cuenta `Edit|Write|MultiEdit`, Codex suma `Bash`), asi que lo
/// que se cuenta son tool-calls que matchearon. Un sistema que existe para no
/// mentirle a las sesiones futuras no empieza mintiendo en su propio mensaje.
pub fn texto_recordatorio(acciones: u64) -> String {
    format!(
        "\n[harness] Van {acciones} acciones en esta feature. ¿Aparecio una tecnica,\n\
         un pitfall o una correccion que una sesion futura necesite?\n\
         Mira el catalogo ('sh harness_cli leccion list') y PATCHEA la que estuvo\n\
         en juego antes de crear otra. Nada que guardar es valido; no es el default.\n"
    )
}

// ---------------------------------------------------------------------------
// Gate del cierre
// ---------------------------------------------------------------------------

/// Lo que el cierre declaro haber aprendido.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Declaracion {
    /// Nombre de la clase, o `ninguna`.
    pub clase: String,
    /// Obligatorio cuando la clase es `ninguna`.
    pub motivo: Option<String>,
}

impl Declaracion {
    pub fn resumen(&self) -> String {
        match &self.motivo {
            Some(m) => format!("{} ({m})", self.clase),
            None => self.clase.clone(),
        }
    }
}

/// Lee `rules.require_leccion` (default false: gate apagado, decision del PRD de
/// aprendizaje del 2026-08-16, para no romper ninguna instalacion previa).
pub fn require_leccion(data: &serde_json::Value) -> bool {
    data.get("rules")
        .and_then(|r| r.get("require_leccion"))
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false)
}

/// Gate de `close --status done`. Devuelve la declaracion a registrar, o `None`
/// cuando no hay nada que registrar (gate apagado y sin `--leccion`).
///
/// Se valida ANTES de mutar la feature: un cierre rechazado no deja el backlog a
/// medio escribir.
pub fn gate(
    paths: &HarnessPaths,
    data: &serde_json::Value,
    status: &str,
    leccion: Option<&str>,
    motivo: Option<&str>,
) -> Result<Option<Declaracion>, Exit> {
    // blocked/pending son valvulas de escape (aparcar o abortar): no se le pide
    // una leccion a algo que no se termino.
    if status != "done" {
        return Ok(None);
    }
    let declarada = leccion.map(str::trim).filter(|s| !s.is_empty());
    let Some(clase) = declarada else {
        if require_leccion(data) {
            return Err(Exit {
                code: 2,
                message: Some(format!(
                    "[GATE] El cierre no declara que se aprendio y la regla require_leccion esta activa.\n    \
                     Dos salidas validas:\n      \
                     --leccion <clase>                        (patcheaste o creaste esa leccion)\n      \
                     --leccion {NINGUNA} --leccion-motivo \"...\"   (no hubo nada que aprender, y por que)\n    \
                     'Ninguna' es una salida real, pero no deberia ser la respuesta por default.\n    \
                     Catalogo: sh harness_cli leccion list"
                )),
            });
        }
        return Ok(None);
    };
    if clase.eq_ignore_ascii_case(NINGUNA) {
        let motivo = motivo.map(str::trim).filter(|s| !s.is_empty());
        let Some(motivo) = motivo else {
            return Err(Exit {
                code: 2,
                message: Some(format!(
                    "[GATE] '--leccion {NINGUNA}' exige --leccion-motivo \"<por que no hubo nada que aprender>\".\n    \
                     Declarar que no se aprendio nada es valido; hacerlo sin motivo, no."
                )),
            });
        };
        return Ok(Some(Declaracion {
            clase: NINGUNA.to_string(),
            motivo: Some(motivo.to_string()),
        }));
    }
    // Una clase inexistente FALLA (decision del usuario 2026-08-16, OBS-2): un
    // typo no puede dejar la declaracion apuntando al vacio.
    if !file_for(paths, clase).is_file() {
        let (todas, _) = scan(paths);
        let mut msg = format!(
            "[GATE] El cierre declara la leccion '{clase}' y no existe ({}).",
            rel_path(clase)
        );
        let cercanas = parecidas(&todas, clase);
        if !cercanas.is_empty() {
            msg.push_str(&format!("\n    ¿Quisiste decir? {}", cercanas.join(", ")));
        }
        msg.push_str(&format!(
            "\n    Crea la clase con 'sh harness_cli leccion nueva {clase}' o corregi el nombre."
        ));
        return Err(Exit {
            code: 2,
            message: Some(msg),
        });
    }
    Ok(Some(Declaracion {
        clase: clase.to_string(),
        motivo: None,
    }))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;
    use serde_json::json;

    #[test]
    fn validar_nombre_should_accept_class_level_names() {
        assert_eq!(
            validar_nombre_de_clase("espejo-de-roles").unwrap(),
            "espejo-de-roles"
        );
        assert_eq!(
            validar_nombre_de_clase("Instalador Idempotente").unwrap(),
            "instalador-idempotente"
        );
        // Un numero corto no descalifica: 'postgres-17' es una clase legitima.
        assert_eq!(
            validar_nombre_de_clase("hub-postgres-17").unwrap(),
            "hub-postgres-17"
        );
    }

    #[test]
    fn validar_nombre_should_reject_session_artifacts() {
        for malo in [
            "fix-espejo-roles",
            "debug-instalador",
            "audit-seguridad",
            "hotfix-hub",
            "espejo-feature-16",
            "leccion-#16",
            "instalador-2026-08-16",
            "error-12345",
        ] {
            let err = validar_nombre_de_clase(malo).unwrap_err();
            assert_eq!(err.code, 2, "{malo} deberia salir con exit 2");
            let msg = err.message.unwrap();
            assert!(msg.contains("no es un nombre de CLASE"), "{malo}: {msg}");
            assert!(msg.contains("espejo-de-roles"), "{malo}: falta el ejemplo");
        }
    }

    #[test]
    fn validar_nombre_should_reject_empty() {
        let err = validar_nombre_de_clase("   ").unwrap_err();
        assert_eq!(err.code, 2);
    }

    #[test]
    fn parse_should_round_trip_unknown_keys_and_body() {
        let text = "---\nnombre: espejo-de-roles\nclave-desconocida: valor\nusos: 3\n---\n\n## Cuando aplica\n\ntexto\n";
        let l = Leccion::parse(Path::new("/x/espejo-de-roles.md"), text).unwrap();
        assert_eq!(l.nombre, "espejo-de-roles");
        assert_eq!(l.usos(), 3);
        assert_eq!(l.render(), text);
    }

    #[test]
    fn parse_should_round_trip_crlf_files() {
        // `.gitattributes` no normaliza *.md: un checkout Windows puede traer
        // CRLF, y re-escribir la telemetria no puede dejar el archivo mixto.
        let text = "---\r\nnombre: a\r\nusos: 1\r\n---\r\n\r\n## Cuando aplica\r\n";
        let mut l = Leccion::parse(Path::new("/x/a.md"), text).unwrap();
        assert_eq!(l.render(), text);
        l.registrar_uso();
        let escrito = l.render();
        assert!(escrito.starts_with("---\r\nnombre: a\r\nusos: 2\r\n"));
        assert!(!escrito.contains("\n\n---"), "quedo una linea en LF suelta");
    }

    #[test]
    fn parse_should_reject_broken_frontmatter() {
        let sin_fm = Leccion::parse(Path::new("/x/a.md"), "## Titulo\n").unwrap_err();
        assert!(sin_fm.contains("no empieza con el frontmatter"));
        let sin_cierre = Leccion::parse(Path::new("/x/a.md"), "---\nnombre: a\n").unwrap_err();
        assert!(sin_cierre.contains("no cierra"));
        let sin_nombre = Leccion::parse(Path::new("/x/a.md"), "---\nusos: 0\n---\n").unwrap_err();
        assert!(sin_nombre.contains("no declara 'nombre'"));
    }

    #[test]
    fn parse_should_reject_name_mismatch() {
        let err = Leccion::parse(
            Path::new("/x/espejo-de-roles.md"),
            "---\nnombre: otra-cosa\n---\n",
        )
        .unwrap_err();
        assert!(err.contains("otra-cosa"), "{err}");
        assert!(err.contains("espejo-de-roles.md"), "{err}");
    }

    #[test]
    fn registrar_uso_should_not_touch_body_nor_ultima_actualizacion() {
        let text = "---\nnombre: a\nusos: 2\nultimo_uso:\nultima_actualizacion: 2026-01-01\n---\n\n## Cuando aplica\n\ncuerpo intacto\n";
        let mut l = Leccion::parse(Path::new("/x/a.md"), text).unwrap();
        let body_antes = l.body.clone();
        l.registrar_uso();
        assert_eq!(l.usos(), 3);
        assert_eq!(l.ultimo_uso(), hoy());
        assert_eq!(l.fm.get("ultima_actualizacion").unwrap(), "2026-01-01");
        assert_eq!(l.body, body_antes);
    }

    #[test]
    fn list_should_read_bracketed_and_bare_values() {
        let text = "---\nnombre: a\ntriggers: [roles, espejo]\nrelacionadas: []\norigen: 7\n---\n";
        let l = Leccion::parse(Path::new("/x/a.md"), text).unwrap();
        assert_eq!(l.fm.list("triggers"), ["roles", "espejo"]);
        assert!(l.fm.list("relacionadas").is_empty());
        assert_eq!(l.fm.list("origen"), ["7"]);
    }

    #[test]
    fn plantilla_should_parse_as_a_valid_leccion() {
        let text = plantilla("espejo-de-roles", Some("17"));
        let l = Leccion::parse(Path::new("/x/espejo-de-roles.md"), &text).unwrap();
        assert_eq!(l.usos(), 0);
        assert_eq!(l.estado(), "activa");
        assert_eq!(l.fm.list("origen"), ["17"]);
        for seccion in ["## Cuando aplica", "## Procedimiento", "## Pitfalls", "## Verificacion"] {
            assert!(l.body.contains(seccion), "falta {seccion}");
        }
    }

    /// Paths de prueba con una leccion ya escrita en `docs/lecciones/`.
    fn paths_con(nombres: &[&str]) -> (tempfile::TempDir, HarnessPaths) {
        let dir = tempfile::tempdir().unwrap();
        let paths = HarnessPaths::from_root(dir.path().to_path_buf());
        std::fs::create_dir_all(super::dir(&paths)).unwrap();
        for n in nombres {
            std::fs::write(file_for(&paths, n), plantilla(n, None)).unwrap();
        }
        (dir, paths)
    }

    /// Leccion con telemetria controlada, para probar el ciclo sin esperar dias.
    fn con_telemetria(usos: u64, ultimo_uso: &str, actualizada: &str, estado: &str) -> Leccion {
        let text = format!(
            "---\nnombre: x\nusos: {usos}\nultimo_uso: {ultimo_uso}\n\
             ultima_actualizacion: {actualizada}\nestado: {estado}\n---\n\ncuerpo\n"
        );
        Leccion::parse(Path::new("/x/x.md"), &text).unwrap()
    }

    #[test]
    fn dias_entre_should_count_calendar_days() {
        assert_eq!(dias_entre("2026-08-01", "2026-08-31"), Some(30));
        assert_eq!(dias_entre("2026-08-31", "2026-08-31"), Some(0));
        assert_eq!(dias_entre("basura", "2026-08-31"), None);
    }

    #[test]
    fn dias_inactiva_should_prefer_last_use_over_last_update() {
        // Usada hace poco pero escrita hace mucho: cuenta el uso.
        let l = con_telemetria(3, "2026-08-15", "2026-01-01", ESTADO_ACTIVA);
        assert_eq!(l.dias_inactiva("2026-08-17"), Some(2));
        // Nunca usada: cuenta desde que se escribio.
        let nueva = con_telemetria(0, "", "2026-08-15", ESTADO_ACTIVA);
        assert_eq!(nueva.dias_inactiva("2026-08-17"), Some(2));
    }

    #[test]
    fn transicion_should_respect_the_exact_stale_threshold() {
        let u = Umbrales::default();
        // 29 dias: todavia no.
        let casi = con_telemetria(1, "2026-07-19", "2026-07-19", ESTADO_ACTIVA);
        assert_eq!(casi.dias_inactiva("2026-08-17"), Some(29));
        assert_eq!(casi.transicion("2026-08-17", u), Transicion::Ninguna);
        // 30 dias exactos: si.
        let justo = con_telemetria(1, "2026-07-18", "2026-07-18", ESTADO_ACTIVA);
        assert_eq!(justo.dias_inactiva("2026-08-17"), Some(30));
        assert_eq!(justo.transicion("2026-08-17", u), Transicion::AStale);
    }

    #[test]
    fn transicion_should_respect_the_exact_archive_threshold() {
        let u = Umbrales::default();
        // 89 dias: sigue stale, no se archiva.
        let casi = con_telemetria(1, "2026-05-20", "2026-05-20", ESTADO_STALE);
        assert_eq!(casi.dias_inactiva("2026-08-17"), Some(89));
        assert_eq!(casi.transicion("2026-08-17", u), Transicion::Ninguna);
        // 90 dias exactos: se archiva.
        let justo = con_telemetria(1, "2026-05-19", "2026-05-19", ESTADO_STALE);
        assert_eq!(justo.dias_inactiva("2026-08-17"), Some(90));
        assert_eq!(justo.transicion("2026-08-17", u), Transicion::AArchivada);
    }

    #[test]
    fn transicion_should_never_touch_a_pinned_lesson() {
        let u = Umbrales::default();
        let mut vieja = con_telemetria(0, "", "2025-01-01", ESTADO_ACTIVA);
        // Sin pin, con 200+ dias, se archivaria.
        assert_eq!(vieja.transicion("2026-08-17", u), Transicion::AArchivada);
        vieja.set_pin(true);
        assert_eq!(vieja.transicion("2026-08-17", u), Transicion::Ninguna);
    }

    #[test]
    fn transicion_should_revive_a_stale_lesson_that_was_used() {
        let u = Umbrales::default();
        let usada = con_telemetria(5, "2026-08-16", "2026-01-01", ESTADO_STALE);
        assert_eq!(usada.transicion("2026-08-17", u), Transicion::AActiva);
    }

    #[test]
    fn transicion_should_not_bring_an_archived_lesson_back_by_itself() {
        // Restaurar es manual: una archivada no vuelve sola aunque se use.
        let u = Umbrales::default();
        let arch = con_telemetria(5, "2026-08-16", "2026-08-16", ESTADO_ARCHIVADA);
        assert_eq!(arch.transicion("2026-08-17", u), Transicion::Ninguna);
    }

    #[test]
    fn transicion_should_be_switchable_off_per_threshold() {
        let vieja = con_telemetria(0, "", "2025-01-01", ESTADO_ACTIVA);
        // Archivo apagado: se queda en stale.
        let sin_archivo = Umbrales { stale: 30, archivo: 0 };
        assert_eq!(vieja.transicion("2026-08-17", sin_archivo), Transicion::AStale);
        // Todo apagado: nada.
        let apagado = Umbrales { stale: 0, archivo: 0 };
        assert_eq!(vieja.transicion("2026-08-17", apagado), Transicion::Ninguna);
    }

    #[test]
    fn umbrales_should_come_from_rules_with_defaults() {
        assert_eq!(Umbrales::from_rules(&json!({})), Umbrales::default());
        let propios = Umbrales::from_rules(&json!({
            "rules": {"leccion_stale_dias": 7, "leccion_archivo_dias": 21}
        }));
        assert_eq!(propios, Umbrales { stale: 7, archivo: 21 });
    }

    #[test]
    fn set_pin_should_not_touch_the_body_nor_telemetry() {
        let mut l = con_telemetria(3, "2026-08-15", "2026-01-01", ESTADO_ACTIVA);
        let cuerpo = l.body.clone();
        l.set_pin(true);
        assert!(l.pinneada());
        assert_eq!(l.usos(), 3);
        assert_eq!(l.ultimo_uso(), "2026-08-15");
        assert_eq!(l.body, cuerpo);
    }

    /// Guia minima pero valida: las dos secciones del contrato con cuerpo.
    fn guia_valida() -> String {
        format!(
            "# Como escribir una leccion\n\nIntro.\n\n\
             {}\n\nPatchea la que estuvo en juego antes de crear otra.\n\n\
             ## Seccion del medio\n\nRuido que NO va al contrato.\n\n\
             {}\n\n1. Fallas del entorno.\n2. Afirmaciones negativas.\n\n\
             ## Sin secretos\n\nNo lleves credenciales.\n",
            CONTRATO_SECCIONES[0], CONTRATO_SECCIONES[1]
        )
    }

    #[test]
    fn contrato_should_extract_only_the_two_sections() {
        let (_d, paths) = paths_con(&[]);
        std::fs::write(guia_path(&paths), guia_valida()).unwrap();
        let c = contrato(&paths).unwrap();
        assert!(c.contains(CONTRATO_SECCIONES[0]));
        assert!(c.contains("Patchea la que estuvo en juego"));
        assert!(c.contains(CONTRATO_SECCIONES[1]));
        assert!(c.contains("Fallas del entorno"));
        // Ni la seccion del medio ni la de despues se cuelan.
        assert!(!c.contains("Ruido que NO va"));
        assert!(!c.contains("No lleves credenciales"));
    }

    #[test]
    fn contrato_should_degrade_when_the_guide_is_unusable() {
        let (_d, paths) = paths_con(&[]);
        // (a) Sin guia.
        assert!(contrato(&paths).is_none());
        // (b) Guia sin las secciones del contrato.
        std::fs::write(guia_path(&paths), "# Guia\n\n## Otra cosa\n\ntexto\n").unwrap();
        assert!(contrato(&paths).is_none());
        // (c) Encabezado presente pero sin cuerpo.
        std::fs::write(
            guia_path(&paths),
            format!("{}\n{}\n", CONTRATO_SECCIONES[0], CONTRATO_SECCIONES[1]),
        )
        .unwrap();
        assert!(contrato(&paths).is_none());
    }

    #[test]
    fn texto_contrato_should_fall_back_to_a_pointer() {
        let (_d, paths) = paths_con(&[]);
        let puntero = texto_contrato_de_cierre(&paths);
        assert!(puntero.contains("SIN declarar que se aprendio"));
        assert!(puntero.contains(&guia_rel()));
        assert!(puntero.lines().filter(|l| !l.is_empty()).count() <= 2);
        // Con guia valida, el contrato completo trae las reglas y los comandos.
        std::fs::write(guia_path(&paths), guia_valida()).unwrap();
        let completo = texto_contrato_de_cierre(&paths);
        assert!(completo.contains("Fallas del entorno"));
        assert!(completo.contains("leccion list"));
        assert!(completo.contains("--leccion ninguna"));
    }

    #[test]
    fn texto_recordatorio_should_stay_short() {
        let t = texto_recordatorio(25);
        assert!(t.contains("25 acciones"));
        assert!(t.contains("leccion list"));
        assert!(t.contains("PATCHEA"));
        assert!(
            t.lines().filter(|l| !l.trim().is_empty()).count() <= 5,
            "el recordatorio tiene que entrar en 5 lineas: {t}"
        );
    }

    #[test]
    fn gate_should_be_silent_without_the_rule() {
        let (_d, paths) = paths_con(&[]);
        let data = json!({"rules": {}});
        assert_eq!(gate(&paths, &data, "done", None, None).unwrap(), None);
        // Sin regla y sin --leccion, el cierre es exactamente el de siempre.
        let sin_rules = json!({});
        assert_eq!(gate(&paths, &sin_rules, "done", None, None).unwrap(), None);
    }

    #[test]
    fn gate_should_block_done_without_declaration_when_rule_is_on() {
        let (_d, paths) = paths_con(&[]);
        let data = json!({"rules": {"require_leccion": true}});
        let err = gate(&paths, &data, "done", None, None).unwrap_err();
        assert_eq!(err.code, 2);
        let msg = err.message.unwrap();
        assert!(msg.contains("--leccion <clase>"), "{msg}");
        assert!(msg.contains("--leccion-motivo"), "{msg}");
    }

    #[test]
    fn gate_should_not_ask_for_a_leccion_on_blocked_or_pending() {
        let (_d, paths) = paths_con(&[]);
        let data = json!({"rules": {"require_leccion": true}});
        assert_eq!(gate(&paths, &data, "blocked", None, None).unwrap(), None);
        assert_eq!(gate(&paths, &data, "pending", None, None).unwrap(), None);
    }

    #[test]
    fn gate_should_accept_an_existing_class() {
        let (_d, paths) = paths_con(&["espejo-de-roles"]);
        let data = json!({"rules": {"require_leccion": true}});
        let decl = gate(&paths, &data, "done", Some("espejo-de-roles"), None)
            .unwrap()
            .unwrap();
        assert_eq!(decl.clase, "espejo-de-roles");
        assert_eq!(decl.motivo, None);
    }

    #[test]
    fn gate_should_reject_a_class_that_does_not_exist() {
        let (_d, paths) = paths_con(&["espejo-de-roles"]);
        let data = json!({"rules": {"require_leccion": true}});
        let err = gate(&paths, &data, "done", Some("espejo-de-rol"), None).unwrap_err();
        assert_eq!(err.code, 2);
        let msg = err.message.unwrap();
        assert!(msg.contains("no existe"), "{msg}");
        assert!(msg.contains("espejo-de-roles"), "falta la sugerencia: {msg}");
    }

    #[test]
    fn gate_should_require_a_motive_for_ninguna() {
        let (_d, paths) = paths_con(&[]);
        let data = json!({"rules": {"require_leccion": true}});
        let err = gate(&paths, &data, "done", Some(NINGUNA), None).unwrap_err();
        assert_eq!(err.code, 2);
        assert!(err.message.unwrap().contains("--leccion-motivo"));
        let decl = gate(&paths, &data, "done", Some(NINGUNA), Some("trabajo mecanico"))
            .unwrap()
            .unwrap();
        assert_eq!(decl.clase, NINGUNA);
        assert_eq!(decl.motivo.as_deref(), Some("trabajo mecanico"));
        assert_eq!(decl.resumen(), "ninguna (trabajo mecanico)");
    }

    #[test]
    fn gate_should_record_a_declaration_even_with_the_rule_off() {
        // Declarar siempre se puede; lo que la regla cambia es si es OBLIGATORIO.
        let (_d, paths) = paths_con(&["espejo-de-roles"]);
        let data = json!({"rules": {"require_leccion": false}});
        let decl = gate(&paths, &data, "done", Some("espejo-de-roles"), None)
            .unwrap()
            .unwrap();
        assert_eq!(decl.clase, "espejo-de-roles");
    }

    #[test]
    fn scan_should_skip_the_guide_and_report_broken_ones() {
        let (_d, paths) = paths_con(&["espejo-de-roles"]);
        std::fs::write(super::dir(&paths).join(GUIA), "# Guia\n").unwrap();
        std::fs::write(super::dir(&paths).join("rota.md"), "sin frontmatter\n").unwrap();
        let (ok, rotas) = scan(&paths);
        assert_eq!(ok.len(), 1);
        assert_eq!(ok[0].nombre, "espejo-de-roles");
        assert_eq!(rotas.len(), 1);
        assert!(rotas[0].0.ends_with("rota.md"));
    }

    #[test]
    fn scan_should_sort_by_uses_desc() {
        let (_d, paths) = paths_con(&["una", "otra"]);
        let mut otra = Leccion::load(&file_for(&paths, "otra")).unwrap();
        otra.registrar_uso();
        otra.save().unwrap();
        let (ok, _) = scan(&paths);
        assert_eq!(
            ok.iter().map(|l| l.nombre.as_str()).collect::<Vec<_>>(),
            ["otra", "una"]
        );
    }

    #[test]
    fn parecidas_should_rank_by_common_prefix() {
        let mk = |n: &str| {
            Leccion::parse(
                Path::new("/x").join(format!("{n}.md")).as_path(),
                &format!("---\nnombre: {n}\n---\n"),
            )
            .unwrap()
        };
        let todas = vec![mk("espejo-de-roles"), mk("espejo-de-hooks"), mk("otra")];
        assert_eq!(
            parecidas(&todas, "espejo-de-rol"),
            ["espejo-de-roles", "espejo-de-hooks"]
        );
    }
}
