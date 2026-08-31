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

/// El PRD tal como vive en la RAIZ del repo, sin importar desde donde se
/// invoque el comando ni si hay un worktree de por medio.
///
/// `file_for` resuelve contra `paths.plans`, que en un worktree apunta al
/// `docs/` de la feature: correcto para el spec, el plan y la evidencia, y
/// EQUIVOCADO para el PRD, que es un documento raiz y compartido por todas las
/// features (feature #60).
pub fn file_en_raiz(repo_root: &Path, slug: &str) -> PathBuf {
    let mut file = repo_root.to_path_buf();
    for parte in rel_path(slug).split('/') {
        file.push(parte);
    }
    file
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
    scan_dir(&prd_dir(paths))
}

/// Igual que `scan`, pero sobre un `docs/prd/` dado. Lo usa `prd doctor`, que
/// audita el arbol de la RAIZ y no el del worktree de turno (feature #60).
pub fn scan_dir(root: &Path) -> Vec<Prd> {
    let root = root.to_path_buf();
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

/// Un puntero candidato a entrar en la bitacora, YA resuelto contra el disco
/// por quien llama: `decidir_vuelta` no toca el filesystem. Esa division es lo
/// que sostiene la promesa "el arnes no escribe un puntero que no resuelve":
/// la sostiene la estructura, no acordarse de comprobarlo (leccion
/// `promesas-estructurales-vs-disciplina`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Candidato {
    /// Como se nombra en la linea (`spec`, `impl`).
    pub etiqueta: String,
    /// Ruta relativa a la RAIZ del repo, con `/` en todas las plataformas.
    pub rel: String,
    /// Si el archivo existe. Lo resuelve quien llama, contra la raiz.
    pub existe: bool,
}

impl Candidato {
    pub fn nuevo(etiqueta: &str, rel: &str, existe: bool) -> Self {
        Self {
            etiqueta: etiqueta.to_string(),
            rel: rel.trim().replace('\\', "/"),
            existe,
        }
    }
}

/// Un puntero que NO entro en la bitacora, con su motivo. Se dice en voz alta:
/// omitirlo en silencio seria el mismo bug con otra cara.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Descarte {
    pub etiqueta: String,
    pub rel: String,
    pub motivo: &'static str,
}

pub const MOTIVO_VACIO: &str = "no se pudo resolver la ruta";
pub const MOTIVO_ESCAPA: &str = "la ruta escapa de la raiz del repo";
pub const MOTIVO_AUSENTE: &str = "el archivo no existe";

/// Lo que la vuelta al PRD va a decir, ya decidido y validado.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlanDeVuelta {
    /// Cabeza de la entrada: el candado de idempotencia.
    pub cabeza: String,
    /// La linea completa de bitacora.
    pub linea: String,
    /// Slug con el que se busca la fila del hito (literal, sin adornos).
    pub slug_hito: String,
    pub fecha: String,
    /// Punteros que quedaron afuera y por que.
    pub descartes: Vec<Descarte>,
}

/// Decide QUE dice la vuelta al PRD.
///
/// Funcion PURA: no lee ni escribe disco, no consulta el entorno. Devuelve el
/// plan que `aplicar_vuelta` — la unica que toca el documento — ejecuta. Un
/// puntero que no resuelve no llega a la linea: queda en `descartes`.
pub fn decidir_vuelta(
    feature_id: &str,
    feature_name: &str,
    date: &str,
    candidatos: &[Candidato],
) -> PlanDeVuelta {
    let cabeza = format!("- #{feature_id} {feature_name} -> done");
    let mut linea = format!("{cabeza} {date}");
    let mut descartes = Vec::new();
    for candidato in candidatos {
        match motivo_de_descarte(candidato) {
            Some(motivo) => descartes.push(Descarte {
                etiqueta: candidato.etiqueta.clone(),
                rel: candidato.rel.clone(),
                motivo,
            }),
            None => linea.push_str(&format!(" {SEP} {}: {}", candidato.etiqueta, candidato.rel)),
        }
    }
    PlanDeVuelta {
        cabeza,
        linea,
        slug_hito: feature_name.to_string(),
        fecha: date.to_string(),
        descartes,
    }
}

/// Separador de punteros en la linea de bitacora (punto medio).
pub const SEP: &str = "\u{b7}";

/// Por que un puntero no entra. `None` = entra.
fn motivo_de_descarte(candidato: &Candidato) -> Option<&'static str> {
    let rel = candidato.rel.trim();
    if rel.is_empty() {
        return Some(MOTIVO_VACIO);
    }
    // Una ruta que sale de la raiz apunta a un arbol que el arnes no controla
    // — tipicamente el worktree que el propio cierre esta por borrar.
    if escapa_de_la_raiz(rel) {
        return Some(MOTIVO_ESCAPA);
    }
    if !candidato.existe {
        return Some(MOTIVO_AUSENTE);
    }
    None
}

/// True si la ruta no es relativa a la raiz del repo: absoluta, con raiz de
/// Windows, o con cualquier `..` en el camino.
pub fn escapa_de_la_raiz(rel: &str) -> bool {
    let rel = rel.replace('\\', "/");
    if rel.starts_with('/') || rel.starts_with("~") {
        return true;
    }
    // `C:/...` y compania.
    if rel.len() >= 2 && rel.as_bytes()[1] == b':' {
        return true;
    }
    rel.split('/').any(|seg| seg == "..")
}

/// Escribe el plan en el PRD. La UNICA funcion que toca el documento del
/// USUARIO: nunca reescribe su cuerpo, solo marca la celda de estado del hito y
/// apendea la bitacora. Idempotente: una feature ya registrada no se vuelve a
/// anotar y la fecha del PRIMER cierre no se reescribe.
pub fn aplicar_vuelta(file: &Path, plan: &PlanDeVuelta) -> anyhow::Result<CloseEcho> {
    let text = std::fs::read_to_string(file)?;
    let mut lines: Vec<String> = text.lines().map(str::to_string).collect();
    let mut echo = CloseEcho::default();

    // (a) La fila del hito cuyo slug de feature coincide.
    if let Some((idx, cells_)) = milestone_rows(&text)
        .into_iter()
        .find(|(_, c)| c.get(2).map(String::as_str) == Some(plan.slug_hito.as_str()))
    {
        let mut updated = cells_;
        if let Some(last) = updated.last_mut() {
            // Ya marcado: la fecha del PRIMER cierre es la que vale. Re-cerrar
            // la misma feature no reescribe la historia del documento.
            if !last.starts_with("done") {
                *last = format!("done ({})", plan.fecha);
                lines[idx] = format!("| {} |", updated.join(" | "));
                echo.milestone_marked = true;
            }
        }
    }

    // (b) La bitacora, sin duplicar la entrada de esta feature.
    let already = lines
        .iter()
        .any(|l| l.trim_start().starts_with(&plan.cabeza));
    if !already {
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
                lines.insert(insert_at, plan.linea.clone());
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
                lines.push(plan.linea.clone());
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
// Leer la bitacora: lo que ya quedo escrito
// ---------------------------------------------------------------------------

/// Una entrada de bitacora ya escrita en un PRD, con sus punteros separados.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EntradaBitacora {
    /// Indice de la linea dentro del archivo (0-based).
    pub idx: usize,
    pub feature_id: String,
    pub feature_name: String,
    pub fecha: String,
    /// `(etiqueta, ruta)` en el orden en que aparecen.
    pub punteros: Vec<(String, String)>,
}

impl EntradaBitacora {
    /// Reconstruye la linea con los punteros dados (los que sobrevivan).
    pub fn con_punteros(&self, punteros: &[(String, String)]) -> String {
        let mut linea = format!(
            "- #{} {} -> done {}",
            self.feature_id, self.feature_name, self.fecha
        );
        for (etiqueta, rel) in punteros {
            linea.push_str(&format!(" {SEP} {etiqueta}: {rel}"));
        }
        linea
    }
}

/// Todas las entradas de bitacora de un PRD. Una linea que no tenga la forma
/// `- #<id> <name> -> done <fecha>` no es una entrada: se ignora sin ruido (el
/// cuerpo del documento es del USUARIO y puede tener cualquier cosa).
pub fn bitacora_entries(text: &str) -> Vec<EntradaBitacora> {
    let mut out = Vec::new();
    for (idx, linea) in text.lines().enumerate() {
        let Some(cuerpo) = linea.trim_start().strip_prefix("- #") else {
            continue;
        };
        let Some((izq, der)) = cuerpo.split_once(" -> done ") else {
            continue;
        };
        let Some((feature_id, feature_name)) = izq.split_once(' ') else {
            continue;
        };
        let mut partes = der.split(SEP);
        let fecha = partes.next().unwrap_or_default().trim().to_string();
        let punteros = partes
            .filter_map(|p| p.trim().split_once(": "))
            .map(|(e, r)| (e.trim().to_string(), r.trim().to_string()))
            .collect();
        out.push(EntradaBitacora {
            idx,
            feature_id: feature_id.trim().to_string(),
            feature_name: feature_name.trim().to_string(),
            fecha,
            punteros,
        });
    }
    out
}

/// True si la fila del hito de `slug` esta marcada `done`. `None` = no hay fila
/// para ese slug (una feature sin hito declarado no es un hallazgo).
pub fn hito_marcado(text: &str, slug: &str) -> Option<bool> {
    milestone_rows(text)
        .into_iter()
        .find(|(_, c)| c.get(2).map(String::as_str) == Some(slug))
        .and_then(|(_, c)| c.last().map(|last| last.starts_with("done")))
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
        // `resuelto-aguas-arriba` cuenta como `superseded` (feature #65): el
        // trabajo NO se hizo en este producto, asi que no puede sumar al
        // numerador de completitud; y dejarlo en el denominador —lo que hacia
        // `blocked`— condena al PRD a no llegar nunca al 100%.
        let st = f.get("status").and_then(Value::as_str);
        if st == Some("superseded") || st == Some(crate::commands::close::AGUAS_ARRIBA) {
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

    /// Envoltorio: arma el plan con los dos punteros dados por existentes.
    /// Los tests de la validacion de punteros son los de `decidir_vuelta`.
    fn escribir_vuelta(
        file: &Path,
        fid: &str,
        name: &str,
        date: &str,
        spec: &str,
        impl_: &str,
    ) -> CloseEcho {
        let plan = decidir_vuelta(
            fid,
            name,
            date,
            &[
                Candidato::nuevo("spec", spec, true),
                Candidato::nuevo("impl", impl_, true),
            ],
        );
        aplicar_vuelta(file, &plan).unwrap()
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

        let echo = escribir_vuelta(
            &file,
            "13",
            "avisar_mora",
            "2026-08-12",
            "docs/spec-feature-13-avisar-mora.md",
            "docs/impl-13.md",
        );
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
        let again = escribir_vuelta(
            &file,
            "13",
            "avisar_mora",
            "2026-08-13",
            "docs/spec-feature-13-avisar-mora.md",
            "docs/impl-13.md",
        );
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
        let echo = escribir_vuelta(&file, "7", "otra_cosa", "2026-08-12", "s.md", "i.md");
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
mod tests_vuelta_al_prd {
    #![allow(clippy::unwrap_used)]
    use super::*;

    /// AC-11: la parte que DECIDE es pura. Se le pasa un directorio vacio como
    /// testigo: si `decidir_vuelta` tocara el disco, quedaria rastro.
    #[test]
    fn decidir_vuelta_es_pura_y_no_escribe() {
        let dir = tempfile::tempdir().unwrap();
        let antes: Vec<_> = std::fs::read_dir(dir.path()).unwrap().collect();
        assert!(antes.is_empty(), "el fixture arranca vacio");

        let plan = decidir_vuelta(
            "60",
            "la_vuelta_no_se_pierde",
            "2026-08-27",
            &[
                Candidato::nuevo("spec", "docs/spec-feature-60-la-vuelta.md", true),
                Candidato::nuevo("impl", "docs/impl-60.md", true),
            ],
        );

        // No escribio nada...
        let despues: Vec<_> = std::fs::read_dir(dir.path()).unwrap().collect();
        assert!(despues.is_empty(), "decidir_vuelta no puede tocar el disco");
        // ...y devolvio el plan completo.
        assert_eq!(plan.cabeza, "- #60 la_vuelta_no_se_pierde -> done");
        assert_eq!(plan.slug_hito, "la_vuelta_no_se_pierde");
        assert!(plan.descartes.is_empty());
        assert_eq!(
            plan.linea,
            "- #60 la_vuelta_no_se_pierde -> done 2026-08-27 \u{b7} spec: docs/spec-feature-60-la-vuelta.md \u{b7} impl: docs/impl-60.md"
        );
    }

    /// AC-4: el puntero al spec es relativo a la raiz. Una ruta que escapa
    /// (tipicamente el worktree que el cierre esta por borrar) NO entra.
    #[test]
    fn punteros_de_bitacora_son_relativos_a_la_raiz() {
        // La forma exacta que tenian los 18 punteros rotos del PRD maestro.
        let del_worktree = "../harness_process-wt/47-features/docs/spec-feature-47-features.md";
        assert!(escapa_de_la_raiz(del_worktree));
        assert!(escapa_de_la_raiz("/abs/docs/spec.md"));
        assert!(escapa_de_la_raiz("C:/docs/spec.md"));
        assert!(escapa_de_la_raiz("docs/../../fuera.md"));
        assert!(!escapa_de_la_raiz("docs/spec-feature-47-features.md"));

        let plan = decidir_vuelta(
            "47",
            "features",
            "2026-08-22",
            &[Candidato::nuevo("spec", del_worktree, true)],
        );
        assert!(
            !plan.linea.contains(".."),
            "la linea no puede llevar una ruta que escapa: {}",
            plan.linea
        );
        assert_eq!(plan.descartes.len(), 1);
        assert_eq!(plan.descartes[0].motivo, MOTIVO_ESCAPA);
    }

    /// AC-5: un puntero que no resuelve se OMITE, no se escribe roto.
    #[test]
    fn bitacora_omite_el_puntero_que_no_resuelve() {
        let plan = decidir_vuelta(
            "60",
            "una_feature",
            "2026-08-27",
            &[
                Candidato::nuevo("spec", "docs/spec-feature-60-una-feature.md", true),
                // El caso del bug #92: impl-<n>.md que nadie creo nunca.
                Candidato::nuevo("impl", "docs/impl-60.md", false),
            ],
        );
        assert!(plan.linea.contains("spec: docs/spec-feature-60-una-feature.md"));
        assert!(!plan.linea.contains("impl:"), "no escribe el puntero ausente");
        assert_eq!(plan.descartes.len(), 1);
        assert_eq!(plan.descartes[0].etiqueta, "impl");
        assert_eq!(plan.descartes[0].motivo, MOTIVO_AUSENTE);

        // Y sin ningun puntero valido, la entrada igual se escribe: la bitacora
        // vale por si misma.
        let pelada = decidir_vuelta("61", "otra", "2026-08-27", &[]);
        assert_eq!(pelada.linea, "- #61 otra -> done 2026-08-27");
    }

    /// AC-10: re-aplicar no duplica ni reescribe la fecha del primer cierre.
    #[test]
    fn vuelta_al_prd_es_idempotente() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("PRD-master.md");
        std::fs::write(
            &file,
            concat!(
                "# PRD Master\n",
                "\n",
                "## 10. Hitos -> features\n",
                "\n",
                "| # | Hito | Slug de feature | Objetivo | Criterio | Estado |\n",
                "| --- | --- | --- | --- | --- | --- |\n",
                "| 1 | La vuelta | la_vuelta | O1 | queda escrita | pendiente |\n",
            ),
        )
        .unwrap();
        let plan = |fecha: &str| {
            decidir_vuelta(
                "60",
                "la_vuelta",
                fecha,
                &[Candidato::nuevo("spec", "docs/spec-feature-60-la-vuelta.md", true)],
            )
        };
        let primero = aplicar_vuelta(&file, &plan("2026-08-27")).unwrap();
        assert!(primero.milestone_marked && primero.logged);
        let segundo = aplicar_vuelta(&file, &plan("2026-09-01")).unwrap();
        assert_eq!(segundo, CloseEcho::default(), "la segunda no hace nada");

        let texto = std::fs::read_to_string(&file).unwrap();
        assert_eq!(texto.matches("- #60 la_vuelta").count(), 1);
        assert!(texto.contains("done (2026-08-27)"), "vale la fecha del primer cierre");
        assert!(!texto.contains("2026-09-01"));
    }

    /// La bitacora ya escrita se puede volver a leer: es lo que le permite a
    /// `prd doctor` auditar y reparar punteros sin tocar el resto de la linea.
    #[test]
    fn bitacora_entries_lee_lo_que_aplicar_vuelta_escribe() {
        let texto = concat!(
            "## Bitacora\n",
            "\n",
            "-\n",
            "- #47 features -> done 2026-08-22 \u{b7} spec: ../wt/47/docs/spec-47.md \u{b7} impl: docs/impl-47.md\n",
            "- una linea cualquiera del usuario\n",
            "- #55 check -> done 2026-08-26\n",
        );
        let entradas = bitacora_entries(texto);
        assert_eq!(entradas.len(), 2, "solo las que tienen la forma de entrada");

        assert_eq!(entradas[0].feature_id, "47");
        assert_eq!(entradas[0].feature_name, "features");
        assert_eq!(entradas[0].fecha, "2026-08-22");
        assert_eq!(entradas[0].punteros.len(), 2);
        assert_eq!(entradas[0].punteros[0].1, "../wt/47/docs/spec-47.md");

        // Reparar = reescribir la linea con los punteros que sobreviven.
        let reparada = entradas[0].con_punteros(&[
            ("spec".to_string(), "docs/spec-47.md".to_string()),
            ("impl".to_string(), "docs/impl-47.md".to_string()),
        ]);
        assert_eq!(
            reparada,
            "- #47 features -> done 2026-08-22 \u{b7} spec: docs/spec-47.md \u{b7} impl: docs/impl-47.md"
        );

        // Una entrada sin punteros se lee igual (la del #55).
        assert!(entradas[1].punteros.is_empty());
        assert_eq!(entradas[1].con_punteros(&[]), "- #55 check -> done 2026-08-26");
    }

    /// `hito_marcado` distingue los tres casos que le importan al doctor.
    #[test]
    fn hito_marcado_distingue_sin_fila_de_sin_marcar() {
        let texto = concat!(
            "## 10. Hitos -> features\n",
            "\n",
            "| # | Hito | Slug de feature | Objetivo | Criterio | Estado |\n",
            "| --- | --- | --- | --- | --- | --- |\n",
            "| 1 | Uno | ya_cerrada | O1 | c | done (2026-08-01) |\n",
            "| 2 | Dos | sin_marcar | O1 | c | pendiente |\n",
        );
        assert_eq!(hito_marcado(texto, "ya_cerrada"), Some(true));
        assert_eq!(hito_marcado(texto, "sin_marcar"), Some(false));
        assert_eq!(hito_marcado(texto, "no_esta_en_la_tabla"), None);
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
    fn prd_tree_ignora_aguas_arriba() {
        // AC-7 (#65): misma decision que `superseded`, por la misma razon — el
        // trabajo NO se hizo en este producto, asi que no puede sumar al
        // numerador; y dejarlo en el denominador (lo que hacia `blocked`)
        // condena al PRD a no llegar nunca al 100%.
        let data = json!({"features": [
            {"id": 1, "name": "hecha", "status": "done"},
            {"id": 2, "name": "pendiente", "status": "pending"},
            {"id": 3, "name": "arreglada-afuera", "status": "resuelto-aguas-arriba",
             "resuelto_en": "harness_process/feature-60"},
            {"id": 4, "name": "trabada", "status": "blocked"}
        ]});
        assert_eq!(feature_counts(&data, ""), (1, 3));

        // Y la comprobacion que motivo la feature: con `blocked` esa misma
        // entrada queda en el denominador PARA SIEMPRE.
        let como_blocked = json!({"features": [
            {"id": 1, "name": "hecha", "status": "done"},
            {"id": 2, "name": "pendiente", "status": "pending"},
            {"id": 3, "name": "arreglada-afuera", "status": "blocked"},
            {"id": 4, "name": "trabada", "status": "blocked"}
        ]});
        assert_eq!(feature_counts(&como_blocked, ""), (1, 4));
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
