//! `harness nudge` (paridad: harness.py cmd_nudge). Best-effort absoluto:
//! cualquier error se traga en silencio y el exit code siempre es 0.
//!
//! Lo invoca el hook `PostToolUse` (matcher `Bash|Edit|Write|apply_patch`), asi
//! que corre muchas veces por sesion: todo lo que hace tiene que ser barato y
//! silencioso salvo cuando de verdad tiene algo que decir.

use std::io::Write;

use crate::features::{active_indices, feature_at, load_features};
use crate::lecciones;
use crate::paths::HarnessPaths;
use crate::plan::{is_plan_stale, plan_staleness_message};
use crate::pycompat::{mtime_f64, now_epoch_f64, py_str};

/// Piso del debounce del aviso "sin feature activa", en segundos.
const BACKOFF_PISO: f64 = 600.0;
/// Techo del backoff (1 hora). Decision del usuario 2026-08-16 (OBS-4 de la #18):
/// se descarto un techo de un dia a proposito, porque el silencio total es justo
/// el escenario en el que no se captura nada.
const BACKOFF_TECHO: f64 = 3600.0;
/// Cada cuantas invocaciones se recuerda mirar las lecciones, si la regla no
/// dice otra cosa. 25 y no 10 (OBS-7): a 10 el aviso se vuelve ruido de fondo.
const RECORDATORIO_DEFAULT: i64 = 25;

pub fn run(paths: &HarnessPaths) -> anyhow::Result<()> {
    let _ = inner(paths);
    Ok(())
}

fn inner(paths: &HarnessPaths) -> anyhow::Result<()> {
    let data = load_features(paths)?;
    let active = active_indices(&data);
    if active.is_empty() {
        aviso_sin_feature(paths);
        return Ok(());
    }

    // Hay feature activa: el backoff del aviso vuelve al piso, asi que el
    // proximo periodo sin feature avisa a los 600 s otra vez (AC-13).
    resetear_backoff(paths);

    let Some(feature) = feature_at(&data, active[0]).as_object() else {
        return Ok(());
    };
    // Aviso de plan stale (multi-LLM). Independiente del recordatorio de
    // lecciones: pueden salir los dos en la misma corrida (OBS-5).
    if is_plan_stale(paths, feature) {
        let _ = std::io::stderr().write_all(
            format!(
                "\n[harness] {}\n[harness] Antes de implementar mas cambios, re-lee el plan y ejecuta:\n    sh harness_cli check-plan\n\n",
                plan_staleness_message(paths, feature)
            )
            .as_bytes(),
        );
    }
    recordatorio_de_lecciones(paths, &data, &py_str(feature.get("id")));
    Ok(())
}

// ---------------------------------------------------------------------------
// Aviso "sin feature activa", con backoff adaptativo
// ---------------------------------------------------------------------------

/// Nivel de backoff guardado en `.last_nudge`. Un archivo vacio o ilegible
/// (formato viejo: el stamp no tenia contenido) vale 0 (AC-14).
fn nivel_backoff(paths: &HarnessPaths) -> u32 {
    std::fs::read_to_string(&paths.nudge_stamp)
        .ok()
        .and_then(|s| s.trim().parse::<u32>().ok())
        .unwrap_or(0)
}

fn intervalo(nivel: u32) -> f64 {
    // 600 -> 1200 -> 2400 -> 3600 (techo).
    let escalado = BACKOFF_PISO * 2f64.powi(nivel.min(16) as i32);
    escalado.min(BACKOFF_TECHO)
}

fn aviso_sin_feature(paths: &HarnessPaths) {
    let nivel = nivel_backoff(paths);
    let ultimo = mtime_f64(&paths.nudge_stamp).unwrap_or(0.0);
    // mtime 0 == nunca se aviso: emitimos ya.
    if ultimo > 0.0 && now_epoch_f64() - ultimo < intervalo(nivel) {
        return;
    }
    if std::fs::create_dir_all(&paths.progress).is_err() {
        return;
    }
    let siguiente = if intervalo(nivel) >= BACKOFF_TECHO {
        nivel // ya estacionado en el techo: no crece mas
    } else {
        nivel + 1
    };
    let _ = std::fs::write(&paths.nudge_stamp, format!("{siguiente}\n"));
    let _ = std::io::stderr().write_all(
        concat!(
            "[harness] Sin feature activa: el avance NO se esta capturando ",
            "(autocheck duerme sin una feature in_progress). Antes de seguir, ",
            "consulta graphify, corre impacto y registra el trabajo con ",
            "'harness_cli add' + 'harness_cli start'.\n"
        )
        .as_bytes(),
    );
}

/// Devuelve el backoff al piso. Solo escribe si hacia falta: este camino corre
/// en cada tool-use y no tiene sentido tocar el archivo cada vez.
fn resetear_backoff(paths: &HarnessPaths) {
    if paths.nudge_stamp.exists() && nivel_backoff(paths) != 0 {
        let _ = std::fs::write(&paths.nudge_stamp, "0\n");
    }
}

// ---------------------------------------------------------------------------
// Recordatorio de lecciones, por volumen de trabajo
// ---------------------------------------------------------------------------

/// `rules.leccion_nudge_interval` (default 25; `<= 0` apaga el recordatorio).
fn intervalo_recordatorio(data: &serde_json::Value) -> i64 {
    data.get("rules")
        .and_then(|r| r.get("leccion_nudge_interval"))
        .and_then(serde_json::Value::as_i64)
        .unwrap_or(RECORDATORIO_DEFAULT)
}

fn recordatorio_de_lecciones(paths: &HarnessPaths, data: &serde_json::Value, fid: &str) {
    // Guarda de entrada: un proyecto que no usa lecciones no se entera de que
    // existen, y ni siquiera gana un archivo en progress/ (AC-3, OBS-1).
    if !lecciones::dir(paths).is_dir() {
        return;
    }
    let intervalo = intervalo_recordatorio(data);
    if intervalo <= 0 {
        return;
    }
    // `<id-feature>:<contador>`. Si la feature activa cambio, el contador
    // arranca de cero (AC-4).
    let previo = std::fs::read_to_string(&paths.nudge_lecciones).unwrap_or_default();
    let cuenta = match previo.trim().split_once(':') {
        Some((id, n)) if id == fid => n.parse::<u64>().unwrap_or(0),
        _ => 0,
    } + 1;
    if std::fs::create_dir_all(&paths.progress).is_err() {
        return;
    }
    if cuenta < intervalo as u64 {
        let _ = std::fs::write(&paths.nudge_lecciones, format!("{fid}:{cuenta}\n"));
        return;
    }
    let _ = std::fs::write(&paths.nudge_lecciones, format!("{fid}:0\n"));
    let _ = std::io::stderr().write_all(lecciones::texto_recordatorio(cuenta).as_bytes());
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;
    use serde_json::json;

    #[test]
    fn intervalo_should_double_until_the_ceiling() {
        assert_eq!(intervalo(0), 600.0);
        assert_eq!(intervalo(1), 1200.0);
        assert_eq!(intervalo(2), 2400.0);
        assert_eq!(intervalo(3), 3600.0);
        // Estacionado: no crece mas alla de una hora.
        assert_eq!(intervalo(4), 3600.0);
        assert_eq!(intervalo(99), 3600.0);
    }

    #[test]
    fn nivel_backoff_should_read_zero_from_a_legacy_empty_stamp() {
        let dir = tempfile::tempdir().unwrap();
        let paths = HarnessPaths::from_root(dir.path().to_path_buf());
        std::fs::create_dir_all(&paths.progress).unwrap();
        // Formato viejo: el stamp existia vacio.
        std::fs::write(&paths.nudge_stamp, "").unwrap();
        assert_eq!(nivel_backoff(&paths), 0);
        std::fs::write(&paths.nudge_stamp, "basura").unwrap();
        assert_eq!(nivel_backoff(&paths), 0);
        std::fs::write(&paths.nudge_stamp, "3\n").unwrap();
        assert_eq!(nivel_backoff(&paths), 3);
    }

    #[test]
    fn aviso_should_escalate_and_park_at_the_ceiling() {
        let dir = tempfile::tempdir().unwrap();
        let paths = HarnessPaths::from_root(dir.path().to_path_buf());
        std::fs::create_dir_all(&paths.progress).unwrap();
        // Primer aviso: sin stamp previo, emite y deja nivel 1.
        aviso_sin_feature(&paths);
        assert_eq!(nivel_backoff(&paths), 1);
        // Inmediatamente despues no vuelve a emitir (debounce vigente).
        aviso_sin_feature(&paths);
        assert_eq!(nivel_backoff(&paths), 1);
        // Con el reloj corrido, escala hasta estacionarse en el techo.
        for esperado in [2, 3, 3, 3] {
            let viejo = filetime::FileTime::from_unix_time(0, 0);
            filetime::set_file_mtime(&paths.nudge_stamp, viejo).unwrap();
            aviso_sin_feature(&paths);
            assert_eq!(nivel_backoff(&paths), esperado);
        }
    }

    #[test]
    fn resetear_backoff_should_return_to_the_floor_only_when_needed() {
        let dir = tempfile::tempdir().unwrap();
        let paths = HarnessPaths::from_root(dir.path().to_path_buf());
        std::fs::create_dir_all(&paths.progress).unwrap();
        // Sin stamp no se crea nada (el camino corre en cada tool-use).
        resetear_backoff(&paths);
        assert!(!paths.nudge_stamp.exists());
        std::fs::write(&paths.nudge_stamp, "3\n").unwrap();
        resetear_backoff(&paths);
        assert_eq!(nivel_backoff(&paths), 0);
        // Ya en el piso: no reescribe (mtime intacto).
        let antes = mtime_f64(&paths.nudge_stamp).unwrap();
        resetear_backoff(&paths);
        assert_eq!(mtime_f64(&paths.nudge_stamp).unwrap(), antes);
    }

    /// Sandbox con `docs/lecciones/` presente.
    fn paths_con_lecciones() -> (tempfile::TempDir, HarnessPaths) {
        let dir = tempfile::tempdir().unwrap();
        let paths = HarnessPaths::from_root(dir.path().to_path_buf());
        std::fs::create_dir_all(lecciones::dir(&paths)).unwrap();
        std::fs::create_dir_all(&paths.progress).unwrap();
        (dir, paths)
    }

    #[test]
    fn recordatorio_should_count_and_reset_at_the_interval() {
        let (_d, paths) = paths_con_lecciones();
        let data = json!({"rules": {"leccion_nudge_interval": 3}});
        for esperado in ["7:1", "7:2"] {
            recordatorio_de_lecciones(&paths, &data, "7");
            let leido = std::fs::read_to_string(&paths.nudge_lecciones).unwrap();
            assert_eq!(leido.trim(), esperado);
        }
        // La tercera llega al intervalo: emite y resetea.
        recordatorio_de_lecciones(&paths, &data, "7");
        let leido = std::fs::read_to_string(&paths.nudge_lecciones).unwrap();
        assert_eq!(leido.trim(), "7:0");
    }

    #[test]
    fn recordatorio_should_restart_when_the_active_feature_changes() {
        let (_d, paths) = paths_con_lecciones();
        let data = json!({"rules": {"leccion_nudge_interval": 10}});
        recordatorio_de_lecciones(&paths, &data, "7");
        recordatorio_de_lecciones(&paths, &data, "7");
        assert_eq!(
            std::fs::read_to_string(&paths.nudge_lecciones).unwrap().trim(),
            "7:2"
        );
        recordatorio_de_lecciones(&paths, &data, "8");
        assert_eq!(
            std::fs::read_to_string(&paths.nudge_lecciones).unwrap().trim(),
            "8:1"
        );
    }

    #[test]
    fn recordatorio_should_do_nothing_without_the_lecciones_dir() {
        let dir = tempfile::tempdir().unwrap();
        let paths = HarnessPaths::from_root(dir.path().to_path_buf());
        std::fs::create_dir_all(&paths.progress).unwrap();
        let data = json!({"rules": {"leccion_nudge_interval": 1}});
        recordatorio_de_lecciones(&paths, &data, "7");
        // Ni siquiera se crea el contador (AC-3).
        assert!(!paths.nudge_lecciones.exists());
    }

    #[test]
    fn recordatorio_should_be_switchable_off() {
        let (_d, paths) = paths_con_lecciones();
        for apagado in [json!(0), json!(-1)] {
            let data = json!({"rules": {"leccion_nudge_interval": apagado}});
            recordatorio_de_lecciones(&paths, &data, "7");
            assert!(!paths.nudge_lecciones.exists());
        }
    }

    #[test]
    fn intervalo_recordatorio_should_default_to_25() {
        assert_eq!(intervalo_recordatorio(&json!({})), 25);
        assert_eq!(intervalo_recordatorio(&json!({"rules": {}})), 25);
        assert_eq!(
            intervalo_recordatorio(&json!({"rules": {"leccion_nudge_interval": 3}})),
            3
        );
    }
}
