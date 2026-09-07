//! Feature #79: `close` refresca el espejo del backlog y de la bitacora.
//!
//! `feature_list.json` y `progress/history.md` estan gitignorados: son los dos
//! datos del proyecto que el instalador no puede regenerar (#78) y los dos que
//! se perdieron el 2026-09-06 cuando el checkout se borro por error. El backlog
//! se recupero de un espejo versionado en `docs/bkp-backlog/` que alguien habia
//! refrescado a mano en el ultimo cierre; la bitacora no tenia espejo y se
//! perdio. Desde esta feature, cada cierre deja los dos espejos byte-identicos,
//! y el usuario los commitea con la raiz (como el sello y la bitacora del PRD).
//!
//! Solo `refrescar` toca el filesystem; la politica y la decision son puras.

use std::path::{Path, PathBuf};

use anyhow::Context;
use serde_json::Value;

/// Directorio del espejo, relativo a la raiz del repo principal (con `/`: es
/// el nombre que documentan spec, README y UPDATING; `dir_del_espejo` lo arma
/// con `Path::join` por componente para no mezclar separadores en Windows).
pub const ESPEJO_DIR: &str = "docs/bkp-backlog";

/// `<raiz>/docs/bkp-backlog`, componente a componente.
pub fn dir_del_espejo(raiz: &Path) -> PathBuf {
    ESPEJO_DIR.split('/').fold(raiz.to_path_buf(), |p, c| p.join(c))
}

/// Ruta para un MENSAJE: relativa a `raiz` si se puede, siempre con `/`, para
/// que lo que se imprime coincida con lo documentado en cualquier plataforma.
pub fn ruta_para_mensaje(ruta: &Path, raiz: &Path) -> String {
    let rel = ruta.strip_prefix(raiz).unwrap_or(ruta);
    rel.components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join("/")
}

/// Lo que dejo un refresco: que se escribio y que fallo, por archivo. Un fallo
/// en la bitacora no oculta que el backlog SI quedo espejado (revision
/// adversarial de la #79): las dos copias se intentan siempre.
#[derive(Debug, Default)]
pub struct Refresco {
    pub escritos: Vec<PathBuf>,
    pub fallos: Vec<(PathBuf, anyhow::Error)>,
}

/// Que hace el cierre con el espejo. Tres estados porque son tres conductas
/// distintas, no un bool con un caso ambiguo: `Auto` mira si el usuario ya
/// opto por el espejo (el directorio existe), `Siempre` lo crea, `Nunca` no
/// lo toca aunque exista.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Politica {
    /// `rules.espejo_backlog` ausente (o no booleano): refresca solo si
    /// `docs/bkp-backlog/` ya existe.
    Auto,
    /// `rules.espejo_backlog: true`: refresca y crea lo que falte.
    Siempre,
    /// `rules.espejo_backlog: false`: no toca nada.
    Nunca,
}

impl Politica {
    /// Lee `rules.espejo_backlog`. Cualquier cosa que no sea `true`/`false`
    /// es `Auto`: la ausencia es el default y un valor mal tipeado no puede
    /// borrar ni crear nada por accidente.
    pub fn from_rules(data: &Value) -> Politica {
        match data
            .get("rules")
            .and_then(|r| r.get("espejo_backlog"))
            .and_then(Value::as_bool)
        {
            Some(true) => Politica::Siempre,
            Some(false) => Politica::Nunca,
            None => Politica::Auto,
        }
    }
}

/// La decision, sin filesystem: `existe_dir` es si `docs/bkp-backlog/` ya esta.
pub fn decidir(politica: Politica, existe_dir: bool) -> bool {
    match politica {
        Politica::Auto => existe_dir,
        Politica::Siempre => true,
        Politica::Nunca => false,
    }
}

/// Refresca los espejos en `raiz/docs/bkp-backlog/`: `feature_list.json` desde
/// `backlog` y `history.md` desde `bitacora`. Copia BYTES, atomica
/// (`write_bytes_atomic`, la misma familia que escribe el backlog): una
/// bitacora con un byte raro se copia igual, sin condicion de codificacion.
/// Una fuente que no existe se salta sin error. Las dos copias se intentan
/// siempre; el resultado dice cual se escribio y cual fallo.
///
/// # Errors
///
/// Solo si el directorio no se puede crear (`Siempre`): sin directorio no hay
/// nada que intentar. Los fallos por archivo van en `Refresco::fallos`, y el
/// que llama decide: `close` los AVISA y sigue, porque un respaldo que impide
/// cerrar es peor que ninguno.
pub fn refrescar(
    raiz: &Path,
    backlog: &Path,
    bitacora: &Path,
    politica: Politica,
) -> anyhow::Result<Refresco> {
    let dir = dir_del_espejo(raiz);
    if !decidir(politica, dir.is_dir()) {
        return Ok(Refresco::default());
    }
    if politica == Politica::Siempre {
        std::fs::create_dir_all(&dir)
            .with_context(|| format!("no se pudo crear {}", dir.display()))?;
    }
    let mut refresco = Refresco::default();
    for (fuente, nombre) in [(backlog, "feature_list.json"), (bitacora, "history.md")] {
        if !fuente.is_file() {
            continue;
        }
        let destino = dir.join(nombre);
        let copia = std::fs::read(fuente)
            .with_context(|| format!("no se pudo leer {}", fuente.display()))
            .and_then(|bytes| {
                crate::features::write_bytes_atomic(&destino, &bytes)
                    .with_context(|| format!("no se pudo escribir {}", destino.display()))
            });
        match copia {
            Ok(()) => refresco.escritos.push(destino),
            Err(err) => refresco.fallos.push((destino, err)),
        }
    }
    Ok(refresco)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;
    use serde_json::json;

    #[test]
    fn espejo_politica_should_be_auto_when_the_rule_is_absent_or_not_a_bool() {
        assert_eq!(Politica::from_rules(&json!({})), Politica::Auto);
        assert_eq!(
            Politica::from_rules(&json!({"rules": {"espejo_backlog": "si"}})),
            Politica::Auto
        );
    }

    #[test]
    fn espejo_politica_should_map_true_to_siempre_and_false_to_nunca() {
        assert_eq!(
            Politica::from_rules(&json!({"rules": {"espejo_backlog": true}})),
            Politica::Siempre
        );
        assert_eq!(
            Politica::from_rules(&json!({"rules": {"espejo_backlog": false}})),
            Politica::Nunca
        );
    }

    #[test]
    fn espejo_politica_should_decide_auto_only_when_the_directory_exists() {
        assert!(decidir(Politica::Auto, true));
        assert!(!decidir(Politica::Auto, false));
    }

    #[test]
    fn espejo_politica_should_decide_siempre_always_and_nunca_never() {
        assert!(decidir(Politica::Siempre, true));
        assert!(decidir(Politica::Siempre, false));
        assert!(!decidir(Politica::Nunca, true));
        assert!(!decidir(Politica::Nunca, false));
    }

    #[test]
    fn refrescar_should_skip_a_missing_source_without_failing() {
        let tmp = tempfile::tempdir().unwrap();
        let raiz = tmp.path();
        std::fs::create_dir_all(raiz.join(ESPEJO_DIR)).unwrap();
        let backlog = raiz.join("feature_list.json");
        std::fs::write(&backlog, "{}\n").unwrap();
        let r = refrescar(raiz, &backlog, &raiz.join("no-existe.md"), Politica::Auto).unwrap();
        assert_eq!(r.escritos, vec![dir_del_espejo(raiz).join("feature_list.json")]);
        assert!(r.fallos.is_empty());
    }

    #[test]
    fn refrescar_should_report_the_failed_copy_and_still_write_the_other_one() {
        // Fallo PARCIAL: el destino del backlog es un directorio, la bitacora
        // se copia igual y el resultado nombra las dos cosas.
        let tmp = tempfile::tempdir().unwrap();
        let raiz = tmp.path();
        let dir = dir_del_espejo(raiz);
        std::fs::create_dir_all(dir.join("feature_list.json")).unwrap();
        let backlog = raiz.join("feature_list.json");
        let bitacora = raiz.join("history.md");
        std::fs::write(&backlog, "{}\n").unwrap();
        std::fs::write(&bitacora, b"- linea con byte raro \xff\n").unwrap();
        let r = refrescar(raiz, &backlog, &bitacora, Politica::Auto).unwrap();
        assert_eq!(r.escritos, vec![dir.join("history.md")]);
        assert_eq!(r.fallos.len(), 1);
        assert_eq!(r.fallos[0].0, dir.join("feature_list.json"));
        assert_eq!(std::fs::read(dir.join("history.md")).unwrap(), b"- linea con byte raro \xff\n");
    }

    #[test]
    fn ruta_para_mensaje_should_be_relative_to_the_root_with_forward_slashes() {
        let raiz = Path::new("/tmp/raiz");
        let ruta = dir_del_espejo(raiz).join("history.md");
        assert_eq!(ruta_para_mensaje(&ruta, raiz), "docs/bkp-backlog/history.md");
    }

    #[test]
    fn refrescar_should_write_nothing_when_auto_and_the_directory_is_missing() {
        let tmp = tempfile::tempdir().unwrap();
        let raiz = tmp.path();
        let backlog = raiz.join("feature_list.json");
        std::fs::write(&backlog, "{}\n").unwrap();
        let r = refrescar(raiz, &backlog, &raiz.join("h.md"), Politica::Auto).unwrap();
        assert!(r.escritos.is_empty() && r.fallos.is_empty());
        assert!(!dir_del_espejo(raiz).exists());
    }
}
