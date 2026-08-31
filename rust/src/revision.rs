//! El paquete de revision (feature #51): todo lo que el reviewer necesita para
//! revisar una feature, y NADA mas.
//!
//! El disparador fue un dato concreto: verificar lo implementado llego a costar
//! **10 millones de tokens**, casi todos gastados explorando el repo y
//! releyendo lo que ya estaba en el spec. Este modulo existe para que el
//! reviewer arranque con el material ya juntado — AC con su estado en verify,
//! evidencia, archivos tocados, diff y rutas protegidas — en vez de salir a
//! buscarlo.
//!
//! Dos reglas de este modulo:
//!
//! - **No escribe nada**: es de solo lectura. El paquete se imprime.
//! - **Nunca recorta en silencio**: si el diff no entra en el presupuesto, el
//!   paquete dice cuanto quedo afuera y donde pedirlo.

use std::path::Path;

use serde_json::{Value, json};

use crate::paths::HarnessPaths;

/// Presupuesto por default: alcanza para el diff de una feature normal de este
/// repo y entra holgado en un turno de revision.
pub const MAX_LINEAS_DEFAULT: usize = 400;

/// Un AC del spec con lo que se sabe de el.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ac {
    pub id: String,
    pub texto: String,
    /// `verde`, `rojo`, `vacio`, `manual`... o `None` si no hay reporte.
    pub estado: Option<String>,
}

/// Lo que se recorto para respetar el presupuesto.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Recorte {
    pub lineas_mostradas: usize,
    pub lineas_totales: usize,
}

/// El paquete completo.
#[derive(Debug, Clone)]
pub struct Paquete {
    pub feature_id: String,
    pub nombre: String,
    pub acs: Vec<Ac>,
    /// Filas de la tabla de evidencia de `impl-<id>.md`, tal cual.
    pub evidencia: Vec<String>,
    pub archivos: Vec<String>,
    pub diff: String,
    pub recorte: Option<Recorte>,
    pub protegidas: Vec<String>,
    /// Que se busco y no estaba (spec, impl, verify, rama).
    pub faltantes: Vec<String>,
}

impl Paquete {
    /// Tamaño del paquete, para que el costo se vea ANTES de gastarlo (AC-12b).
    /// La estimacion de tokens usa la regla practica de ~4 caracteres por token.
    pub fn tamano(&self) -> (usize, usize) {
        let texto = self.render_texto();
        let lineas = texto.lines().count();
        let tokens = texto.chars().count() / 4;
        (lineas, tokens)
    }

    /// El paquete en texto, que es lo que lee el reviewer.
    pub fn render_texto(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!(
            "== Paquete de revision - Feature #{}: {} ==\n\n",
            self.feature_id, self.nombre
        ));

        out.push_str("## AC del spec (y su estado en verify)\n\n");
        if self.acs.is_empty() {
            out.push_str("(el spec no declara AC-n todavia)\n");
        }
        for ac in &self.acs {
            let estado = ac.estado.as_deref().unwrap_or("sin verificar");
            out.push_str(&format!("- [{estado}] {}: {}\n", ac.id, ac.texto));
        }

        out.push_str("\n## Evidencia declarada (impl)\n\n");
        if self.evidencia.is_empty() {
            out.push_str("(sin tabla de evidencia)\n");
        }
        for fila in &self.evidencia {
            out.push_str(fila);
            out.push('\n');
        }

        out.push_str("\n## Archivos tocados\n\n");
        if self.archivos.is_empty() {
            out.push_str("(ninguno)\n");
        }
        for a in &self.archivos {
            out.push_str(&format!("- {a}\n"));
        }

        if !self.protegidas.is_empty() {
            out.push_str("\n## RUTAS PROTEGIDAS TOCADAS\n\n");
            for p in &self.protegidas {
                out.push_str(&format!("- {p}\n"));
            }
            out.push_str(
                "\nSon documentos del usuario: el veredicto es `blocked` salvo que lo haya pedido explicitamente.\n",
            );
        }

        out.push_str("\n## Diff\n\n");
        if self.diff.trim().is_empty() {
            out.push_str("(sin diff: la rama no tiene commits propios)\n");
        } else {
            out.push_str("```diff\n");
            out.push_str(&self.diff);
            if !self.diff.ends_with('\n') {
                out.push('\n');
            }
            out.push_str("```\n");
        }
        if let Some(r) = &self.recorte {
            out.push_str(&format!(
                "\n[recortado] se muestran {} de {} lineas del diff. Para el resto, a mano:\n  git diff <base>...<rama> -- <archivo>\n",
                r.lineas_mostradas, r.lineas_totales
            ));
        }

        if !self.faltantes.is_empty() {
            out.push_str("\n## Falta\n\n");
            for f in &self.faltantes {
                out.push_str(&format!("- {f}\n"));
            }
        }
        out
    }

    /// El mismo contenido en JSON, para que un agente no tenga que parsear
    /// texto (AC-14).
    pub fn render_json(&self) -> Value {
        let (lineas, tokens) = self.tamano();
        json!({
            "feature": self.feature_id,
            "nombre": self.nombre,
            "acs": self.acs.iter().map(|a| json!({
                "id": a.id,
                "texto": a.texto,
                "estado": a.estado,
            })).collect::<Vec<_>>(),
            "evidencia": self.evidencia,
            "archivos": self.archivos,
            "diff": self.diff,
            "recorte": self.recorte.as_ref().map(|r| json!({
                "lineas_mostradas": r.lineas_mostradas,
                "lineas_totales": r.lineas_totales,
            })),
            "protegidas": self.protegidas,
            "faltantes": self.faltantes,
            "tamano": {"lineas": lineas, "tokens_estimados": tokens},
        })
    }
}

/// Estado por AC leido de `docs/verify-<id>.md` (tabla `| AC-n | estado | ...`).
pub fn estados_de_verify(texto: &str) -> Vec<(String, String)> {
    texto
        .lines()
        .filter(|l| l.trim_start().starts_with("| AC-"))
        .filter_map(|l| {
            let celdas: Vec<&str> = l.trim().trim_matches('|').split('|').collect();
            match celdas.as_slice() {
                [ac, estado, ..] => Some((ac.trim().to_string(), estado.trim().to_string())),
                _ => None,
            }
        })
        .collect()
}

/// Filas de la tabla de evidencia de `impl-<id>.md`: las que empiezan por
/// `| AC-`. La cabecera y el separador se descartan solos.
pub fn filas_de_evidencia(texto: &str) -> Vec<String> {
    texto
        .lines()
        .map(str::trim)
        .filter(|l| l.starts_with("| AC-"))
        .map(str::to_string)
        .collect()
}

/// Recorta el diff al presupuesto. Devuelve el texto y, si hubo recorte, cuanto
/// quedo afuera: el paquete siempre lo declara (AC-12).
pub fn recortar(diff: &str, max_lineas: usize) -> (String, Option<Recorte>) {
    let lineas: Vec<&str> = diff.lines().collect();
    if lineas.len() <= max_lineas {
        return (diff.to_string(), None);
    }
    let mostradas: Vec<&str> = lineas.iter().take(max_lineas).copied().collect();
    (
        mostradas.join("\n"),
        Some(Recorte {
            lineas_mostradas: max_lineas,
            lineas_totales: lineas.len(),
        }),
    )
}

/// Arma el paquete leyendo lo que exista. Nunca falla por una ausencia: la
/// anota en `faltantes` (AC-13).
pub fn armar(
    paths: &HarnessPaths,
    feature: &serde_json::Map<String, Value>,
    max_lineas: usize,
) -> Paquete {
    use crate::pycompat::py_str;

    let fid = py_str(feature.get("id"));
    let nombre = py_str(feature.get("name"));
    let mut faltantes = Vec::new();

    // AC del spec + su estado en verify.
    let spec_path = crate::spec::spec_path(paths, feature);
    let acs_texto = match std::fs::read_to_string(&spec_path) {
        Ok(t) => crate::atlassian::emit::parse_acceptance_criteria(&t),
        Err(_) => {
            faltantes.push(format!("el spec ({})", spec_path.display()));
            Vec::new()
        }
    };
    let verify_path = paths.plans.join(format!("verify-{fid}.md"));
    let estados = match std::fs::read_to_string(&verify_path) {
        Ok(t) => estados_de_verify(&t),
        Err(_) => {
            faltantes.push(format!(
                "el reporte de verify ({}): los AC figuran sin verificar",
                verify_path.display()
            ));
            Vec::new()
        }
    };
    let acs = acs_texto
        .into_iter()
        .map(|(id, texto)| {
            let estado = estados
                .iter()
                .find(|(ac, _)| *ac == id)
                .map(|(_, e)| e.clone());
            Ac { id, texto, estado }
        })
        .collect();

    // Evidencia declarada.
    let impl_path = paths.plans.join(format!("impl-{fid}.md"));
    let evidencia = match std::fs::read_to_string(&impl_path) {
        Ok(t) => filas_de_evidencia(&t),
        Err(_) => {
            faltantes.push(format!("la evidencia ({})", impl_path.display()));
            Vec::new()
        }
    };

    // Diff y archivos tocados de la rama de la feature.
    let (archivos, diff_bruto) = cambios_de_la_rama(paths, feature, &mut faltantes);
    let (diff, recorte) = recortar(&diff_bruto, max_lineas);

    // Rutas protegidas entre lo tocado.
    let protegidas = {
        let data = crate::features::load_features(paths).unwrap_or(Value::Null);
        let patrones = crate::rutas::patrones(&data);
        archivos
            .iter()
            .filter(|a| crate::rutas::esta_protegida(a, &paths.repo_root, &patrones))
            .cloned()
            .collect()
    };

    Paquete {
        feature_id: fid,
        nombre,
        acs,
        evidencia,
        archivos,
        diff,
        recorte,
        protegidas,
        faltantes,
    }
}

/// Archivos y diff de la rama de la feature contra su base. Sin rama (modo
/// clasico o repo sin git) devuelve lo que haya sin commitear.
fn cambios_de_la_rama(
    paths: &HarnessPaths,
    feature: &serde_json::Map<String, Value>,
    faltantes: &mut Vec<String>,
) -> (Vec<String>, String) {
    let Some(principal) = crate::git::repo_principal(&paths.repo_root) else {
        faltantes.push("el repo git (no se puede calcular el diff)".to_string());
        return (Vec::new(), String::new());
    };
    let rama = feature.get("branch").and_then(Value::as_str);
    let worktree = feature
        .get("worktree")
        .and_then(Value::as_str)
        .map(std::path::PathBuf::from)
        .filter(|wt| wt.is_dir());
    match rama {
        Some(rama) if crate::git::rama_existe(&principal, rama) => {
            let base = crate::git::rama_base(&principal, None).unwrap_or_else(|| "main".to_string());
            // Se compara contra la BASE desde el worktree de la feature: asi el
            // paquete incluye tanto lo ya commiteado en la rama como lo que
            // todavia esta sin commitear. El reviewer revisa ANTES del cierre,
            // que es justo cuando el trabajo puede no estar commiteado — con
            // `base...rama` el paquete decia "archivos tocados: ninguno".
            let (dir, rango) = match &worktree {
                Some(wt) => (wt.clone(), base.clone()),
                None => (principal.clone(), format!("{base}...{rama}")),
            };
            let mut archivos = git_lineas(&dir, &["diff", "--name-only", &rango]);
            // Los archivos NUEVOS sin `git add` tambien son trabajo de la
            // feature: si el paquete no los nombra, el reviewer no se entera de
            // que existen. No se incluye su contenido en el diff (los agrega
            // git recien al indexarlos), pero se listan marcados.
            for nuevo in git_lineas(&dir, &["ls-files", "--others", "--exclude-standard"]) {
                archivos.push(format!("{nuevo} (nuevo, sin git add)"));
            }
            let diff = git_texto(&dir, &["diff", &rango]);
            (archivos, diff)
        }
        _ => {
            faltantes.push(
                "la rama de la feature (se usa el estado sin commitear del checkout)".to_string(),
            );
            let archivos = git_lineas(&principal, &["diff", "--name-only", "HEAD"]);
            let diff = git_texto(&principal, &["diff", "HEAD"]);
            (archivos, diff)
        }
    }
}

fn git_texto(dir: &Path, args: &[&str]) -> String {
    std::process::Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
        .unwrap_or_default()
}

fn git_lineas(dir: &Path, args: &[&str]) -> Vec<String> {
    git_texto(dir, args)
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(str::to_string)
        .collect()
}


// ===========================================================================
// El veredicto del reviewer (feature #64)
// ===========================================================================
//
// La promesa: `close --status done` no puede decir "revisado" sin que alguien
// haya revisado. Dos piezas la sostienen, y ninguna es disciplina:
//
// 1. **El sello lo escribe el binario.** El gate lee UNICAMENTE la linea
//    `Revisado:` que estampa `revision --veredicto`; la prosa del archivo
//    —incluido un `Veredicto: approved` tipeado a mano— no cuenta. De los 40
//    reviews que ya existen, 7 no son parseables y `docs/review-3.md:3` dice
//    "approved" y "cierre BLOQUEADO" en la misma linea: parsear prosa de un
//    agente es leer, no verificar.
// 2. **La cobertura por AC.** Estampar exige una fila por cada AC-n que declara
//    el SPEC (no el review), y cada fila tiene que citar `archivo:linea`. Un
//    review escrito en cinco segundos no puede citar lineas que existan.
//
// Lo que NO hace, y por que: no compara mtime contra `docs/impl-<id>.md`.
// `documentos.rs:23-26` ya rechazo esa comparacion por deadlock, y aca el
// deadlock es el ciclo normal (el reviewer pide cambios -> el implementer
// corrige -> el impl queda mas nuevo -> el gate bloquea para siempre), con una
// unica salida barata: `touch`. La regla entrenaria el `touch`. Ademas no
// detecta nada: de los 40 pares existentes, cero tienen el review mas viejo.

/// Prefijo de la linea que estampa el binario. Es lo UNICO que el gate lee.
pub const SELLO_REVIEW: &str = "Revisado:";

/// Los tres veredictos de `roles/reviewer.md`.
pub const VEREDICTOS: [&str; 3] = ["approved", "changes_requested", "blocked"];

/// Lee `rules.require_review` (default false: la regla nace apagada, como las
/// otras cuatro, para no romper instalaciones existentes).
pub fn require_review(data: &Value) -> bool {
    data.get("rules")
        .and_then(|r| r.get("require_review"))
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

/// `docs/review-<id>.md`, relativo (para los mensajes).
pub fn review_rel(fid: &str) -> String {
    format!("docs/review-{fid}.md")
}

/// `docs/review-<id>.md`, absoluto.
pub fn review_path(paths: &HarnessPaths, fid: &str) -> std::path::PathBuf {
    paths.plans.join(format!("review-{fid}.md"))
}

/// El veredicto ESTAMPADO, o `None` si el archivo no lleva sello del binario.
///
/// Deliberadamente NO mira ninguna linea `Veredicto:` en prosa (AC-2).
pub fn veredicto_estampado(texto: &str) -> Option<String> {
    for linea in lineas_fuera_de_bloque(texto) {
        let Some(resto) = linea.trim_start().strip_prefix(SELLO_REVIEW) else {
            continue;
        };
        // `continue`, no `?`: el `?` salia de la FUNCION ENTERA, asi que una
        // primera linea `Revisado:` sin nada detras abortaba el barrido y el
        // gate decia "no lleva el sello" con el sello tres lineas mas abajo. Un
        // mensaje de gate que afirma algo que el archivo desmiente es lo que la
        // #63 vino a cerrar.
        let Some(v) = resto.trim().split(['·', ' ']).find(|p| !p.is_empty()) else {
            continue;
        };
        if VEREDICTOS.contains(&v) {
            return Some(v.to_string());
        }
    }
    None
}

/// Re-export del parser UNICO (feature #67).
///
/// Antes esta funcion era una implementacion propia, y `commands::revision`
/// tenia otra, y `verificacion` una tercera. Discrepaban en el 37% de los
/// documentos de siete lineas.
pub use crate::markdown::lineas_fuera_de_bloque;


/// La linea canonica del sello. La escribe SOLO el binario.
pub fn linea_sello(veredicto: &str, stamp: &str) -> String {
    format!("{SELLO_REVIEW} {veredicto} · {stamp} · estampado por `harness revision --veredicto`")
}

/// `AC-1` no lo menciona una fila de `AC-12`: el match es de token completo.
///
/// Sin esto, un spec de 12 AC daba por cubierto el AC-1 con la fila del AC-10,
/// que es un gate que aprueba lo que no reviso.
fn menciona(linea: &str, ac: &str) -> bool {
    let mut desde = 0;
    while let Some(i) = linea[desde..].find(ac) {
        let fin = desde + i + ac.len();
        if !linea[fin..].starts_with(|c: char| c.is_ascii_digit()) {
            return true;
        }
        desde = fin;
    }
    false
}

/// Las raices contra las que puede resolver una cita del review.
///
/// La tercera es la que importa y la que faltaba: cuando la feature vive en un
/// worktree, el review cita archivos DEL WORKTREE (`rust/src/revision.rs:602`),
/// pero `root`/`repo_root` apuntan al checkout principal, donde ese archivo
/// existe con otro contenido y la linea 602 no existe. Es el mismo defecto de
/// worktree-vs-raiz que arreglaron la #60 y la #63, y aparecio la primera vez
/// que esta feature se uso de verdad: su propio review quedo rechazado con seis
/// AC "sin cita que resuelva", citando archivos que si estaban ahi.
/// `paths.plans` es el `docs/` de la feature, asi que su padre es esa raiz.
pub fn raices_de_citas(paths: &HarnessPaths) -> Vec<&Path> {
    raices_desde(&paths.plans, &paths.repo_root, &paths.root)
}

/// La parte pura, para poder testearla sin armar un `HarnessPaths` entero.
pub fn raices_desde<'a>(plans: &'a Path, repo_root: &'a Path, root: &'a Path) -> Vec<&'a Path> {
    let mut out: Vec<&Path> = Vec::new();
    if let Some(raiz_feature) = plans.parent() {
        out.push(raiz_feature);
    }
    for r in [repo_root, root] {
        if !out.contains(&r) {
            out.push(r);
        }
    }
    out
}

/// Los candidatos a cita `archivo:linea` de una fila, con su numero.
fn citas_de(linea: &str) -> Vec<(String, usize)> {
    let mut out = Vec::new();
    for token in linea.split(|c: char| c.is_whitespace() || c == '|' || c == '`') {
        let Some((ruta, num)) = token.rsplit_once(':') else {
            continue;
        };
        let num: String = num.chars().take_while(char::is_ascii_digit).collect();
        let Ok(n) = num.parse::<usize>() else { continue };
        let ruta = ruta.trim_matches(|c: char| c == '(' || c == ')' || c == ',' || c == '`');
        if n > 0 && !ruta.is_empty() {
            out.push((ruta.to_string(), n));
        }
    }
    out
}

/// Que se pudo averiguar de una cita `archivo:linea`.
///
/// Son TRES respuestas y no dos a proposito (feature #67). Antes el tope de
/// lectura devolvia `false` —"la linea no existe"— sobre citas correctas cuya
/// linea caia mas alla del tope: la linea existia y `sed` la mostraba. Es el
/// patron 127-vs-124 de `docs/lecciones/criterios-de-cierre-que-se-pueden-fallar.md`:
/// traducir "no pude comprobar" a "no". El tope se conserva —sacarlo cuesta
/// 10,5 s por 2 GB dentro de un gate sin timeout, que es la familia de la #66—
/// pero deja de mentir.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cita {
    Resuelve,
    NoResuelve,
    NoSePudoComprobar,
}

/// Tope de lectura por cita. No es memoria —el buffer fijo ya la acota— es
/// TIEMPO: un review de 20 AC citando archivos enormes correria dentro de
/// `close`, que no tiene timeout.
const TOPE_LECTURA: u64 = 8 * 1024 * 1024;

/// ¿La cita resuelve, no resuelve, o no se pudo comprobar?
fn evaluar_cita(raices: &[&Path], ruta: &str, linea: usize) -> Cita {
    if linea == 0 || ruta.contains("..") || Path::new(ruta).is_absolute() {
        return Cita::NoResuelve;
    }
    let mut sin_comprobar = false;
    for r in raices {
        let candidato = r.join(ruta);
        // `is_file()` antes de abrir: un FIFO dejaba el proceso colgado y un
        // symlink a /dev/zero agotaba la memoria.
        if !std::fs::metadata(&candidato).is_ok_and(|m| m.is_file()) {
            continue;
        }
        let Ok(f) = std::fs::File::open(&candidato) else {
            continue;
        };
        match contar_hasta(f, linea) {
            Cita::Resuelve => return Cita::Resuelve,
            Cita::NoSePudoComprobar => sin_comprobar = true,
            Cita::NoResuelve => {}
        }
    }
    if sin_comprobar {
        Cita::NoSePudoComprobar
    } else {
        Cita::NoResuelve
    }
}

/// Cuenta saltos por BYTES hasta encontrar la linea o agotar el tope.
///
/// `lines()` materializa cada linea entera, asi que un blob de 200 MB en UNA
/// sola linea costaba 211 MB de RSS aunque solo hiciera falta saber si existe la
/// linea 1.
fn contar_hasta(f: std::fs::File, linea: usize) -> Cita {
    // Lineas vistas hasta aca. Un archivo de N lineas terminado en salto tiene N
    // saltos, no N+1: contar `saltos + 1` siempre hacia que la cita a la linea
    // N+1 resolviera en cualquier archivo normal (reproducido: `evidencia.txt:4`
    // en un archivo de 3 lineas). La ultima linea solo suma si NO hay salto
    // final.
    let vistas = |saltos: usize, termina_en_salto: bool| {
        if termina_en_salto { saltos } else { saltos + 1 }
    };
    let mut leidos = 0u64;
    let mut saltos = 0usize;
    let mut termina_en_salto = true;
    let mut buf = [0u8; 64 * 1024];
    let mut r = std::io::BufReader::new(f);
    loop {
        let Ok(n) = std::io::Read::read(&mut r, &mut buf) else {
            return Cita::NoSePudoComprobar;
        };
        if n == 0 {
            return if vistas(saltos, termina_en_salto) >= linea {
                Cita::Resuelve
            } else {
                Cita::NoResuelve
            };
        }
        saltos += buf[..n].iter().filter(|b| **b == b'\n').count();
        termina_en_salto = buf[n - 1] == b'\n';
        // El MISMO conteo que en el EOF, y no `saltos >= linea`: la linea en
        // curso ya existe —se leyo un byte suyo— asi que exigirle su salto final
        // hacia que un archivo de una sola linea larga no pudiera confirmar ni
        // su linea 1 antes de agotar el tope. Encontrado por el test del AC-6:
        // es el mismo error que el AC-6 arregla —reportar "no pude" sobre algo
        // que si se puede— un paso antes.
        if vistas(saltos, termina_en_salto) >= linea {
            return Cita::Resuelve;
        }
        leidos += n as u64;
        if leidos > TOPE_LECTURA {
            return Cita::NoSePudoComprobar;
        }
    }
}

/// ¿Una fila responde por este AC con una cita que resuelve?
///
/// El corte es la CITA: sin un `archivo:linea` que exista de verdad, la fila es
/// una afirmacion, y una afirmacion es justo lo que un review de cinco segundos
/// sabe escribir.
fn fila_responde(raices: &[&Path], linea: &str, ac: &str) -> bool {
    menciona(linea, ac)
        && citas_de(linea)
            .iter()
            .any(|(ruta, n)| evaluar_cita(raices, ruta, *n) == Cita::Resuelve)
}

/// Los AC del SPEC que el review no responde con una cita. Vacio = cubierto.
///
/// La lista sale del spec, no del review: si saliera del review, un review
/// vacio estaria "completo".
pub fn acs_sin_fila(raices: &[&Path], spec: &str, review: &str) -> Vec<String> {
    let filas: Vec<&str> = lineas_fuera_de_bloque(review);
    crate::verificacion::parsear(spec)
        .into_iter()
        .map(|v| v.ac)
        .filter(|ac| !filas.iter().any(|l| fila_responde(raices, l, ac)))
        .collect()
}

/// Gate de cierre: con la regla activa, `done` exige un review estampado y
/// `approved`. Solo LEE; estampar es `revision --veredicto`.
pub fn gate(
    paths: &HarnessPaths,
    data: &Value,
    status: &str,
    feature: &serde_json::Map<String, Value>,
    fid: &str,
) -> Result<(), crate::exit::Exit> {
    use crate::exit::Exit;
    if status != "done" || !require_review(data) {
        return Ok(());
    }
    let rel = review_rel(fid);
    let Ok(texto) = std::fs::read_to_string(review_path(paths, fid)) else {
        return Err(Exit {
            code: 2,
            message: Some(format!(
                "[GATE] Falta el veredicto del reviewer: {rel}.\n    \
                 La regla require_review esta activa: cerrar como done exige que\n    \
                 alguien haya revisado, con una fila por cada AC-n del spec.\n    \
                 Arranca por el paquete: sh harness_cli revision --feature {fid}\n    \
                 y registra el veredicto: sh harness_cli revision --feature {fid} --veredicto approved"
            )),
        });
    };
    let Some(veredicto) = veredicto_estampado(&texto) else {
        return Err(Exit {
            code: 2,
            message: Some(format!(
                "[GATE] {rel} no lleva el sello del arnes.\n    \
                 El gate lee unicamente la linea `{SELLO_REVIEW} ...` que estampa el\n    \
                 binario; un `Veredicto:` escrito a mano no cuenta como revision.\n    \
                 Registralo con: sh harness_cli revision --feature {fid} --veredicto approved"
            )),
        });
    };
    // El sello dice QUE se decidio; la cobertura dice que se MIRO. El gate
    // re-verifica las dos, y no le alcanza con la primera: la linea del sello es
    // texto y un agente decidido la puede tipear (lo encontro el reviewer de
    // esta misma feature). Lo que no puede fabricar en cinco segundos es una
    // fila por cada AC del spec con su cita. Por eso la barrera que sostiene la
    // promesa es esta, y se comprueba de nuevo aca aunque `revision --veredicto`
    // ya la haya comprobado al estampar.
    // La misma guarda que `estampar` (commands/revision.rs:84): un spec ilegible
    // o sin AC no es "todo cubierto", es que no hay contra que medir. Sin esto,
    // `unwrap_or_default()` + 0 AC = 0 faltantes, y el cierre pasaba con el
    // sello solo — o sea que B1 se reabria por otra puerta.
    let spec_path = crate::spec::spec_path(paths, feature);
    let Ok(spec) = std::fs::read_to_string(&spec_path) else {
        return Err(Exit {
            code: 2,
            message: Some(format!(
                "[GATE] No se pudo leer el spec de la feature #{fid}: {}.\n    \
                 Sin spec no hay AC contra que medir el review, asi que el\n    \
                 veredicto no se puede comprobar.",
                spec_path.display()
            )),
        });
    };
    if crate::verificacion::parsear(&spec).is_empty() {
        return Err(Exit {
            code: 2,
            message: Some(format!(
                "[GATE] El spec de la feature #{fid} no declara ningun AC-n.\n    \
                 Sin AC, comprobar la cobertura del review no comprueba nada."
            )),
        });
    }
    let faltan = acs_sin_fila(&raices_de_citas(paths), &spec, &texto);
    if !faltan.is_empty() {
        return Err(Exit {
            code: 2,
            message: Some(format!(
                "[GATE] {rel} no responde por {} AC del spec: {}.\n    \
                 Cada AC-n necesita una fila que lo nombre y cite `archivo:linea`;\n    \
                 una fila sin cita es una afirmacion, no una verificacion.\n    \
                 Completa el review y volve a registrarlo:\n      \
                 sh harness_cli revision --feature {fid} --veredicto approved",
                faltan.len(),
                faltan.join(", ")
            )),
        });
    }
    if veredicto != "approved" {
        return Err(Exit {
            code: 2,
            message: Some(format!(
                "[GATE] El veredicto de {rel} es `{veredicto}`, no `approved`.\n    \
                 Un cierre `done` exige un review aprobado. Si el trabajo quedo trabado,\n    \
                 cerra con --status blocked; si lo absorbio otra feature, con\n    \
                 --status superseded --absorbida-por <id>."
            )),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;

    #[test]
    fn estados_de_verify_should_read_the_table() {
        let texto = "# Verificacion\n\n| AC | Estado | Comando |\n| --- | --- | --- |\n| AC-1 | verde | `x` |\n| AC-2 | rojo | `y` |\n| AC-3 | vacio | `z` |\n";
        let estados = estados_de_verify(texto);
        assert_eq!(
            estados,
            vec![
                ("AC-1".to_string(), "verde".to_string()),
                ("AC-2".to_string(), "rojo".to_string()),
                ("AC-3".to_string(), "vacio".to_string()),
            ]
        );
    }

    #[test]
    fn filas_de_evidencia_should_keep_only_ac_rows() {
        let texto = "# Evidencia\n\n| AC | Estado | Evidencia |\n| --- | --- | --- |\n| AC-1 | OK | test x |\n\ntexto suelto\n| AC-2 | OK | test y |\n";
        let filas = filas_de_evidencia(texto);
        assert_eq!(filas.len(), 2);
        assert!(filas[0].contains("AC-1"));
        assert!(filas[1].contains("AC-2"));
    }

    #[test]
    fn uncommitted_work_should_be_visible_in_the_package() {
        // El reviewer revisa ANTES del cierre: si el paquete solo mirara
        // `base...rama`, el trabajo sin commitear del worktree seria invisible
        // y el paquete diria "archivos tocados: ninguno". Lo encontro la
        // verificacion en vivo de la propia feature #51.
        let cmd = |dir: &std::path::Path, args: &[&str]| {
            std::process::Command::new("git")
                .args(args)
                .current_dir(dir)
                .output()
                .unwrap();
        };
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path();
        cmd(p, &["init", "-q", "-b", "main"]);
        cmd(p, &["config", "user.email", "t@e.cl"]);
        cmd(p, &["config", "user.name", "T"]);
        std::fs::write(p.join("a.txt"), "uno\n").unwrap();
        cmd(p, &["add", "-A"]);
        cmd(p, &["commit", "-q", "-m", "init"]);
        let wt = p.join("wt");
        cmd(p, &["worktree", "add", "-b", "feature/1-x", &wt.to_string_lossy(), "main"]);
        // Trabajo SIN commitear dentro del worktree.
        std::fs::write(wt.join("b.txt"), "nuevo\n").unwrap();
        cmd(&wt, &["add", "-A"]);

        let archivos = git_lineas(&wt, &["diff", "--name-only", "main"]);
        assert!(
            archivos.iter().any(|a| a == "b.txt"),
            "el trabajo sin commitear tiene que verse: {archivos:?}"
        );
    }

    #[test]
    fn recortar_should_declare_what_was_left_out() {
        // AC-12: el recorte NUNCA es silencioso.
        let diff = (1..=10).map(|i| format!("linea {i}")).collect::<Vec<_>>().join("\n");
        let (texto, recorte) = recortar(&diff, 4);
        assert_eq!(texto.lines().count(), 4);
        let Some(r) = recorte else {
            panic!("un diff mas largo que el presupuesto tiene que declarar el recorte");
        };
        assert_eq!(r.lineas_mostradas, 4);
        assert_eq!(r.lineas_totales, 10);

        // Y si entra, no hay recorte ni perdida.
        let (completo, sin_recorte) = recortar(&diff, 50);
        assert_eq!(completo, diff);
        assert!(sin_recorte.is_none());
    }

    fn paquete_demo() -> Paquete {
        Paquete {
            feature_id: "51".to_string(),
            nombre: "demo".to_string(),
            acs: vec![
                Ac {
                    id: "AC-1".to_string(),
                    texto: "Given algo, When otra cosa, Then resultado.".to_string(),
                    estado: Some("verde".to_string()),
                },
                Ac {
                    id: "AC-2".to_string(),
                    texto: "Given otro, When mas, Then final.".to_string(),
                    estado: None,
                },
            ],
            evidencia: vec!["| AC-1 | OK | test x |".to_string()],
            archivos: vec!["rust/src/revision.rs".to_string()],
            diff: "+linea nueva".to_string(),
            recorte: Some(Recorte {
                lineas_mostradas: 1,
                lineas_totales: 900,
            }),
            protegidas: vec!["docs/prd/PRD-master.md".to_string()],
            faltantes: vec!["el reporte de verify".to_string()],
        }
    }

    #[test]
    fn render_should_show_state_missing_pieces_and_the_cut() {
        let texto = paquete_demo().render_texto();
        // AC-11: las cinco piezas.
        assert!(texto.contains("AC-1"), "los AC");
        assert!(texto.contains("[verde]"), "el estado del verify");
        assert!(texto.contains("sin verificar"), "el AC sin reporte se marca");
        assert!(texto.contains("| AC-1 | OK | test x |"), "la evidencia");
        assert!(texto.contains("rust/src/revision.rs"), "los archivos");
        assert!(texto.contains("+linea nueva"), "el diff");
        // Rutas protegidas: se nombran y se explica que significan.
        assert!(texto.contains("RUTAS PROTEGIDAS TOCADAS"));
        assert!(texto.contains("docs/prd/PRD-master.md"));
        // AC-12: el recorte se declara con numeros.
        assert!(texto.contains("se muestran 1 de 900"));
        // AC-13: lo que falta se nombra.
        assert!(texto.contains("el reporte de verify"));
    }

    #[test]
    fn tamano_should_report_the_cost_before_spending_it() {
        // AC-12b: el paquete dice cuanto cuesta leerlo.
        let (lineas, tokens) = paquete_demo().tamano();
        assert!(lineas > 10, "cuenta lineas reales: {lineas}");
        assert!(tokens > 0 && tokens < 5_000, "estimacion razonable: {tokens}");
    }

    #[test]
    fn json_should_carry_the_same_information() {
        // AC-14: sin parsear texto.
        let j = paquete_demo().render_json();
        assert_eq!(j["feature"], "51");
        assert_eq!(j["acs"][0]["estado"], "verde");
        assert!(j["acs"][1]["estado"].is_null());
        assert_eq!(j["recorte"]["lineas_totales"], 900);
        assert_eq!(j["protegidas"][0], "docs/prd/PRD-master.md");
        assert!(j["tamano"]["tokens_estimados"].as_u64().unwrap_or(0) > 0);
    }

    // ---- El veredicto del reviewer (feature #64) ----------------------------

    #[test]
    fn require_review_default_false() {
        // AC-6: una instalacion vieja sin la clave no se rompe.
        assert!(!require_review(&json!({})));
        assert!(!require_review(&json!({"rules": {}})));
        assert!(!require_review(&json!({"rules": {"require_review": false}})));
        assert!(require_review(&json!({"rules": {"require_review": true}})));
    }

    #[test]
    fn gate_review_ignora_prosa() {
        // AC-2: la prosa no cuenta, por mas que diga approved.
        assert_eq!(veredicto_estampado("Veredicto: approved"), None);
        assert_eq!(veredicto_estampado("**Veredicto: APROBADO para cierre.**"), None);
        // El falso positivo real de docs/review-3.md:3.
        assert_eq!(
            veredicto_estampado("Veredicto: approved (implementacion) - cierre BLOQUEADO"),
            None
        );
        // Solo el sello del binario cuenta.
        let sello = linea_sello("approved", "2026-08-28T00:00:00Z");
        assert_eq!(veredicto_estampado(&sello), Some("approved".into()));
    }

    #[test]
    fn gate_review_solo_approved() {
        // AC-5: el gate distingue los tres veredictos.
        for v in ["approved", "changes_requested", "blocked"] {
            let t = linea_sello(v, "2026-08-28T00:00:00Z");
            assert_eq!(veredicto_estampado(&t), Some(v.to_string()), "veredicto {v}");
        }
        // Un veredicto inventado no se acepta ni aunque venga con el prefijo.
        assert_eq!(veredicto_estampado("Revisado: aprobadisimo · 2026"), None);
    }

    /// Repo de mentira con dos archivos reales, para que las citas RESUELVAN.
    fn repo_de_prueba() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("close.rs"), "a\n".repeat(200)).unwrap();
        std::fs::create_dir_all(dir.path().join("src")).unwrap();
        std::fs::write(dir.path().join("src/revision.rs"), "b\n".repeat(500)).unwrap();
        dir
    }

    #[test]
    fn veredicto_exige_cobertura_de_ac() {
        // AC-3: la lista sale del SPEC, no del review.
        let dir = repo_de_prueba();
        let root = dir.path();
        let spec = "- AC-1: Given a, When b, Then c.\n- AC-2: Given d, When e, Then f.\n";

        // Review vacio: faltan los dos (si saliera del review, estaria "completo").
        assert_eq!(acs_sin_fila(&[root], spec, ""), vec!["AC-1", "AC-2"]);

        // Fila que nombra el AC pero NO cita: no responde.
        let sin_cita = "| AC-1 | cubierto, anda bien |\n| AC-2 | idem |\n";
        assert_eq!(acs_sin_fila(&[root], spec, sin_cita), vec!["AC-1", "AC-2"]);

        // Fila con cita que RESUELVE: responde.
        let con_cita = "| AC-1 | close.rs:101 | cubierto |\n| AC-2 | src/revision.rs:469 | cubierto |\n";
        assert!(acs_sin_fila(&[root], spec, con_cita).is_empty());

        // Cobertura parcial: nombra solo el que falta.
        let parcial = "| AC-1 | close.rs:101 | cubierto |\n";
        assert_eq!(acs_sin_fila(&[root], spec, parcial), vec!["AC-2"]);
    }

    #[test]
    fn la_cita_necesita_archivo_y_linea() {
        let dir = repo_de_prueba();
        let root = dir.path();
        // El corte es `algo:N` que RESUELVE: ni el numero suelto ni el archivo solo.
        assert!(fila_responde(&[root], "| AC-1 | close.rs:101 |", "AC-1"));
        assert!(!fila_responde(&[root], "| AC-1 | close.rs |", "AC-1"));
        assert!(!fila_responde(&[root], "| AC-1 | linea 101 |", "AC-1"));
        // Y no confunde un AC con otro.
        assert!(!fila_responde(&[root], "| AC-11 | close.rs:101 |", "AC-1"));
    }

    #[test]
    fn las_citas_resuelven_contra_la_raiz_de_la_FEATURE_primero() {
        // El bug que aparecio la primera vez que la feature se uso de verdad:
        // el review de la #64 citaba `rust/src/revision.rs:602`, que existe en
        // el worktree (927 lineas) y no en el checkout principal (507). El gate
        // resolvia contra el principal y rechazaba seis AC con citas correctas.
        let feature_root = Path::new("/tmp/wt/64-algo");
        let plans = feature_root.join("docs");
        let principal = Path::new("/tmp/principal");
        let raices = raices_desde(&plans, principal, principal);
        assert_eq!(
            raices.first().copied(),
            Some(feature_root),
            "la raiz de la feature tiene que ir PRIMERO"
        );
        // Y no se duplica cuando root == repo_root.
        assert_eq!(raices.len(), 2);
    }

    #[test]
    fn la_cita_tiene_que_apuntar_a_algo_que_existe() {
        // Lo que encontro el reviewer de la #64: el gate comprobaba la FORMA de
        // la cita, no que apuntara a algo.
        let dir = repo_de_prueba();
        let root = dir.path();
        // Archivo inexistente.
        assert!(!fila_responde(&[root], "| AC-1 | inventado.rs:99 |", "AC-1"));
        // Archivo real, linea que no existe (close.rs tiene 200).
        assert!(!fila_responde(&[root], "| AC-1 | close.rs:99999 |", "AC-1"));
        assert!(fila_responde(&[root], "| AC-1 | close.rs:200 |", "AC-1"));
        // El falso positivo de un numero de version.
        assert!(!fila_responde(&[root], "| AC-1 | version 3.14:15 |", "AC-1"));
        // Y no se sale del repo.
        assert!(!fila_responde(&[root], "| AC-1 | ../../etc/passwd:1 |", "AC-1"));
    }


    // ---------------------------------------------------------------------
    // Feature #67: las tres respuestas de una cita, el off-by-one del EOF y el
    // sello que se encontraba a medias.
    // ---------------------------------------------------------------------

    /// Un archivo de `lineas` lineas, mas relleno hasta pasarse del tope.
    fn archivo_gordo(dir: &std::path::Path, nombre: &str) -> usize {
        // Una sola linea larguisima: el tope se agota ANTES de ver un solo
        // salto, que es el caso donde la respuesta vieja ("la linea no existe")
        // era mas falsa. Son 12 MB contra un tope de 8 MB.
        let mut texto = "x".repeat(12 * 1024 * 1024);
        texto.push('\n');
        texto.push_str("la linea 2 existe de verdad\n");
        std::fs::write(dir.join(nombre), &texto).unwrap();
        2
    }

    #[test]
    fn cita_grande_no_se_pudo_comprobar() {
        // AC-6: la tercera respuesta. Antes esto devolvia "no resuelve" —o sea,
        // "la linea no existe"— sobre una cita CORRECTA cuya linea cae mas alla
        // del tope. La linea existe y `sed` la muestra. Es el patron 127-vs-124
        // de `docs/lecciones/criterios-de-cierre-que-se-pueden-fallar.md`:
        // traducir "no pude comprobar" a "no".
        let dir = tempfile::tempdir().unwrap();
        let raiz = dir.path();
        let existe = archivo_gordo(raiz, "gordo.txt");
        assert_eq!(
            evaluar_cita(&[raiz], "gordo.txt", existe),
            Cita::NoSePudoComprobar,
            "una cita que no se alcanzo a leer no puede reportarse como inexistente"
        );
        // Y la linea 1 SI se puede comprobar sin agotar el tope: el tope no es
        // una excusa para no mirar.
        assert_eq!(evaluar_cita(&[raiz], "gordo.txt", 1), Cita::Resuelve);
        // Un archivo chico sigue dando las dos respuestas de siempre.
        std::fs::write(raiz.join("chico.txt"), "uno\ndos\n").unwrap();
        assert_eq!(evaluar_cita(&[raiz], "chico.txt", 2), Cita::Resuelve);
        assert_eq!(evaluar_cita(&[raiz], "chico.txt", 9), Cita::NoResuelve);
    }

    #[test]
    fn cita_grande_no_cuelga_el_cierre() {
        // AC-7: el cierre DECIDE. No cuelga (el tope se conserva: sacarlo cuesta
        // 10,5 s por 2 GB dentro de un gate sin timeout) y no muere.
        //
        // Y decide en la direccion honesta: una cita que no se pudo comprobar NO
        // cuenta como cobertura. La alternativa —darla por buena— dejaria pasar
        // un review citando un archivo enorme cualquiera.
        let dir = tempfile::tempdir().unwrap();
        let raiz = dir.path();
        let existe = archivo_gordo(raiz, "gordo.txt");
        let spec = "- AC-1: Given algo, When pasa, Then otra.\n";
        let review = format!("| AC-1 | gordo.txt:{existe} | cubierto |\n");

        let t0 = std::time::Instant::now();
        let faltan = acs_sin_fila(&[raiz], spec, &review);
        let ms = t0.elapsed().as_millis();

        assert_eq!(
            faltan,
            vec!["AC-1".to_string()],
            "una cita sin comprobar no puede contar como cobertura"
        );
        assert!(ms < 10_000, "el gate tardo {ms} ms: el tope no esta cortando");
    }

    #[test]
    fn la_cita_no_acepta_la_linea_siguiente_al_eof() {
        // AC-8: un archivo de N lineas terminado en salto tiene N saltos, no
        // N+1. Contar `saltos + 1` hacia que la cita a la linea N+1 resolviera
        // en CUALQUIER archivo normal —reproducido con `evidencia.txt:4` en un
        // archivo de 3 lineas—, o sea que el gate aceptaba como evidencia una
        // linea que no existe.
        let dir = tempfile::tempdir().unwrap();
        let raiz = dir.path();

        std::fs::write(raiz.join("con_salto.txt"), "uno\ndos\ntres\n").unwrap();
        assert_eq!(evaluar_cita(&[raiz], "con_salto.txt", 3), Cita::Resuelve);
        assert_eq!(
            evaluar_cita(&[raiz], "con_salto.txt", 4),
            Cita::NoResuelve,
            "la linea 4 de un archivo de 3 lineas no existe"
        );

        // Sin salto final, la ultima linea SI cuenta: son 3 lineas igual.
        std::fs::write(raiz.join("sin_salto.txt"), "uno\ndos\ntres").unwrap();
        assert_eq!(evaluar_cita(&[raiz], "sin_salto.txt", 3), Cita::Resuelve);
        assert_eq!(evaluar_cita(&[raiz], "sin_salto.txt", 4), Cita::NoResuelve);

        // Un archivo vacio no tiene linea 1.
        std::fs::write(raiz.join("vacio.txt"), "").unwrap();
        assert_eq!(evaluar_cita(&[raiz], "vacio.txt", 1), Cita::NoResuelve);
        // Y uno de una sola linea sin salto, si.
        std::fs::write(raiz.join("una.txt"), "sola").unwrap();
        assert_eq!(evaluar_cita(&[raiz], "una.txt", 1), Cita::Resuelve);
        assert_eq!(evaluar_cita(&[raiz], "una.txt", 2), Cita::NoResuelve);
    }

    #[test]
    fn el_sello_se_encuentra_aunque_haya_lineas_peladas() {
        // AC-9: el `?` salia de la FUNCION ENTERA en la primera linea `Revisado:`
        // sin valor detras, asi que el gate decia "no lleva el sello del arnes"
        // con el sello tres lineas mas abajo. Un mensaje de gate que el archivo
        // desmiente es justo lo que la #63 vino a cerrar.
        let sello = linea_sello("approved", "2026-08-30 12:00");
        let texto = format!("# Review\nRevisado:\nprosa\n{sello}\n");
        assert_eq!(
            veredicto_estampado(&texto).as_deref(),
            Some("approved"),
            "el sello esta en el archivo y el gate no lo vio"
        );

        // Variantes de linea pelada que tampoco pueden abortar el barrido.
        for pelada in ["Revisado:", "Revisado:   ", "Revisado: · ·", "Revisado: fulano"] {
            let texto = format!("# Review\n{pelada}\n{sello}\n");
            assert_eq!(
                veredicto_estampado(&texto).as_deref(),
                Some("approved"),
                "aborto el barrido en {pelada:?}"
            );
        }

        // Y sigue sin inventar: sin sello real, no hay veredicto.
        assert_eq!(veredicto_estampado("# Review\nRevisado:\nprosa\n"), None);
        assert_eq!(veredicto_estampado("# Review\nVeredicto: approved\n"), None);
    }
}
