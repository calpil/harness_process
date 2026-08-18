//! Curador de lecciones: el mantenimiento de la biblioteca (feature #21).
//!
//! Tres limites que definen esta pieza tanto como lo que hace:
//!
//! - **Nunca borra.** Archivar es MOVER a `docs/lecciones/archivo/`. No existe
//!   ningun camino que elimine una leccion.
//! - **Nada se mueve sin `--aplicar`.** La pasada por defecto solo informa
//!   (decision del usuario 2026-08-17, OBS-3): mover archivos de alguien en un
//!   hook, sin que lo pida, no es curar.
//! - **Toda pasada mutante deja backup**, y el rollback tambien es reversible.
//!
//! Sin modelo y sin hub: las transiciones son aritmetica de fechas.

use std::path::{Path, PathBuf};

use crate::exit::Exit;
use crate::lecciones::{self, Leccion, Transicion, Umbrales};
use crate::paths::HarnessPaths;
use crate::pycompat::env_nonempty;

/// Que le pasa a UNA leccion en esta pasada.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Accion {
    pub nombre: String,
    pub transicion: Transicion,
    pub dias: i64,
    pub estado_actual: String,
}

/// El plan de una pasada. Se calcula leyendo, sin mutar nada.
#[derive(Debug, Clone, Default)]
pub struct Plan {
    pub acciones: Vec<Accion>,
    pub pinneadas: Vec<String>,
    pub evaluadas: usize,
}

impl Plan {
    pub fn vacio(&self) -> bool {
        self.acciones.is_empty()
    }
}

/// Calcula que haria la pasada. **No toca el filesystem** mas que para leer.
pub fn planificar(paths: &HarnessPaths, hoy: &str, umbrales: Umbrales) -> Plan {
    let (lecciones_ok, _rotas) = lecciones::scan(paths);
    let mut plan = Plan {
        evaluadas: lecciones_ok.len(),
        ..Plan::default()
    };
    for l in &lecciones_ok {
        if l.pinneada() {
            plan.pinneadas.push(l.nombre.clone());
            continue;
        }
        let transicion = l.transicion(hoy, umbrales);
        if !transicion.muta() {
            continue;
        }
        plan.acciones.push(Accion {
            nombre: l.nombre.clone(),
            transicion,
            dias: l.dias_inactiva(hoy).unwrap_or(0),
            estado_actual: l.estado(),
        });
    }
    plan.acciones.sort_by(|a, b| a.nombre.cmp(&b.nombre));
    plan
}

// ---------------------------------------------------------------------------
// Backups
// ---------------------------------------------------------------------------

/// `bkp/lecciones/` (honra `HARNESS_BKP_DIR`, igual que el instalador, para no
/// inventar una segunda convencion de respaldos).
pub fn backups_dir(paths: &HarnessPaths) -> PathBuf {
    let base = match env_nonempty("HARNESS_BKP_DIR") {
        Some(v) => PathBuf::from(v),
        None => paths.root.join("bkp"),
    };
    base.join("lecciones")
}

#[derive(Debug, Clone)]
pub struct Backup {
    pub id: String,
    pub motivo: String,
    pub path: PathBuf,
}

pub fn listar_backups(paths: &HarnessPaths) -> Vec<Backup> {
    let root = backups_dir(paths);
    let Ok(entries) = std::fs::read_dir(&root) else {
        return Vec::new();
    };
    let mut out: Vec<Backup> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .map(|p| {
            let id = p
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            let motivo = std::fs::read_to_string(p.join("MOTIVO.txt"))
                .unwrap_or_default()
                .trim()
                .to_string();
            Backup { id, motivo, path: p }
        })
        .collect();
    out.sort_by(|a, b| a.id.cmp(&b.id));
    out
}

/// Copia el arbol de lecciones a `bkp/lecciones/<ts>/`. Devuelve el id.
pub fn respaldar(paths: &HarnessPaths, id: &str, motivo: &str) -> anyhow::Result<PathBuf> {
    let destino = backups_dir(paths).join(id);
    std::fs::create_dir_all(&destino)?;
    copiar_arbol(&lecciones::dir(paths), &destino.join("lecciones"))?;
    std::fs::write(destino.join("MOTIVO.txt"), format!("{motivo}\n"))?;
    Ok(destino)
}

fn copiar_arbol(origen: &Path, destino: &Path) -> anyhow::Result<()> {
    if !origen.is_dir() {
        return Ok(());
    }
    std::fs::create_dir_all(destino)?;
    for entry in std::fs::read_dir(origen)?.flatten() {
        let path = entry.path();
        let nombre = entry.file_name();
        if path.is_dir() {
            copiar_arbol(&path, &destino.join(nombre))?;
        } else {
            std::fs::copy(&path, destino.join(nombre))?;
        }
    }
    Ok(())
}

fn borrar_contenido(dir: &Path) -> anyhow::Result<()> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Ok(());
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            std::fs::remove_dir_all(&path)?;
        } else {
            std::fs::remove_file(&path)?;
        }
    }
    Ok(())
}

/// Restaura el backup indicado (o el mas reciente). **Antes** de restaurar toma
/// un backup del estado actual, asi que deshacer tambien se deshace (AC-11).
pub fn rollback(paths: &HarnessPaths, id: Option<&str>, ts: &str) -> Result<Backup, Exit> {
    let disponibles = listar_backups(paths);
    let elegido = match id {
        Some(pedido) => disponibles.iter().find(|b| b.id == pedido).cloned(),
        None => disponibles.last().cloned(),
    };
    let Some(backup) = elegido else {
        return Err(Exit {
            code: 2,
            message: Some(match id {
                Some(pedido) => format!(
                    "No existe el backup '{pedido}'.\n    Vealos con 'sh harness_cli lecciones rollback --list'."
                ),
                None => "No hay backups de lecciones todavia: ninguna pasada del curador mutó nada.".to_string(),
            }),
        });
    };
    // El rollback tambien es reversible.
    if respaldar(paths, &format!("{ts}-pre-rollback"), &format!("estado previo al rollback a {}", backup.id)).is_err() {
        return Err(Exit {
            code: 2,
            message: Some("No se pudo respaldar el estado actual: se aborta el rollback.".to_string()),
        });
    }
    let destino = lecciones::dir(paths);
    let origen = backup.path.join("lecciones");
    if borrar_contenido(&destino).is_err() || copiar_arbol(&origen, &destino).is_err() {
        return Err(Exit {
            code: 2,
            message: Some(format!(
                "Fallo la restauracion desde {}. El estado previo quedo respaldado en {}.",
                backup.id,
                backups_dir(paths).join(format!("{ts}-pre-rollback")).display()
            )),
        });
    }
    Ok(backup)
}

// ---------------------------------------------------------------------------
// Aplicar
// ---------------------------------------------------------------------------

pub struct Aplicado {
    pub backup: PathBuf,
    pub reporte: PathBuf,
    pub aplicadas: usize,
}

/// Ejecuta el plan: respalda, muta y escribe el reporte. Sin acciones no crea ni
/// backup ni reporte (AC-17): correr un chequeo no puede ensuciar el repo.
pub fn aplicar(
    paths: &HarnessPaths,
    plan: &Plan,
    hoy: &str,
    ts: &str,
    umbrales: Umbrales,
) -> anyhow::Result<Option<Aplicado>> {
    if plan.vacio() {
        return Ok(None);
    }
    let backup = respaldar(paths, ts, "pasada del curador")?;
    let archivo = lecciones::archivo_dir(paths);
    let mut aplicadas = 0usize;
    for accion in &plan.acciones {
        let origen = lecciones::file_for(paths, &accion.nombre);
        let Ok(mut leccion) = Leccion::load(&origen) else {
            continue;
        };
        let Some(destino_estado) = accion.transicion.estado_destino() else {
            continue;
        };
        leccion.set_estado(destino_estado);
        if accion.transicion == Transicion::AArchivada {
            // Archivar es MOVER: se escribe en el destino y se saca del activo.
            std::fs::create_dir_all(&archivo)?;
            let destino = archivo.join(format!("{}.md", accion.nombre));
            crate::features::write_text_atomic(&destino, &leccion.render())?;
            std::fs::remove_file(&origen)?;
        } else {
            leccion.save()?;
        }
        aplicadas += 1;
    }
    let reporte = escribir_reporte(paths, plan, hoy, ts, umbrales, &backup)?;
    Ok(Some(Aplicado {
        backup,
        reporte,
        aplicadas,
    }))
}

fn escribir_reporte(
    paths: &HarnessPaths,
    plan: &Plan,
    hoy: &str,
    ts: &str,
    umbrales: Umbrales,
    backup: &Path,
) -> anyhow::Result<PathBuf> {
    let dir = paths.progress.join("lecciones").join(ts);
    std::fs::create_dir_all(&dir)?;
    let path = dir.join("REPORT.md");
    let mut out = format!(
        "# Pasada del curador - {ts}\n\n\
         Fecha de evaluacion: {hoy}\n\
         Umbrales: stale >= {} dias, archivo >= {} dias\n\
         Lecciones evaluadas: {}\n\
         Backup previo: `{}`\n\n\
         ## Transiciones aplicadas\n\n",
        umbrales.stale,
        umbrales.archivo,
        plan.evaluadas,
        backup.display()
    );
    if plan.acciones.is_empty() {
        out.push_str("- (ninguna)\n");
    }
    for a in &plan.acciones {
        let que = match a.transicion {
            Transicion::AStale => "activa -> stale",
            Transicion::AArchivada => "-> archivada (movida a docs/lecciones/archivo/)",
            Transicion::AActiva => "stale -> activa (volvio a usarse)",
            Transicion::Ninguna => "sin cambio",
        };
        out.push_str(&format!(
            "- `{}`: {que} — {} dia(s) de inactividad, estado previo `{}`\n",
            a.nombre, a.dias, a.estado_actual
        ));
    }
    out.push_str("\n## Salteadas por pin\n\n");
    if plan.pinneadas.is_empty() {
        out.push_str("- (ninguna)\n");
    }
    for n in &plan.pinneadas {
        out.push_str(&format!("- `{n}`\n"));
    }
    out.push_str(
        "\n---\n\nNada se borro: archivar es mover. Para deshacer esta pasada:\n\
         `sh harness_cli lecciones rollback`\n",
    );
    std::fs::write(&path, out)?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;

    /// Sandbox con lecciones de telemetria controlada.
    fn sandbox(lecciones_: &[(&str, &str, &str)]) -> (tempfile::TempDir, HarnessPaths) {
        let dir = tempfile::tempdir().unwrap();
        let paths = HarnessPaths::from_root(dir.path().to_path_buf());
        std::fs::create_dir_all(lecciones::dir(&paths)).unwrap();
        for (nombre, ultimo_uso, estado) in lecciones_ {
            let text = format!(
                "---\nnombre: {nombre}\nusos: 1\nultimo_uso: {ultimo_uso}\n\
                 ultima_actualizacion: {ultimo_uso}\nestado: {estado}\n---\n\ncuerpo de {nombre}\n"
            );
            std::fs::write(lecciones::file_for(&paths, nombre), text).unwrap();
        }
        (dir, paths)
    }

    #[test]
    fn planificar_should_not_touch_anything() {
        let (_d, paths) = sandbox(&[("vieja", "2026-01-01", "activa")]);
        let file = lecciones::file_for(&paths, "vieja");
        let antes = std::fs::read_to_string(&file).unwrap();
        let plan = planificar(&paths, "2026-08-17", Umbrales::default());
        assert_eq!(plan.acciones.len(), 1);
        assert_eq!(std::fs::read_to_string(&file).unwrap(), antes);
    }

    #[test]
    fn planificar_should_list_pinned_lessons_apart() {
        let (_d, paths) = sandbox(&[("vieja", "2026-01-01", "activa")]);
        let mut l = Leccion::load(&lecciones::file_for(&paths, "vieja")).unwrap();
        l.set_pin(true);
        l.save().unwrap();
        let plan = planificar(&paths, "2026-08-17", Umbrales::default());
        assert!(plan.acciones.is_empty());
        assert_eq!(plan.pinneadas, ["vieja"]);
    }

    #[test]
    fn aplicar_should_move_the_lesson_instead_of_deleting_it() {
        let (_d, paths) = sandbox(&[("vencida", "2026-01-01", "stale")]);
        let plan = planificar(&paths, "2026-08-17", Umbrales::default());
        let hecho = aplicar(&paths, &plan, "2026-08-17", "20260817-000000", Umbrales::default())
            .unwrap()
            .unwrap();
        assert_eq!(hecho.aplicadas, 1);
        // Ya no esta en el activo...
        assert!(!lecciones::file_for(&paths, "vencida").exists());
        // ...pero SI en el archivo, con su cuerpo intacto.
        let archivada = lecciones::archivo_dir(&paths).join("vencida.md");
        assert!(archivada.exists(), "archivar tiene que MOVER, no borrar");
        let l = Leccion::load(&archivada).unwrap();
        assert_eq!(l.estado(), lecciones::ESTADO_ARCHIVADA);
        assert!(l.body.contains("cuerpo de vencida"));
    }

    #[test]
    fn aplicar_should_do_nothing_without_actions() {
        let (_d, paths) = sandbox(&[("fresca", "2026-08-16", "activa")]);
        let plan = planificar(&paths, "2026-08-17", Umbrales::default());
        assert!(plan.vacio());
        let hecho = aplicar(&paths, &plan, "2026-08-17", "20260817-000000", Umbrales::default()).unwrap();
        assert!(hecho.is_none());
        // Ni backup ni reporte: correr un chequeo no ensucia el repo.
        assert!(!backups_dir(&paths).exists());
        assert!(!paths.progress.join("lecciones").exists());
    }

    #[test]
    fn rollback_should_restore_the_tree_and_stay_reversible() {
        let (_d, paths) = sandbox(&[("vencida", "2026-01-01", "stale")]);
        let original = std::fs::read_to_string(lecciones::file_for(&paths, "vencida")).unwrap();
        let plan = planificar(&paths, "2026-08-17", Umbrales::default());
        aplicar(&paths, &plan, "2026-08-17", "20260817-000000", Umbrales::default()).unwrap();
        assert!(!lecciones::file_for(&paths, "vencida").exists());

        let backup = rollback(&paths, None, "20260817-000100").unwrap();
        assert_eq!(backup.id, "20260817-000000");
        // Volvio a su lugar, con el contenido EXACTO.
        let restaurado = std::fs::read_to_string(lecciones::file_for(&paths, "vencida")).unwrap();
        assert_eq!(restaurado, original);
        // Y el rollback dejo su propio backup: deshacer se deshace.
        let ids: Vec<String> = listar_backups(&paths).into_iter().map(|b| b.id).collect();
        assert!(ids.iter().any(|i| i.ends_with("pre-rollback")), "{ids:?}");
    }

    #[test]
    fn rollback_should_fail_cleanly_without_backups() {
        let (_d, paths) = sandbox(&[("x", "2026-08-16", "activa")]);
        let err = rollback(&paths, None, "20260817-000000").unwrap_err();
        assert_eq!(err.code, 2);
        assert!(err.message.unwrap().contains("No hay backups"));
    }

    #[test]
    fn rollback_should_reject_an_unknown_id() {
        let (_d, paths) = sandbox(&[("vencida", "2026-01-01", "stale")]);
        let plan = planificar(&paths, "2026-08-17", Umbrales::default());
        aplicar(&paths, &plan, "2026-08-17", "20260817-000000", Umbrales::default()).unwrap();
        let err = rollback(&paths, Some("no-existe"), "20260817-000100").unwrap_err();
        assert_eq!(err.code, 2);
        assert!(err.message.unwrap().contains("--list"));
    }

    #[test]
    fn el_reporte_should_explain_each_transition_and_the_pins() {
        let (_d, paths) = sandbox(&[
            ("vencida", "2026-01-01", "stale"),
            ("enfriandose", "2026-07-01", "activa"),
            ("protegida", "2025-01-01", "activa"),
        ]);
        let mut p = Leccion::load(&lecciones::file_for(&paths, "protegida")).unwrap();
        p.set_pin(true);
        p.save().unwrap();
        let plan = planificar(&paths, "2026-08-17", Umbrales::default());
        let hecho = aplicar(&paths, &plan, "2026-08-17", "20260817-000000", Umbrales::default())
            .unwrap()
            .unwrap();
        let texto = std::fs::read_to_string(&hecho.reporte).unwrap();
        assert!(texto.contains("archivada"), "{texto}");
        assert!(texto.contains("activa -> stale"), "{texto}");
        assert!(texto.contains("dia(s) de inactividad"), "{texto}");
        assert!(texto.contains("## Salteadas por pin"), "{texto}");
        assert!(texto.contains("`protegida`"), "{texto}");
        assert!(texto.contains("Nada se borro"), "{texto}");
    }
}
