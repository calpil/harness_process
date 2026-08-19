//! PRDs anidados: el arbol de producto que vive en `docs/prd/`.
//!
//! La identidad de un PRD es su **cadena de segmentos** (`cobranza/mora`), y de
//! ella salen las dos rutas, sin registro intermedio: la carpeta lleva el
//! segmento propio y el archivo la cadena completa.
//!
//! ```text
//! docs/prd/PRD-master.md                       ""              (la raiz)
//! docs/prd/cobranza/PRD-cobranza.md            "cobranza"
//! docs/prd/cobranza/mora/PRD-cobranza-mora.md  "cobranza/mora"
//! ```
//!
//! El FILESYSTEM es la fuente de verdad; el `Padre:` del encabezado es una
//! declaracion que `harness_check.sh` contrasta contra la ubicacion real.

use std::path::{Path, PathBuf};

use serde_json::{Map, Value};

use crate::exit::Exit;
use crate::features::{features_slice, write_text_atomic};
use crate::paths::HarnessPaths;
use crate::plan::slugify;

/// Referencia canonica del PRD maestro (raiz del arbol).
pub const MASTER: &str = "master";

/// Un PRD del arbol, tal como esta en disco.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Prd {
    /// Cadena canonica de segmentos: "" para el maestro, "cobranza/mora" si no.
    pub slug: String,
    /// Ruta absoluta al archivo `PRD-*.md`.
    pub file: PathBuf,
}

impl Prd {
    /// Segmentos de la cadena ([] para el maestro).
    pub fn segments(&self) -> Vec<&str> {
        segments(&self.slug)
    }

    /// Cadena canonica del padre: `None` para el maestro, `Some("")` para un
    /// hijo directo del maestro.
    pub fn parent_slug(&self) -> Option<String> {
        let segs = self.segments();
        match segs.split_last() {
            None => None,
            Some((_, head)) => Some(head.join("/")),
        }
    }

    /// Etiqueta legible: el nombre del archivo sin extension (`PRD-cobranza-mora`).
    pub fn label(&self) -> String {
        self.file
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| "PRD".to_string())
    }

    /// Como se escribe esta referencia en la linea de comandos.
    pub fn reference(&self) -> String {
        if self.slug.is_empty() {
            MASTER.to_string()
        } else {
            self.slug.clone()
        }
    }
}

/// `docs/prd/` de la RAIZ.
pub fn prd_dir(paths: &HarnessPaths) -> PathBuf {
    paths.plans.join("prd")
}

/// Segmentos de una cadena canonica ("" -> []).
pub fn segments(slug: &str) -> Vec<&str> {
    slug.split('/').filter(|s| !s.is_empty()).collect()
}

/// Carpeta de un PRD: `docs/prd/` + un nivel por segmento.
pub fn dir_for(paths: &HarnessPaths, segs: &[&str]) -> PathBuf {
    let mut dir = prd_dir(paths);
    for seg in segs {
        dir.push(seg);
    }
    dir
}

/// Nombre de archivo: `PRD-` + la cadena unida por `-` (`PRD-master.md` en la
/// raiz). Es unico en todo el repo, asi que se puede grepear sin ambiguedad.
pub fn file_name_for(segs: &[&str]) -> String {
    if segs.is_empty() {
        return format!("PRD-{MASTER}.md");
    }
    format!("PRD-{}.md", segs.join("-"))
}

/// Ruta absoluta del PRD de una cadena de segmentos.
pub fn file_for(paths: &HarnessPaths, segs: &[&str]) -> PathBuf {
    dir_for(paths, segs).join(file_name_for(segs))
}

/// Ruta del PRD relativa a la RAIZ (`docs/prd/cobranza/PRD-cobranza.md`), con
/// separador `/` en todas las plataformas: es texto para documentos, no una
/// ruta del sistema.
pub fn rel_path(slug: &str) -> String {
    let segs = segments(slug);
    let mut parts = vec!["docs".to_string(), "prd".to_string()];
    parts.extend(segs.iter().map(|s| (*s).to_string()));
    parts.push(file_name_for(&segs));
    parts.join("/")
}

/// Normaliza el nombre que escribio el usuario a UN segmento de la cadena.
/// Reusa la `slugify` de planes y specs, asi que `cobranza_mora` -> `cobranza-mora`
/// y cualquier `../` o separador se disuelve antes de tocar el filesystem.
pub fn normalize_segment(name: &str) -> Result<String, Exit> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(Exit::msg("El nombre del PRD no puede estar vacio."));
    }
    let slug = slugify(trimmed);
    // slugify() devuelve "feature" ante una entrada sin alfanumericos: ahi no
    // hay nombre que valga, y nombrar un PRD "feature" seria peor que fallar.
    if slug == "feature" && !trimmed.to_lowercase().contains("feature") {
        return Err(Exit::msg(format!(
            "Nombre de PRD invalido: '{name}' no deja ninguna letra ni numero utilizable."
        )));
    }
    Ok(slug)
}

/// Resuelve la referencia del padre (`master`, "" o una cadena) a su cadena
/// canonica.
pub fn normalize_parent(parent: Option<&str>) -> String {
    match parent {
        None => String::new(),
        Some(p) => {
            let p = p.trim().trim_matches('/');
            if p.is_empty() || p.eq_ignore_ascii_case(MASTER) {
                String::new()
            } else {
                p.to_string()
            }
        }
    }
}

/// Recorre `docs/prd/` y devuelve los PRDs bien formados (los que estan donde
/// dice su cadena), ordenados por cadena. Un arbol inexistente devuelve vacio:
/// no tener PRDs no es un error.
pub fn scan(paths: &HarnessPaths) -> Vec<Prd> {
    let root = prd_dir(paths);
    let mut found = Vec::new();
    let master = root.join(file_name_for(&[]));
    if master.is_file() {
        found.push(Prd {
            slug: String::new(),
            file: master,
        });
    }
    walk(&root, &mut Vec::new(), &mut found);
    found.sort_by(|a, b| a.slug.cmp(&b.slug));
    found
}

fn walk(dir: &Path, segs: &mut Vec<String>, out: &mut Vec<Prd>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let mut subdirs: Vec<PathBuf> = entries
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    subdirs.sort();
    for sub in subdirs {
        let Some(name) = sub.file_name().map(|n| n.to_string_lossy().into_owned()) else {
            continue;
        };
        segs.push(name);
        let chain: Vec<&str> = segs.iter().map(String::as_str).collect();
        let file = sub.join(file_name_for(&chain));
        if file.is_file() {
            out.push(Prd {
                slug: chain.join("/"),
                file,
            });
        }
        walk(&sub, segs, out);
        segs.pop();
    }
}

/// Resuelve la referencia que escribio el usuario: cadena completa
/// (`cobranza/mora`), `master`, o el ultimo segmento si es UNICO en el arbol
/// (`mora`). Ambigua o inexistente -> error que lista los candidatos.
pub fn resolve(paths: &HarnessPaths, reference: &str) -> Result<Prd, Exit> {
    let tree = scan(paths);
    let want = normalize_parent(Some(reference));
    if let Some(hit) = tree.iter().find(|p| p.slug == want) {
        return Ok(hit.clone());
    }
    // Por ultimo segmento, solo si no hay dos ramas que lo compartan.
    let by_tail: Vec<&Prd> = tree
        .iter()
        .filter(|p| p.segments().last() == Some(&want.as_str()))
        .collect();
    match by_tail.as_slice() {
        [single] => Ok((*single).clone()),
        [] => Err(Exit::msg(format!(
            "PRD no encontrado: '{reference}'.{}",
            available(&tree)
        ))),
        many => Err(Exit::msg(format!(
            "Referencia ambigua: '{reference}' existe en {} ramas ({}). Usa la ruta completa.",
            many.len(),
            many.iter()
                .map(|p| p.reference())
                .collect::<Vec<_>>()
                .join(", ")
        ))),
    }
}

fn available(tree: &[Prd]) -> String {
    if tree.is_empty() {
        return " No hay ningun PRD todavia: empeza por docs/prd/PRD-master.md.".to_string();
    }
    format!(
        " PRDs disponibles: {}.",
        tree.iter()
            .map(|p| p.reference())
            .collect::<Vec<_>>()
            .join(", ")
    )
}

// ---------------------------------------------------------------------------
// Plantilla del PRD hijo
// ---------------------------------------------------------------------------

/// PRD hijo con las MISMAS 12 secciones del metodo que `PRD-master.md`, mas el
/// `Padre:` que lo cuelga del arbol. Es un documento del USUARIO desde el
/// momento en que se crea: nadie lo vuelve a escribir.
pub fn child_template(slug: &str, parent_slug: &str) -> String {
    let segs = segments(slug);
    let parent_ref = if parent_slug.is_empty() {
        MASTER.to_string()
    } else {
        parent_slug.to_string()
    };
    let parent_rel = rel_path(parent_slug);
    let depth = segs.len();
    // Los punteros a docs/ suben tantos niveles como profundidad tenga el PRD.
    let up = "../".repeat(depth + 1);
    format!(
        r#"# PRD - <nombre de esta parte>

Estado: Borrador
Padre: {parent_ref}
Duenno: <quien responde por esta parte>
Ultima actualizacion: <YYYY-MM-DD>
Alcance: <en una linea: que abarca esta parte y que NO toca>
Como se escribe: {up}prd/COMO-ESCRIBIR-UN-PRD.md
PRD padre: {parent_rel}
Diseno tecnico: {up}prd/SDD-master.md
Constitution: {up}constitution.md

> PRD anidado (`{slug}`): una parte del producto, con su propia historia. Es un
> documento del USUARIO: el arnes lo creo una vez y solo vuelve a tocarlo para
> marcar un hito cerrado y dejar bitacora. Todo lo demas lo escribis vos.
>
> Si esta parte sigue siendo demasiado grande para una sola historia, partila:
> `sh harness_cli prd add --name <parte> --parent {parent_ref_child}`

---

**LA REGLA DURA: SIN CODIGO. SOLO PSEUDO-CODIGO.** Este documento fija la
**estructura** — la historia, que entidades se tocan y como cambian — en
pseudo-codigo y explicaciones. Nunca lleva codigo final, la implementacion
exacta, pantallas terminadas ni configuracion.

---

## 1. Resumen (hoy -> despues)

<El dibujo mas barato que existe: dos lineas.>

- **Hoy:** <que pasa hoy, y que no pasa>
- **Despues:** <que pasa cuando esta parte exista>

## 2. La historia

<El corazon del documento: una persona con nombre, un momento concreto, sin
tecnicismos. Si la historia no convence, el resto no importa.>

**ANTES**

<que le pasa hoy a esa persona, y por que duele>

**DESPUES**

<que vive esa misma persona cuando esta parte exista>

## 3. Objetivos / No-objetivos

| ID | Objetivo | Como se ve cumplido |
| --- | --- | --- |
| O1 | <lo que tiene que lograr esta parte> | <senal observable> |

| ID | No-objetivo | Por que no |
| --- | --- | --- |
| NO1 | <lo que explicitamente NO se hace aca> | <razon> |

## 4. Usuarios y jobs-to-be-done

| Usuario | Que intenta lograr | Como lo resuelve hoy | Por que no alcanza |
| --- | --- | --- | --- |
| <rol> | <job> | <workaround actual> | <limitacion> |

## 5. Metricas de exito

| Metrica | Hoy | Objetivo | Mide | Como se mide |
| --- | --- | --- | --- | --- |
| <metrica> | <valor> | <valor> | <O1> | <log/dashboard> |

## 6. Como funciona hoy -> como va a funcionar

```
HOY                          DESPUES
<evento> -> (nada)           <evento> -> <lo que ahora ocurre>
                                  |__ <componente> -> <componente>
```

## 7. Los datos

| Que | Entidad / campo | Para que |
| --- | --- | --- |
| disparador | <el evento que arranca el flujo> | <que arranca> |
| interruptor | <flag por cliente/entorno> | <apagarlo en 1 clic> |
| candado | <campo que evita repetir> | <evitar repetir la accion> |

## 8. Pseudo-codigo (el acuerdo)

<La receta en palabras: que lo dispara, que lo frena y que promete. El detalle
vinculante de cada cambio vive en su `docs/spec-feature-<id>-<slug>.md`.>

```
CUANDO <ocurre el disparador>

  ¿<esta activado para este caso>?  -> si no, no hacemos nada
  ¿<ya lo hicimos antes>?           -> si si, no hacemos nada

  ENTONCES <que hacemos, en una frase>,
           con <la restriccion que lo hace aceptable>.
```

**Promesas:** <una sola vez por caso> · <limite temporal> · <que NO hace>.

## 9. Restricciones y supuestos

- Tecnicas: <stack obligado, sistemas con los que hay que integrar>
- Negocio / legales: <plazos, normativa, contratos>
- Supuestos: <lo que damos por cierto y habria que validar>

## 10. Hitos -> features

<Cada fila se carga al backlog con:
 sh harness_cli add --name <slug> --service <servicio> --acceptance "<criterio>" --prd {reference}
y al arrancarla (`start`) su spec nace citando este PRD. Al cerrarla
(`close --status done`) el arnes marca aca su Estado y deja bitacora.>

| # | Hito | Slug de feature | Objetivo que cumple | Criterio de aceptacion (resumen) | Estado |
| --- | --- | --- | --- | --- | --- |
| 1 | <hito> | <slug_snake_case> | <O1> | <que tiene que ser cierto> | pendiente |

## 11. Riesgos

| Riesgo | Impacto | Mitigacion |
| --- | --- | --- |
| <riesgo> | <alto/medio/bajo> | <que se hace al respecto> |

## 12. Decisiones abiertas

- <pregunta> — DECIDIDO (<usuario>, <fecha>): <respuesta>
- <pregunta> — ABIERTA
"#,
        parent_ref_child = if slug.is_empty() {
            MASTER.to_string()
        } else {
            slug.to_string()
        },
        reference = if slug.is_empty() {
            MASTER.to_string()
        } else {
            slug.to_string()
        },
    )
}

// ---------------------------------------------------------------------------
// Enlace en el PRD padre
// ---------------------------------------------------------------------------

/// Titulo de la seccion donde el padre lista a sus hijos.
pub const CHILDREN_SECTION: &str = "## PRDs anidados";

/// Engancha al hijo en la seccion `## PRDs anidados` del padre. Es un documento
/// del USUARIO: solo se agrega una fila (o la seccion entera al final si
/// falta), nunca se reordena ni se reescribe otra linea. Idempotente: si la
/// fila del hijo ya esta, no hace nada.
///
/// Devuelve `true` si escribio.
pub fn link_child(parent_file: &Path, parent_slug: &str, child_slug: &str) -> anyhow::Result<bool> {
    let text = std::fs::read_to_string(parent_file)?;
    let child_ref = if child_slug.is_empty() {
        MASTER.to_string()
    } else {
        child_slug.to_string()
    };
    // Ruta del hijo RELATIVA al padre: el link funciona al abrir el documento.
    let child_rel = relative_child_link(parent_slug, child_slug);
    if text
        .lines()
        .any(|l| l.starts_with('|') && cells(l).first().map(|c| c.as_str()) == Some(&child_ref))
    {
        return Ok(false);
    }
    let row = format!("| {child_ref} | [{child_rel}]({child_rel}) | <en una linea: que cuenta este PRD> |");
    let updated = match section_rows_end(&text) {
        Some(insert_at) => {
            let mut lines: Vec<String> = text.lines().map(str::to_string).collect();
            lines.insert(insert_at, row);
            let mut joined = lines.join("\n");
            if text.ends_with('\n') {
                joined.push('\n');
            }
            joined
        }
        None => {
            let mut joined = text.trim_end().to_string();
            joined.push_str("\n\n");
            joined.push_str(CHILDREN_SECTION);
            joined.push_str(
                "\n\n<Cada fila la agrega `sh harness_cli prd add --name <parte> --parent <ruta>`.\n\
                 Cada hijo cuenta su propia historia; este documento no carga con todo el peso.>\n\n\
                 | PRD | Archivo | Que cuenta |\n| --- | --- | --- |\n",
            );
            joined.push_str(&row);
            joined.push('\n');
            joined
        }
    };
    write_text_atomic(parent_file, &updated)?;
    Ok(true)
}

/// Ruta del hijo relativa a la carpeta del padre (`mora/PRD-cobranza-mora.md`).
fn relative_child_link(parent_slug: &str, child_slug: &str) -> String {
    let child_segs = segments(child_slug);
    let parent_len = segments(parent_slug).len();
    let mut parts: Vec<String> = child_segs
        .iter()
        .skip(parent_len)
        .map(|s| (*s).to_string())
        .collect();
    parts.push(file_name_for(&child_segs));
    parts.join("/")
}

/// Indice de la linea DESPUES de la ultima fila de la tabla de
/// `## PRDs anidados` (donde entra una fila nueva). `None` si no hay seccion.
fn section_rows_end(text: &str) -> Option<usize> {
    let lines: Vec<&str> = text.lines().collect();
    let start = lines
        .iter()
        .position(|l| l.trim_end() == CHILDREN_SECTION)?;
    let mut last_row = None;
    for (i, line) in lines.iter().enumerate().skip(start + 1) {
        if line.starts_with("## ") {
            break;
        }
        if line.starts_with('|') {
            last_row = Some(i);
        }
    }
    // Sin tabla todavia: la fila va al final de la seccion.
    Some(match last_row {
        Some(i) => i + 1,
        None => lines
            .iter()
            .enumerate()
            .skip(start + 1)
            .find(|(_, l)| l.starts_with("## "))
            .map_or(lines.len(), |(i, _)| i),
    })
}

/// Celdas de una fila Markdown, sin los pipes de los extremos.
pub fn cells(line: &str) -> Vec<String> {
    line.trim()
        .trim_start_matches('|')
        .trim_end_matches('|')
        .split('|')
        .map(|c| c.trim().to_string())
        .collect()
}

// ---------------------------------------------------------------------------
// Hitos
// ---------------------------------------------------------------------------

/// Titulo de la tabla de hitos (la que alimenta el backlog).
pub const MILESTONES_SECTION: &str = "## 10. Hitos -> features";

/// Filas de datos de la tabla de hitos: se ignoran el encabezado, el separador
/// y las filas que siguen siendo el ejemplo de la plantilla (`<...>`).
/// Devuelve (indice de linea, celdas).
pub fn milestone_rows(text: &str) -> Vec<(usize, Vec<String>)> {
    let lines: Vec<&str> = text.lines().collect();
    let Some(start) = lines
        .iter()
        .position(|l| l.trim_end().starts_with(MILESTONES_SECTION))
    else {
        return Vec::new();
    };
    let mut rows = Vec::new();
    for (i, line) in lines.iter().enumerate().skip(start + 1) {
        if line.starts_with("## ") {
            break;
        }
        if !line.starts_with('|') {
            continue;
        }
        let c = cells(line);
        if c.len() < 2 {
            continue;
        }
        let hito = c[1].as_str();
        // Encabezado, separador y placeholder de la plantilla no son hitos.
        if hito.eq_ignore_ascii_case("Hito") || hito.starts_with("---") || hito.is_empty() {
            continue;
        }
        if hito.starts_with('<') && hito.ends_with('>') {
            continue;
        }
        rows.push((i, c));
    }
    rows
}

/// Cuantos hitos declara este PRD.
pub fn milestone_count(file: &Path) -> usize {
    std::fs::read_to_string(file)
        .map(|t| milestone_rows(&t).len())
        .unwrap_or(0)
}

/// `Padre:` declarado en el encabezado (primeras lineas), si lo hay.
pub fn declared_parent(file: &Path) -> Option<String> {
    let text = std::fs::read_to_string(file).ok()?;
    text.lines()
        .take(15)
        .find_map(|l| l.strip_prefix("Padre:"))
        .map(|v| v.trim().to_string())
}

// ---------------------------------------------------------------------------
// Vuelta del cierre: marcar el hito + bitacora
// ---------------------------------------------------------------------------

/// Titulo de la bitacora que deja el cierre.
pub const LOG_SECTION: &str = "## Bitacora";

/// Resultado de escribir la vuelta del cierre en el PRD.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct CloseEcho {
    /// La fila del hito quedo marcada `done (fecha)`.
    pub milestone_marked: bool,
    /// Se agrego la linea de bitacora.
    pub logged: bool,
}

/// Marca el hito de la feature y deja bitacora en el PRD de origen. NUNCA
/// reescribe el cuerpo del documento. Idempotente: una feature ya registrada no
/// se vuelve a anotar.
pub fn echo_close(
    file: &Path,
    feature_id: &str,
    feature_name: &str,
    date: &str,
    spec_rel: &str,
    impl_rel: &str,
) -> anyhow::Result<CloseEcho> {
    let text = std::fs::read_to_string(file)?;
    let mut lines: Vec<String> = text.lines().map(str::to_string).collect();
    let mut echo = CloseEcho::default();

    // (a) La fila del hito cuyo slug de feature coincide.
    if let Some((idx, cells_)) = milestone_rows(&text)
        .into_iter()
        .find(|(_, c)| c.get(2).map(String::as_str) == Some(feature_name))
    {
        let mut updated = cells_;
        if let Some(last) = updated.last_mut() {
            // Ya marcado: la fecha del PRIMER cierre es la que vale. Re-cerrar
            // la misma feature no reescribe la historia del documento.
            if !last.starts_with("done") {
                *last = format!("done ({date})");
                lines[idx] = format!("| {} |", updated.join(" | "));
                echo.milestone_marked = true;
            }
        }
    }

    // (b) La bitacora, sin duplicar la entrada de esta feature.
    let entry_head = format!("- #{feature_id} {feature_name} -> done");
    let already = lines.iter().any(|l| l.trim_start().starts_with(&entry_head));
    if !already {
        let entry = format!("{entry_head} {date} · spec: {spec_rel} · impl: {impl_rel}");
        match lines.iter().position(|l| l.trim_end() == LOG_SECTION) {
            Some(start) => {
                let end = lines
                    .iter()
                    .enumerate()
                    .skip(start + 1)
                    .find(|(_, l)| l.starts_with("## "))
                    .map_or(lines.len(), |(i, _)| i);
                let insert_at = (start + 1..end)
                    .rev()
                    .find(|&i| !lines[i].trim().is_empty())
                    .map_or(end, |i| i + 1);
                lines.insert(insert_at, entry);
            }
            None => {
                while lines.last().is_some_and(|l| l.trim().is_empty()) {
                    lines.pop();
                }
                lines.push(String::new());
                lines.push(LOG_SECTION.to_string());
                lines.push(String::new());
                lines.push(
                    "<Lo que el arnes cerro contra este PRD. Si lo implementado difiere de lo que"
                        .to_string(),
                );
                lines.push(
                    " promete este documento, actualiza el documento: esa parte es tuya.>"
                        .to_string(),
                );
                lines.push(String::new());
                lines.push(entry);
            }
        }
        echo.logged = true;
    }

    if echo.milestone_marked || echo.logged {
        let mut joined = lines.join("\n");
        joined.push('\n');
        write_text_atomic(file, &joined)?;
    }
    Ok(echo)
}

// ---------------------------------------------------------------------------
// Arbol
// ---------------------------------------------------------------------------

/// Cadena canonica del PRD que declara una feature ("" = el maestro, que es
/// donde caen tambien las features sin `--prd`).
pub fn feature_prd_slug(feature: &Value) -> String {
    normalize_parent(feature.get("prd").and_then(Value::as_str))
}

/// Cuenta features del backlog por PRD: (cerradas como done, total).
fn feature_counts(data: &Value, slug: &str) -> (usize, usize) {
    let mut done = 0;
    let mut total = 0;
    for f in features_slice(data) {
        if feature_prd_slug(f) != slug {
            continue;
        }
        // Una feature `superseded` no cuenta NI arriba NI abajo (feature #37,
        // decision del usuario OBS-1): no es trabajo hecho —nunca tuvo spec ni
        // evidencia propia— ni pendiente, es una entrada que se plego en otra.
        // Contarla solo en el denominador hacia que el PRD pareciera menos
        // completo de lo que esta.
        if f.get("status").and_then(Value::as_str) == Some("superseded") {
            continue;
        }
        total += 1;
        if f.get("status").and_then(Value::as_str) == Some("done") {
            done += 1;
        }
    }
    (done, total)
}

/// Dibuja el arbol desde `root_slug` con sus hitos y el estado de sus features.
pub fn render_tree(paths: &HarnessPaths, data: &Value, root: &Prd) -> String {
    let tree = scan(paths);
    let mut rows: Vec<(String, String)> = Vec::new();
    collect_rows(data, &tree, root, String::new(), true, true, &mut rows);
    let width = rows.iter().map(|(l, _)| l.chars().count()).max().unwrap_or(0);
    let mut out = String::new();
    for (label, note) in rows {
        let pad = " ".repeat(width - label.chars().count());
        out.push_str(&format!("{label}{pad}  {note}\n"));
    }
    out
}

fn collect_rows(
    data: &Value,
    tree: &[Prd],
    node: &Prd,
    prefix: String,
    is_last: bool,
    is_root: bool,
    out: &mut Vec<(String, String)>,
) {
    let branch = if is_root {
        String::new()
    } else if is_last {
        format!("{prefix}`-- ")
    } else {
        format!("{prefix}|-- ")
    };
    out.push((format!("{branch}{}", node.label()), note_for(data, node)));

    let children: Vec<&Prd> = tree
        .iter()
        .filter(|p| p.parent_slug().as_deref() == Some(node.slug.as_str()))
        .collect();
    let child_prefix = if is_root {
        " ".to_string()
    } else if is_last {
        format!("{prefix}    ")
    } else {
        format!("{prefix}|   ")
    };
    for (i, child) in children.iter().enumerate() {
        collect_rows(
            data,
            tree,
            child,
            child_prefix.clone(),
            i + 1 == children.len(),
            false,
            out,
        );
    }
}

fn note_for(data: &Value, node: &Prd) -> String {
    let hitos = milestone_count(&node.file);
    let (done, total) = feature_counts(data, &node.slug);
    let mut note = if hitos == 0 {
        "[!] sin hitos".to_string()
    } else if hitos == 1 {
        "1 hito".to_string()
    } else {
        format!("{hitos} hitos")
    };
    if total > 0 {
        note.push_str(&format!(" | features: {done}/{total} done"));
    } else if hitos > 0 {
        note.push_str(" | sin features");
    }
    // El encabezado que miente sobre su ubicacion: el arbol real manda.
    let declared = declared_parent(&node.file);
    if let Some(declared) = declared {
        let expected = node
            .parent_slug()
            .map(|p| if p.is_empty() { MASTER.to_string() } else { p });
        if let Some(expected) = expected
            && declared != expected
        {
            note.push_str(&format!(
                " [!] declara Padre: {declared} (su lugar dice {expected})"
            ));
        }
    }
    note
}

/// Ruta relativa a la RAIZ del PRD que declara una feature; el maestro por
/// defecto (las features sin `--prd` cuentan para el producto entero).
pub fn feature_prd_rel(feature: &Map<String, Value>) -> String {
    let slug = normalize_parent(feature.get("prd").and_then(Value::as_str));
    rel_path(&slug)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;
    use serde_json::json;

    /// Mismo fixture que los tests de spec: el arnes vive en <dir>/hp con
    /// marker `subdir`, asi que la RAIZ (y su docs/) es <dir>. Sin tocar el
    /// entorno: `HARNESS_REPO_ROOT` es global al proceso de test y contaminaria
    /// a los demas modulos.
    fn paths_in(dir: &Path) -> HarnessPaths {
        let harness = dir.join("hp");
        std::fs::create_dir_all(&harness).unwrap();
        std::fs::write(harness.join(".harness_layout"), "subdir").unwrap();
        HarnessPaths::from_root(harness)
    }

    fn seed_master(paths: &HarnessPaths) {
        let dir = prd_dir(paths);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("PRD-master.md"),
            "# PRD Master - demo\n\nEstado: draft\n\n## 10. Hitos -> features\n\n\
             | # | Hito | Slug de feature | Objetivo | Criterio | Estado |\n\
             | --- | --- | --- | --- | --- | --- |\n\
             | 1 | <hito> | <slug_snake_case> | <O1> | <criterio> | pendiente |\n",
        )
        .unwrap();
    }

    #[test]
    fn paths_should_derive_from_the_segment_chain() {
        let dir = tempfile::tempdir().unwrap();
        let paths = paths_in(dir.path());
        assert_eq!(file_name_for(&[]), "PRD-master.md");
        assert_eq!(file_name_for(&["cobranza"]), "PRD-cobranza.md");
        assert_eq!(
            file_name_for(&["cobranza", "mora"]),
            "PRD-cobranza-mora.md"
        );
        assert_eq!(
            file_for(&paths, &["cobranza", "mora"]),
            prd_dir(&paths)
                .join("cobranza")
                .join("mora")
                .join("PRD-cobranza-mora.md")
        );
        assert_eq!(
            rel_path("cobranza/mora"),
            "docs/prd/cobranza/mora/PRD-cobranza-mora.md"
        );
        assert_eq!(rel_path(""), "docs/prd/PRD-master.md");
    }

    #[test]
    fn normalize_segment_should_slugify_and_reject_empty_names() {
        assert_eq!(normalize_segment("cobranza_mora").unwrap(), "cobranza-mora");
        assert_eq!(normalize_segment("  Mora Temprana ").unwrap(), "mora-temprana");
        // Un slug hostil se disuelve antes de tocar el filesystem.
        assert_eq!(normalize_segment("../../etc").unwrap(), "etc");
        assert!(normalize_segment("   ").is_err());
        assert!(normalize_segment("///").is_err());
    }

    #[test]
    fn scan_should_find_the_tree_and_ignore_misplaced_files() {
        let dir = tempfile::tempdir().unwrap();
        let paths = paths_in(dir.path());
        seed_master(&paths);
        let cob = prd_dir(&paths).join("cobranza");
        std::fs::create_dir_all(cob.join("mora")).unwrap();
        std::fs::write(cob.join("PRD-cobranza.md"), "x").unwrap();
        std::fs::write(cob.join("mora").join("PRD-cobranza-mora.md"), "x").unwrap();
        // Archivo mal ubicado: el nombre no corresponde a su cadena.
        std::fs::write(cob.join("PRD-otro.md"), "x").unwrap();
        let tree = scan(&paths);
        let slugs: Vec<&str> = tree.iter().map(|p| p.slug.as_str()).collect();
        assert_eq!(slugs, ["", "cobranza", "cobranza/mora"]);
    }

    #[test]
    fn resolve_should_accept_path_tail_and_reject_ambiguity() {
        let dir = tempfile::tempdir().unwrap();
        let paths = paths_in(dir.path());
        seed_master(&paths);
        for chain in [
            vec!["cobranza"],
            vec!["cobranza", "mora"],
            vec!["ventas"],
            vec!["ventas", "mora"],
        ] {
            std::fs::create_dir_all(dir_for(&paths, &chain)).unwrap();
            std::fs::write(file_for(&paths, &chain), "x").unwrap();
        }
        assert_eq!(resolve(&paths, "master").unwrap().slug, "");
        assert_eq!(resolve(&paths, "cobranza").unwrap().slug, "cobranza");
        assert_eq!(
            resolve(&paths, "cobranza/mora").unwrap().slug,
            "cobranza/mora"
        );
        let err = resolve(&paths, "mora").unwrap_err();
        assert!(err.message.unwrap().contains("ambigua"));
        let err = resolve(&paths, "inexistente").unwrap_err();
        let msg = err.message.unwrap();
        assert!(msg.contains("PRD no encontrado"));
        assert!(msg.contains("cobranza/mora"));
    }

    #[test]
    fn resolve_should_accept_a_unique_tail() {
        let dir = tempfile::tempdir().unwrap();
        let paths = paths_in(dir.path());
        seed_master(&paths);
        let chain = ["cobranza", "mora"];
        std::fs::create_dir_all(dir_for(&paths, &chain)).unwrap();
        std::fs::write(file_for(&paths, &chain), "x").unwrap();
        std::fs::create_dir_all(dir_for(&paths, &["cobranza"])).unwrap();
        std::fs::write(file_for(&paths, &["cobranza"]), "x").unwrap();
        assert_eq!(resolve(&paths, "mora").unwrap().slug, "cobranza/mora");
    }

    #[test]
    fn child_template_should_declare_parent_and_keep_the_twelve_sections() {
        let t = child_template("cobranza/mora", "cobranza");
        assert!(t.contains("Padre: cobranza"));
        assert!(t.contains("Estado: Borrador"));
        assert!(t.contains("PRD padre: docs/prd/cobranza/PRD-cobranza.md"));
        assert!(t.contains("--prd cobranza/mora"));
        let order = [
            "## 1. Resumen",
            "## 2. La historia",
            "## 3. Objetivos / No-objetivos",
            "## 4. Usuarios y jobs-to-be-done",
            "## 5. Metricas de exito",
            "## 6. Como funciona hoy -> como va a funcionar",
            "## 7. Los datos",
            "## 8. Pseudo-codigo (el acuerdo)",
            "## 9. Restricciones y supuestos",
            "## 10. Hitos -> features",
            "## 11. Riesgos",
            "## 12. Decisiones abiertas",
        ];
        let mut cursor = 0;
        for section in order {
            let at = t[cursor..]
                .find(section)
                .unwrap_or_else(|| panic!("falta {section} despues de la posicion {cursor}"));
            cursor += at + section.len();
        }
        // Hijo directo del maestro: Padre: master y punteros un nivel arriba.
        let root_child = child_template("cobranza", "");
        assert!(root_child.contains("Padre: master"));
        assert!(root_child.contains("PRD padre: docs/prd/PRD-master.md"));
    }

    #[test]
    fn link_child_should_create_the_section_then_append_without_duplicating() {
        let dir = tempfile::tempdir().unwrap();
        let parent = dir.path().join("PRD-master.md");
        std::fs::write(&parent, "# PRD Master\n\n## 12. Decisiones abiertas\n\n- ninguna\n").unwrap();

        assert!(link_child(&parent, "", "cobranza").unwrap());
        let text = std::fs::read_to_string(&parent).unwrap();
        assert!(text.contains(CHILDREN_SECTION));
        assert!(text.contains("| cobranza | [cobranza/PRD-cobranza.md](cobranza/PRD-cobranza.md) |"));
        // El cuerpo original quedo intacto y en su lugar.
        assert!(text.starts_with("# PRD Master\n\n## 12. Decisiones abiertas\n\n- ninguna\n"));

        assert!(link_child(&parent, "", "onboarding").unwrap());
        let text = std::fs::read_to_string(&parent).unwrap();
        assert_eq!(text.matches("## PRDs anidados").count(), 1);
        assert!(text.contains("| onboarding |"));

        // Idempotente.
        assert!(!link_child(&parent, "", "cobranza").unwrap());
        let again = std::fs::read_to_string(&parent).unwrap();
        assert_eq!(again.matches("| cobranza |").count(), 1);
    }

    #[test]
    fn link_child_should_use_a_path_relative_to_the_parent() {
        let dir = tempfile::tempdir().unwrap();
        let parent = dir.path().join("PRD-cobranza.md");
        std::fs::write(&parent, "# PRD - cobranza\n").unwrap();
        link_child(&parent, "cobranza", "cobranza/mora").unwrap();
        let text = std::fs::read_to_string(&parent).unwrap();
        assert!(text.contains("| cobranza/mora | [mora/PRD-cobranza-mora.md](mora/PRD-cobranza-mora.md) |"));
    }

    #[test]
    fn milestone_rows_should_ignore_header_separator_and_template_example() {
        let text = "## 10. Hitos -> features\n\n\
            | # | Hito | Slug de feature | Objetivo | Criterio | Estado |\n\
            | --- | --- | --- | --- | --- | --- |\n\
            | 1 | <hito> | <slug_snake_case> | <O1> | <criterio> | pendiente |\n\
            | 2 | Avisar la mora | avisar_mora | O1 | llega el aviso | pendiente |\n\
            \n## 11. Riesgos\n";
        let rows = milestone_rows(text);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].1[2], "avisar_mora");
    }

    #[test]
    fn echo_close_should_mark_the_milestone_and_log_once() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("PRD-cobranza.md");
        std::fs::write(
            &file,
            "# PRD - cobranza\n\n## 10. Hitos -> features\n\n\
             | # | Hito | Slug de feature | Objetivo | Criterio | Estado |\n\
             | --- | --- | --- | --- | --- | --- |\n\
             | 1 | Avisar la mora | avisar_mora | O1 | llega el aviso | pendiente |\n",
        )
        .unwrap();

        let echo = echo_close(
            &file,
            "13",
            "avisar_mora",
            "2026-08-12",
            "docs/spec-feature-13-avisar-mora.md",
            "docs/impl-13.md",
        )
        .unwrap();
        assert_eq!(
            echo,
            CloseEcho {
                milestone_marked: true,
                logged: true
            }
        );
        let text = std::fs::read_to_string(&file).unwrap();
        assert!(text.contains("| 1 | Avisar la mora | avisar_mora | O1 | llega el aviso | done (2026-08-12) |"));
        assert!(text.contains("## Bitacora"));
        assert!(text.contains(
            "- #13 avisar_mora -> done 2026-08-12 · spec: docs/spec-feature-13-avisar-mora.md · impl: docs/impl-13.md"
        ));

        // Idempotente: ni marca de nuevo ni duplica la bitacora.
        let again = echo_close(
            &file,
            "13",
            "avisar_mora",
            "2026-08-13",
            "docs/spec-feature-13-avisar-mora.md",
            "docs/impl-13.md",
        )
        .unwrap();
        assert_eq!(again, CloseEcho::default());
        let text2 = std::fs::read_to_string(&file).unwrap();
        assert_eq!(text2.matches("- #13 avisar_mora").count(), 1);
        assert!(text2.contains("done (2026-08-12)"));
    }

    #[test]
    fn echo_close_should_log_even_without_a_milestone_row() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("PRD-master.md");
        std::fs::write(&file, "# PRD Master\n\n## 12. Decisiones abiertas\n\n- ninguna\n").unwrap();
        let echo = echo_close(&file, "7", "otra_cosa", "2026-08-12", "s.md", "i.md").unwrap();
        assert_eq!(
            echo,
            CloseEcho {
                milestone_marked: false,
                logged: true
            }
        );
        let text = std::fs::read_to_string(&file).unwrap();
        assert!(text.contains("## Bitacora"));
        assert!(text.contains("- #7 otra_cosa -> done 2026-08-12"));
    }

    #[test]
    fn render_tree_should_draw_children_with_milestones_and_feature_state() {
        let dir = tempfile::tempdir().unwrap();
        let paths = paths_in(dir.path());
        seed_master(&paths);
        for (chain, parent) in [
            (vec!["cobranza"], ""),
            (vec!["cobranza", "mora"], "cobranza"),
        ] {
            std::fs::create_dir_all(dir_for(&paths, &chain)).unwrap();
            let slug = chain.join("/");
            std::fs::write(file_for(&paths, &chain), child_template(&slug, parent)).unwrap();
        }
        // Un hito real en el hijo (la plantilla trae solo el placeholder).
        let mora = file_for(&paths, &["cobranza", "mora"]);
        let text = std::fs::read_to_string(&mora)
            .unwrap()
            .replace(
                "| 1 | <hito> | <slug_snake_case> | <O1> | <que tiene que ser cierto> | pendiente |",
                "| 1 | Avisar la mora | avisar_mora | O1 | llega el aviso | pendiente |",
            );
        std::fs::write(&mora, text).unwrap();

        let data = json!({"features": [
            {"id": 1, "name": "avisar_mora", "prd": "cobranza/mora", "status": "done"},
            {"id": 2, "name": "otra", "prd": "cobranza/mora", "status": "pending"},
            {"id": 3, "name": "suelta", "status": "done"}
        ]});
        let master = resolve(&paths, MASTER).unwrap();
        let out = render_tree(&paths, &data, &master);
        let lines: Vec<&str> = out.lines().collect();
        assert!(lines[0].starts_with("PRD-master"));
        // La feature sin --prd cuenta para el maestro.
        assert!(lines[0].contains("features: 1/1 done"), "{out}");
        assert!(lines[1].starts_with(" `-- PRD-cobranza"), "{out}");
        assert!(lines[1].contains("[!] sin hitos"), "{out}");
        assert!(lines[2].starts_with("     `-- PRD-cobranza-mora"), "{out}");
        assert!(lines[2].contains("1 hito | features: 1/2 done"), "{out}");
    }

    #[test]
    fn render_tree_should_flag_a_header_that_lies_about_its_parent() {
        let dir = tempfile::tempdir().unwrap();
        let paths = paths_in(dir.path());
        seed_master(&paths);
        let chain = ["cobranza"];
        std::fs::create_dir_all(dir_for(&paths, &chain)).unwrap();
        // Declara un padre que no es su ubicacion real.
        std::fs::write(
            file_for(&paths, &chain),
            "# PRD - cobranza\n\nEstado: Borrador\nPadre: ventas\n",
        )
        .unwrap();
        let master = resolve(&paths, MASTER).unwrap();
        let out = render_tree(&paths, &json!({"features": []}), &master);
        assert!(out.contains("[!] declara Padre: ventas (su lugar dice master)"), "{out}");
    }
}

#[cfg(test)]
mod tests_superseded {
    #![allow(clippy::unwrap_used)]
    use super::*;
    use serde_json::json;

    #[test]
    fn prd_tree_should_ignore_superseded_features() {
        // Decision del usuario (OBS-1 de la #37): una feature absorbida no
        // cuenta ni arriba ni abajo. Contarla solo en el denominador hacia que
        // el PRD pareciera menos completo de lo que esta.
        let data = json!({"features": [
            {"id": 1, "name": "hecha", "status": "done"},
            {"id": 2, "name": "pendiente", "status": "pending"},
            {"id": 3, "name": "absorbida", "status": "superseded", "superseded_by": "1"},
            {"id": 4, "name": "trabada", "status": "blocked"}
        ]});
        // Sin la superseded: 1 done sobre 3 (done + pending + blocked).
        assert_eq!(feature_counts(&data, ""), (1, 3));
    }

    #[test]
    fn prd_tree_should_still_count_blocked_features() {
        // Regresion: `blocked` sigue contando en el total, como siempre.
        let data = json!({"features": [
            {"id": 1, "status": "done"},
            {"id": 2, "status": "blocked"}
        ]});
        assert_eq!(feature_counts(&data, ""), (1, 2));
    }
}
