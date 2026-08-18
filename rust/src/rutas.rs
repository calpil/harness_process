//! Rutas protegidas (feature #26).
//!
//! Los PRD y la constitution son del USUARIO. Hasta esta feature eso era buena
//! fe: el README lo decia y no habia un solo gate. Aca hay una lista y tres
//! capas que la hacen valer, **cada una con su alcance declarado**:
//!
//! | Capa | Que puede | Que NO puede |
//! | --- | --- | --- |
//! | `PreToolUse` | impedir la escritura | existir donde el backend no tiene el evento |
//! | `PostToolUse` | avisar con el comando de reversion | impedir: corre DESPUES |
//! | `harness_check.sh` | bloquear el cierre (exit 2) | actuar en el momento del dano |
//!
//! **Lo que se protege son las herramientas del AGENTE, no el binario del
//! arnes.** `close` escribe en `docs/prd/PRD-master.md` cada vez que marca un
//! hito: si la proteccion lo alcanzara, el arnes se bloquearia a si mismo.
//!
//! `esta_protegida()` es **pura** y el modulo no importa nada que escriba, asi
//! que "esto no muta nada" lo sostiene la estructura y no la disciplina
//! (leccion `promesas-estructurales-vs-disciplina`).

use std::path::Path;

use serde_json::Value;

/// Las tres por defecto. Deliberadamente pocas: esta es la primera feature del
/// arnes que puede **impedirle trabajar al agente**, y una lista ancha por
/// defecto dejaria proyectos trabados sin que nadie lo pidiera.
pub const DEFAULTS: [&str; 3] = ["docs/prd/**", "docs/constitution.md", ".env"];

/// Lee `rules.rutas_protegidas`. Tres estados **distintos**, y confundirlos
/// dejaria un proyecto desprotegido creyendo lo contrario:
///
/// - clave ausente -> los defaults (una instalacion que no configura nada queda
///   protegida igual)
/// - lista propia  -> exactamente esa
/// - lista **vacia** -> proteccion apagada, porque el usuario lo pidio
pub fn patrones(data: &Value) -> Vec<String> {
    let Some(valor) = data.get("rules").and_then(|r| r.get("rutas_protegidas")) else {
        return DEFAULTS.iter().map(|s| (*s).to_string()).collect();
    };
    let Some(lista) = valor.as_array() else {
        // Valor con tipo equivocado: se ignora y valen los defaults. Nunca
        // desproteger por un error de tipeo en la config.
        return DEFAULTS.iter().map(|s| (*s).to_string()).collect();
    };
    lista
        .iter()
        .filter_map(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect()
}

/// `true` si la ruta cae bajo algun patron. Funcion **pura**.
///
/// `ruta` puede venir absoluta (como la da un hook) o relativa a la raiz (como
/// la da `git status`): las dos formas tienen que decidir lo mismo, porque de lo
/// contrario la proteccion dependeria de como se escribio la ruta.
pub fn esta_protegida(ruta: &str, raiz: &Path, patrones: &[String]) -> bool {
    let Some(rel) = relativa(ruta, raiz) else {
        return false; // fuera de la raiz: no es asunto del arnes
    };
    patrones.iter().any(|p| matchea(&rel, p))
}

/// Normaliza a una ruta relativa a la raiz, con separadores `/`.
fn relativa(ruta: &str, raiz: &Path) -> Option<String> {
    let limpia = ruta.trim();
    if limpia.is_empty() {
        return None;
    }
    let path = Path::new(limpia);
    let rel = if path.is_absolute() {
        // `strip_prefix` sobre la raiz canonica si se puede; si no, lexica.
        let raiz_c = std::fs::canonicalize(raiz).unwrap_or_else(|_| raiz.to_path_buf());
        let path_c = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
        path_c
            .strip_prefix(&raiz_c)
            .or_else(|_| path.strip_prefix(raiz))
            .ok()?
            .to_path_buf()
    } else {
        // `./docs/x` y `docs/x` son la misma ruta.
        path.strip_prefix("./").unwrap_or(path).to_path_buf()
    };
    let texto = rel.to_string_lossy().replace('\\', "/");
    (!texto.is_empty()).then_some(texto)
}

/// Glob por segmentos: `*` cubre un segmento, `**` cualquier profundidad
/// (incluida ninguna, para que `docs/prd/**` cubra tambien `docs/prd`).
fn matchea(rel: &str, patron: &str) -> bool {
    let r: Vec<&str> = rel.split('/').filter(|s| !s.is_empty()).collect();
    let p: Vec<&str> = patron
        .trim_start_matches("./")
        .split('/')
        .filter(|s| !s.is_empty())
        .collect();
    matchea_segmentos(&r, &p)
}

fn matchea_segmentos(rel: &[&str], pat: &[&str]) -> bool {
    match (rel.first(), pat.first()) {
        (_, Some(&"**")) => {
            // `**` consume cero o mas segmentos: se prueban todos los cortes.
            (0..=rel.len()).any(|n| matchea_segmentos(&rel[n..], &pat[1..]))
        }
        (Some(r), Some(p)) if segmento_matchea(r, p) => {
            matchea_segmentos(&rel[1..], &pat[1..])
        }
        (None, None) => true,
        _ => false,
    }
}

/// `*` dentro de un segmento (`*.env`, `PRD-*.md`).
fn segmento_matchea(seg: &str, patron: &str) -> bool {
    if patron == "*" {
        return true;
    }
    if !patron.contains('*') {
        return seg == patron;
    }
    let partes: Vec<&str> = patron.split('*').collect();
    let mut resto = seg;
    for (i, parte) in partes.iter().enumerate() {
        if parte.is_empty() {
            continue;
        }
        if i == 0 {
            let Some(r) = resto.strip_prefix(parte) else {
                return false;
            };
            resto = r;
        } else if i == partes.len() - 1 {
            return resto.ends_with(parte) && resto.len() >= parte.len();
        } else {
            let Some(pos) = resto.find(parte) else {
                return false;
            };
            resto = &resto[pos + parte.len()..];
        }
    }
    true
}

/// Las rutas protegidas que estan modificadas y sin commitear, **descontando
/// las que escribio el propio arnes** (`exentas`).
///
/// Recibe la salida de `git status --porcelain` y la lista de exenciones ya
/// resueltas, en vez de invocar git y leer archivos: asi sigue siendo pura y se
/// puede probar sin un repo.
///
/// Las exenciones son el AC-9/AC-10 y no son un detalle: `close` escribe en
/// `docs/prd/PRD-master.md` cada vez que marca un hito, y sin esto el arnes se
/// reportaria a si mismo como violacion en el turno siguiente a cada cierre.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Violacion {
    pub ruta: String,
    /// `git status` la conoce (`M`, `A`, `R`...) o es `??`. Cambia el remedio:
    /// `git checkout --` **no revierte un archivo sin trackear**, y ofrecerlo
    /// seria dar un comando que no hace nada (hallazgo de la primera corrida
    /// real sobre este repo).
    pub trackeada: bool,
}

pub fn violaciones(
    git_porcelain: &str,
    raiz: &Path,
    patrones: &[String],
    exentas: &[String],
) -> Vec<Violacion> {
    let mut out: Vec<Violacion> = Vec::new();
    for linea in git_porcelain.lines() {
        // `XY ruta` o `XY vieja -> nueva` (renombres).
        if linea.len() < 4 {
            continue;
        }
        let estado = &linea[..2];
        let ruta = linea[3..].trim();
        let ruta = ruta.rsplit(" -> ").next().unwrap_or(ruta);
        let ruta = ruta.trim_matches('"');
        if !esta_protegida(ruta, raiz, patrones) {
            continue;
        }
        if exentas.iter().any(|e| e == ruta) {
            continue;
        }
        if out.iter().any(|v| v.ruta == ruta) {
            continue;
        }
        out.push(Violacion {
            ruta: ruta.to_string(),
            trackeada: estado != "??",
        });
    }
    out
}

/// Una linea del registro de escrituras del arnes: `<ruta>\t<mtime_nanos>`.
///
/// El mtime es lo que hace que la exencion **se auto-limpie**: vale solo
/// mientras nadie vuelva a tocar el archivo. Si el agente lo edita despues de
/// que lo escribio el arnes, el mtime cambia y la exencion deja de aplicar.
pub fn linea_registro(ruta: &str, mtime_nanos: u128) -> String {
    format!("{ruta}\t{mtime_nanos}")
}

/// Resuelve que rutas siguen exentas: las que el arnes escribio y **nadie toco
/// desde entonces**.
pub fn exentas(registro: &str, mtime_actual: impl Fn(&str) -> Option<u128>) -> Vec<String> {
    registro
        .lines()
        .filter_map(|l| {
            let (ruta, stamp) = l.split_once('\t')?;
            let stamp: u128 = stamp.trim().parse().ok()?;
            (mtime_actual(ruta) == Some(stamp)).then(|| ruta.to_string())
        })
        .collect()
}

/// Que hacer con una ruta protegida que se toco. Nunca se ejecuta solo
/// (decision del usuario 2026-08-17, OBS-2).
///
/// **Primero mirar, despues decidir**, y esto no es cortesia: durante el
/// desarrollo de esta feature la version anterior imprimia `git checkout --`
/// a secas, se corrio tal cual, y **borro los hitos de tres features que
/// estaban sin commitear**. `git checkout` no revierte "el cambio del agente":
/// revierte el archivo entero a HEAD, incluido el trabajo legitimo que hubiera
/// encima. Un remedio destructivo tiene que decir que destruye.
pub fn remedio(v: &Violacion) -> String {
    if v.trackeada {
        format!(
            "mira que cambio: git diff -- {} | y si no fue tuyo: git checkout -- {} (DESCARTA todo lo no commiteado de ese archivo)",
            v.ruta, v.ruta
        )
    } else {
        format!(
            "es nueva y sin trackear: revisala | y si no la pusiste vos: rm -r {} (BORRA el archivo)",
            v.ruta
        )
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;
    use serde_json::json;

    fn pats(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| (*s).to_string()).collect()
    }

    fn rutas_de(v: &[Violacion]) -> Vec<String> {
        v.iter().map(|x| x.ruta.clone()).collect()
    }

    #[test]
    fn deny_should_protect_the_three_defaults() {
        let raiz = Path::new("/proyecto");
        let p = patrones(&json!({}));
        assert_eq!(p, pats(&DEFAULTS));
        for ruta in [
            "docs/prd/PRD-master.md",
            "docs/prd/aprendizaje/PRD-aprendizaje.md",
            "docs/constitution.md",
            ".env",
        ] {
            assert!(esta_protegida(ruta, raiz, &p), "{ruta} deberia estar protegida");
        }
    }

    #[test]
    fn deny_should_match_globs_at_any_depth() {
        let raiz = Path::new("/proyecto");
        let p = pats(&["docs/prd/**"]);
        // `**` cubre cualquier profundidad, incluida la propia carpeta.
        assert!(esta_protegida("docs/prd/PRD-master.md", raiz, &p));
        assert!(esta_protegida("docs/prd/a/b/c/PRD-x.md", raiz, &p));
        assert!(esta_protegida("docs/prd", raiz, &p));
        assert!(!esta_protegida("docs/plan-feature-1.md", raiz, &p));
        // `*` cubre UN segmento, no varios.
        let un_segmento = pats(&["docs/*/notas.md"]);
        assert!(esta_protegida("docs/prd/notas.md", raiz, &un_segmento));
        assert!(!esta_protegida("docs/prd/sub/notas.md", raiz, &un_segmento));
        // `*` dentro de un segmento.
        let parcial = pats(&["docs/PRD-*.md"]);
        assert!(esta_protegida("docs/PRD-master.md", raiz, &parcial));
        assert!(!esta_protegida("docs/plan.md", raiz, &parcial));
    }

    #[test]
    fn deny_should_normalize_absolute_and_relative_paths() {
        // La forma de escribir la ruta no puede cambiar si esta protegida: un
        // hook la manda absoluta y `git status` la manda relativa.
        let dir = tempfile::tempdir().unwrap();
        let raiz = dir.path();
        std::fs::create_dir_all(raiz.join("docs/prd")).unwrap();
        std::fs::write(raiz.join("docs/prd/PRD-master.md"), "x").unwrap();
        let p = pats(&["docs/prd/**"]);
        let absoluta = raiz.join("docs/prd/PRD-master.md");
        assert!(esta_protegida(&absoluta.to_string_lossy(), raiz, &p));
        assert!(esta_protegida("docs/prd/PRD-master.md", raiz, &p));
        assert!(esta_protegida("./docs/prd/PRD-master.md", raiz, &p));
        // Fuera de la raiz: no es asunto del arnes.
        assert!(!esta_protegida("/otro/lado/docs/prd/PRD-master.md", raiz, &p));
    }

    #[test]
    fn deny_should_not_guess_beyond_the_list() {
        // Sin heuristicas: "parece un PRD" no alcanza.
        let raiz = Path::new("/proyecto");
        let p = pats(&["docs/constitution.md"]);
        assert!(!esta_protegida("docs/prd/PRD-master.md", raiz, &p));
        assert!(!esta_protegida("PRD-otro.md", raiz, &p));
        assert!(!esta_protegida("docs/constitution.md.bak", raiz, &p));
        assert!(esta_protegida("docs/constitution.md", raiz, &p));
    }

    #[test]
    fn deny_should_read_user_defined_paths() {
        let raiz = Path::new("/proyecto");
        let p = patrones(&json!({"rules": {"rutas_protegidas": ["infra/**", "secretos.yml"]}}));
        assert_eq!(p, pats(&["infra/**", "secretos.yml"]));
        assert!(esta_protegida("infra/terraform/main.tf", raiz, &p));
        assert!(esta_protegida("secretos.yml", raiz, &p));
        // Y los defaults YA NO aplican: la lista propia reemplaza, no suma.
        assert!(!esta_protegida("docs/constitution.md", raiz, &p));
    }

    #[test]
    fn deny_should_fall_back_to_defaults_when_unconfigured() {
        // Clave ausente y `rules` ausente: los dos caen en los defaults.
        assert_eq!(patrones(&json!({})), pats(&DEFAULTS));
        assert_eq!(patrones(&json!({"rules": {}})), pats(&DEFAULTS));
        // Tipo equivocado: tampoco desprotege.
        assert_eq!(
            patrones(&json!({"rules": {"rutas_protegidas": "docs/prd/**"}})),
            pats(&DEFAULTS)
        );
    }

    #[test]
    fn deny_should_be_disablable_with_an_empty_list() {
        // Vacia EXPLICITA es distinto de ausente: el usuario lo pidio.
        let raiz = Path::new("/proyecto");
        let p = patrones(&json!({"rules": {"rutas_protegidas": []}}));
        assert!(p.is_empty());
        assert!(!esta_protegida("docs/prd/PRD-master.md", raiz, &p));
        assert!(!esta_protegida("docs/constitution.md", raiz, &p));
    }

    #[test]
    fn violaciones_should_read_git_porcelain() {
        let raiz = Path::new("/proyecto");
        let p = pats(&DEFAULTS);
        let salida = " M docs/prd/PRD-master.md\n\
                      ?? docs/plan-feature-26.md\n\
                      A  docs/constitution.md\n\
                      M  rust/src/rutas.rs\n";
        assert_eq!(
            rutas_de(&violaciones(salida, raiz, &p, &[])),
            ["docs/prd/PRD-master.md", "docs/constitution.md"]
        );
    }

    #[test]
    fn violaciones_should_handle_renames_and_quotes() {
        let raiz = Path::new("/proyecto");
        let p = pats(&DEFAULTS);
        let salida = "R  docs/viejo.md -> docs/constitution.md\n";
        assert_eq!(rutas_de(&violaciones(salida, raiz, &p, &[])), ["docs/constitution.md"]);
    }

    #[test]
    fn violaciones_should_be_empty_without_protected_changes() {
        let raiz = Path::new("/proyecto");
        let p = pats(&DEFAULTS);
        assert!(violaciones(" M rust/src/main.rs\n?? nuevo.txt\n", raiz, &p, &[]).is_empty());
        assert!(violaciones("", raiz, &p, &[]).is_empty());
    }

    #[test]
    fn violaciones_should_exempt_what_the_harness_itself_wrote() {
        // AC-9/AC-10: `close` escribe el PRD al marcar un hito. Sin esto, el
        // arnes se reportaria a si mismo despues de cada cierre.
        let raiz = Path::new("/proyecto");
        let p = pats(&DEFAULTS);
        let salida = " M docs/prd/PRD-master.md\n M docs/constitution.md\n";
        let exentas = vec!["docs/prd/PRD-master.md".to_string()];
        assert_eq!(rutas_de(&violaciones(salida, raiz, &p, &exentas)), ["docs/constitution.md"]);
    }

    #[test]
    fn exenciones_should_expire_when_the_file_changes_again() {
        // La exencion vale mientras NADIE vuelva a tocar el archivo: si el
        // agente lo edita despues, el mtime cambia y vuelve a ser violacion.
        let registro = linea_registro("docs/prd/PRD-master.md", 1000);
        assert_eq!(
            exentas(&registro, |_| Some(1000)),
            ["docs/prd/PRD-master.md"],
            "sin cambios posteriores, sigue exenta"
        );
        assert!(
            exentas(&registro, |_| Some(2000)).is_empty(),
            "el agente lo toco despues: la exencion caduca"
        );
        assert!(
            exentas(&registro, |_| None).is_empty(),
            "archivo ausente: nada que eximir"
        );
        assert!(exentas("", |_| Some(1000)).is_empty());
        assert!(exentas("basura sin tab\n", |_| Some(1000)).is_empty());
    }

    #[test]
    fn remedio_should_show_the_diff_before_the_destructive_command() {
        // Encoded aca porque ya paso: la version anterior imprimia
        // `git checkout --` a secas, se corrio, y borro los hitos de tres
        // features que estaban sin commitear.
        let v = Violacion { ruta: "docs/constitution.md".into(), trackeada: true };
        let r = remedio(&v);
        assert!(r.contains("git diff -- docs/constitution.md"), "{r}");
        assert!(r.contains("git checkout -- docs/constitution.md"), "{r}");
        assert!(r.contains("DESCARTA"), "un remedio destructivo tiene que decirlo: {r}");
        assert!(
            r.find("git diff").unwrap() < r.find("git checkout").unwrap(),
            "primero mirar, despues decidir: {r}"
        );
    }

    #[test]
    fn remedio_should_not_offer_checkout_for_an_untracked_path() {
        // Hallazgo de la primera corrida real: `git checkout -- docs/prd/nueva/`
        // sobre un directorio sin trackear no revierte nada. Un remedio que no
        // remedia es peor que ninguno.
        let nueva = Violacion { ruta: "docs/prd/nueva/".into(), trackeada: false };
        let r = remedio(&nueva);
        assert!(!r.contains("git checkout"), "{r}");
        assert!(r.contains("sin trackear"), "{r}");
        assert!(r.contains("rm -r docs/prd/nueva/"), "{r}");
        assert!(r.contains("BORRA"), "un remedio destructivo tiene que decirlo: {r}");
    }

    #[test]
    fn violaciones_should_mark_untracked_paths() {
        let raiz = Path::new("/proyecto");
        let p = pats(&DEFAULTS);
        let v = violaciones("?? docs/prd/nueva/\n M docs/constitution.md\n", raiz, &p, &[]);
        assert_eq!(v.len(), 2);
        assert!(!v[0].trackeada, "?? es sin trackear: {v:?}");
        assert!(v[1].trackeada, "M es trackeada: {v:?}");
    }
}
