//! `harness check-spec`: gate SDD para harness_check.sh, hooks y roles.
//! Exit codes: 0 = regla apagada (informa) o spec aprobado y fresco;
//! 1 = sin feature in_progress; 2 = spec stale O (regla activa y spec
//! ausente/draft/no aprobado). Solo el USUARIO aprueba (Estado: approved).

use crate::exit::Exit;
use crate::features::{active_feature_index, feature_at, load_features};
use crate::paths::HarnessPaths;
use crate::pycompat::relpath;
use crate::spec::{
    SpecState, is_spec_stale, require_spec_approved, spec_path, spec_staleness_message, spec_state,
};

pub fn run(paths: &HarnessPaths, feature: Option<&str>) -> anyhow::Result<()> {
    let data = load_features(paths)?;
    let idx = active_feature_index(&data, feature)?;
    let Some(feature) = feature_at(&data, idx).as_object() else {
        anyhow::bail!("feature_list.json: feature invalida");
    };
    // Frescura primero: un spec editado por otro LLM invalida cualquier estado
    // hasta re-leerlo y re-firmarlo (start/advance/autocheck).
    if is_spec_stale(paths, feature) {
        println!("{}", spec_staleness_message(paths, feature));
        return Err(Exit::code(2).into());
    }
    let state = spec_state(paths, feature);
    let path = spec_path(paths, feature);
    let rel = relpath(&path, &paths.repo_root).unwrap_or_else(|| path.clone());
    if !require_spec_approved(&data) {
        println!(
            "[spec] Regla require_spec_approved apagada: gate no aplica. Estado del spec: {} ({}).",
            state.label(),
            rel.display()
        );
        return Ok(());
    }
    if state == SpecState::Approved {
        println!("[OK] Spec aprobado y fresco: {}", rel.display());
        return Ok(());
    }
    println!(
        "[GATE] Spec sin aprobar: {} (estado: {}).",
        rel.display(),
        state.label()
    );
    println!(
        "    Completa el spec y pide al USUARIO aprobarlo editando `Estado: approved` (solo el usuario aprueba; los agentes no)."
    );
    Err(Exit::code(2).into())
}
