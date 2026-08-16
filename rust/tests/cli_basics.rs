//! Tests de integracion del binario: exit codes y textos exactos de los
//! comandos de solo archivo (el gate fuerte cross-implementacion es
//! tests/parity_smoke.sh, con Python como oraculo).
#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};
use std::process::Command;

use assert_cmd::prelude::*;
use predicates::prelude::*;

/// Copia el binario al sandbox para que ROOT (dir del exe) sea el sandbox,
/// igual que `dirname(abspath(__file__))` en harness.py.
fn sandbox_with_binary() -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let harness_dir = dir.path().join("hp");
    std::fs::create_dir_all(&harness_dir).unwrap();
    std::fs::write(harness_dir.join(".harness_layout"), "subdir").unwrap();
    let built = assert_cmd::cargo::cargo_bin("harness");
    let target = harness_dir.join(if cfg!(windows) { "harness.exe" } else { "harness" });
    std::fs::copy(&built, &target).unwrap();
    (dir, target)
}

fn cmd(bin: &Path) -> Command {
    let mut c = Command::new(bin);
    // DB_* fuera: el registro al hub debe degradar con el mensaje best-effort
    for var in ["DB_HOST", "DB_USER", "DB_PASSWORD", "HARNESS_REPO_ROOT", "HARNESS_HUB"] {
        c.env_remove(var);
    }
    c.env("HARNESS_HUB", bin.parent().unwrap().join("hub"));
    c
}

/// Fixture "checkout FUENTE del arnes": marker subdir + senales de fuente
/// (templates/harness_cli + rust/) y padre SIN huella de instalacion. La
/// resolucion debe quedarse en el propio checkout (feature #7 / AC-8).
fn sandbox_source_checkout() -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let harness_dir = dir.path().join("hp");
    std::fs::create_dir_all(harness_dir.join("templates")).unwrap();
    std::fs::create_dir_all(harness_dir.join("rust")).unwrap();
    std::fs::write(harness_dir.join("templates/harness_cli"), "#!/bin/sh\n").unwrap();
    std::fs::write(harness_dir.join(".harness_layout"), "subdir").unwrap();
    let built = assert_cmd::cargo::cargo_bin("harness");
    let target = harness_dir.join(if cfg!(windows) { "harness.exe" } else { "harness" });
    std::fs::copy(&built, &target).unwrap();
    (dir, target)
}

/// Fixture "instalacion subdir que perdio el marker" (feature #10): el arnes
/// vive en `<raiz>/hp` SIN `.harness_layout` -el estado en que queda cualquier
/// instalacion que hizo `git pull` tras la feature #7- y la raiz tiene huella
/// de instalacion (`docs/constitution.md` + `CLAUDE.md`).
fn sandbox_lost_marker_install(with_footprint: bool) -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let harness_dir = dir.path().join("hp");
    std::fs::create_dir_all(&harness_dir).unwrap();
    if with_footprint {
        std::fs::create_dir_all(dir.path().join("docs")).unwrap();
        std::fs::write(dir.path().join("docs/constitution.md"), "# constitution\n").unwrap();
        std::fs::write(dir.path().join("CLAUDE.md"), "# proyecto\n").unwrap();
    }
    assert!(!harness_dir.join(".harness_layout").exists());
    let built = assert_cmd::cargo::cargo_bin("harness");
    let target = harness_dir.join(if cfg!(windows) { "harness.exe" } else { "harness" });
    std::fs::copy(&built, &target).unwrap();
    (dir, target)
}

#[test]
fn start_should_infer_subdir_root_when_marker_is_missing() {
    // Feature #10 / AC-1 + AC-2: sin marker y con huella en el padre, los
    // artefactos van al docs/ del PROYECTO (no a <arnes>/docs) y se avisa.
    let (dir, bin) = sandbox_lost_marker_install(true);
    cmd(&bin).args(["add", "--name", "Demo"]).assert().success();
    cmd(&bin)
        .args(["start", "--feature", "1"])
        .assert()
        .success()
        .stderr(predicate::str::contains(
            "[i] .harness_layout ausente: layout subdir inferido por la huella de instalacion del padre",
        ))
        .stderr(predicate::str::contains("para regenerar el marker"));
    assert!(dir.path().join("docs/plan-feature-1-demo.md").exists());
    assert!(dir.path().join("docs/spec-feature-1-demo.md").exists());
    assert!(!dir.path().join("hp/docs").exists());
}

#[test]
fn missing_marker_without_footprint_should_stay_local() {
    // Feature #10 / AC-4: sin marker y sin huella no se infiere nada; la raiz
    // es el propio dir del arnes y no hay aviso.
    let (dir, bin) = sandbox_lost_marker_install(false);
    cmd(&bin).args(["add", "--name", "Demo"]).assert().success();
    cmd(&bin)
        .args(["start", "--feature", "1"])
        .assert()
        .success()
        .stderr(predicate::str::contains(".harness_layout ausente").not());
    assert!(dir.path().join("hp/docs/plan-feature-1-demo.md").exists());
    assert!(!dir.path().join("docs").exists());
}

#[test]
fn explicit_root_marker_should_never_infer_subdir() {
    // Feature #10 / AC-3: con un marker EXPLICITO ('root') no hay inferencia
    // aunque el padre tenga huella; la raiz es el dir del arnes, sin aviso.
    let (dir, bin) = sandbox_lost_marker_install(true);
    std::fs::write(dir.path().join("hp/.harness_layout"), "root\n").unwrap();
    cmd(&bin).args(["add", "--name", "Demo"]).assert().success();
    cmd(&bin)
        .args(["start", "--feature", "1"])
        .assert()
        .success()
        .stderr(predicate::str::contains(".harness_layout ausente").not());
    assert!(dir.path().join("hp/docs/plan-feature-1-demo.md").exists());
    assert!(!dir.path().join("docs/plan-feature-1-demo.md").exists());
}

#[test]
fn home_parent_should_block_marker_inference() {
    // Feature #10 / AC-5: la guarda de $HOME de la feature #7 aplica tambien a
    // la inferencia; con el escape explicito, la huella vuelve a mandar.
    let (dir, bin) = sandbox_lost_marker_install(true);
    cmd(&bin)
        .env("HOME", dir.path())
        .env("USERPROFILE", dir.path())
        .arg("status")
        .assert()
        .success()
        .stderr(predicate::str::contains(".harness_layout ausente").not());
    cmd(&bin)
        .env("HOME", dir.path())
        .env("USERPROFILE", dir.path())
        .env("HARNESS_ALLOW_HOME_SURFACE", "1")
        .arg("status")
        .assert()
        .success()
        .stderr(predicate::str::contains(".harness_layout ausente"));
}

#[test]
fn env_override_should_beat_marker_inference() {
    // Feature #10 / AC-6: HARNESS_REPO_ROOT manda sobre la inferencia y el
    // aviso [i] no aparece.
    let (dir, bin) = sandbox_lost_marker_install(true);
    let target_root = dir.path().join("otra-raiz");
    std::fs::create_dir_all(&target_root).unwrap();
    cmd(&bin)
        .env("HARNESS_REPO_ROOT", &target_root)
        .args(["add", "--name", "Demo"])
        .assert()
        .success();
    cmd(&bin)
        .env("HARNESS_REPO_ROOT", &target_root)
        .args(["start", "--feature", "1"])
        .assert()
        .success()
        .stderr(predicate::str::contains(".harness_layout ausente").not());
    assert!(target_root.join("docs/plan-feature-1-demo.md").exists());
    assert!(!dir.path().join("docs/plan-feature-1-demo.md").exists());
}

#[test]
fn status_should_print_empty_backlog() {
    let (_dir, bin) = sandbox_with_binary();
    cmd(&bin).arg("status").assert().success().stdout(
        "Backlog: 0 feature(s) | active=0 pending=0 blocked=0 done=0\n",
    );
}

#[test]
fn next_should_report_no_pending_features() {
    let (_dir, bin) = sandbox_with_binary();
    cmd(&bin)
        .arg("next")
        .assert()
        .success()
        .stdout("No hay features pending.\n");
}

#[test]
fn add_should_create_feature_and_next_should_print_python_style_json() {
    let (_dir, bin) = sandbox_with_binary();
    cmd(&bin)
        .args(["add", "--name", "Pago QR", "--service", "demo/ms-pagos-service"])
        .assert()
        .success()
        .stdout("Feature #1 agregada.\n");
    let expected = "{\n  \"id\": 1,\n  \"name\": \"Pago QR\",\n  \"microservicios\": [\n    \"demo/ms-pagos-service\"\n  ],\n  \"acceptance\": [],\n  \"status\": \"pending\"\n}\n";
    cmd(&bin).arg("next").assert().success().stdout(expected);
}

#[test]
fn check_plan_should_exit_one_without_active_feature() {
    let (_dir, bin) = sandbox_with_binary();
    cmd(&bin)
        .arg("check-plan")
        .assert()
        .code(1)
        .stderr("No hay feature in_progress. Inicia una: harness.py start --feature <id>\n");
}

#[test]
fn close_should_reject_invalid_status_with_usage_exit_two() {
    let (_dir, bin) = sandbox_with_binary();
    cmd(&bin)
        .args(["close", "--feature", "1", "--status", "bogus"])
        .assert()
        .code(2);
}

#[test]
fn start_should_create_plan_and_spec_sign_both_and_check_plan_should_pass() {
    let (dir, bin) = sandbox_with_binary();
    cmd(&bin)
        .args(["add", "--name", "Pago QR"])
        .assert()
        .success();
    cmd(&bin)
        .args(["start", "--feature", "1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Feature #1 iniciada. Plan: docs/plan-feature-1-pago-qr.md"))
        .stdout(predicate::str::contains("Spec (draft) generado: docs/spec-feature-1-pago-qr.md"))
        .stderr(predicate::str::contains("El Memory Hub PostgreSQL requiere: DB_HOST, DB_USER, DB_PASSWORD"));
    assert!(dir.path().join("docs/plan-feature-1-pago-qr.md").exists());
    // AC-1: el spec nace plano junto al plan y en draft
    let spec = std::fs::read_to_string(dir.path().join("docs/spec-feature-1-pago-qr.md")).unwrap();
    assert!(spec.contains("Estado: draft"));
    assert!(spec.contains("## Criterios de aceptacion (Given/When/Then)"));
    // current.md referencia el spec ademas del plan
    let current = std::fs::read_to_string(dir.path().join("hp/progress/current.md")).unwrap();
    assert!(current.contains("Plan: docs/plan-feature-1-pago-qr.md\nSpec: docs/spec-feature-1-pago-qr.md\n"));
    cmd(&bin)
        .arg("check-plan")
        .assert()
        .success()
        .stdout("Plan fresco (sin cambios desde la ultima firma registrada).\nSpec fresco (sin cambios desde la ultima firma registrada).\n[spec] Estado: draft\n[OK] Plan fresco para implementacion.\n");
}

#[test]
fn check_plan_should_exit_two_when_plan_edited_by_another_agent() {
    let (dir, bin) = sandbox_with_binary();
    cmd(&bin).args(["add", "--name", "Demo"]).assert().success();
    cmd(&bin).args(["start", "--feature", "1"]).assert().success();
    let plan = dir.path().join("docs/plan-feature-1-demo.md");
    // Otro LLM edita el plan y el mtime queda claramente fuera de tolerancia
    let mut content = std::fs::read_to_string(&plan).unwrap();
    content.push_str("\n## Cambio de otro agente\n");
    std::fs::write(&plan, content).unwrap();
    let past = filetime::FileTime::from_unix_time(1_700_000_000, 0);
    filetime::set_file_mtime(&plan, past).unwrap();
    cmd(&bin)
        .arg("check-plan")
        .assert()
        .code(2)
        .stdout(predicate::str::contains("PLAN ACTUALIZADO POR OTRO LLM"));
}

#[test]
fn start_should_reject_second_in_progress_feature() {
    let (_dir, bin) = sandbox_with_binary();
    cmd(&bin).args(["add", "--name", "Uno"]).assert().success();
    cmd(&bin).args(["add", "--name", "Dos"]).assert().success();
    cmd(&bin).args(["start", "--feature", "1"]).assert().success();
    cmd(&bin)
        .args(["start", "--feature", "2"])
        .assert()
        .code(1)
        .stderr("Ya hay feature in_progress: #1 Uno\n");
}

/// Activa la regla require_spec_approved en el feature_list.json del sandbox
/// (el gate SDD es opt-in: add/start no la escriben solos).
fn enable_spec_rule(harness_dir: &Path) {
    let path = harness_dir.join("feature_list.json");
    let text = std::fs::read_to_string(&path).unwrap();
    let mut data: serde_json::Value = serde_json::from_str(&text).unwrap();
    data.as_object_mut().unwrap().insert(
        "rules".to_string(),
        serde_json::json!({"require_spec_approved": true}),
    );
    std::fs::write(&path, serde_json::to_string_pretty(&data).unwrap()).unwrap();
}

#[test]
fn check_spec_should_exit_one_without_active_feature() {
    let (_dir, bin) = sandbox_with_binary();
    cmd(&bin)
        .arg("check-spec")
        .assert()
        .code(1)
        .stderr("No hay feature in_progress. Inicia una: harness.py start --feature <id>\n");
}

#[test]
fn check_spec_should_pass_informing_when_rule_is_off() {
    let (_dir, bin) = sandbox_with_binary();
    cmd(&bin).args(["add", "--name", "Demo"]).assert().success();
    cmd(&bin).args(["start", "--feature", "1"]).assert().success();
    // Sin la regla (compat instalaciones previas): rc=0 pero informa el estado
    cmd(&bin)
        .arg("check-spec")
        .assert()
        .success()
        .stdout(predicate::str::contains("Regla require_spec_approved apagada"))
        .stdout(predicate::str::contains("draft"));
}

#[test]
fn check_plan_should_exit_two_when_spec_edited_by_another_agent() {
    let (dir, bin) = sandbox_with_binary();
    cmd(&bin).args(["add", "--name", "Demo"]).assert().success();
    cmd(&bin).args(["start", "--feature", "1"]).assert().success();
    let spec = dir.path().join("docs/spec-feature-1-demo.md");
    // Otro LLM edita el spec; el plan sigue fresco: stdout distingue cual fue
    let mut content = std::fs::read_to_string(&spec).unwrap();
    content.push_str("\n## Cambio de otro agente\n");
    std::fs::write(&spec, content).unwrap();
    let past = filetime::FileTime::from_unix_time(1_700_000_000, 0);
    filetime::set_file_mtime(&spec, past).unwrap();
    cmd(&bin)
        .arg("check-plan")
        .assert()
        .code(2)
        .stdout(predicate::str::contains("Plan fresco"))
        .stdout(predicate::str::contains("SPEC ACTUALIZADO POR OTRO LLM"));
    cmd(&bin)
        .arg("check-spec")
        .assert()
        .code(2)
        .stdout(predicate::str::contains("SPEC ACTUALIZADO POR OTRO LLM"));
}

#[test]
fn spec_gate_should_block_advance_and_close_done_until_user_approves() {
    let (dir, bin) = sandbox_with_binary();
    cmd(&bin).args(["add", "--name", "Demo"]).assert().success();
    cmd(&bin).args(["start", "--feature", "1"]).assert().success();
    enable_spec_rule(&dir.path().join("hp"));
    // Spec draft + regla activa: advance y close --status done bloquean
    cmd(&bin)
        .args(["advance", "--nota", "intento", "--no-graphify"])
        .assert()
        .code(1)
        .stderr(predicate::str::contains("[GATE] Spec sin aprobar"));
    cmd(&bin)
        .args(["close", "--feature", "1", "--status", "done", "--note", "x"])
        .assert()
        .code(1)
        .stderr(predicate::str::contains("[GATE] Spec sin aprobar"));
    cmd(&bin)
        .arg("check-spec")
        .assert()
        .code(2)
        .stdout(predicate::str::contains("[GATE] Spec sin aprobar"));
    // El usuario aprueba (draft -> approved): advance pasa y re-firma el spec
    let spec = dir.path().join("docs/spec-feature-1-demo.md");
    let approved = std::fs::read_to_string(&spec)
        .unwrap()
        .replace("Estado: draft", "Estado: approved");
    std::fs::write(&spec, approved).unwrap();
    cmd(&bin)
        .args(["advance", "--nota", "ok", "--no-graphify"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Avance registrado en feature #1"));
    cmd(&bin)
        .arg("check-spec")
        .assert()
        .success()
        .stdout(predicate::str::contains("[OK] Spec aprobado y fresco"));
    cmd(&bin)
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains("[spec] #1 approved (fresco)"));
}

#[test]
fn approve_spec_should_refuse_without_explicit_user_confirmation() {
    // AC-3: la barrera del Articulo 2 en codigo. Sin --yes no hay aprobacion,
    // y el spec queda intacto en draft.
    let (dir, bin) = sandbox_with_binary();
    cmd(&bin).args(["add", "--name", "Demo"]).assert().success();
    cmd(&bin).args(["start", "--feature", "1"]).assert().success();
    enable_spec_rule(&dir.path().join("hp"));
    cmd(&bin)
        .arg("approve-spec")
        .assert()
        .code(2)
        .stdout(predicate::str::contains(
            "[GATE] approve-spec exige la confirmacion explicita del USUARIO.",
        ))
        .stdout(predicate::str::contains(
            "sh harness_cli approve-spec --yes",
        ));
    let spec = std::fs::read_to_string(dir.path().join("docs/spec-feature-1-demo.md")).unwrap();
    assert!(spec.contains("Estado: draft"));
    assert!(!spec.contains("Aprobado:"));
}

#[test]
fn approve_spec_should_register_approval_and_leave_check_spec_clean() {
    // AC-1 + AC-2 + AC-6: el agente registra el si del usuario, el spec queda
    // approved con sello, y check-spec NO reporta la falsa alarma multi-LLM.
    let (dir, bin) = sandbox_with_binary();
    cmd(&bin).args(["add", "--name", "Demo"]).assert().success();
    cmd(&bin).args(["start", "--feature", "1"]).assert().success();
    enable_spec_rule(&dir.path().join("hp"));
    cmd(&bin)
        .args(["approve-spec", "--yes", "--nota", "aprobado en chat"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "[OK] Aprobacion del USUARIO registrada: docs/spec-feature-1-demo.md (Estado: approved).",
        ))
        .stdout(predicate::str::contains("Firma del spec actualizada"));
    let spec = std::fs::read_to_string(dir.path().join("docs/spec-feature-1-demo.md")).unwrap();
    assert!(spec.contains("Estado: approved"));
    assert!(spec.contains("por USUARIO (confirmacion explicita) - aprobado en chat"));
    // AC-2: el gate sale limpio inmediatamente despues, sin advance de por medio
    cmd(&bin)
        .arg("check-spec")
        .assert()
        .success()
        .stdout(predicate::str::contains("[OK] Spec aprobado y fresco"));
    // AC-6: rastro en la bitacora append-only
    let history = std::fs::read_to_string(dir.path().join("hp/progress/history.md")).unwrap();
    assert!(history.contains("approve-spec feature #1 estado=approved nota=aprobado en chat"));
    // El gate de implementacion queda abierto: advance ya no bloquea
    cmd(&bin)
        .args(["advance", "--nota", "ok", "--no-graphify"])
        .assert()
        .success();
}

#[test]
fn approve_spec_should_be_idempotent() {
    // AC-4: re-aprobar informa y no duplica el sello.
    let (dir, bin) = sandbox_with_binary();
    cmd(&bin).args(["add", "--name", "Demo"]).assert().success();
    cmd(&bin).args(["start", "--feature", "1"]).assert().success();
    cmd(&bin).args(["approve-spec", "--yes"]).assert().success();
    cmd(&bin)
        .args(["approve-spec", "--yes"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "[OK] El spec ya estaba aprobado: docs/spec-feature-1-demo.md (sello no duplicado).",
        ));
    let spec = std::fs::read_to_string(dir.path().join("docs/spec-feature-1-demo.md")).unwrap();
    assert_eq!(spec.matches("Aprobado: ").count(), 1);
}

#[test]
fn approve_spec_should_exit_one_without_active_feature_and_two_without_spec() {
    // AC-5: mismos exit codes que check-spec (1 sin feature, 2 sin spec).
    let (dir, bin) = sandbox_with_binary();
    cmd(&bin)
        .args(["approve-spec", "--yes"])
        .assert()
        .code(1)
        .stderr(predicate::str::contains("No hay feature in_progress"));
    cmd(&bin).args(["add", "--name", "Demo"]).assert().success();
    cmd(&bin).args(["start", "--feature", "1"]).assert().success();
    std::fs::remove_file(dir.path().join("docs/spec-feature-1-demo.md")).unwrap();
    cmd(&bin)
        .args(["approve-spec", "--yes"])
        .assert()
        .code(2)
        .stdout(predicate::str::contains(
            "[GATE] No existe el spec: docs/spec-feature-1-demo.md.",
        ))
        .stdout(predicate::str::contains("start --feature 1"));
}

#[test]
fn approve_spec_should_resign_a_spec_approved_by_hand() {
    // Rescate del flujo viejo: si el usuario ya edito `Estado: approved` a mano,
    // el spec queda stale; approve-spec --yes re-firma y limpia la falsa alarma.
    let (dir, bin) = sandbox_with_binary();
    cmd(&bin).args(["add", "--name", "Demo"]).assert().success();
    cmd(&bin).args(["start", "--feature", "1"]).assert().success();
    enable_spec_rule(&dir.path().join("hp"));
    let spec = dir.path().join("docs/spec-feature-1-demo.md");
    let approved = std::fs::read_to_string(&spec)
        .unwrap()
        .replace("Estado: draft", "Estado: approved");
    std::fs::write(&spec, approved).unwrap();
    let past = filetime::FileTime::from_unix_time(1_700_000_000, 0);
    filetime::set_file_mtime(&spec, past).unwrap();
    cmd(&bin)
        .arg("check-spec")
        .assert()
        .code(2)
        .stdout(predicate::str::contains("SPEC ACTUALIZADO POR OTRO LLM"));
    cmd(&bin)
        .args(["approve-spec", "--yes"])
        .assert()
        .success()
        .stdout(predicate::str::contains("ya estaba aprobado"));
    cmd(&bin)
        .arg("check-spec")
        .assert()
        .success()
        .stdout(predicate::str::contains("[OK] Spec aprobado y fresco"));
}

#[test]
fn close_blocked_should_pass_without_approved_spec() {
    let (dir, bin) = sandbox_with_binary();
    cmd(&bin).args(["add", "--name", "Demo"]).assert().success();
    cmd(&bin).args(["start", "--feature", "1"]).assert().success();
    enable_spec_rule(&dir.path().join("hp"));
    // Valvula de escape: blocked/pending no exigen spec aprobado
    cmd(&bin)
        .args(["close", "--feature", "1", "--status", "blocked", "--note", "aparcada"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Feature #1 cerrada como blocked."));
}

#[test]
fn start_should_stay_inside_source_checkout_and_not_touch_parent() {
    // Feature #7 / AC-8: en un checkout fuente (marker subdir incoherente con
    // el entorno) los artefactos de start van a <checkout>/docs/ y el padre
    // (que hacia de $HOME en el incidente real) queda intacto.
    let (dir, bin) = sandbox_source_checkout();
    cmd(&bin).args(["add", "--name", "Demo"]).assert().success();
    cmd(&bin)
        .args(["start", "--feature", "1"])
        .assert()
        .success()
        .stderr(predicate::str::contains("[i] Checkout fuente del arnes detectado"));
    assert!(dir.path().join("hp/docs/plan-feature-1-demo.md").exists());
    assert!(dir.path().join("hp/docs/spec-feature-1-demo.md").exists());
    assert!(!dir.path().join("docs").exists());
}

#[test]
fn env_override_should_beat_source_checkout_guardrail() {
    // Feature #7 / AC-9: HARNESS_REPO_ROOT sigue mandando sobre cualquier
    // deteccion; con el override, los artefactos van a la raiz indicada.
    let (dir, bin) = sandbox_source_checkout();
    cmd(&bin)
        .env("HARNESS_REPO_ROOT", dir.path())
        .args(["add", "--name", "Demo"])
        .assert()
        .success();
    cmd(&bin)
        .env("HARNESS_REPO_ROOT", dir.path())
        .args(["start", "--feature", "1"])
        .assert()
        .success()
        .stderr(predicate::str::contains("[i] Checkout fuente").not());
    assert!(dir.path().join("docs/plan-feature-1-demo.md").exists());
    assert!(!dir.path().join("hp/docs").exists());
}

#[test]
fn home_parent_should_trigger_source_guardrail_even_with_footprint() {
    // Feature #7 / AC-6 (regla $HOME): si el padre ES $HOME, la huella que
    // haya ahi (~/CLAUDE.md, ~/.claude/settings.json) no lo convierte en raiz
    // de instalacion; sin HARNESS_ALLOW_HOME_SURFACE=1 se resuelve local.
    let (dir, bin) = sandbox_source_checkout();
    std::fs::write(dir.path().join("CLAUDE.md"), "# global del usuario\n").unwrap();
    cmd(&bin)
        .env("HOME", dir.path())
        .env("USERPROFILE", dir.path())
        .arg("status")
        .assert()
        .success()
        .stderr(predicate::str::contains("[i] Checkout fuente del arnes detectado"));
    // Con el escape explicito, la misma huella vuelve a mandar (padre = raiz).
    cmd(&bin)
        .env("HOME", dir.path())
        .env("USERPROFILE", dir.path())
        .env("HARNESS_ALLOW_HOME_SURFACE", "1")
        .arg("status")
        .assert()
        .success()
        .stderr(predicate::str::contains("[i] Checkout fuente").not());
}

#[test]
fn close_should_archive_current_state_and_reset_it() {
    let (dir, bin) = sandbox_with_binary();
    cmd(&bin).args(["add", "--name", "Demo"]).assert().success();
    cmd(&bin).args(["start", "--feature", "1"]).assert().success();
    cmd(&bin)
        .args(["close", "--feature", "1", "--status", "done", "--note", "ok"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Feature #1 cerrada como done. Estado archivado en docs/estado-feature-1-demo.md.",
        ));
    let current = std::fs::read_to_string(dir.path().join("hp/progress/current.md")).unwrap();
    assert!(current.starts_with("# Estado Actual\n\nSin feature activa.\n"));
    assert!(dir.path().join("docs/estado-feature-1-demo.md").exists());
}

// ---------------------------------------------------------------------------
// Feature #15: binding con Atlassian, outbox y ejecutor con agente MCP.
// ---------------------------------------------------------------------------

/// Escribe un binding activo en la raiz del sandbox (lo que dejaria el
/// instalador con --atlassian-site/--jira-project/--confluence-space).
fn write_binding(root: &Path, project: &str) {
    std::fs::write(
        root.join("atlassian.json"),
        format!(
            r#"{{"site":"calpil.atlassian.net","enabled":true,"jira":{{"project_key":"{project}"}},"confluence":{{"space_key":"SD"}}}}"#
        ),
    )
    .unwrap();
}

#[test]
fn atlassian_should_stay_invisible_without_binding() {
    // AC-4: sin binding el flujo se comporta exactamente como hoy y no crea
    // ni la carpeta de la outbox.
    let (dir, bin) = sandbox_with_binary();
    cmd(&bin).args(["add", "--name", "Demo"]).assert().success();
    cmd(&bin)
        .args(["start", "--feature", "1"])
        .assert()
        .success();
    cmd(&bin)
        .args(["advance", "--nota", "un avance"])
        .assert()
        .success();
    assert!(
        !dir.path().join("hp/progress/atlassian").exists(),
        "sin binding no se escribe nada de Atlassian"
    );
    // Y `status` lo dice sin fallar.
    cmd(&bin)
        .args(["atlassian", "status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("no tiene binding"));
}

#[test]
fn atlassian_bind_should_refuse_to_guess_the_project() {
    // AC-5: sin proyecto el comando se niega con exit 2 y dice que preguntar.
    let (_dir, bin) = sandbox_with_binary();
    cmd(&bin)
        .args(["atlassian", "bind", "--site", "calpil.atlassian.net"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("no lo voy a adivinar"))
        .stderr(predicate::str::contains("Preguntale al USUARIO"));
}

#[test]
fn atlassian_bind_should_write_the_binding_and_status_should_show_it() {
    // AC-1 + AC-12 por la via del CLI (la del instalador la cubre el smoke).
    let (dir, bin) = sandbox_with_binary();
    cmd(&bin)
        .args([
            "atlassian",
            "bind",
            "--site",
            "calpil.atlassian.net",
            "--jira-project",
            "ADR",
            "--confluence-space",
            "SD",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("binding registrado"));
    let written = std::fs::read_to_string(dir.path().join("atlassian.json")).unwrap();
    assert!(written.contains("\"project_key\": \"ADR\""));
    // Decision OBS-6: Story por default.
    assert!(written.contains("\"feature\": \"Story\""));

    cmd(&bin)
        .args(["atlassian", "status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("ADR"))
        // AC-16: del token solo se informa presencia.
        .stdout(predicate::str::contains("Token      : ausente"));
}

#[test]
fn flow_should_emit_intents_and_drain_should_plan_them_in_order() {
    // AC-6 + AC-7 + AC-9: add deja epic + historia, start suma las subtasks de
    // los AC-n del spec, y drain los ordena por dependencia sin mutar nada.
    let (dir, bin) = sandbox_with_binary();
    write_binding(dir.path(), "ADR");

    cmd(&bin)
        .args(["add", "--name", "Demo", "--acceptance", "algo verificable"])
        .assert()
        .success();
    cmd(&bin)
        .args(["start", "--feature", "1"])
        .assert()
        .success();
    // El spec nace como plantilla: los AC-n reales los escribe el lider antes
    // de pedir la aprobacion, y es ahi donde bajan como subtasks (AC-7).
    let spec = dir.path().join("docs/spec-feature-1-demo.md");
    let text = std::fs::read_to_string(&spec).unwrap();
    std::fs::write(
        &spec,
        text.replace(
            "- AC-1: Given <contexto>, When <accion>, Then <resultado observable>.",
            "- AC-1: Given un repo, When corro add, Then queda el intent.\n- AC-2: Given el intent, When corro drain, Then aparece en el plan.",
        ),
    )
    .unwrap();
    cmd(&bin)
        .args(["approve-spec", "--yes", "--nota", "aprobado en el test"])
        .assert()
        .success();

    let outbox = dir.path().join("hp/progress/atlassian/outbox");
    assert!(outbox.is_dir(), "la outbox existe con binding activo");

    let out = cmd(&bin).args(["atlassian", "drain"]).output().unwrap();
    assert!(out.status.success());
    let plan: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let kinds: Vec<&str> = plan["plan"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["call"]["tool"].as_str().unwrap())
        .collect();
    assert!(!kinds.is_empty(), "hay llamadas planificadas");
    // El primero siempre es el epic del PRD (rank 0), y las subtasks de los
    // AC-n van despues de la historia.
    let whats: Vec<&str> = plan["plan"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["what"].as_str().unwrap())
        .collect();
    assert_eq!(whats[0], "epic del PRD master");
    let historia = whats.iter().position(|w| w.starts_with("historia")).unwrap();
    let ac1 = whats.iter().position(|w| w.contains("AC-1")).unwrap();
    let ac2 = whats.iter().position(|w| w.contains("AC-2")).unwrap();
    assert!(historia < ac1 && historia < ac2, "las subtasks van despues de su historia: {whats:?}");
    assert_eq!(plan["project"].as_str().unwrap(), "ADR");
    // AC-9: drain NO muta (los intents siguen pendientes).
    let after = cmd(&bin).args(["atlassian", "drain"]).output().unwrap();
    let plan2: serde_json::Value = serde_json::from_slice(&after.stdout).unwrap();
    assert_eq!(plan["pending"], plan2["pending"]);
}

#[test]
fn ack_should_record_the_key_and_dedupe_the_next_run() {
    // AC-10 + AC-11: la clave vuelve al state, el intent se archiva y el mismo
    // comando del flujo no vuelve a emitirlo.
    let (dir, bin) = sandbox_with_binary();
    write_binding(dir.path(), "ADR");
    cmd(&bin).args(["add", "--name", "Demo"]).assert().success();

    let out = cmd(&bin).args(["atlassian", "drain"]).output().unwrap();
    let plan: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let first = plan["plan"][0]["intent"].as_str().unwrap().to_string();

    cmd(&bin)
        .args(["atlassian", "ack", "--intent", &first, "--key", "ADR-1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("ADR-1"));

    let state: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(dir.path().join("hp/progress/atlassian/state.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(state["prds"]["master"].as_str().unwrap(), "ADR-1");

    // El intent ya no aparece en drain y quedo archivado (no borrado).
    let out2 = cmd(&bin).args(["atlassian", "drain"]).output().unwrap();
    let plan2: serde_json::Value = serde_json::from_slice(&out2.stdout).unwrap();
    let ids: Vec<&str> = plan2["plan"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["intent"].as_str().unwrap())
        .collect();
    assert!(!ids.contains(&first.as_str()));
    assert!(dir.path().join("hp/progress/atlassian/applied").is_dir());

    // Un ack repetido es inofensivo (idempotencia del AC-10).
    cmd(&bin)
        .args(["atlassian", "ack", "--intent", &first, "--key", "ADR-1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("ya no esta pendiente"));
}

#[test]
fn apply_should_refuse_without_token_and_point_to_the_agent_route() {
    // AC-18: sin credenciales, `apply` sale con 2 y nombra la alternativa.
    let (dir, bin) = sandbox_with_binary();
    write_binding(dir.path(), "ADR");
    cmd(&bin).args(["add", "--name", "Demo"]).assert().success();
    cmd(&bin)
        .args(["atlassian", "apply"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("atlassian drain"))
        .stderr(predicate::str::contains("HARNESS_ATLASSIAN_TOKEN"));
}

#[test]
fn close_should_emit_transition_and_comment() {
    // AC-8: cerrar deja la transicion al estado final y la nota como comentario.
    let (dir, bin) = sandbox_with_binary();
    write_binding(dir.path(), "ADR");
    cmd(&bin).args(["add", "--name", "Demo"]).assert().success();
    cmd(&bin)
        .args(["close", "--feature", "1", "--status", "blocked", "--note", "trabada"])
        .assert()
        .success();

    let out = cmd(&bin).args(["atlassian", "drain"]).output().unwrap();
    let plan: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let text = plan.to_string();
    // Decision OBS-7: blocked se marca con el flag Impediment, no transiciona.
    assert!(text.contains("Impediment"), "blocked usa el flag: {text}");
    assert!(text.contains("trabada"), "la nota viaja como comentario");
}
