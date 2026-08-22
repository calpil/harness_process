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
}
