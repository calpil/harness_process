//! `harness approve-spec --yes`: registra la aprobacion del USUARIO sobre el
//! spec de la feature activa. La DECISION es del usuario y solo del usuario; el
//! agente la REGISTRA despues de mostrarle el spec y recibir su si explicito
//! (Articulo 2 de docs/constitution.md). Sin `--yes` el comando se niega.
//!
//! Ademas re-firma `last_spec_sig`: aprobar cambia el hash del spec y, sin la
//! re-firma, `check-spec` reportaria la aprobacion del propio usuario como
//! "SPEC ACTUALIZADO POR OTRO LLM".
//!
//! Exit codes: 0 = aprobado o ya estaba aprobado; 1 = sin feature in_progress;
//! 2 = sin confirmacion explicita, o spec ausente.

use crate::exit::Exit;
use crate::features::{
    active_feature_index, feature_at, feature_mut, load_features, save_features,
};
use crate::memories::hub_register;
use crate::paths::HarnessPaths;
use crate::progress::{log, now_stamp};
use crate::pycompat::{py_str, relpath};
use crate::spec::{
    ApprovalOutcome, SpecState, approve_spec, spec_path, spec_state, update_spec_sig,
};

pub fn run(
    paths: &HarnessPaths,
    fid: Option<&str>,
    yes: bool,
    nota: Option<&str>,
) -> anyhow::Result<()> {
    let mut data = load_features(paths)?;
    let idx = active_feature_index(&data, fid)?;
    let (feature_id, path, state) = {
        let Some(feature) = feature_at(&data, idx).as_object() else {
            anyhow::bail!("feature_list.json: feature invalida");
        };
        (
            py_str(feature.get("id")),
            spec_path(paths, feature),
            spec_state(paths, feature),
        )
    };
    let rel = relpath(&path, &paths.repo_root).unwrap_or_else(|| path.clone());
    // Barrera: ningun agente aprueba por su cuenta. El flag no es una formalidad,
    // es la traduccion del Articulo 2 a codigo.
    if !yes {
        println!("[GATE] approve-spec exige la confirmacion explicita del USUARIO.");
        println!(
            "    1) Mostrale el spec ({}) en el chat y abriselo en su editor.",
            rel.display()
        );
        println!("    2) Preguntale si lo aprueba.");
        println!("    3) Solo con su SI: sh harness_cli approve-spec --yes");
        return Err(Exit::code(2).into());
    }
    if state == SpecState::Missing {
        println!("[GATE] No existe el spec: {}.", rel.display());
        println!("    Sembralo con: sh harness_cli start --feature {feature_id}");
        return Err(Exit::code(2).into());
    }
    let stamp = now_stamp();
    let nota = nota.unwrap_or_default();
    let outcome = {
        let Some(feature) = feature_at(&data, idx).as_object() else {
            anyhow::bail!("feature_list.json: feature invalida");
        };
        approve_spec(paths, feature, &stamp, nota)?
    };
    // Re-firma SIEMPRE, incluso si ya estaba aprobado: cubre el caso del usuario
    // que edito `Estado: approved` a mano y quedo con la falsa alarma pendiente.
    {
        let feature = feature_mut(&mut data, idx)?;
        update_spec_sig(paths, feature);
    }
    save_features(paths, &data)?;
    match outcome {
        ApprovalOutcome::Registered => {
            println!(
                "[OK] Aprobacion del USUARIO registrada: {} (Estado: approved).",
                rel.display()
            );
            println!("    Sello: {}", crate::spec::approval_stamp_line(&stamp, nota));
        }
        ApprovalOutcome::AlreadyApproved => {
            println!(
                "[OK] El spec ya estaba aprobado: {} (sello no duplicado).",
                rel.display()
            );
        }
    }
    println!("    Firma del spec actualizada: check-spec sale limpio (sin falsa alarma multi-LLM).");
    log(
        paths,
        &format!("approve-spec feature #{feature_id} estado=approved nota={nota}"),
    )?;
    // Feature #15: la aprobacion del USUARIO tambien queda del otro lado (AC-8).
    if let Some(feature) = feature_at(&data, idx).as_object() {
        crate::atlassian::emit::on_approve_spec(
            paths,
            feature,
            &crate::spec::approval_stamp_line(&stamp, nota),
        );
    }
    crate::atlassian::push::push_bg(paths);
    // Hub best-effort (nunca bloquea la aprobacion): mismo criterio que advance.
    hub_register(
        "approve-spec",
        "approved",
        &format!("feature-{feature_id}"),
        nota,
    );
    Ok(())
}
