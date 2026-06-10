//! `harness nudge` (paridad: harness.py cmd_nudge). Best-effort absoluto:
//! cualquier error se traga en silencio y el exit code siempre es 0.

use std::io::Write;

use crate::features::{active_indices, feature_at, load_features};
use crate::paths::HarnessPaths;
use crate::plan::{is_plan_stale, plan_staleness_message};
use crate::pycompat::{mtime_f64, now_epoch_f64};

pub fn run(paths: &HarnessPaths) -> anyhow::Result<()> {
    let _ = inner(paths);
    Ok(())
}

fn inner(paths: &HarnessPaths) -> anyhow::Result<()> {
    let data = load_features(paths)?;
    let active = active_indices(&data);
    if active.is_empty() {
        // Caso sin feature: recordatorio con debounce de 600s.
        let last = mtime_f64(&paths.nudge_stamp).unwrap_or(0.0);
        if now_epoch_f64() - last < 600.0 {
            return Ok(());
        }
        std::fs::create_dir_all(&paths.progress)?;
        std::fs::File::create(&paths.nudge_stamp)?;
        let mut err = std::io::stderr();
        let _ = err.write_all(
            concat!(
                "[harness] Sin feature activa: el avance NO se esta capturando ",
                "(autocheck duerme sin una feature in_progress). Antes de seguir, ",
                "consulta graphify, corre impacto y registra el trabajo con ",
                "'harness.py add' + 'harness.py start'.\n"
            )
            .as_bytes(),
        );
        return Ok(());
    }

    // Hay feature activa: chequeo de frescura del plan (multi-LLM)
    let Some(feature) = feature_at(&data, active[0]).as_object() else {
        return Ok(());
    };
    if is_plan_stale(paths, feature) {
        let mut err = std::io::stderr();
        let _ = err.write_all(
            format!(
                "\n[harness] {}\n[harness] Antes de implementar mas cambios, re-lee el plan y ejecuta:\n    python3 harness.py check-plan\n\n",
                plan_staleness_message(paths, feature)
            )
            .as_bytes(),
        );
    }
    Ok(())
}
