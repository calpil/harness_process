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
    // Aislamiento de credenciales (feature #16): el binario busca el token en
    // el entorno y en ~/.config/harness/config. Un test JAMAS puede tomar las
    // credenciales reales de la maquina ni, mucho menos, hablarle a la API de
    // verdad: HOME apunta al sandbox y las variables se limpian.
    let home = bin.parent().unwrap().join("fake-home");
    let _ = std::fs::create_dir_all(&home);
    c.env("HOME", &home);
    c.env("USERPROFILE", &home);
    c.env_remove("HARNESS_ATLASSIAN_EMAIL");
    c.env_remove("HARNESS_ATLASSIAN_TOKEN");
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
    // Feature #47 (AC-8/AC-9): el estado vivo es de la feature y current.md es
    // el indice de lo que hay abierto.
    let current = std::fs::read_to_string(dir.path().join("hp/progress/current-1.md")).unwrap();
    assert!(current.contains("Plan: docs/plan-feature-1-pago-qr.md\nSpec: docs/spec-feature-1-pago-qr.md\n"));
    let indice = std::fs::read_to_string(dir.path().join("hp/progress/current.md")).unwrap();
    assert!(indice.contains("#1 Pago QR"), "el indice lista la activa: {indice}");
    assert!(indice.contains("current-1.md"));
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
fn start_should_allow_a_second_feature_in_parallel() {
    // Feature #47 / AC-1: se acabo el "Ya hay feature in_progress". Las dos
    // quedan activas, cada una con SU estado vivo (AC-8), y current.md pasa a
    // ser el indice de ambas (AC-9).
    let (dir, bin) = sandbox_with_binary();
    cmd(&bin).args(["add", "--name", "Uno"]).assert().success();
    cmd(&bin).args(["add", "--name", "Dos"]).assert().success();
    cmd(&bin).args(["start", "--feature", "1"]).assert().success();
    cmd(&bin)
        .args(["start", "--feature", "2"])
        .assert()
        .success()
        .stdout(predicate::str::contains("En paralelo con: #1 Uno"));

    let backlog: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(dir.path().join("hp/feature_list.json")).unwrap(),
    )
    .unwrap();
    let activas: Vec<&str> = backlog["features"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|f| f["status"] == "in_progress")
        .map(|f| f["name"].as_str().unwrap())
        .collect();
    assert_eq!(activas, vec!["Uno", "Dos"], "las dos quedan en curso");

    assert!(dir.path().join("hp/progress/current-1.md").is_file());
    assert!(dir.path().join("hp/progress/current-2.md").is_file());
    let indice = std::fs::read_to_string(dir.path().join("hp/progress/current.md")).unwrap();
    assert!(indice.contains("#1 Uno") && indice.contains("#2 Dos"), "{indice}");
}

#[test]
fn close_should_not_touch_the_state_of_the_other_active_feature() {
    // AC-11 (el bug de la feature #45, ahora imposible): cerrar una no puede
    // pisar el estado vivo de la otra.
    let (dir, bin) = sandbox_with_binary();
    cmd(&bin).args(["add", "--name", "Uno"]).assert().success();
    cmd(&bin).args(["add", "--name", "Dos"]).assert().success();
    cmd(&bin).args(["start", "--feature", "1"]).assert().success();
    cmd(&bin).args(["start", "--feature", "2"]).assert().success();

    let vivo_2 = dir.path().join("hp/progress/current-2.md");
    let antes = std::fs::read_to_string(&vivo_2).unwrap();

    cmd(&bin)
        .args(["close", "--feature", "1", "--status", "blocked", "--note", "aparcada"])
        .assert()
        .success();

    // El estado de la #2 quedo intacto...
    assert_eq!(std::fs::read_to_string(&vivo_2).unwrap(), antes);
    // ...y su stamp de autocheck tampoco se borro (AC-10).
    assert!(dir.path().join("hp/progress/.last_autocheck-2").exists());
    // El archivado de la #1 se llevo SU estado, no el de la #2.
    let archivado = std::fs::read_to_string(
        dir.path().join("docs/estado-feature-1-uno.md"),
    )
    .unwrap();
    assert!(archivado.contains("Feature #1"), "{archivado}");
    assert!(!archivado.contains("Feature #2: Dos"), "no se llevo el estado ajeno");
    // Y el indice ya solo lista la que sigue viva.
    let indice = std::fs::read_to_string(dir.path().join("hp/progress/current.md")).unwrap();
    assert!(indice.contains("#2 Dos") && !indice.contains("#1 Uno"), "{indice}");
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
    // Spec draft + regla activa: advance y close --status done bloquean con
    // exit 2. Salia 1 hasta la feature #36, que unifico los tres gates de
    // `close` en el mismo codigo (el de leccion y el de verify ya salian 2).
    cmd(&bin)
        .args(["advance", "--nota", "intento", "--no-graphify"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("[GATE] Spec sin aprobar"));
    cmd(&bin)
        .args(["close", "--feature", "1", "--status", "done", "--note", "x"])
        .assert()
        .code(2)
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

// ---------------------------------------------------------------------------
// Feature #16: envio automatico, --kind y el interruptor.
// ---------------------------------------------------------------------------

#[test]
fn add_should_reject_an_invalid_kind_before_touching_the_backlog() {
    // AC-10: exit 2 con la lista de validos, y el backlog intacto.
    let (dir, bin) = sandbox_with_binary();
    cmd(&bin)
        .args(["add", "--name", "Demo", "--kind", "epica"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("--kind invalido"))
        .stderr(predicate::str::contains("feature, bug, task"));
    // El backlog ni se crea: el rechazo ocurre antes de tocarlo.
    let backlog = std::fs::read_to_string(dir.path().join("hp/feature_list.json"))
        .unwrap_or_default();
    assert!(!backlog.contains("Demo"), "una feature invalida no entra al backlog");
}

#[test]
fn add_kind_should_be_optional_and_map_to_the_right_issue_type() {
    // AC-8 + AC-9: con --kind bug queda registrado y el plan usa el tipo Bug;
    // sin --kind, el backlog queda exactamente como antes.
    let (dir, bin) = sandbox_with_binary();
    write_binding(dir.path(), "ADR");
    cmd(&bin).args(["add", "--name", "Normal"]).assert().success();
    cmd(&bin)
        .args(["add", "--name", "Arreglo", "--kind", "bug"])
        .assert()
        .success();

    let backlog: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(dir.path().join("hp/feature_list.json")).unwrap())
            .unwrap();
    let features = backlog["features"].as_array().unwrap();
    assert!(features[0].get("kind").is_none(), "sin --kind no se agrega el campo");
    assert_eq!(features[1]["kind"], "bug");

    let out = cmd(&bin).args(["atlassian", "drain"]).output().unwrap();
    let plan: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let tipos: Vec<&str> = plan["plan"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|p| p["call"]["args"]["issueTypeName"].as_str())
        .collect();
    assert!(tipos.contains(&"Story"), "la feature normal va como Story: {tipos:?}");
    assert!(tipos.contains(&"Bug"), "el bug va como Bug: {tipos:?}");
}

#[test]
fn prd_add_should_emit_its_epic_without_waiting_for_a_feature() {
    // AC-3: el PRD nuevo nace como epic apenas se crea.
    let (dir, bin) = sandbox_with_binary();
    write_binding(dir.path(), "ADR");
    std::fs::create_dir_all(dir.path().join("docs/prd")).unwrap();
    std::fs::write(
        dir.path().join("docs/prd/PRD-master.md"),
        "# PRD maestro\n\n## 10. Hitos -> features\n",
    )
    .unwrap();

    cmd(&bin)
        .args(["prd", "add", "--name", "cobranza"])
        .assert()
        .success();

    let out = cmd(&bin).args(["atlassian", "drain"]).output().unwrap();
    let plan: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let what: Vec<&str> = plan["plan"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["what"].as_str().unwrap())
        .collect();
    assert!(
        what.iter().any(|w| w.contains("epic del PRD cobranza")),
        "el PRD nuevo deja su epic: {what:?}"
    );
}

#[test]
fn auto_push_should_be_reported_and_switchable() {
    // AC-13 + AC-14: `status` dice por que no se empuja, y el env lo apaga.
    let (dir, bin) = sandbox_with_binary();
    write_binding(dir.path(), "ADR");
    // Sin token (los tests estan aislados): apagado por falta de credenciales.
    cmd(&bin)
        .args(["atlassian", "status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Auto push  : apagado (sin token"));

    // Con credenciales falsas en el repo, se enciende...
    std::fs::write(
        dir.path().join(".harness.env"),
        "HARNESS_ATLASSIAN_EMAIL=a@b.cl\nHARNESS_ATLASSIAN_TOKEN=secreto\n",
    )
    .unwrap();
    cmd(&bin)
        .args(["atlassian", "status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Auto push  : encendido"));

    // ...y la variable de entorno lo apaga para esa corrida (AC-14).
    cmd(&bin)
        .env("HARNESS_ATLASSIAN_AUTO", "0")
        .args(["atlassian", "status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("HARNESS_ATLASSIAN_AUTO=0"));
}

#[test]
fn transitions_should_keep_their_exit_codes_with_auto_push_on() {
    // AC-2: con el envio automatico encendido (credenciales falsas, sin red
    // alcanzable) los comandos del flujo siguen saliendo 0 y con su salida.
    let (dir, bin) = sandbox_with_binary();
    write_binding(dir.path(), "ADR");
    std::fs::write(
        dir.path().join(".harness.env"),
        "HARNESS_ATLASSIAN_EMAIL=a@b.cl\nHARNESS_ATLASSIAN_TOKEN=secreto\n",
    )
    .unwrap();
    cmd(&bin)
        .args(["add", "--name", "Demo"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Feature #1 agregada."));
    cmd(&bin)
        .args(["start", "--feature", "1"])
        .assert()
        .success();
    cmd(&bin)
        .args(["close", "--feature", "1", "--status", "blocked", "--note", "x"])
        .assert()
        .success()
        .stdout(predicate::str::contains("cerrada como blocked"));
}

#[test]
fn backfill_should_load_prds_and_backlog_without_touching_the_network() {
    // AC-24 + AC-27: el backfill emite epics de los PRDs, historias de TODAS
    // las features y sus transiciones de estado, sin token (solo escribe la
    // outbox; aplicarlo es otro paso).
    let (dir, bin) = sandbox_with_binary();
    write_binding(dir.path(), "ADR");
    std::fs::create_dir_all(dir.path().join("docs/prd")).unwrap();
    std::fs::write(
        dir.path().join("docs/prd/PRD-master.md"),
        "# PRD maestro\n\n## 10. Hitos -> features\n",
    )
    .unwrap();

    // Backlog con historia: una cerrada y una en curso.
    cmd(&bin).args(["add", "--name", "Vieja"]).assert().success();
    cmd(&bin).args(["add", "--name", "Nueva"]).assert().success();
    cmd(&bin)
        .args(["start", "--feature", "2"])
        .assert()
        .success();
    // Limpiamos la outbox para partir de cero y probar SOLO el backfill.
    let outbox = dir.path().join("hp/progress/atlassian/outbox");
    std::fs::remove_dir_all(&outbox).unwrap();

    cmd(&bin)
        .args(["atlassian", "backfill"])
        .assert()
        .success()
        .stdout(predicate::str::contains("backfill:"));

    let out = cmd(&bin).args(["atlassian", "drain"]).output().unwrap();
    let plan: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let what: Vec<String> = plan["plan"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["what"].as_str().unwrap().to_string())
        .collect();
    assert!(what.iter().any(|w| w.contains("epic del PRD master")), "{what:?}");
    assert!(what.iter().any(|w| w.contains("feature #1")), "{what:?}");
    assert!(what.iter().any(|w| w.contains("feature #2")), "{what:?}");
    assert!(
        what.iter().any(|w| w.contains("-> In Progress")),
        "la feature en curso lleva su estado al board: {what:?}"
    );
}

#[test]
fn backfill_should_be_idempotent_and_respect_sin_acs() {
    // AC-25 + AC-28: repetirlo no duplica, y --sin-acs omite las subtasks.
    let (dir, bin) = sandbox_with_binary();
    write_binding(dir.path(), "ADR");
    cmd(&bin).args(["add", "--name", "Demo"]).assert().success();
    cmd(&bin)
        .args(["start", "--feature", "1"])
        .assert()
        .success();
    let spec = dir.path().join("docs/spec-feature-1-demo.md");
    let text = std::fs::read_to_string(&spec).unwrap();
    std::fs::write(
        &spec,
        text.replace(
            "- AC-1: Given <contexto>, When <accion>, Then <resultado observable>.",
            "- AC-1: Given algo, When otra cosa, Then un resultado.",
        ),
    )
    .unwrap();
    std::fs::remove_dir_all(dir.path().join("hp/progress/atlassian/outbox")).unwrap();

    // Con --sin-acs no bajan subtasks...
    cmd(&bin)
        .args(["atlassian", "backfill", "--sin-acs"])
        .assert()
        .success();
    let out = cmd(&bin).args(["atlassian", "drain"]).output().unwrap();
    let plan: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let text_plan = plan.to_string();
    assert!(!text_plan.contains("subtask AC-1"), "--sin-acs no baja los AC");

    // ...y sin el flag si, sumandose a lo ya emitido sin duplicarlo.
    cmd(&bin).args(["atlassian", "backfill"]).assert().success();
    let out2 = cmd(&bin).args(["atlassian", "drain"]).output().unwrap();
    let plan2: serde_json::Value = serde_json::from_slice(&out2.stdout).unwrap();
    assert!(plan2.to_string().contains("subtask AC-1"));

    let claves: Vec<&str> = plan2["plan"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["key"].as_str().unwrap())
        .collect();
    let mut unicas = claves.clone();
    unicas.sort_unstable();
    unicas.dedup();
    assert_eq!(claves.len(), unicas.len(), "el backfill no duplica intents: {claves:?}");
}

#[test]
fn bind_should_report_that_it_cannot_verify_without_token() {
    // AC-18: sin credenciales no se puede validar contra la API; se dice y se
    // escribe el binding igual.
    let (_dir, bin) = sandbox_with_binary();
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
        .stdout(predicate::str::contains("verificacion: omitida"))
        .stdout(predicate::str::contains("binding registrado"));
}

#[test]
fn backfill_should_refuse_without_binding() {
    // Sin binding no hay a donde cargar: exit 2 con la pregunta para el usuario.
    let (_dir, bin) = sandbox_with_binary();
    cmd(&bin)
        .args(["atlassian", "backfill"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("no lo voy a adivinar"));
}

// ---------------------------------------------------------------------------
// Lecciones (feature #17)
// ---------------------------------------------------------------------------

/// Activa la regla del gate de aprendizaje sin tocar las demas.
fn enable_leccion_rule(harness_dir: &Path) {
    let path = harness_dir.join("feature_list.json");
    let text = std::fs::read_to_string(&path).unwrap();
    let mut data: serde_json::Value = serde_json::from_str(&text).unwrap();
    let obj = data.as_object_mut().unwrap();
    let rules = obj
        .entry("rules".to_string())
        .or_insert_with(|| serde_json::json!({}));
    rules
        .as_object_mut()
        .unwrap()
        .insert("require_leccion".to_string(), serde_json::json!(true));
    std::fs::write(&path, serde_json::to_string_pretty(&data).unwrap()).unwrap();
}

#[test]
fn leccion_nueva_should_reject_session_names_without_writing_anything() {
    let (dir, bin) = sandbox_with_binary();
    cmd(&bin)
        .args(["leccion", "nueva", "fix-espejo-16"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("no es un nombre de CLASE"))
        .stderr(predicate::str::contains("espejo-de-roles"));
    // AC-4: ni el archivo ni la carpeta se crean ante un nombre rechazado.
    assert!(!dir.path().join("docs/lecciones").exists());
}

#[test]
fn leccion_should_create_list_use_and_refuse_duplicates() {
    let (dir, bin) = sandbox_with_binary();
    cmd(&bin).args(["add", "--name", "Demo"]).assert().success();
    cmd(&bin).args(["start", "--feature", "1"]).assert().success();
    // AC-3: crea desde plantilla y siembra `origen` con la feature activa.
    cmd(&bin)
        .args(["leccion", "nueva", "espejo-de-roles"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Leccion creada: docs/lecciones/espejo-de-roles.md",
        ));
    let file = dir.path().join("docs/lecciones/espejo-de-roles.md");
    let text = std::fs::read_to_string(&file).unwrap();
    assert!(text.contains("nombre: espejo-de-roles"));
    assert!(text.contains("origen: [1]"));
    assert!(text.contains("usos: 0"));
    for seccion in ["## Cuando aplica", "## Procedimiento", "## Pitfalls", "## Verificacion"] {
        assert!(text.contains(seccion), "falta {seccion}");
    }
    // AC-5: crear otra vez empuja a patchear, no duplica.
    cmd(&bin)
        .args(["leccion", "nueva", "espejo-de-roles"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("ya existe"))
        .stderr(predicate::str::contains("Patchea esa"));
    // AC-8: usar suma telemetria sin tocar el cuerpo.
    cmd(&bin)
        .args(["leccion", "usar", "espejo-de-roles"])
        .assert()
        .success()
        .stdout(predicate::str::contains("1 usos"));
    let usado = std::fs::read_to_string(&file).unwrap();
    assert!(usado.contains("usos: 1"));
    assert!(usado.contains("## Cuando aplica"));
    // AC-6: el catalogo lista la leccion; --json la emite estructurada.
    cmd(&bin)
        .args(["leccion", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("espejo-de-roles"));
    cmd(&bin)
        .args(["leccion", "list", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"usos\": 1"));
    // AC-7: show imprime la leccion; un typo sugiere la buena.
    cmd(&bin)
        .args(["leccion", "show", "espejo-de-roles"])
        .assert()
        .success()
        .stdout(predicate::str::contains("nombre: espejo-de-roles"));
    cmd(&bin)
        .args(["leccion", "show", "espejo-de-rol"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("¿Quisiste decir? espejo-de-roles"));
}

#[test]
fn leccion_list_should_explain_how_to_start_when_empty() {
    let (_dir, bin) = sandbox_with_binary();
    cmd(&bin)
        .args(["leccion", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Sin lecciones todavia"));
}

#[test]
fn close_should_stay_identical_without_the_leccion_rule() {
    // AC-10: sin la regla, cerrar como siempre no pide nada ni escribe campos.
    let (dir, bin) = sandbox_with_binary();
    cmd(&bin).args(["add", "--name", "Demo"]).assert().success();
    cmd(&bin).args(["start", "--feature", "1"]).assert().success();
    cmd(&bin)
        .args(["close", "--feature", "1", "--status", "done", "--note", "x"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Feature #1 cerrada como done"));
    let text = std::fs::read_to_string(dir.path().join("hp/feature_list.json")).unwrap();
    assert!(!text.contains("\"leccion\""));
}

#[test]
fn close_gate_should_demand_a_declaration_and_accept_both_exits() {
    let (dir, bin) = sandbox_with_binary();
    let harness_dir = dir.path().join("hp");
    cmd(&bin).args(["add", "--name", "Demo"]).assert().success();
    cmd(&bin).args(["add", "--name", "Otra"]).assert().success();
    cmd(&bin).args(["start", "--feature", "1"]).assert().success();
    enable_leccion_rule(&harness_dir);
    // AC-11: sin declaracion, el cierre bloquea y NO cierra.
    cmd(&bin)
        .args(["close", "--feature", "1", "--status", "done", "--note", "x"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("[GATE] El cierre no declara"))
        .stderr(predicate::str::contains("--leccion ninguna"));
    let text = std::fs::read_to_string(harness_dir.join("feature_list.json")).unwrap();
    assert!(text.contains("\"status\": \"in_progress\""));
    // AC-12: una clase inexistente falla y sugiere crearla.
    cmd(&bin)
        .args([
            "close", "--feature", "1", "--status", "done", "--leccion", "espejo-de-roles",
        ])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("y no existe"))
        .stderr(predicate::str::contains("leccion nueva espejo-de-roles"));
    // AC-13: 'ninguna' sin motivo se niega; con motivo, cierra.
    cmd(&bin)
        .args(["close", "--feature", "1", "--status", "done", "--leccion", "ninguna"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("--leccion-motivo"));
    cmd(&bin)
        .args([
            "close",
            "--feature",
            "1",
            "--status",
            "done",
            "--leccion",
            "ninguna",
            "--leccion-motivo",
            "trabajo mecanico",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Leccion declarada: ninguna"));
    let text = std::fs::read_to_string(harness_dir.join("feature_list.json")).unwrap();
    assert!(text.contains("\"leccion\": \"ninguna\""));
    assert!(text.contains("\"leccion_motivo\": \"trabajo mecanico\""));
    let history = std::fs::read_to_string(harness_dir.join("progress/history.md")).unwrap();
    assert!(history.contains("leccion=ninguna (trabajo mecanico)"));
    // AC-12 (rama feliz): con la clase creada, el cierre la registra.
    cmd(&bin).args(["start", "--feature", "2"]).assert().success();
    cmd(&bin)
        .args(["leccion", "nueva", "espejo-de-roles"])
        .assert()
        .success();
    cmd(&bin)
        .args([
            "close", "--feature", "2", "--status", "done", "--leccion", "espejo-de-roles",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Leccion declarada: espejo-de-roles"));
    let text = std::fs::read_to_string(harness_dir.join("feature_list.json")).unwrap();
    assert!(text.contains("\"leccion\": \"espejo-de-roles\""));
}

#[test]
fn close_gate_should_not_ask_for_a_leccion_when_blocking_a_feature() {
    let (dir, bin) = sandbox_with_binary();
    cmd(&bin).args(["add", "--name", "Demo"]).assert().success();
    cmd(&bin).args(["start", "--feature", "1"]).assert().success();
    enable_leccion_rule(&dir.path().join("hp"));
    cmd(&bin)
        .args(["close", "--feature", "1", "--status", "blocked", "--note", "aparcada"])
        .assert()
        .success();
}

#[test]
fn leccion_should_work_with_the_hub_unreachable() {
    // AC-9 / D10: hub apuntando a un host muerto -> mismos exit codes y salida.
    let (dir, bin) = sandbox_with_binary();
    let run = |args: &[&str], hub_down: bool| {
        let mut c = cmd(&bin);
        if hub_down {
            c.env("DB_HOST", "127.0.0.1")
                .env("DB_PORT", "1")
                .env("DB_USER", "nadie")
                .env("DB_PASSWORD", "nada")
                .env("DB_NAME", "nada");
        }
        c.args(args).output().unwrap()
    };
    cmd(&bin)
        .args(["leccion", "nueva", "espejo-de-roles"])
        .assert()
        .success();
    for args in [
        vec!["leccion", "list"],
        vec!["leccion", "show", "espejo-de-roles"],
        vec!["leccion", "usar", "espejo-de-roles"],
        vec!["leccion", "nueva", "fix-esto"],
    ] {
        let con_hub = run(&args, false);
        let sin_hub = run(&args, true);
        assert_eq!(
            con_hub.status.code(),
            sin_hub.status.code(),
            "exit distinto con el hub caido para {args:?}"
        );
        assert_eq!(
            String::from_utf8_lossy(&con_hub.stderr),
            String::from_utf8_lossy(&sin_hub.stderr),
            "stderr distinto con el hub caido para {args:?}"
        );
    }
    assert!(dir.path().join("docs/lecciones/espejo-de-roles.md").exists());
}

// ---------------------------------------------------------------------------
// Nudge de aprendizaje (feature #18)
// ---------------------------------------------------------------------------

/// Copia la guia REAL del repo al sandbox. Es lo que convierte al test en un
/// gate anti-drift: si alguien renombra una seccion de la guia, el contrato deja
/// de encontrarla y esto se pone rojo.
fn seed_guia_real(root: &Path) {
    let guia = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../templates/docs/lecciones/COMO-ESCRIBIR-UNA-LECCION.md");
    let destino = root.join("docs/lecciones");
    std::fs::create_dir_all(&destino).unwrap();
    std::fs::copy(&guia, destino.join("COMO-ESCRIBIR-UNA-LECCION.md")).unwrap();
}

#[test]
fn close_should_emit_the_contract_read_from_the_real_guide() {
    let (dir, bin) = sandbox_with_binary();
    seed_guia_real(dir.path());
    cmd(&bin).args(["add", "--name", "Demo"]).assert().success();
    cmd(&bin).args(["start", "--feature", "1"]).assert().success();
    let out = cmd(&bin)
        .args(["close", "--feature", "1", "--status", "done", "--note", "x"])
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr);
    // AC-6: el contrato sale, y su texto viene de la guia (no de una copia).
    assert!(stderr.contains("SIN declarar que se aprendio"), "{stderr}");
    assert!(
        stderr.contains("primero patchear, crear al final"),
        "falta el orden de preferencia: {stderr}"
    );
    assert!(
        stderr.contains("Que NO capturar"),
        "falta la lista anti-veneno: {stderr}"
    );
    // Las cinco reglas de la guia real llegan al contrato.
    for regla in [
        "Fallas dependientes del entorno",
        "Afirmaciones negativas sobre herramientas",
        "Errores transitorios",
        "Narrativas de una tarea unica",
        "Fracasos no resueltos",
    ] {
        assert!(stderr.contains(regla), "el contrato no trae '{regla}': {stderr}");
    }
    assert!(stderr.contains("leccion list"), "{stderr}");
    // Lo que NO tiene que colarse: otras secciones de la guia.
    assert!(!stderr.contains("Sin secretos"), "se colo una seccion de mas");
    // AC-10: el exit code y el stdout del cierre no cambian.
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("Feature #1 cerrada como done"));
    assert!(!stdout.contains("SIN declarar"), "el contrato no va a stdout");
}

#[test]
fn close_should_not_emit_the_contract_when_the_lesson_was_declared() {
    let (dir, bin) = sandbox_with_binary();
    seed_guia_real(dir.path());
    cmd(&bin).args(["add", "--name", "Demo"]).assert().success();
    cmd(&bin).args(["add", "--name", "Otra"]).assert().success();
    cmd(&bin).args(["start", "--feature", "1"]).assert().success();
    cmd(&bin)
        .args(["leccion", "nueva", "espejo-de-roles"])
        .assert()
        .success();
    // AC-7: con clase declarada, no hay contrato.
    let con_clase = cmd(&bin)
        .args([
            "close", "--feature", "1", "--status", "done", "--leccion", "espejo-de-roles",
        ])
        .output()
        .unwrap();
    assert!(!String::from_utf8_lossy(&con_clase.stderr).contains("SIN declarar"));
    // AC-7: 'ninguna' con motivo tampoco lo dispara.
    cmd(&bin).args(["start", "--feature", "2"]).assert().success();
    let con_ninguna = cmd(&bin)
        .args([
            "close",
            "--feature",
            "2",
            "--status",
            "done",
            "--leccion",
            "ninguna",
            "--leccion-motivo",
            "trabajo mecanico",
        ])
        .output()
        .unwrap();
    assert!(!String::from_utf8_lossy(&con_ninguna.stderr).contains("SIN declarar"));
}

#[test]
fn close_should_not_emit_the_contract_on_blocked_or_without_lecciones_dir() {
    // AC-8: blocked no pide leccion.
    let (dir, bin) = sandbox_with_binary();
    seed_guia_real(dir.path());
    cmd(&bin).args(["add", "--name", "Demo"]).assert().success();
    cmd(&bin).args(["start", "--feature", "1"]).assert().success();
    let bloqueada = cmd(&bin)
        .args(["close", "--feature", "1", "--status", "blocked", "--note", "aparcada"])
        .output()
        .unwrap();
    assert!(!String::from_utf8_lossy(&bloqueada.stderr).contains("SIN declarar"));

    // AC-9: sin docs/lecciones/, un cierre done sin declaracion no emite nada.
    let (dir2, bin2) = sandbox_with_binary();
    assert!(!dir2.path().join("docs/lecciones").exists());
    cmd(&bin2).args(["add", "--name", "Demo"]).assert().success();
    cmd(&bin2).args(["start", "--feature", "1"]).assert().success();
    let sin_lecciones = cmd(&bin2)
        .args(["close", "--feature", "1", "--status", "done", "--note", "x"])
        .output()
        .unwrap();
    assert!(!String::from_utf8_lossy(&sin_lecciones.stderr).contains("SIN declarar"));
}

#[test]
fn close_should_degrade_to_a_pointer_when_the_guide_is_missing() {
    // AC-21: docs/lecciones/ existe pero la guia no: puntero, no error.
    let (dir, bin) = sandbox_with_binary();
    std::fs::create_dir_all(dir.path().join("docs/lecciones")).unwrap();
    cmd(&bin).args(["add", "--name", "Demo"]).assert().success();
    cmd(&bin).args(["start", "--feature", "1"]).assert().success();
    let out = cmd(&bin)
        .args(["close", "--feature", "1", "--status", "done", "--note", "x"])
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("SIN declarar que se aprendio"), "{stderr}");
    assert!(stderr.contains("COMO-ESCRIBIR-UNA-LECCION.md"), "{stderr}");
    // Degrada: no inventa el contenido del contrato.
    assert!(!stderr.contains("Que NO capturar"), "{stderr}");
    assert!(out.status.success(), "el cierre no puede fallar por la guia");
}

#[test]
fn nudge_should_stay_silent_until_the_interval_and_never_fail() {
    let (dir, bin) = sandbox_with_binary();
    seed_guia_real(dir.path());
    cmd(&bin).args(["add", "--name", "Demo"]).assert().success();
    cmd(&bin).args(["start", "--feature", "1"]).assert().success();
    // Intervalo chico para no correr 25 veces.
    let features = dir.path().join("hp/feature_list.json");
    let text = std::fs::read_to_string(&features).unwrap();
    let mut data: serde_json::Value = serde_json::from_str(&text).unwrap();
    data.as_object_mut().unwrap().insert(
        "rules".to_string(),
        serde_json::json!({"leccion_nudge_interval": 3}),
    );
    std::fs::write(&features, serde_json::to_string_pretty(&data).unwrap()).unwrap();
    // AC-2: las dos primeras no dicen nada de lecciones.
    for _ in 0..2 {
        let out = cmd(&bin).arg("nudge").output().unwrap();
        assert!(out.status.success());
        assert!(!String::from_utf8_lossy(&out.stderr).contains("acciones en esta feature"));
    }
    // AC-1: la tercera emite el recordatorio corto, y sigue saliendo con 0.
    let out = cmd(&bin).arg("nudge").output().unwrap();
    assert!(out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("3 acciones en esta feature"), "{stderr}");
    assert!(stderr.contains("leccion list"), "{stderr}");
}

#[test]
fn nudge_should_never_fail_even_with_a_corrupt_backlog() {
    // AC-15: best-effort absoluto.
    let (dir, bin) = sandbox_with_binary();
    std::fs::write(dir.path().join("hp/feature_list.json"), "{ roto").unwrap();
    cmd(&bin).arg("nudge").assert().success();
}

// ---------------------------------------------------------------------------
// Perfil de usuario (feature #19)
// ---------------------------------------------------------------------------

#[test]
fn the_seeded_template_should_match_what_the_binary_writes() {
    // Anti-drift: el instalador siembra templates/docs/perfil-usuario.md y el
    // binario usa su propia plantilla cuando el archivo no esta. Si divergen, un
    // repo instalado y uno migrado tendrian encabezados distintos.
    let plantilla = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../templates/docs/perfil-usuario.md"),
    )
    .unwrap();
    let (dir, bin) = sandbox_with_binary();
    cmd(&bin)
        .args(["perfil", "add", "--texto", "Entrada de prueba.", "--yes"])
        .assert()
        .success();
    let escrito = std::fs::read_to_string(dir.path().join("docs/perfil-usuario.md")).unwrap();
    let encabezado: String = escrito
        .lines()
        .take_while(|l| !l.starts_with("- "))
        .collect::<Vec<_>>()
        .join("\n");
    assert_eq!(
        encabezado.trim_end(),
        plantilla.trim_end(),
        "la plantilla del instalador y la del binario divergieron"
    );
}

#[test]
fn perfil_writes_should_refuse_without_the_user_yes() {
    let (dir, bin) = sandbox_with_binary();
    // AC-6: los tres comandos de escritura se niegan sin --yes.
    for args in [
        vec!["perfil", "add", "--texto", "algo"],
        vec!["perfil", "replace", "--old", "x", "--texto", "algo"],
        vec!["perfil", "remove", "--old", "x"],
    ] {
        cmd(&bin)
            .args(&args)
            .assert()
            .code(2)
            .stdout(predicate::str::contains("exige la confirmacion explicita del USUARIO"));
    }
    // Y no se creo nada.
    assert!(!dir.path().join("docs/perfil-usuario.md").exists());
}

#[test]
fn perfil_should_add_show_replace_and_remove() {
    let (dir, bin) = sandbox_with_binary();
    cmd(&bin)
        .args(["perfil", "show"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Perfil vacio"));
    cmd(&bin)
        .args(["perfil", "add", "--texto", "Elige la opcion segura. (#14)", "--yes"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Entrada agregada"))
        .stdout(predicate::str::contains("se refrescan"));
    // AC-7: duplicado exacto no duplica.
    cmd(&bin)
        .args(["perfil", "add", "--texto", "Elige la opcion segura. (#14)", "--yes"])
        .assert()
        .success()
        .stdout(predicate::str::contains("no se duplico"));
    cmd(&bin)
        .args(["perfil", "add", "--texto", "Prefiere features amplias. (#15)", "--yes"])
        .assert()
        .success();
    cmd(&bin)
        .args(["perfil", "show"])
        .assert()
        .success()
        .stdout(predicate::str::contains("1. Elige la opcion segura. (#14)"))
        .stdout(predicate::str::contains("2. Prefiere features amplias. (#15)"))
        .stdout(predicate::str::contains("/1500 chars"));
    // AC-8: subcadena que no matachea y subcadena ambigua.
    cmd(&bin)
        .args(["perfil", "remove", "--old", "inexistente", "--yes"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("Ninguna entrada"));
    cmd(&bin)
        .args(["perfil", "remove", "--old", "e", "--yes"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("usa un fragmento mas especifico"));
    cmd(&bin)
        .args([
            "perfil", "replace", "--old", "amplias", "--texto", "Prefiere features completas. (#15)",
            "--yes",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Ahora: Prefiere features completas"));
    cmd(&bin)
        .args(["perfil", "remove", "--old", "opcion segura", "--yes"])
        .assert()
        .success();
    let texto = std::fs::read_to_string(dir.path().join("docs/perfil-usuario.md")).unwrap();
    assert!(!texto.contains("opcion segura"));
    assert!(texto.contains("Prefiere features completas"));
    assert!(texto.starts_with("# Perfil de usuario"), "se perdio el encabezado");
    // AC-9: toda escritura queda en la bitacora.
    let history = std::fs::read_to_string(dir.path().join("hp/progress/history.md")).unwrap();
    for esperado in ["perfil add", "perfil replace", "perfil remove"] {
        assert!(history.contains(esperado), "falta '{esperado}' en la bitacora");
    }
}

#[test]
fn perfil_should_refuse_entries_that_look_like_secrets() {
    // AC-10: bloquea ANTES de escribir.
    let (dir, bin) = sandbox_with_binary();
    cmd(&bin)
        .args([
            "perfil", "add", "--texto", "el api_key: abc123 del hub", "--yes",
        ])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("no entra al perfil"))
        .stderr(predicate::str::contains("nombra la VARIABLE de entorno"));
    assert!(!dir.path().join("docs/perfil-usuario.md").exists());
}

#[test]
fn perfil_should_refuse_to_exceed_the_hard_limit() {
    // AC-3: falla mostrando las entradas actuales; no recorta.
    let (dir, bin) = sandbox_with_binary();
    // Dos textos DISTINTOS: con el mismo, ganaria el chequeo de duplicado y el
    // limite nunca se ejercitaria.
    let primera = "x".repeat(900);
    let segunda = "y".repeat(900);
    cmd(&bin)
        .args(["perfil", "add", "--texto", &primera, "--yes"])
        .assert()
        .success();
    cmd(&bin)
        .args(["perfil", "add", "--texto", &segunda, "--yes"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("el limite es duro"))
        .stderr(predicate::str::contains("en este mismo turno"))
        .stderr(predicate::str::contains("Entradas actuales:"));
    let texto = std::fs::read_to_string(dir.path().join("docs/perfil-usuario.md")).unwrap();
    assert!(texto.contains(&primera));
    assert!(!texto.contains(&segunda), "no se escribio la segunda");
}

#[test]
fn perfil_bloque_should_be_empty_without_entries_and_carry_them_after() {
    let (_dir, bin) = sandbox_with_binary();
    let vacio = cmd(&bin).args(["perfil", "bloque"]).output().unwrap();
    assert!(vacio.status.success());
    assert!(
        String::from_utf8_lossy(&vacio.stdout).trim().is_empty(),
        "sin entradas no hay bloque"
    );
    cmd(&bin)
        .args(["perfil", "add", "--texto", "Una preferencia. (#14)", "--yes"])
        .assert()
        .success();
    cmd(&bin)
        .args(["perfil", "bloque"])
        .assert()
        .success()
        .stdout(predicate::str::contains("<!-- harness:perfil:inicio -->"))
        .stdout(predicate::str::contains("- Una preferencia. (#14)"))
        .stdout(predicate::str::contains("<!-- harness:perfil:fin -->"));
}

#[test]
fn perfil_sugerir_should_report_evidence_and_emit_the_contract() {
    let (dir, bin) = sandbox_with_binary();
    let hp = dir.path().join("hp");
    std::fs::create_dir_all(hp.join("progress")).unwrap();
    std::fs::write(
        hp.join("progress/history.md"),
        "- 2026-08-14T03:43:37Z approve-spec feature #14 nota=Alan eligio la opcion segura\n\
         - 2026-08-14T04:10:09Z close feature #14 status=done note=hub por lotes\n",
    )
    .unwrap();
    cmd(&bin)
        .args(["perfil", "sugerir"])
        .assert()
        .success()
        .stdout(predicate::str::contains("feature #14"))
        .stdout(predicate::str::contains("eligio la opcion segura"))
        .stdout(predicate::str::contains("COMO DESTILAR UNA ENTRADA"))
        .stdout(predicate::str::contains("perfil add --texto"))
        // La linea sin senal de decision no entra.
        .stdout(predicate::str::contains("hub por lotes").not());
    // AC-14: no escribe nada.
    assert!(!dir.path().join("docs/perfil-usuario.md").exists());
}

#[test]
fn perfil_sugerir_should_say_so_without_material() {
    // AC-16.
    let (_dir, bin) = sandbox_with_binary();
    cmd(&bin)
        .args(["perfil", "sugerir"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Sin material todavia"));
}

// ---------------------------------------------------------------------------
// buscar (feature #20)
// ---------------------------------------------------------------------------

/// Siembra un corpus chico con una fuente de cada nivel del ranking.
fn seed_corpus(root: &Path) {
    let docs = root.join("docs");
    std::fs::create_dir_all(docs.join("lecciones")).unwrap();
    std::fs::create_dir_all(docs.join("adr")).unwrap();
    std::fs::write(
        docs.join("lecciones/espejo-de-roles.md"),
        "---\nnombre: espejo-de-roles\ntriggers: [espejo]\n---\n\nEl espejo de roles se regenera.\n",
    )
    .unwrap();
    std::fs::write(
        docs.join("adr/ADR-0001-cliente-http.md"),
        "# ADR-0001: cliente HTTP para el espejo\n\nDecision tecnica.\n",
    )
    .unwrap();
    std::fs::write(
        docs.join("spec-feature-9-x.md"),
        "# Spec\n\nEl espejo de roles se valida en el gate.\n",
    )
    .unwrap();
    std::fs::write(docs.join("impl-9.md"), "Se toco el espejo.\n").unwrap();
}

#[test]
fn buscar_should_refuse_an_empty_query() {
    let (_dir, bin) = sandbox_with_binary();
    cmd(&bin)
        .args(["buscar", "   "])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("Falta la consulta"))
        .stderr(predicate::str::contains("Uso: sh harness_cli buscar"));
}

#[test]
fn buscar_should_rank_curated_knowledge_first() {
    let (dir, bin) = sandbox_with_binary();
    seed_corpus(dir.path());
    let out = cmd(&bin).args(["buscar", "espejo", "--json"]).output().unwrap();
    assert!(out.status.success());
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let filas = json["resultados"].as_array().unwrap();
    let fuentes: Vec<&str> = filas
        .iter()
        .map(|r| r["fuente"].as_str().unwrap_or_default())
        .collect();
    // La leccion antes que el ADR, el ADR antes que impl, y la bitacora al final.
    let pos = |f: &str| fuentes.iter().position(|x| *x == f);
    assert!(pos("leccion").unwrap() < pos("adr").unwrap(), "{fuentes:?}");
    assert!(pos("adr").unwrap() < pos("impl").unwrap(), "{fuentes:?}");
    // AC-7: el score es auditable.
    assert!(filas[0]["score"].as_i64().unwrap() > 0);
    assert!(filas[0]["archivo"].as_str().unwrap().contains("lecciones/"));
}

#[test]
fn buscar_should_report_a_partial_match_instead_of_pretending() {
    let (dir, bin) = sandbox_with_binary();
    seed_corpus(dir.path());
    cmd(&bin)
        .args(["buscar", "espejo inexistente"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Ninguna linea tiene TODOS los terminos"));
}

#[test]
fn buscar_should_exit_zero_without_matches() {
    // AC-10: no encontrar no es un error.
    let (dir, bin) = sandbox_with_binary();
    seed_corpus(dir.path());
    cmd(&bin)
        .args(["buscar", "zzzznoexiste"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Sin coincidencias"))
        .stdout(predicate::str::contains("menos terminos"));
}

#[test]
fn buscar_json_should_stay_valid_without_matches() {
    // AC-11: un script no deberia manejar dos formatos.
    let (dir, bin) = sandbox_with_binary();
    seed_corpus(dir.path());
    let out = cmd(&bin)
        .args(["buscar", "zzzznoexiste", "--json"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json["resultados"].as_array().unwrap().len(), 0);
    assert_eq!(json["total"], 0);
}

#[test]
fn buscar_should_never_truncate_silently() {
    // AC-9: mas de 20 resultados => se dice cuantos quedaron fuera.
    let (dir, bin) = sandbox_with_binary();
    let docs = dir.path().join("docs");
    std::fs::create_dir_all(&docs).unwrap();
    let mut cuerpo = String::new();
    for i in 0..30 {
        cuerpo.push_str(&format!("linea {i} con marcador\n"));
    }
    std::fs::write(docs.join("impl-1.md"), &cuerpo).unwrap();
    cmd(&bin)
        .args(["buscar", "marcador"])
        .assert()
        .success()
        .stdout(predicate::str::contains("30 resultado(s)"))
        .stdout(predicate::str::contains("10 resultado(s) mas"))
        .stdout(predicate::str::contains("--todos"));
    // Con --todos no queda el aviso.
    cmd(&bin)
        .args(["buscar", "marcador", "--todos"])
        .assert()
        .success()
        .stdout(predicate::str::contains("resultado(s) mas").not());
}

#[test]
fn buscar_should_say_so_without_a_corpus() {
    // AC-16.
    let (_dir, bin) = sandbox_with_binary();
    cmd(&bin)
        .args(["buscar", "lo que sea"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No hay corpus"));
}

#[test]
fn buscar_should_write_nothing_and_ignore_the_hub() {
    // AC-13 / AC-14: mismo resultado con el hub caido, y ni un archivo tocado.
    let (dir, bin) = sandbox_with_binary();
    seed_corpus(dir.path());
    let antes = std::fs::read_dir(dir.path().join("docs")).unwrap().count();
    let normal = cmd(&bin).args(["buscar", "espejo", "--json"]).output().unwrap();
    let sin_hub = cmd(&bin)
        .env("DB_HOST", "127.0.0.1")
        .env("DB_PORT", "1")
        .env("DB_USER", "nadie")
        .env("DB_PASSWORD", "nada")
        .env("DB_NAME", "nada")
        .args(["buscar", "espejo", "--json"])
        .output()
        .unwrap();
    assert_eq!(normal.status.code(), sin_hub.status.code());
    assert_eq!(normal.stdout, sin_hub.stdout, "el hub no puede cambiar el resultado");
    let despues = std::fs::read_dir(dir.path().join("docs")).unwrap().count();
    assert_eq!(antes, despues, "buscar no puede crear archivos");
    // Y no aparecio ningun indice.
    assert!(!dir.path().join("docs/.buscar-index").exists());
}

#[test]
fn buscar_should_skip_backup_directories() {
    // AC-1: bkp/ tiene copias viejas que contaminarian el resultado.
    let (dir, bin) = sandbox_with_binary();
    seed_corpus(dir.path());
    std::fs::create_dir_all(dir.path().join("bkp")).unwrap();
    std::fs::write(dir.path().join("bkp/viejo.md"), "espejo de una version vieja\n").unwrap();
    let out = cmd(&bin).args(["buscar", "espejo", "--json"]).output().unwrap();
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let archivos: Vec<&str> = json["resultados"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["archivo"].as_str().unwrap_or_default())
        .collect();
    assert!(archivos.iter().all(|a| !a.contains("bkp/")), "{archivos:?}");
}

// ---------------------------------------------------------------------------
// Curador de lecciones (feature #21)
// ---------------------------------------------------------------------------

/// Escribe una leccion con telemetria controlada, para ejercitar el ciclo de
/// vida sin esperar 90 dias.
fn seed_leccion(root: &Path, nombre: &str, ultimo_uso: &str, estado: &str, pin: bool) {
    let dir = root.join("docs/lecciones");
    std::fs::create_dir_all(&dir).unwrap();
    let extra = if pin { "pinneada: true\n" } else { "" };
    std::fs::write(
        dir.join(format!("{nombre}.md")),
        format!(
            "---\nnombre: {nombre}\ndescripcion: Leccion de prueba.\ntriggers: [marcador-{nombre}]\n\
             usos: 1\nultimo_uso: {ultimo_uso}\nultima_actualizacion: {ultimo_uso}\n\
             estado: {estado}\n{extra}---\n\n## Cuando aplica\n\ncuerpo de {nombre}\n"
        ),
    )
    .unwrap();
}

#[test]
fn lecciones_should_say_so_without_a_library() {
    // AC-2.
    let (_dir, bin) = sandbox_with_binary();
    for args in [
        vec!["lecciones", "status"],
        vec!["lecciones", "curar"],
        vec!["lecciones", "pin", "x"],
    ] {
        cmd(&bin)
            .args(&args)
            .assert()
            .success()
            .stdout(predicate::str::contains("Todavia no hay biblioteca"));
    }
}

#[test]
fn lecciones_curar_should_not_touch_anything_without_aplicar() {
    // AC-9: el criterio central de esta feature.
    let (dir, bin) = sandbox_with_binary();
    seed_leccion(dir.path(), "vencida", "2020-01-01", "stale", false);
    let file = dir.path().join("docs/lecciones/vencida.md");
    let antes = std::fs::read_to_string(&file).unwrap();
    let mtime_antes = std::fs::metadata(&file).unwrap().modified().unwrap();
    cmd(&bin)
        .args(["lecciones", "curar"])
        .assert()
        .success()
        .stdout(predicate::str::contains("ARCHIVAR"))
        .stdout(predicate::str::contains("no se toco ningun archivo"));
    assert_eq!(std::fs::read_to_string(&file).unwrap(), antes);
    assert_eq!(std::fs::metadata(&file).unwrap().modified().unwrap(), mtime_antes);
    assert!(!dir.path().join("docs/lecciones/archivo").exists());
}

#[test]
fn lecciones_curar_aplicar_should_move_backup_and_report() {
    // AC-5, AC-10, AC-16.
    let (dir, bin) = sandbox_with_binary();
    seed_leccion(dir.path(), "vencida", "2020-01-01", "stale", false);
    let original = std::fs::read_to_string(dir.path().join("docs/lecciones/vencida.md")).unwrap();
    cmd(&bin)
        .args(["lecciones", "curar", "--aplicar"])
        .assert()
        .success()
        .stdout(predicate::str::contains("1 transicion(es) aplicada(s)"))
        .stdout(predicate::str::contains("Backup previo"))
        .stdout(predicate::str::contains("Para deshacer"));
    // Movida, no borrada.
    assert!(!dir.path().join("docs/lecciones/vencida.md").exists());
    let archivada = dir.path().join("docs/lecciones/archivo/vencida.md");
    assert!(archivada.exists(), "archivar tiene que MOVER");
    assert!(std::fs::read_to_string(&archivada).unwrap().contains("cuerpo de vencida"));
    // Backup y reporte existen.
    let backups = dir.path().join("hp/bkp/lecciones");
    assert!(backups.is_dir(), "falta el backup previo");
    let reportes = dir.path().join("hp/progress/lecciones");
    assert!(reportes.is_dir(), "falta el reporte");
    // El backup guarda el original intacto.
    let copia = std::fs::read_dir(&backups)
        .unwrap()
        .flatten()
        .next()
        .unwrap()
        .path()
        .join("lecciones/vencida.md");
    assert_eq!(std::fs::read_to_string(copia).unwrap(), original);
}

#[test]
fn lecciones_rollback_should_restore_exactly_and_stay_reversible() {
    // AC-11, AC-12: el criterio de cierre mas importante.
    let (dir, bin) = sandbox_with_binary();
    seed_leccion(dir.path(), "vencida", "2020-01-01", "stale", false);
    let file = dir.path().join("docs/lecciones/vencida.md");
    let original = std::fs::read_to_string(&file).unwrap();
    cmd(&bin).args(["lecciones", "curar", "--aplicar"]).assert().success();
    cmd(&bin)
        .args(["lecciones", "rollback"])
        .assert()
        .success()
        .stdout(predicate::str::contains("tambien se puede deshacer"));
    // Diff vacio: el contenido volvio EXACTO.
    assert_eq!(std::fs::read_to_string(&file).unwrap(), original);
    // Y el rollback dejo su propio backup.
    cmd(&bin)
        .args(["lecciones", "rollback", "--list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("pre-rollback"));
}

#[test]
fn lecciones_pin_should_survive_a_pass() {
    // AC-7: 200+ dias de inactividad y sigue activa.
    let (dir, bin) = sandbox_with_binary();
    seed_leccion(dir.path(), "protegida", "2020-01-01", "activa", true);
    cmd(&bin)
        .args(["lecciones", "curar", "--aplicar"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Salteadas por pin: protegida"));
    let texto = std::fs::read_to_string(dir.path().join("docs/lecciones/protegida.md")).unwrap();
    assert!(texto.contains("estado: activa"), "el pin no protegio: {texto}");
    assert!(dir.path().join("docs/lecciones/protegida.md").exists());
}

#[test]
fn lecciones_pin_and_unpin_should_toggle_without_touching_the_body() {
    // AC-13.
    let (dir, bin) = sandbox_with_binary();
    seed_leccion(dir.path(), "x", "2026-08-01", "activa", false);
    let file = dir.path().join("docs/lecciones/x.md");
    cmd(&bin).args(["lecciones", "pin", "x"]).assert().success();
    assert!(std::fs::read_to_string(&file).unwrap().contains("pinneada: true"));
    cmd(&bin).args(["lecciones", "unpin", "x"]).assert().success();
    let texto = std::fs::read_to_string(&file).unwrap();
    assert!(texto.contains("pinneada: false"));
    assert!(texto.contains("cuerpo de x"), "se toco el cuerpo");
    assert!(texto.contains("usos: 1"), "se toco la telemetria");
}

#[test]
fn lecciones_manual_archive_and_restore_should_round_trip() {
    // AC-14.
    let (dir, bin) = sandbox_with_binary();
    seed_leccion(dir.path(), "x", "2026-08-01", "activa", false);
    let original = std::fs::read_to_string(dir.path().join("docs/lecciones/x.md")).unwrap();
    cmd(&bin)
        .args(["lecciones", "archivar", "x"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No se borro nada"));
    assert!(dir.path().join("docs/lecciones/archivo/x.md").exists());
    // Archivar dos veces falla.
    cmd(&bin)
        .args(["lecciones", "archivar", "x"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("ya esta archivada"));
    cmd(&bin).args(["lecciones", "restaurar", "x"]).assert().success();
    let vuelto = std::fs::read_to_string(dir.path().join("docs/lecciones/x.md")).unwrap();
    assert_eq!(vuelto, original, "el round trip cambio el contenido");
    // Restaurar algo que no esta archivado falla.
    cmd(&bin)
        .args(["lecciones", "restaurar", "x"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("no esta archivada"));
}

#[test]
fn lecciones_should_suggest_similar_names() {
    // AC-15.
    let (dir, bin) = sandbox_with_binary();
    seed_leccion(dir.path(), "espejo-de-roles", "2026-08-01", "activa", false);
    cmd(&bin)
        .args(["lecciones", "pin", "espejo-de-rol"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("¿Quisiste decir? espejo-de-roles"));
}

#[test]
fn an_archived_lesson_should_stay_searchable_below_an_active_one() {
    // AC-18: el motivo por el que el archivo es VISIBLE y no oculto.
    let (dir, bin) = sandbox_with_binary();
    seed_leccion(dir.path(), "vieja", "2020-01-01", "stale", false);
    seed_leccion(dir.path(), "vigente", "2026-08-17", "activa", false);
    // Un termino que esta en las dos.
    std::fs::write(
        dir.path().join("docs/lecciones/vigente.md"),
        std::fs::read_to_string(dir.path().join("docs/lecciones/vigente.md"))
            .unwrap()
            .replace("cuerpo de vigente", "el termino compartido"),
    )
    .unwrap();
    std::fs::write(
        dir.path().join("docs/lecciones/vieja.md"),
        std::fs::read_to_string(dir.path().join("docs/lecciones/vieja.md"))
            .unwrap()
            .replace("cuerpo de vieja", "el termino compartido"),
    )
    .unwrap();
    cmd(&bin).args(["lecciones", "curar", "--aplicar"]).assert().success();
    assert!(dir.path().join("docs/lecciones/archivo/vieja.md").exists());
    let out = cmd(&bin)
        .args(["buscar", "termino compartido", "--json"])
        .output()
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let filas = json["resultados"].as_array().unwrap();
    let fuentes: Vec<&str> = filas.iter().map(|r| r["fuente"].as_str().unwrap()).collect();
    // Sigue apareciendo...
    assert!(fuentes.contains(&"leccion-archivada"), "{fuentes:?}");
    // ...pero por debajo de la activa.
    let pos_activa = fuentes.iter().position(|f| *f == "leccion").unwrap();
    let pos_arch = fuentes.iter().position(|f| *f == "leccion-archivada").unwrap();
    assert!(pos_activa < pos_arch, "{fuentes:?}");
    // Y el catalogo activo ya no la lista.
    cmd(&bin)
        .args(["leccion", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("vieja").not());
    cmd(&bin)
        .args(["leccion", "list", "--archivadas"])
        .assert()
        .success()
        .stdout(predicate::str::contains("vieja"));
}

#[test]
fn lecciones_status_should_report_days_to_the_next_transition() {
    // AC-1, AC-3.
    let (dir, bin) = sandbox_with_binary();
    seed_leccion(dir.path(), "fresca", &lecciones_hoy(), "activa", false);
    cmd(&bin)
        .args(["lecciones", "status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Umbrales: stale >= 30d"))
        .stdout(predicate::str::contains("-> stale en 30d"))
        .stdout(predicate::str::contains("Candidatas HOY: 0 a stale, 0 a archivar"));
    let out = cmd(&bin).args(["lecciones", "status", "--json"]).output().unwrap();
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let l = &json["lecciones"][0];
    assert_eq!(l["nombre"], "fresca");
    assert_eq!(l["dias_inactiva"], 0);
    assert_eq!(l["proxima_transicion"], "stale");
    assert_eq!(l["dias_para_transicion"], 30);
}

/// Fecha de hoy en el mismo formato que usa el binario.
fn lecciones_hoy() -> String {
    chrono::Utc::now().format("%Y-%m-%d").to_string()
}

// ---------------------------------------------------------------------------
// journey (feature #22)
// ---------------------------------------------------------------------------

/// Repo con los tres almacenes coherentes entre si.
fn seed_journey(root: &Path, hp: &Path) {
    std::fs::write(
        hp.join("feature_list.json"),
        r#"{"features": [
            {"id": 17, "name": "lecciones", "status": "done",
             "closed_at": "2026-08-16T20:00:00Z", "leccion": "docs-generados"},
            {"id": 19, "name": "perfil", "status": "done",
             "closed_at": "2026-08-17T10:00:00Z", "leccion": "documentos-del-usuario"}
        ]}"#,
    )
    .unwrap();
    let lec = root.join("docs/lecciones");
    std::fs::create_dir_all(&lec).unwrap();
    for (nombre, origen, fecha) in [
        ("docs-generados", "17", "2026-08-16"),
        ("hitos-del-prd", "17", "2026-08-16"),
        ("documentos-del-usuario", "19", "2026-08-17"),
    ] {
        std::fs::write(
            lec.join(format!("{nombre}.md")),
            format!(
                "---\nnombre: {nombre}\ndescripcion: Sobre {nombre}.\norigen: [{origen}]\n\
                 usos: 0\nultimo_uso:\nultima_actualizacion: {fecha}\nestado: activa\n---\n\ncuerpo\n"
            ),
        )
        .unwrap();
    }
    std::fs::write(
        root.join("docs/perfil-usuario.md"),
        "# Perfil de usuario\n\nEntradas:\n\n- Una preferencia durable. (#17, #19)\n",
    )
    .unwrap();
}

#[test]
fn journey_should_say_so_on_a_fresh_repo() {
    // AC-15.
    let (_dir, bin) = sandbox_with_binary();
    cmd(&bin)
        .arg("journey")
        .assert()
        .success()
        .stdout(predicate::str::contains("Todavia no hay nada que mapear"));
}

#[test]
fn journey_should_show_both_lessons_of_a_feature_without_duplicating() {
    // AC-2 + el criterio de cierre: la feature que declaro una leccion y pario
    // otra tiene que mostrar LAS DOS, y la declarada una sola vez.
    let (dir, bin) = sandbox_with_binary();
    seed_journey(dir.path(), &dir.path().join("hp"));
    let out = cmd(&bin).arg("journey").output().unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("[leccion declarada] docs-generados"), "{stdout}");
    assert!(stdout.contains("[leccion (origen)] hitos-del-prd"), "{stdout}");
    // Cuenta las LINEAS de nodo hijo, no la subcadena (que tambien aparece en la
    // descripcion "Sobre docs-generados.").
    assert_eq!(
        stdout.matches("] docs-generados").count(),
        1,
        "la declarada sale duplicada:\n{stdout}"
    );
}

#[test]
fn journey_should_anchor_the_profile_entry_to_its_most_recent_feature() {
    // AC-4 + criterio de cierre: cuelga de la #19, no de la #17.
    let (dir, bin) = sandbox_with_binary();
    seed_journey(dir.path(), &dir.path().join("hp"));
    let out = cmd(&bin).arg("journey").output().unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(
        stdout.matches("Una preferencia durable").count(),
        1,
        "la entrada se repite en cada feature citada:\n{stdout}"
    );
    // Aparece despues de la #19 en el texto.
    let pos_19 = stdout.find("#19").unwrap();
    let pos_entrada = stdout.find("Una preferencia durable").unwrap();
    assert!(pos_entrada > pos_19, "{stdout}");
}

#[test]
fn journey_should_report_a_coherent_repo_as_such() {
    // AC-10.
    let (dir, bin) = sandbox_with_binary();
    seed_journey(dir.path(), &dir.path().join("hp"));
    cmd(&bin)
        .arg("journey")
        .assert()
        .success()
        .stdout(predicate::str::contains("Sin huecos"));
}

#[test]
fn journey_should_report_gaps_with_the_command_that_fixes_them() {
    // AC-6, AC-12: el mapa no poda, senala el comando del almacen.
    let (dir, bin) = sandbox_with_binary();
    seed_journey(dir.path(), &dir.path().join("hp"));
    std::fs::write(
        dir.path().join("docs/lecciones/rota.md"),
        "---\nnombre: rota\ndescripcion: Con origen inexistente.\norigen: [99]\n\
         usos: 0\nultimo_uso:\nultima_actualizacion: 2026-08-17\nestado: activa\n---\n\ncuerpo\n",
    )
    .unwrap();
    cmd(&bin)
        .arg("journey")
        .assert()
        .success()
        .stdout(predicate::str::contains("[enlace-roto]"))
        .stdout(predicate::str::contains("#99"))
        .stdout(predicate::str::contains("harness_cli leccion show rota"))
        .stdout(predicate::str::contains("journey no escribe nada"));
}

#[test]
fn journey_json_should_expose_nodes_links_and_gaps() {
    // AC-13.
    let (dir, bin) = sandbox_with_binary();
    seed_journey(dir.path(), &dir.path().join("hp"));
    let out = cmd(&bin).args(["journey", "--json"]).output().unwrap();
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert!(!json["nodos"].as_array().unwrap().is_empty());
    let clases: Vec<&str> = json["enlaces"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["clase"].as_str().unwrap())
        .collect();
    assert!(clases.contains(&"declarada"), "{clases:?}");
    assert!(clases.contains(&"origen"), "{clases:?}");
    assert!(clases.contains(&"cita"), "{clases:?}");
    assert_eq!(json["huecos"].as_array().unwrap().len(), 0);
}

#[test]
fn journey_should_write_nothing_and_ignore_the_hub() {
    // AC-11, AC-14.
    let (dir, bin) = sandbox_with_binary();
    seed_journey(dir.path(), &dir.path().join("hp"));
    let antes = std::fs::read_dir(dir.path().join("docs/lecciones")).unwrap().count();
    let normal = cmd(&bin).args(["journey", "--json"]).output().unwrap();
    let sin_hub = cmd(&bin)
        .env("DB_HOST", "127.0.0.1")
        .env("DB_PORT", "1")
        .env("DB_USER", "nadie")
        .env("DB_PASSWORD", "nada")
        .env("DB_NAME", "nada")
        .args(["journey", "--json"])
        .output()
        .unwrap();
    assert_eq!(normal.stdout, sin_hub.stdout, "el hub no puede cambiar el mapa");
    assert_eq!(
        std::fs::read_dir(dir.path().join("docs/lecciones")).unwrap().count(),
        antes,
        "journey no puede crear archivos"
    );
}

#[test]
fn journey_should_not_report_closes_from_before_the_lessons_existed() {
    // El hallazgo que redujo 16 huecos a 0: las features viejas no son huecos.
    let (dir, bin) = sandbox_with_binary();
    let hp = dir.path().join("hp");
    seed_journey(dir.path(), &hp);
    let texto = std::fs::read_to_string(hp.join("feature_list.json")).unwrap();
    let con_vieja = texto.replace(
        r#""features": ["#,
        r#""features": [
            {"id": 3, "name": "prehistorica", "status": "done", "closed_at": "2026-07-01T00:00:00Z"},"#,
    );
    std::fs::write(hp.join("feature_list.json"), con_vieja).unwrap();
    cmd(&bin)
        .arg("journey")
        .assert()
        .success()
        .stdout(predicate::str::contains("prehistorica"))
        .stdout(predicate::str::contains("Sin huecos"));
}

// ---------------------------------------------------------------------------
// Feature #23: AC ejecutables (`verify`) y el gate de cierre.
//
// Es el unico comando del arnes que ejecuta shell. Los tests de aca abajo son,
// sobre todo, tests de las BARRERAS: que no ejecuta sin spec aprobado y que el
// cierre no ejecuta nunca. Los dos se comprueban con un comando que dejaria
// rastro en el disco: si el archivo no aparece, no corrio.
// ---------------------------------------------------------------------------

fn enable_verify_rule(harness_dir: &Path) {
    let path = harness_dir.join("feature_list.json");
    let text = std::fs::read_to_string(&path).unwrap();
    let mut data: serde_json::Value = serde_json::from_str(&text).unwrap();
    let obj = data.as_object_mut().unwrap();
    let rules = obj
        .entry("rules".to_string())
        .or_insert_with(|| serde_json::json!({}));
    rules
        .as_object_mut()
        .unwrap()
        .insert("require_verify_green".to_string(), serde_json::json!(true));
    std::fs::write(&path, serde_json::to_string_pretty(&data).unwrap()).unwrap();
}

/// Reemplaza los AC del spec por los que le pasemos, manteniendo el resto.
fn escribir_acs(spec: &Path, acs: &str) {
    let texto = std::fs::read_to_string(spec).unwrap();
    let marca = "- AC-1: Given <contexto>, When <accion>, Then <resultado observable>.\n  Comando: `<como se prueba, ejecutable desde la raiz>`";
    assert!(texto.contains(marca), "cambio la plantilla del spec");
    std::fs::write(spec, texto.replace(marca, acs)).unwrap();
}

fn feature_con_spec(acs: &str) -> (tempfile::TempDir, PathBuf, PathBuf) {
    let (dir, bin) = sandbox_with_binary();
    cmd(&bin).args(["add", "--name", "Demo"]).assert().success();
    cmd(&bin).args(["start", "--feature", "1"]).assert().success();
    let spec = dir.path().join("docs/spec-feature-1-demo.md");
    escribir_acs(&spec, acs);
    (dir, bin, spec)
}

#[test]
fn verify_should_refuse_to_run_commands_from_a_draft_spec() {
    // AC-5, la barrera central: el comando escribiria `rastro.txt`. Si el
    // archivo no existe, es que verify no llego a ejecutarlo.
    let (dir, bin, _spec) = feature_con_spec(
        "- AC-1: Given algo, Then otra.\n  Comando: `touch rastro.txt`",
    );
    cmd(&bin)
        .args(["verify", "--feature", "1"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("[BARRERA] Spec sin aprobar"))
        .stderr(predicate::str::contains("approve-spec"));
    assert!(
        !dir.path().join("rastro.txt").exists(),
        "verify ejecuto un comando de un spec en draft"
    );
    assert!(!dir.path().join("docs/verify-1.md").exists());
}

/// Spec de tres AC: uno verde, uno rojo con salida, uno sin comando.
const TRES_AC: &str = "- AC-1: Given algo, Then otra.\n  Comando: `true`\n\
     - AC-2: Given algo, Then falla.\n  Comando: `echo se-rompio-esto >&2; exit 4`\n\
     - AC-3: Given algo, Then a mano.";

#[test]
fn verify_should_print_each_command_before_running_it() {
    // AC-4: nada se corre a ciegas. El comando aparece en stdout junto a su AC.
    let (_dir, bin, _spec) = feature_con_spec(TRES_AC);
    cmd(&bin).args(["approve-spec", "--yes"]).assert().success();
    cmd(&bin)
        .args(["verify", "--feature", "1"])
        .assert()
        .code(1)
        .stdout(predicate::str::contains("AC-1  $ true"))
        .stdout(predicate::str::contains("AC-2  $ echo se-rompio-esto >&2; exit 4"))
        .stdout(predicate::str::contains("1 verde(s), 1 en rojo, 1 manual(es)"))
        .stderr(predicate::str::contains("AC en rojo: AC-2"));
}

#[test]
fn verify_should_write_a_report_per_ac() {
    // AC-8: numero, comando, exit code, duracion y estado, por cada AC.
    let (dir, bin, _spec) = feature_con_spec(TRES_AC);
    cmd(&bin).args(["approve-spec", "--yes"]).assert().success();
    cmd(&bin).args(["verify", "--feature", "1"]).assert().code(1);
    let reporte = std::fs::read_to_string(dir.path().join("docs/verify-1.md")).unwrap();
    assert!(reporte.contains("| AC-1 | verde | `true` | 0 |"), "{reporte}");
    assert!(reporte.contains("| AC-2 | rojo |"), "{reporte}");
    assert!(reporte.contains("| 4 |"), "falta el exit code: {reporte}");
    assert!(reporte.contains("| AC-3 | manual | `(verificacion manual)`"), "{reporte}");
}

#[test]
fn verify_should_include_output_of_failures() {
    // AC-9: se diagnostica leyendo el reporte, sin re-correr.
    let (dir, bin, _spec) = feature_con_spec(TRES_AC);
    cmd(&bin).args(["approve-spec", "--yes"]).assert().success();
    cmd(&bin)
        .args(["verify", "--feature", "1"])
        .assert()
        .code(1)
        .stdout(predicate::str::contains("se-rompio-esto"));
    let reporte = std::fs::read_to_string(dir.path().join("docs/verify-1.md")).unwrap();
    assert!(reporte.contains("### AC-2 (rojo)"), "{reporte}");
    assert!(reporte.contains("se-rompio-esto"), "{reporte}");
    // El verde no ensucia el reporte con su salida: solo se guarda lo que fallo.
    assert!(!reporte.contains("### AC-1"), "{reporte}");
}

#[test]
fn verify_should_keep_going_after_a_failure() {
    // AC-6: un AC roto no corta la corrida; el valor esta en ver todo lo roto.
    let (dir, bin, _spec) = feature_con_spec(
        "- AC-1: uno.\n  Comando: `false`\n\
         - AC-2: dos.\n  Comando: `touch corrio-el-segundo.txt`",
    );
    cmd(&bin).args(["approve-spec", "--yes"]).assert().success();
    cmd(&bin).args(["verify", "--feature", "1"]).assert().code(1);
    assert!(
        dir.path().join("corrio-el-segundo.txt").exists(),
        "el fallo del AC-1 corto la corrida"
    );
}

#[test]
fn verify_should_do_nothing_without_declared_commands() {
    // AC-2: los 310 AC ya escritos no declaran comandos y no son un error.
    let (dir, bin, _spec) = feature_con_spec("- AC-1: Given algo, Then otra.");
    cmd(&bin).args(["approve-spec", "--yes"]).assert().success();
    cmd(&bin)
        .args(["verify", "--feature", "1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("ninguno declara"));
    assert!(!dir.path().join("docs/verify-1.md").exists());
}

#[test]
fn verify_should_run_a_single_ac_on_demand() {
    // AC-11: iterar sobre uno solo mientras se lo arregla.
    let (dir, bin, _spec) = feature_con_spec(
        "- AC-1: uno.\n  Comando: `touch no-deberia.txt`\n\
         - AC-2: dos.\n  Comando: `touch si-deberia.txt`",
    );
    cmd(&bin).args(["approve-spec", "--yes"]).assert().success();
    cmd(&bin)
        .args(["verify", "--feature", "1", "--solo", "AC-2"])
        .assert()
        .success();
    assert!(dir.path().join("si-deberia.txt").exists());
    assert!(!dir.path().join("no-deberia.txt").exists());
    cmd(&bin)
        .args(["verify", "--feature", "1", "--solo", "AC-9"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("no declara AC-9"));
}

#[test]
fn verify_json_should_expose_the_result_per_ac() {
    // AC-10: JSON parseable con el estado de cada AC.
    let (_dir, bin, _spec) = feature_con_spec(
        "- AC-1: uno.\n  Comando: `exit 7`\n- AC-2: dos.",
    );
    cmd(&bin).args(["approve-spec", "--yes"]).assert().success();
    let out = cmd(&bin)
        .args(["verify", "--feature", "1", "--json"])
        .output()
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["verde"], serde_json::json!(false));
    assert_eq!(v["resultados"][0]["ac"], "AC-1");
    assert_eq!(v["resultados"][0]["estado"], "rojo");
    assert_eq!(v["resultados"][0]["exit"], 7);
    assert_eq!(v["resultados"][0]["comando"], "exit 7");
    assert_eq!(v["resultados"][1]["estado"], "manual");
    assert_eq!(v["reporte"], "docs/verify-1.md");
}

#[test]
fn verify_should_time_out_a_hung_command() {
    // AC-6: el timeout sale de rules y el AC queda en timeout, no colgado.
    let (dir, bin, _spec) = feature_con_spec("- AC-1: uno.\n  Comando: `sleep 30`");
    let path = dir.path().join("hp/feature_list.json");
    let mut data: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    data.as_object_mut().unwrap().insert(
        "rules".to_string(),
        serde_json::json!({"verify_timeout_segundos": 1}),
    );
    std::fs::write(&path, serde_json::to_string_pretty(&data).unwrap()).unwrap();
    cmd(&bin).args(["approve-spec", "--yes"]).assert().success();
    cmd(&bin).args(["verify", "--feature", "1"]).assert().code(1);
    let reporte = std::fs::read_to_string(dir.path().join("docs/verify-1.md")).unwrap();
    assert!(reporte.contains("| AC-1 | timeout |"), "{reporte}");
}

#[test]
fn close_should_demand_a_verify_report() {
    // AC-12: con comandos declarados y la regla activa, falta el reporte.
    let (dir, bin, _spec) = feature_con_spec("- AC-1: uno.\n  Comando: `true`");
    cmd(&bin).args(["approve-spec", "--yes"]).assert().success();
    enable_verify_rule(&dir.path().join("hp"));
    cmd(&bin)
        .args(["close", "--feature", "1", "--status", "done"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("Falta el reporte de verificacion"))
        .stderr(predicate::str::contains("docs/verify-1.md"));
}

#[test]
fn close_should_not_gate_a_spec_without_commands_even_with_the_rule_on() {
    // AC-13: la regla activa NO rompe las features cuyos AC no declaran nada.
    let (dir, bin, _spec) = feature_con_spec("- AC-1: uno, sin comando.");
    cmd(&bin).args(["approve-spec", "--yes"]).assert().success();
    enable_verify_rule(&dir.path().join("hp"));
    cmd(&bin)
        .args(["close", "--feature", "1", "--status", "done"])
        .assert()
        .success();
}

#[test]
fn close_should_stay_identical_without_the_verify_rule() {
    // AC-12: sin la regla, cerrar es exactamente lo de siempre — aunque el spec
    // declare comandos y no exista ningun reporte.
    let (dir, bin, _spec) = feature_con_spec("- AC-1: uno.\n  Comando: `false`");
    cmd(&bin).args(["approve-spec", "--yes"]).assert().success();
    assert!(!dir.path().join("docs/verify-1.md").exists());
    cmd(&bin)
        .args(["close", "--feature", "1", "--status", "done"])
        .assert()
        .success();
}

#[test]
fn close_should_block_on_a_red_report() {
    // AC-14: nombra CUALES fallaron, no solo que algo fallo.
    let (dir, bin, _spec) = feature_con_spec(
        "- AC-1: uno.\n  Comando: `false`\n- AC-2: dos.\n  Comando: `true`",
    );
    cmd(&bin).args(["approve-spec", "--yes"]).assert().success();
    cmd(&bin).args(["verify", "--feature", "1"]).assert().code(1);
    enable_verify_rule(&dir.path().join("hp"));
    cmd(&bin)
        .args(["close", "--feature", "1", "--status", "done"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("Hay AC en rojo: AC-1"))
        .stderr(predicate::str::contains("AC-2").not());
}

#[test]
fn close_should_never_execute_verify_commands() {
    // AC-16: el reporte esta VERDE, asi que el cierre pasa. Si en el camino
    // ejecutara los comandos del spec, el rastro volveria a aparecer.
    let (dir, bin, _spec) = feature_con_spec(
        "- AC-1: uno.\n  Comando: `touch rastro-del-cierre.txt`",
    );
    cmd(&bin).args(["approve-spec", "--yes"]).assert().success();
    cmd(&bin).args(["verify", "--feature", "1"]).assert().success();
    std::fs::remove_file(dir.path().join("rastro-del-cierre.txt")).unwrap();
    enable_verify_rule(&dir.path().join("hp"));
    cmd(&bin)
        .args(["close", "--feature", "1", "--status", "done"])
        .assert()
        .success();
    assert!(
        !dir.path().join("rastro-del-cierre.txt").exists(),
        "el cierre ejecuto un comando del spec"
    );
}

#[test]
fn start_should_document_the_command_line_in_the_spec_template() {
    // AC-17: el proximo spec ofrece la linea solo, sin que nadie la recuerde.
    let (dir, bin) = sandbox_with_binary();
    cmd(&bin).args(["add", "--name", "Demo"]).assert().success();
    cmd(&bin).args(["start", "--feature", "1"]).assert().success();
    let spec = std::fs::read_to_string(dir.path().join("docs/spec-feature-1-demo.md")).unwrap();
    assert!(spec.contains("Comando: `<como se prueba"), "{spec}");
    assert!(spec.contains("harness_cli verify --feature"), "{spec}");
    assert!(spec.contains("no declarar comando NO es un fallo"), "{spec}");
}

#[test]
fn close_should_block_on_a_stale_report() {
    // AC-15: el spec cambio despues de la corrida; lo verificado ya no aplica.
    let (dir, bin, spec) = feature_con_spec("- AC-1: uno.\n  Comando: `true`");
    cmd(&bin).args(["approve-spec", "--yes"]).assert().success();
    cmd(&bin).args(["verify", "--feature", "1"]).assert().success();
    let despues = filetime::FileTime::from_unix_time(
        filetime::FileTime::from_last_modification_time(
            &std::fs::metadata(dir.path().join("docs/verify-1.md")).unwrap(),
        )
        .unix_seconds()
            + 60,
        0,
    );
    filetime::set_file_mtime(&spec, despues).unwrap();
    enable_verify_rule(&dir.path().join("hp"));
    cmd(&bin)
        .args(["close", "--feature", "1", "--status", "done"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("mas viejo que el spec"));
}

#[test]
fn close_should_pass_with_a_fresh_green_report() {
    let (dir, bin, _spec) = feature_con_spec("- AC-1: uno.\n  Comando: `true`");
    cmd(&bin).args(["approve-spec", "--yes"]).assert().success();
    cmd(&bin).args(["verify", "--feature", "1"]).assert().success();
    enable_verify_rule(&dir.path().join("hp"));
    cmd(&bin)
        .args(["close", "--feature", "1", "--status", "done"])
        .assert()
        .success();
}

#[test]
fn only_verify_should_execute_declared_commands() {
    // AC-7 de la #23 ("la invocacion de verify es manual"), reescrito en la #24
    // como CONTRATO DE COMPORTAMIENTO.
    //
    // La version anterior grepeaba src/**/*.rs y setup_harness.sh buscando la
    // cadena "verify::run". Violaba la regla "prohibido leer el codigo fuente en
    // un test" de docs/conventions.md, y de las dos maneras que la regla
    // describe: pasaba aunque verify estuviera mal cableado (bastaba invocarlo
    // por otro camino) y fallaba ante un refactor correcto (renombrar la
    // funcion). Ademas empezo a fallar cuando la #23 DOCUMENTO verify en las
    // superficies, que es obligatorio, y hubo que ensenarle a distinguir prosa
    // de codigo — sintoma de que estaba probando la forma del fuente.
    //
    // Ahora se mira el disco: el spec declara un comando que dejaria un archivo,
    // y se corre el arnes entero. Si algun comando ejecutara lo declarado, el
    // archivo aparece. Sobrevive a cualquier reescritura de la implementacion.
    let (dir, bin) = sandbox_with_binary();
    cmd(&bin).args(["add", "--name", "Demo"]).assert().success();
    cmd(&bin).args(["start", "--feature", "1"]).assert().success();
    let spec = dir.path().join("docs/spec-feature-1-demo.md");
    escribir_acs(&spec, "- AC-1: uno.\n  Comando: `touch rastro-de-ejecucion.txt`");
    cmd(&bin).args(["approve-spec", "--yes"]).assert().success();
    let rastro = dir.path().join("rastro-de-ejecucion.txt");

    // Todos los comandos que corren en el flujo normal, incluidos los que
    // disparan los hooks (autocheck, nudge) y el cierre.
    for args in [
        vec!["status"],
        vec!["next"],
        vec!["check-plan"],
        vec!["check-spec"],
        vec!["autocheck"],
        vec!["nudge"],
        vec!["advance", "--nota", "sin ejecutar nada"],
        vec!["leccion", "list"],
        vec!["journey"],
        vec!["buscar", "verify"],
        vec!["close", "--feature", "1", "--status", "done"],
    ] {
        let _ = cmd(&bin).args(&args).output();
        assert!(
            !rastro.exists(),
            "`{}` ejecuto el Comando: declarado en el spec; solo verify puede",
            args.join(" ")
        );
    }

    // Control positivo: sin esto, el test pasaria igual si el rastro fuera
    // imposible de crear, y no estaria probando nada.
    cmd(&bin).args(["verify", "--feature", "1"]).assert().success();
    assert!(
        rastro.exists(),
        "verify no ejecuto el comando declarado: el test no prueba nada"
    );
}

// ---------------------------------------------------------------------------
// Feature #25: `doctor` diagnostica la INSTALACION.
//
// Contrato de comportamiento, no de forma (docs/conventions.md, regla 2): se
// corre el comando y se mira su salida y el disco, nunca el texto del fuente.
// ---------------------------------------------------------------------------

fn doctor_json(bin: &Path) -> serde_json::Value {
    let out = cmd(bin).args(["doctor", "--json"]).output().unwrap();
    serde_json::from_slice(&out.stdout).unwrap_or_else(|e| {
        panic!(
            "doctor --json tiene que ser JSON valido ({e}): {}",
            String::from_utf8_lossy(&out.stdout)
        )
    })
}

#[test]
fn doctor_should_report_every_area_on_a_healthy_install() {
    // AC-1: las siete areas, cada una con su estado.
    let (_dir, bin) = sandbox_with_binary();
    let v = doctor_json(&bin);
    let areas: Vec<String> = v["areas"]
        .as_array()
        .unwrap()
        .iter()
        .map(|a| a["area"].as_str().unwrap().to_string())
        .collect();
    for esperada in [
        "binario",
        "hooks",
        "superficies",
        "marker",
        "hub",
        "herramientas",
        "graphify",
        "rutas_protegidas",
    ] {
        assert!(areas.contains(&esperada.to_string()), "falta {esperada}: {areas:?}");
    }
}

#[test]
fn doctor_should_print_an_exact_remedy_for_every_problem() {
    // AC-2: una falla sin comando de remedio es una queja, no un diagnostico.
    // Se siembra un problema real (backend instalado sin superficie ni hooks)
    // para que el test tenga fallas que revisar y no solo estados ok.
    let (dir, bin) = sandbox_with_binary();
    std::fs::create_dir_all(dir.path().join(".claude")).unwrap();
    std::fs::write(dir.path().join(".claude/settings.json"), "{}").unwrap();
    let v = doctor_json(&bin);
    assert!(v["fallas"].as_u64().unwrap() > 0, "el test no sembro ninguna falla: {v}");
    for area in v["areas"].as_array().unwrap() {
        let estado = area["estado"].as_str().unwrap();
        if estado == "falla" || estado == "aviso" {
            let remedio = area["remedio"].as_str().unwrap_or("");
            assert!(
                !remedio.trim().is_empty(),
                "{} en {estado} sin remedio",
                area["area"]
            );
        }
    }
}

#[test]
fn doctor_should_separate_failures_from_warnings() {
    // AC-3: solo las fallas cambian el exit code. En un sandbox sano puede
    // haber avisos (hub sin configurar, herramientas opcionales) y aun asi 0.
    let (_dir, bin) = sandbox_with_binary();
    let v = doctor_json(&bin);
    let fallas = v["fallas"].as_u64().unwrap();
    let salida = cmd(&bin).arg("doctor").output().unwrap();
    let code = salida.status.code().unwrap_or(-1);
    if fallas == 0 {
        assert_eq!(code, 0, "sin fallas el exit tiene que ser 0");
    } else {
        assert_eq!(code, 2, "con fallas el exit tiene que ser 2");
    }
}

#[test]
fn doctor_json_should_expose_area_state_and_remedy() {
    // AC-4: los cuatro campos por area, para que un script pueda decidir.
    let (_dir, bin) = sandbox_with_binary();
    let v = doctor_json(&bin);
    let primera = &v["areas"][0];
    for campo in ["area", "estado", "detalle", "remedio"] {
        assert!(primera.get(campo).is_some(), "falta el campo {campo}: {primera}");
    }
    assert!(v["sana"].is_boolean());
}

#[test]
fn doctor_should_detect_a_binary_older_than_the_scripts() {
    // AC-5: el caso que ya rompio dos veces (`git pull` sin re-instalar).
    let (dir, bin) = sandbox_with_binary();
    let hp = dir.path().join("hp");
    std::fs::write(hp.join("harness_cli"), "#!/bin/sh\n").unwrap();
    let viejo = filetime::FileTime::from_unix_time(1_600_000_000, 0);
    filetime::set_file_mtime(&bin, viejo).unwrap();
    let v = doctor_json(&bin);
    let binario = v["areas"]
        .as_array()
        .unwrap()
        .iter()
        .find(|a| a["area"] == "binario")
        .unwrap();
    assert_eq!(binario["estado"], "falla", "{binario}");
    assert!(
        binario["detalle"].as_str().unwrap().contains("git pull"),
        "{binario}"
    );
    assert_eq!(binario["remedio"], "bash setup_harness.sh");
}

#[test]
fn doctor_should_detect_a_hook_pointing_nowhere() {
    // AC-6: backend instalado y el runtime de hooks ausente.
    let (dir, bin) = sandbox_with_binary();
    std::fs::create_dir_all(dir.path().join(".claude")).unwrap();
    std::fs::write(dir.path().join(".claude/settings.json"), "{}").unwrap();
    let v = doctor_json(&bin);
    let hooks = v["areas"]
        .as_array()
        .unwrap()
        .iter()
        .find(|a| a["area"] == "hooks")
        .unwrap();
    assert_eq!(hooks["estado"], "falla", "{hooks}");
    assert!(hooks["detalle"].as_str().unwrap().contains("claude"), "{hooks}");
}

#[test]
fn doctor_should_only_demand_surfaces_the_backend_uses() {
    // AC-7: con solo Claude instalado, la falta de GEMINI.md no es problema.
    let (dir, bin) = sandbox_with_binary();
    std::fs::create_dir_all(dir.path().join(".claude")).unwrap();
    std::fs::write(dir.path().join(".claude/settings.json"), "{}").unwrap();
    let v = doctor_json(&bin);
    let sup = v["areas"]
        .as_array()
        .unwrap()
        .iter()
        .find(|a| a["area"] == "superficies")
        .unwrap();
    let detalle = sup["detalle"].as_str().unwrap();
    assert!(detalle.contains("CLAUDE.md"), "{detalle}");
    assert!(!detalle.contains("GEMINI.md"), "no debe pedir Gemini: {detalle}");
}

#[test]
fn doctor_should_explain_which_root_it_resolved_and_why() {
    // AC-8: lo que costo la feature #10 entera.
    let (_dir, bin) = sandbox_with_binary();
    let v = doctor_json(&bin);
    let marker = v["areas"]
        .as_array()
        .unwrap()
        .iter()
        .find(|a| a["area"] == "marker")
        .unwrap();
    assert!(
        marker["detalle"].as_str().unwrap().contains("raiz resuelta"),
        "{marker}"
    );
}

#[test]
fn doctor_should_treat_an_unreachable_hub_as_a_warning() {
    // AC-9: el hub caido nunca puede hacer mentir al exit code.
    let (_dir, bin) = sandbox_with_binary();
    let v = doctor_json(&bin);
    let hub = v["areas"]
        .as_array()
        .unwrap()
        .iter()
        .find(|a| a["area"] == "hub")
        .unwrap();
    assert_ne!(hub["estado"], "falla", "el hub nunca bloquea: {hub}");
}

#[test]
fn doctor_should_split_required_and_optional_tools() {
    // AC-10: `git` es requerida; las opcionales solo avisan.
    let (_dir, bin) = sandbox_with_binary();
    let v = doctor_json(&bin);
    let herr = v["areas"]
        .as_array()
        .unwrap()
        .iter()
        .find(|a| a["area"] == "herramientas")
        .unwrap();
    assert_ne!(herr["estado"], "falla", "git deberia estar presente: {herr}");
}

#[test]
fn doctor_should_report_graphify_as_optional() {
    // AC-11: el arnes funciona sin graphify.
    let (_dir, bin) = sandbox_with_binary();
    let v = doctor_json(&bin);
    let g = v["areas"]
        .as_array()
        .unwrap()
        .iter()
        .find(|a| a["area"] == "graphify")
        .unwrap();
    assert_ne!(g["estado"], "falla", "graphify nunca bloquea: {g}");
}

#[test]
fn doctor_should_not_demand_surfaces_in_a_source_checkout() {
    // AC-12: en el repo del arnes, su ausencia es lo CORRECTO.
    let (_dir, bin) = sandbox_source_checkout();
    let v = doctor_json(&bin);
    for area in ["superficies", "hooks"] {
        let a = v["areas"]
            .as_array()
            .unwrap()
            .iter()
            .find(|x| x["area"] == area)
            .unwrap();
        assert_eq!(a["estado"], "no_aplica", "{a}");
    }
    assert_eq!(v["fallas"], 0, "un checkout fuente no puede tener fallas: {v}");
}

#[test]
fn doctor_should_not_duplicate_the_process_checks() {
    // AC-14: doctor mira la instalacion; harness_check.sh mira el proceso. Si
    // doctor hablara de specs o lecciones, serian dos herramientas diciendo lo
    // mismo con palabras distintas, que confunde mas que una sola.
    //
    // El no-solapamiento se asserta sobre el CONJUNTO DE AREAS, no sobre las
    // palabras: doctor no agrega un area de spec, plan, PRD, leccion ni perfil.
    // Dos intentos anteriores fallaron por grepear prosa — el primero por la
    // linea que remite a harness_check.sh (que el AC exige), el segundo porque
    // el area del hub explica que "lecciones, perfil, buscar y journey son
    // archivos", que es informacion util y no un diagnostico del proceso.
    let (_dir, bin) = sandbox_with_binary();
    let v = doctor_json(&bin);
    let areas: std::collections::BTreeSet<String> = v["areas"]
        .as_array()
        .unwrap()
        .iter()
        .map(|a| a["area"].as_str().unwrap().to_string())
        .collect();
    let esperadas: std::collections::BTreeSet<String> = [
        "binario",
        "hooks",
        "superficies",
        "marker",
        "hub",
        "herramientas",
        "graphify",
        "rutas_protegidas",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    assert_eq!(
        areas, esperadas,
        "doctor solo revisa la instalacion; el proceso es de harness_check.sh"
    );
    // Y la salida humana remite explicitamente a donde SI se revisa el proceso.
    let salida = cmd(&bin).arg("doctor").output().unwrap();
    let texto = String::from_utf8_lossy(&salida.stdout);
    assert!(texto.contains("harness_check.sh"), "{texto}");
}

#[test]
fn doctor_should_not_write_anything() {
    // AC-15: solo lee. Se compara el arbol entero antes y despues.
    let (dir, bin) = sandbox_with_binary();
    let antes = huella_del_arbol(dir.path());
    cmd(&bin).arg("doctor").output().unwrap();
    cmd(&bin).args(["doctor", "--json"]).output().unwrap();
    assert_eq!(antes, huella_del_arbol(dir.path()), "doctor modifico el arbol");
}

/// Ruta + mtime de cada archivo, ordenado: detecta creaciones, borrados y
/// escrituras.
fn huella_del_arbol(raiz: &Path) -> Vec<String> {
    let mut out = Vec::new();
    let mut pendientes = vec![raiz.to_path_buf()];
    while let Some(dir) = pendientes.pop() {
        let Ok(entradas) = std::fs::read_dir(&dir) else {
            continue;
        };
        for e in entradas.flatten() {
            let p = e.path();
            if p.is_dir() {
                pendientes.push(p);
            } else if let Ok(m) = std::fs::metadata(&p) {
                let t = m
                    .modified()
                    .ok()
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_nanos())
                    .unwrap_or(0);
                out.push(format!("{} {t} {}", p.display(), m.len()));
            }
        }
    }
    out.sort();
    out
}

// ---------------------------------------------------------------------------
// Feature #26: rutas protegidas. El arnes no se bloquea a si mismo.
// ---------------------------------------------------------------------------

#[test]
fn close_should_still_write_the_prd_milestone_when_protected() {
    // AC-9: `docs/prd/**` esta protegida por defecto Y `close` escribe ahi al
    // marcar el hito. Si la proteccion alcanzara al binario, el arnes se
    // bloquearia a si mismo en cada cierre.
    let (dir, bin) = sandbox_with_binary();
    let prd = dir.path().join("docs/prd");
    std::fs::create_dir_all(&prd).unwrap();
    std::fs::write(
        prd.join("PRD-master.md"),
        concat!(
            "# PRD\n\n",
            "## 10. Hitos -> features\n\n",
            "| # | Hito | Slug de feature | Objetivo | Criterio | Estado |\n",
            "| --- | --- | --- | --- | --- | --- |\n",
            "| 1 | Algo | demo | <O1> | que pase | pendiente |\n\n",
            "## Bitacora\n\n-\n",
        ),
    )
    .unwrap();
    cmd(&bin).args(["add", "--name", "demo"]).assert().success();
    cmd(&bin).args(["start", "--feature", "1"]).assert().success();
    cmd(&bin)
        .args(["close", "--feature", "1", "--status", "done"])
        .assert()
        .success();
    let texto = std::fs::read_to_string(prd.join("PRD-master.md")).unwrap();
    assert!(
        texto.contains("done ("),
        "el arnes no pudo marcar el hito en una ruta protegida: {texto}"
    );
    // Y la escritura quedo registrada como propia, asi que no es violacion.
    let registro = std::fs::read_to_string(dir.path().join("hp/progress/.rutas_arnes"))
        .unwrap_or_default();
    assert!(
        registro.contains("docs/prd/PRD-master.md"),
        "close no registro su propia escritura: {registro}"
    );
}

#[test]
fn rutas_should_answer_whether_a_path_is_protected() {
    let (_dir, bin) = sandbox_with_binary();
    // Protegida -> exit 2 y la nombra.
    cmd(&bin)
        .args(["rutas", "--check", "docs/constitution.md"])
        .assert()
        .code(2)
        .stdout(predicate::str::contains("docs/constitution.md"));
    // No protegida -> exit 0 y no dice nada.
    cmd(&bin)
        .args(["rutas", "--check", "rust/src/main.rs"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

#[test]
fn rutas_should_list_the_active_configuration() {
    let (_dir, bin) = sandbox_with_binary();
    cmd(&bin)
        .arg("rutas")
        .assert()
        .success()
        .stdout(predicate::str::contains("docs/prd/**"))
        .stdout(predicate::str::contains("docs/constitution.md"));
}

// ---------------------------------------------------------------------------
// Feature #36: las seis deudas que el arnes se anoto en sus propios impl.
// ---------------------------------------------------------------------------

#[test]
fn close_gates_should_share_one_exit_code() {
    // Deuda de impl-23. La nota decia "1 / 1 / 2" y al medirla estaba mal: el
    // gate de leccion ya salia 2. El unico distinto era el de spec.
    let (dir, bin) = sandbox_with_binary();
    let hp = dir.path().join("hp");
    cmd(&bin).args(["add", "--name", "Demo"]).assert().success();
    cmd(&bin).args(["start", "--feature", "1"]).assert().success();

    // Gate 1: spec sin aprobar.
    enable_spec_rule(&hp);
    cmd(&bin)
        .args(["close", "--feature", "1", "--status", "done"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("Spec sin aprobar"));

    // Gate 2: leccion sin declarar.
    cmd(&bin).args(["approve-spec", "--yes"]).assert().success();
    enable_leccion_rule(&hp);
    cmd(&bin)
        .args(["close", "--feature", "1", "--status", "done"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("no declara que se aprendio"));
}

#[test]
fn close_should_keep_usage_errors_separate_from_gates() {
    // El cambio de exit code no puede borrar la diferencia entre "el arnes te
    // frena" y "escribiste mal el comando": el segundo lo maneja clap.
    let (_dir, bin) = sandbox_with_binary();
    cmd(&bin)
        .args(["close", "--feature", "1", "--status", "inventado"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("Spec sin aprobar").not());
}

#[test]
fn verify_solo_should_accept_several_acs() {
    // Deuda de impl-23: iterar sobre dos AC obligaba a dos corridas.
    let (dir, bin) = sandbox_with_binary();
    let spec = dir.path().join("docs/spec-feature-1-demo.md");
    cmd(&bin).args(["add", "--name", "demo"]).assert().success();
    cmd(&bin).args(["start", "--feature", "1"]).assert().success();
    escribir_acs(
        &spec,
        "- AC-1: uno.\n  Comando: `touch uno.txt`\n\
         - AC-2: dos.\n  Comando: `touch dos.txt`\n\
         - AC-3: tres.\n  Comando: `touch tres.txt`",
    );
    cmd(&bin).args(["approve-spec", "--yes"]).assert().success();
    cmd(&bin)
        .args(["verify", "--feature", "1", "--solo", "AC-1,AC-3"])
        .assert()
        .success();
    assert!(dir.path().join("uno.txt").exists(), "no corrio AC-1");
    assert!(dir.path().join("tres.txt").exists(), "no corrio AC-3");
    assert!(!dir.path().join("dos.txt").exists(), "corrio AC-2, que no se pidio");
}

#[test]
fn verify_solo_should_name_the_missing_ac() {
    // Con varios pedidos, "no existe" a secas obliga a probar de a uno.
    let (dir, bin) = sandbox_with_binary();
    let spec = dir.path().join("docs/spec-feature-1-demo.md");
    cmd(&bin).args(["add", "--name", "demo"]).assert().success();
    cmd(&bin).args(["start", "--feature", "1"]).assert().success();
    escribir_acs(&spec, "- AC-1: uno.\n  Comando: `true`");
    cmd(&bin).args(["approve-spec", "--yes"]).assert().success();
    cmd(&bin)
        .args(["verify", "--feature", "1", "--solo", "AC-1,AC-9"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("AC-9"))
        .stderr(predicate::str::contains("AC-1").not());
}

#[test]
fn leccion_list_should_size_the_column_to_the_longest_name() {
    // Hito #27: el ancho fijo de 28 desbordaba con nombres de CLASE largos.
    let (dir, bin) = sandbox_with_binary();
    let lecciones = dir.path().join("docs/lecciones");
    std::fs::create_dir_all(&lecciones).unwrap();
    let largo = "una-clase-con-un-nombre-deliberadamente-larguisimo";
    for nombre in [largo, "corta"] {
        std::fs::write(
            lecciones.join(format!("{nombre}.md")),
            format!(
                "---\nnombre: {nombre}\ndescripcion: d\ntriggers: [x]\nusos: 0\nestado: activa\n---\n\ncuerpo\n"
            ),
        )
        .unwrap();
    }
    let salida = cmd(&bin).args(["leccion", "list"]).output().unwrap();
    let texto = String::from_utf8_lossy(&salida.stdout);
    // La columna de "usos" empieza en la misma posicion en las dos filas: eso
    // es lo que significa que la tabla no desborda.
    let columnas: Vec<usize> = texto
        .lines()
        .filter(|l| l.contains(" usos |"))
        .filter_map(|l| l.find(" usos |"))
        .collect();
    assert_eq!(columnas.len(), 2, "esperaba dos filas: {texto}");
    assert_eq!(columnas[0], columnas[1], "las columnas no alinean: {texto}");
    assert!(
        columnas[0] > largo.chars().count(),
        "el nombre largo desborda la columna: {texto}"
    );
}

#[test]
fn leccion_list_should_not_change_order_fields_or_json() {
    // Es formato de salida y nada mas: orden, campos, --json y exit codes
    // quedan como estaban.
    let (dir, bin) = sandbox_with_binary();
    let lecciones = dir.path().join("docs/lecciones");
    std::fs::create_dir_all(&lecciones).unwrap();
    for (nombre, usos) in [("mas-usada", 5), ("menos-usada", 1)] {
        std::fs::write(
            lecciones.join(format!("{nombre}.md")),
            format!(
                "---\nnombre: {nombre}\ndescripcion: d\ntriggers: [x]\nusos: {usos}\nestado: activa\n---\n\ncuerpo\n"
            ),
        )
        .unwrap();
    }
    let salida = cmd(&bin).args(["leccion", "list"]).output().unwrap();
    let texto = String::from_utf8_lossy(&salida.stdout);
    let pos_mas = texto.find("mas-usada").unwrap();
    let pos_menos = texto.find("menos-usada").unwrap();
    assert!(pos_mas < pos_menos, "cambio el orden por uso: {texto}");
    let json_out = cmd(&bin).args(["leccion", "list", "--json"]).output().unwrap();
    let v: serde_json::Value = serde_json::from_slice(&json_out.stdout).unwrap();
    assert!(v["lecciones"].is_array(), "cambio la forma del --json: {v}");
    assert_eq!(v["lecciones"][0]["nombre"], "mas-usada");
}

// ---------------------------------------------------------------------------
// Feature #29: que el PRD, el SDD y architecture.md no queden mintiendo.
// ---------------------------------------------------------------------------

/// Sandbox con el arbol de documentos que el alcance espera.
fn sandbox_con_documentos() -> (tempfile::TempDir, PathBuf) {
    let (dir, bin) = sandbox_with_binary();
    let prd = dir.path().join("docs/prd");
    std::fs::create_dir_all(&prd).unwrap();
    std::fs::write(prd.join("PRD-master.md"), "# PRD\n\ncuerpo del prd\n").unwrap();
    std::fs::write(prd.join("SDD-master.md"), "# SDD\n\ncuerpo del sdd\n").unwrap();
    std::fs::write(
        dir.path().join("docs/architecture.md"),
        "# Arquitectura\n\n- `viejo.rs`: lo de siempre\n",
    )
    .unwrap();
    cmd(&bin).args(["add", "--name", "demo"]).assert().success();
    cmd(&bin).args(["start", "--feature", "1"]).assert().success();
    (dir, bin)
}

fn enable_docs_rule(harness_dir: &Path) {
    let path = harness_dir.join("feature_list.json");
    let mut data: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    let obj = data.as_object_mut().unwrap();
    let rules = obj.entry("rules".to_string()).or_insert_with(|| serde_json::json!({}));
    rules
        .as_object_mut()
        .unwrap()
        .insert("require_docs_al_dia".to_string(), serde_json::json!(true));
    std::fs::write(&path, serde_json::to_string_pretty(&data).unwrap()).unwrap();
}

#[test]
fn prd_propose_should_seed_one_block_per_document() {
    // AC-4: un bloque por documento del alcance, todos PENDIENTE, exit 2.
    let (dir, bin) = sandbox_con_documentos();
    cmd(&bin).args(["prd", "propose", "--feature", "1"]).assert().code(2);
    let texto = std::fs::read_to_string(dir.path().join("docs/prd-diff-1.md")).unwrap();
    for esperado in [
        "## Documento: docs/prd/PRD-master.md",
        "## Documento: docs/prd/SDD-master.md",
        "## Documento: docs/architecture.md",
    ] {
        assert!(texto.contains(esperado), "falta {esperado}: {texto}");
    }
    assert_eq!(texto.matches("Veredicto: PENDIENTE").count(), 3, "{texto}");
}

#[test]
fn prd_propose_should_not_clobber_existing_verdicts() {
    // AC-5: correr propose de nuevo no puede borrar lo ya contestado.
    let (dir, bin) = sandbox_con_documentos();
    let propuesta = dir.path().join("docs/prd-diff-1.md");
    cmd(&bin).args(["prd", "propose", "--feature", "1"]).assert().code(2);
    let contestado = std::fs::read_to_string(&propuesta)
        .unwrap()
        .replacen("Veredicto: PENDIENTE", "Veredicto: no-aplica la feature no toca el producto", 1);
    std::fs::write(&propuesta, &contestado).unwrap();
    cmd(&bin).args(["prd", "propose", "--feature", "1"]).assert().code(2);
    let despues = std::fs::read_to_string(&propuesta).unwrap();
    assert!(despues.contains("no-aplica la feature no toca el producto"), "{despues}");
    assert_eq!(despues.matches("## Documento:").count(), 3, "duplico bloques: {despues}");
}

#[test]
fn prd_propose_should_precompute_presence_signals() {
    // AC-6: el BINARIO precomputa Presente/Ausente para que el agente no parta
    // de cero.
    let (dir, bin) = sandbox_con_documentos();
    // El PRD menciona la feature; el SDD no.
    let prd = dir.path().join("docs/prd/PRD-master.md");
    std::fs::write(&prd, "# PRD\n\nla feature demo ya esta contada aca\n").unwrap();
    cmd(&bin).args(["prd", "propose", "--feature", "1"]).assert().code(2);
    let texto = std::fs::read_to_string(dir.path().join("docs/prd-diff-1.md")).unwrap();
    assert!(texto.contains("Presente en: docs/prd/PRD-master.md:3"), "{texto}");
    assert!(texto.contains("no menciona 'demo'"), "{texto}");
}

/// Contesta los tres bloques de una propuesta ya sembrada.
fn contestar(propuesta: &Path, veredictos: [&str; 3]) {
    let texto = std::fs::read_to_string(propuesta).unwrap();
    let mut i = 0;
    let nuevo: String = texto
        .lines()
        .map(|l| {
            if l.trim() == "Veredicto: PENDIENTE" && i < 3 {
                let v = veredictos[i];
                i += 1;
                v.to_string()
            } else {
                l.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    std::fs::write(propuesta, nuevo).unwrap();
}

#[test]
fn prd_apply_without_yes_should_show_and_refuse_to_write() {
    // AC-12: muestra lo que escribiria y NO escribe un byte.
    let (dir, bin) = sandbox_con_documentos();
    let propuesta = dir.path().join("docs/prd-diff-1.md");
    cmd(&bin).args(["prd", "propose", "--feature", "1"]).assert().code(2);
    contestar(
        &propuesta,
        [
            "Veredicto: no-aplica no cambia el producto",
            "Veredicto: no-aplica no cambia el diseno",
            "Veredicto: cambio\nAntes:\n- `viejo.rs`: lo de siempre\nDespues:\n- `viejo.rs`: lo de siempre\n- `nuevo.rs`: novedad",
        ],
    );
    let arch = dir.path().join("docs/architecture.md");
    let antes = std::fs::read_to_string(&arch).unwrap();
    cmd(&bin)
        .args(["prd", "apply", "--feature", "1"])
        .assert()
        .code(2)
        .stdout(predicate::str::contains("[GATE] prd apply exige la confirmacion explicita"))
        .stdout(predicate::str::contains("docs/architecture.md"));
    assert_eq!(std::fs::read_to_string(&arch).unwrap(), antes, "escribio sin --yes");
}

#[test]
fn prd_apply_with_yes_should_write_seal_and_log() {
    // AC-13: escribe, sella y deja bitacora.
    let (dir, bin) = sandbox_con_documentos();
    let propuesta = dir.path().join("docs/prd-diff-1.md");
    cmd(&bin).args(["prd", "propose", "--feature", "1"]).assert().code(2);
    contestar(
        &propuesta,
        [
            "Veredicto: no-aplica x",
            "Veredicto: no-aplica y",
            "Veredicto: cambio\nAntes:\n- `viejo.rs`: lo de siempre\nDespues:\n- `viejo.rs`: lo de siempre\n- `nuevo.rs`: novedad",
        ],
    );
    cmd(&bin).args(["prd", "apply", "--feature", "1", "--yes"]).assert().success();
    let arch = std::fs::read_to_string(dir.path().join("docs/architecture.md")).unwrap();
    assert!(arch.contains("nuevo.rs"), "{arch}");
    let sellada = std::fs::read_to_string(&propuesta).unwrap();
    assert!(sellada.contains("Aplicado:"), "{sellada}");
    assert!(sellada.contains("por USUARIO (confirmacion explicita)"), "{sellada}");
    let history = std::fs::read_to_string(dir.path().join("hp/progress/history.md")).unwrap();
    assert!(history.contains("prd apply feature #1"), "{history}");
}

#[test]
fn prd_apply_should_refuse_a_citation_that_does_not_hold() {
    // AC-9: la mentira mas probable del agente, refutada por maquina.
    let (dir, bin) = sandbox_con_documentos();
    let propuesta = dir.path().join("docs/prd-diff-1.md");
    cmd(&bin).args(["prd", "propose", "--feature", "1"]).assert().code(2);
    contestar(
        &propuesta,
        [
            "Veredicto: ya-esta docs/prd/PRD-master.md:900-999",
            "Veredicto: no-aplica y",
            "Veredicto: no-aplica z",
        ],
    );
    cmd(&bin)
        .args(["prd", "apply", "--feature", "1"])
        .assert()
        .code(2)
        .stdout(predicate::str::contains("no se sostiene"));
}

#[test]
fn prd_apply_should_reject_a_tampered_block_list() {
    // AC-7: el agente no puede colapsar N preguntas en una respuesta.
    let (dir, bin) = sandbox_con_documentos();
    let propuesta = dir.path().join("docs/prd-diff-1.md");
    cmd(&bin).args(["prd", "propose", "--feature", "1"]).assert().code(2);
    let texto = std::fs::read_to_string(&propuesta).unwrap();
    let sin_uno: String = texto
        .split("## Documento: docs/architecture.md")
        .next()
        .unwrap()
        .to_string();
    std::fs::write(&propuesta, sin_uno.replace("Veredicto: PENDIENTE", "Veredicto: no-aplica x")).unwrap();
    cmd(&bin)
        .args(["prd", "apply", "--feature", "1"])
        .assert()
        .code(2)
        .stdout(predicate::str::contains("no coincide con el alcance real"))
        .stdout(predicate::str::contains("docs/architecture.md"));
}

#[test]
fn prd_diff_should_live_outside_the_protected_path() {
    // AC-15: la propuesta del agente NUNCA se escribe dentro de docs/prd/**.
    let (dir, bin) = sandbox_con_documentos();
    cmd(&bin).args(["prd", "propose", "--feature", "1"]).assert().code(2);
    assert!(dir.path().join("docs/prd-diff-1.md").is_file());
    assert!(!dir.path().join("docs/prd/prd-diff-1.md").exists());
    // Y el binario lo confirma: esa ruta no esta protegida.
    cmd(&bin)
        .args(["rutas", "--check", "docs/prd-diff-1.md"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

#[test]
fn prd_apply_should_register_its_own_writes() {
    // AC-16: el binario escribe en docs/prd/** y lo registra, para no
    // dispararse a si mismo la red de seguridad de la #26.
    let (dir, bin) = sandbox_con_documentos();
    let propuesta = dir.path().join("docs/prd-diff-1.md");
    cmd(&bin).args(["prd", "propose", "--feature", "1"]).assert().code(2);
    contestar(
        &propuesta,
        [
            "Veredicto: cambio\nAntes:\ncuerpo del prd\nDespues:\ncuerpo del prd, ahora al dia",
            "Veredicto: no-aplica y",
            "Veredicto: no-aplica z",
        ],
    );
    cmd(&bin).args(["prd", "apply", "--feature", "1", "--yes"]).assert().success();
    let registro =
        std::fs::read_to_string(dir.path().join("hp/progress/.rutas_arnes")).unwrap_or_default();
    assert!(
        registro.contains("docs/prd/PRD-master.md"),
        "no registro su escritura sobre una ruta protegida: {registro}"
    );
}

#[test]
fn close_should_demand_the_docs_proposal_when_the_rule_is_on() {
    // AC-17: sin propuesta no cierra; con la regla apagada, cierra como siempre.
    let (dir, bin) = sandbox_con_documentos();
    let hp = dir.path().join("hp");
    // Regla apagada: cierra.
    cmd(&bin)
        .args(["close", "--feature", "1", "--status", "done"])
        .assert()
        .success();
    // Con la regla y sin propuesta: bloquea nombrando el comando.
    let (dir2, bin2) = sandbox_con_documentos();
    enable_docs_rule(&dir2.path().join("hp"));
    cmd(&bin2)
        .args(["close", "--feature", "1", "--status", "done"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("prd propose --feature 1"));
    let _ = hp;
}

#[test]
fn close_should_demand_the_user_seal_not_just_the_answers() {
    // AC-17 (OBS-1): contestada no alcanza; hace falta el SI del usuario.
    let (dir, bin) = sandbox_con_documentos();
    let propuesta = dir.path().join("docs/prd-diff-1.md");
    cmd(&bin).args(["prd", "propose", "--feature", "1"]).assert().code(2);
    contestar(
        &propuesta,
        ["Veredicto: no-aplica x", "Veredicto: no-aplica y", "Veredicto: no-aplica z"],
    );
    enable_docs_rule(&dir.path().join("hp"));
    cmd(&bin)
        .args(["close", "--feature", "1", "--status", "done"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("todavia no la aprobo"));
    cmd(&bin).args(["prd", "apply", "--feature", "1", "--yes"]).assert().success();
    cmd(&bin)
        .args(["close", "--feature", "1", "--status", "done"])
        .assert()
        .success();
}

#[test]
fn no_spec_command_should_invoke_prd_apply_yes() {
    // AC-19: la trampa que este repo se puso solo. `verify` ejecuta los
    // `Comando:` de los AC con `sh -c`, asi que un AC que invocara
    // `prd apply --yes` aplicaria la propuesta SIN el si del usuario,
    // salteandose el ritual entero. Se corre sobre los specs REALES.
    let docs = Path::new(env!("CARGO_MANIFEST_DIR")).join("../docs");
    let Ok(entradas) = std::fs::read_dir(&docs) else {
        return;
    };
    let mut revisados = 0usize;
    for e in entradas.flatten() {
        let path = e.path();
        if !path
            .file_name()
            .is_some_and(|n| n.to_string_lossy().starts_with("spec-feature-"))
        {
            continue;
        }
        let Ok(texto) = std::fs::read_to_string(&path) else {
            continue;
        };
        revisados += 1;
        for (n, linea) in texto.lines().enumerate() {
            let t = linea.trim();
            if !t.starts_with("Comando:") {
                continue;
            }
            assert!(
                !(t.contains("prd apply") && t.contains("--yes")),
                "{}:{} declara un Comando: que aplica la propuesta sin el si del usuario: {t}",
                path.display(),
                n + 1
            );
        }
    }
    assert!(revisados > 5, "esperaba varios specs reales, revise {revisados}");
}

// ---------------------------------------------------------------------------
// Feature #28: consolidacion de lecciones. La mitad que MUTA se verifica sin
// backend y de forma determinista; la del modelo vive en
// tests/consolidar_check.sh, que habla con un backend real.
// ---------------------------------------------------------------------------

fn sembrar_leccion(dir: &Path, nombre: &str, triggers: &str, cuerpo: &str) {
    let lecciones = dir.join("docs/lecciones");
    std::fs::create_dir_all(&lecciones).unwrap();
    std::fs::write(
        lecciones.join(format!("{nombre}.md")),
        format!(
            "---\nnombre: {nombre}\ndescripcion: Una oracion sobre {nombre}.\n\
             triggers: [{triggers}]\nrelacionadas: []\norigen: [1]\nusos: 0\n\
             ultimo_uso:\nultima_actualizacion: 2026-08-18\nestado: activa\n---\n\n{cuerpo}\n"
        ),
    )
    .unwrap();
}

#[test]
fn consolidar_aplicar_should_take_the_merge_from_argv() {
    // La fusion NO sale de lo que dijo el modelo: sale de argv. Por eso este
    // test corre sin backend y es determinista.
    let (dir, bin) = sandbox_with_binary();
    sembrar_leccion(dir.path(), "paraguas", "a, b", "cuerpo real del paraguas. Ver [[miembro]].");
    sembrar_leccion(dir.path(), "miembro", "a, b", "cuerpo de la miembro.");
    cmd(&bin)
        .args([
            "lecciones", "consolidar", "--aplicar",
            "--en", "paraguas", "--de", "miembro",
            "--motivo", "cuentan lo mismo",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Consolidacion aplicada"));
    assert!(
        dir.path().join("docs/lecciones/archivo/miembro.md").is_file(),
        "no archivo la miembro"
    );
    assert!(
        !dir.path().join("docs/lecciones/miembro.md").exists(),
        "la miembro sigue en el catalogo activo"
    );
    assert!(dir.path().join("docs/lecciones/paraguas.md").is_file(), "borro el paraguas");
}

#[test]
fn consolidar_aplicar_should_demand_a_motivo() {
    // Una fusion sin motivo escrito es la que nadie va a poder revisar despues.
    let (dir, bin) = sandbox_with_binary();
    sembrar_leccion(dir.path(), "paraguas", "a", "cuerpo. [[miembro]]");
    sembrar_leccion(dir.path(), "miembro", "a", "cuerpo.");
    cmd(&bin)
        .args(["lecciones", "consolidar", "--aplicar", "--en", "paraguas", "--de", "miembro"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("Falta --motivo"));
    assert!(!dir.path().join("docs/lecciones/archivo").exists(), "archivo sin motivo");
}

#[test]
fn consolidar_should_refuse_a_skeleton_umbrella() {
    // Archivar contra un esqueleto perderia el conocimiento de forma estructural.
    let (dir, bin) = sandbox_with_binary();
    cmd(&bin).args(["leccion", "nueva", "paraguas"]).assert().success();
    sembrar_leccion(dir.path(), "miembro", "a", "cuerpo.");
    cmd(&bin)
        .args([
            "lecciones", "consolidar", "--aplicar", "--en", "paraguas",
            "--de", "miembro", "--motivo", "x",
        ])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("placeholders"));
    assert!(!dir.path().join("docs/lecciones/archivo").exists());
}

#[test]
fn consolidar_should_demand_the_union_of_triggers() {
    // Sin heredar los triggers, el conocimiento deja de encontrarse: `buscar`
    // puntua una leccion activa 100 y una archivada 30.
    let (dir, bin) = sandbox_with_binary();
    sembrar_leccion(dir.path(), "paraguas", "comun", "cuerpo real. [[miembro]]");
    sembrar_leccion(dir.path(), "miembro", "comun, propio", "cuerpo.");
    cmd(&bin)
        .args([
            "lecciones", "consolidar", "--aplicar", "--en", "paraguas",
            "--de", "miembro", "--motivo", "x",
        ])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("'propio'"))
        .stderr(predicate::str::contains("100"));
}

#[test]
fn consolidar_should_demand_a_pointer_to_each_member() {
    let (dir, bin) = sandbox_with_binary();
    sembrar_leccion(dir.path(), "paraguas", "a", "cuerpo sin citar a nadie.");
    sembrar_leccion(dir.path(), "miembro", "a", "cuerpo.");
    cmd(&bin)
        .args([
            "lecciones", "consolidar", "--aplicar", "--en", "paraguas",
            "--de", "miembro", "--motivo", "x",
        ])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("[[miembro]]"));
}

#[test]
fn consolidar_should_allow_an_existing_member_as_the_umbrella() {
    // Es lo que manda la guia ("patchea el paraguas existente") y es la forma
    // del unico solapamiento real de este repo: el paraguas va tambien en --de.
    let (dir, bin) = sandbox_with_binary();
    sembrar_leccion(dir.path(), "paraguas", "a, b", "cuerpo real. [[miembro]]");
    sembrar_leccion(dir.path(), "miembro", "a, b", "cuerpo.");
    cmd(&bin)
        .args([
            "lecciones", "consolidar", "--aplicar", "--en", "paraguas",
            "--de", "paraguas,miembro", "--motivo", "el paraguas es una de las dos",
        ])
        .assert()
        .success();
    assert!(dir.path().join("docs/lecciones/paraguas.md").is_file(), "archivo el paraguas");
    assert!(dir.path().join("docs/lecciones/archivo/miembro.md").is_file());
}

#[test]
fn consolidar_should_archive_byte_for_byte_with_backup() {
    // Nunca borra, y se puede comprobar byte a byte.
    let (dir, bin) = sandbox_with_binary();
    sembrar_leccion(dir.path(), "paraguas", "a", "cuerpo real. [[miembro]]");
    sembrar_leccion(dir.path(), "miembro", "a", "cuerpo con un PITFALL que no se puede perder.");
    let antes = std::fs::read_to_string(dir.path().join("docs/lecciones/miembro.md")).unwrap();
    let cuerpo_antes = antes.split("---\n").nth(2).unwrap().to_string();
    cmd(&bin)
        .args([
            "lecciones", "consolidar", "--aplicar", "--en", "paraguas",
            "--de", "miembro", "--motivo", "x",
        ])
        .assert()
        .success();
    let archivada =
        std::fs::read_to_string(dir.path().join("docs/lecciones/archivo/miembro.md")).unwrap();
    let cuerpo_despues = archivada.split("---\n").nth(2).unwrap().to_string();
    assert_eq!(cuerpo_antes, cuerpo_despues, "el cuerpo archivado cambio");
    assert!(archivada.contains("PITFALL que no se puede perder"));
}

#[test]
fn consolidar_report_should_list_each_merge_with_its_reason() {
    let (dir, bin) = sandbox_with_binary();
    sembrar_leccion(dir.path(), "paraguas", "a", "cuerpo real. [[miembro]]");
    sembrar_leccion(dir.path(), "miembro", "a", "cuerpo.");
    cmd(&bin)
        .args([
            "lecciones", "consolidar", "--aplicar", "--en", "paraguas",
            "--de", "miembro", "--motivo", "este es el motivo exacto",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("este es el motivo exacto"))
        .stdout(predicate::str::contains("Backup:"));
    let history = std::fs::read_to_string(dir.path().join("hp/progress/history.md")).unwrap();
    assert!(history.contains("lecciones consolidar"), "{history}");
}

#[test]
fn consolidar_should_be_undoable_with_rollback() {
    let (dir, bin) = sandbox_with_binary();
    sembrar_leccion(dir.path(), "paraguas", "a", "cuerpo real. [[miembro]]");
    sembrar_leccion(dir.path(), "miembro", "a", "cuerpo.");
    cmd(&bin)
        .args([
            "lecciones", "consolidar", "--aplicar", "--en", "paraguas",
            "--de", "miembro", "--motivo", "x",
        ])
        .assert()
        .success();
    assert!(!dir.path().join("docs/lecciones/miembro.md").exists());
    cmd(&bin).args(["lecciones", "rollback"]).assert().success();
    assert!(
        dir.path().join("docs/lecciones/miembro.md").is_file(),
        "rollback no devolvio la leccion al catalogo activo"
    );
}

// ---------------------------------------------------------------------------
// Feature #37: el estado `superseded`, para lo que se hizo en OTRA feature.
//
// Varios de estos son tests de REGRESION: fijan que agregar un valor al campo
// `status` no rompe a los cuatro consumidores que ya lo miran. Sin ellos, "no
// rompe nada" seria una afirmacion; con ellos, es un contrato.
// ---------------------------------------------------------------------------

fn dos_features(bin: &Path) {
    cmd(bin).args(["add", "--name", "absorbida"]).assert().success();
    cmd(bin).args(["add", "--name", "absorbente"]).assert().success();
}

#[test]
fn close_should_accept_the_superseded_status() {
    let (dir, bin) = sandbox_with_binary();
    dos_features(&bin);
    cmd(&bin)
        .args(["close", "--feature", "1", "--status", "superseded", "--absorbida-por", "2"])
        .assert()
        .success();
    let texto = std::fs::read_to_string(dir.path().join("hp/feature_list.json")).unwrap();
    let v: serde_json::Value = serde_json::from_str(&texto).unwrap();
    assert_eq!(v["features"][0]["status"], "superseded");
}

#[test]
fn superseded_should_demand_the_absorbing_feature() {
    // La trazabilidad es el punto entero del estado: sin ella es `blocked`.
    let (_dir, bin) = sandbox_with_binary();
    dos_features(&bin);
    cmd(&bin)
        .args(["close", "--feature", "1", "--status", "superseded"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("--absorbida-por"));
}

#[test]
fn superseded_should_refuse_an_unknown_absorber() {
    // Una referencia rota es peor que ninguna.
    let (_dir, bin) = sandbox_with_binary();
    dos_features(&bin);
    cmd(&bin)
        .args(["close", "--feature", "1", "--status", "superseded", "--absorbida-por", "99"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("no existe"));
    // Y tampoco puede absorberse a si misma.
    cmd(&bin)
        .args(["close", "--feature", "1", "--status", "superseded", "--absorbida-por", "1"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("a si misma"));
}

#[test]
fn superseded_should_record_the_absorbing_feature() {
    let (dir, bin) = sandbox_with_binary();
    dos_features(&bin);
    cmd(&bin)
        .args(["close", "--feature", "1", "--status", "superseded", "--absorbida-por", "2"])
        .assert()
        .success();
    let v: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(dir.path().join("hp/feature_list.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(
        v["features"][0]["superseded_by"], "2",
        "la trazabilidad tiene que ser un campo, no prosa en `note`"
    );
}

#[test]
fn superseded_should_not_trigger_the_done_gates() {
    // REGRESION: con las cuatro reglas de `done` encendidas, cerrar como
    // superseded pasa igual. El trabajo y su evidencia viven en la que
    // absorbio; exigirle spec propio seria justo el problema que el estado
    // vino a resolver.
    let (dir, bin) = sandbox_with_binary();
    let hp = dir.path().join("hp");
    dos_features(&bin);
    enable_spec_rule(&hp);
    enable_leccion_rule(&hp);
    cmd(&bin)
        .args(["close", "--feature", "1", "--status", "superseded", "--absorbida-por", "2"])
        .assert()
        .success();
}

#[test]
fn next_should_not_offer_a_superseded_feature() {
    // REGRESION: `next` solo ofrece `pending`, asi que esto ya era cierto antes
    // de la #37. El test lo fija.
    //
    // Y DISCRIMINA: no alcanza con que la superseded no aparezca (eso pasaria
    // igual con `--status kfjhds`, y seria el detector-de-cambios que la regla
    // 3 de docs/conventions.md prohibe). Se comprueba ADEMAS que la pending SI
    // se ofrezca, en el mismo catalogo: asi el test distingue "next filtra
    // bien" de "next no ofrece nada".
    let (_dir, bin) = sandbox_with_binary();
    dos_features(&bin);
    cmd(&bin)
        .args(["close", "--feature", "1", "--status", "superseded", "--absorbida-por", "2"])
        .assert()
        .success();
    cmd(&bin)
        .arg("next")
        .assert()
        .success()
        .stdout(predicate::str::contains("absorbente"))
        .stdout(predicate::str::contains("absorbida\"").not());
}

#[test]
fn status_should_show_who_absorbed_a_superseded_feature() {
    let (_dir, bin) = sandbox_with_binary();
    dos_features(&bin);
    cmd(&bin)
        .args(["close", "--feature", "1", "--status", "superseded", "--absorbida-por", "2"])
        .assert()
        .success();
    cmd(&bin)
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains("[superseded por #2]"));
}

#[test]
fn blocked_features_should_stay_blocked() {
    // REGRESION: la migracion es explicita, no automatica. Una feature trabada
    // de verdad no se toca.
    let (_dir, bin) = sandbox_with_binary();
    dos_features(&bin);
    cmd(&bin)
        .args(["close", "--feature", "1", "--status", "blocked", "--note", "esperando a alguien"])
        .assert()
        .success();
    cmd(&bin)
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains("[blocked]"))
        .stdout(predicate::str::contains("superseded").not());
}

// ---------------------------------------------------------------------------
// Feature #44: un AC que salio 0 sin ejecutar ningun caso no es evidencia.
// ---------------------------------------------------------------------------

/// La salida real de libtest cuando el filtro no matchea nada, tal como la
/// imprimio `cargo test consolidar_without_aplicar_should_not_touch_anything`.
/// Sale 0, dice `ok`, y no corrio nada: ese es el falso verde.
/// En UNA linea con `\n` escapados, porque un `Comando:` del spec es una sola
/// linea: lo multilinea lo arma `printf` al ejecutarse, no el spec.
const COMANDO_FILTRO_VACIO: &str = "printf 'running 0 tests\\ntest result: ok. 0 passed; \
     0 failed; 0 ignored; 0 measured; 322 filtered out; finished in 0.00s\\n'";

#[test]
fn verify_should_mark_an_ac_that_ran_nothing_as_vacio() {
    let (dir, bin, _spec) = feature_con_spec(&format!(
        "- AC-1: el que no midio nada.\n  Comando: `{COMANDO_FILTRO_VACIO}`\n\
         - AC-2: el que si.\n  Comando: `true`"
    ));
    cmd(&bin).args(["approve-spec", "--yes"]).assert().success();
    // Exit 1: `vacio` bloquea, igual que un rojo.
    cmd(&bin).args(["verify", "--feature", "1"]).assert().code(1);
    let reporte = std::fs::read_to_string(dir.path().join("docs/verify-1.md")).unwrap();
    assert!(reporte.contains("| AC-1 | vacio |"), "{reporte}");
    assert!(reporte.contains("| AC-2 | verde |"), "{reporte}");
    // El resumen lo muestra aparte en vez de esconderlo entre los rojos.
    assert!(reporte.contains("1 sin casos"), "{reporte}");
}

#[test]
fn close_should_block_on_an_empty_verification() {
    // AC-10: el cierre lee el reporte y trata `vacio` como bloqueante, sin
    // ejecutar nada. Nombra el AC para que se sepa cual arreglar.
    let (dir, bin, _spec) = feature_con_spec(&format!(
        "- AC-1: el que no midio nada.\n  Comando: `{COMANDO_FILTRO_VACIO}`"
    ));
    cmd(&bin).args(["approve-spec", "--yes"]).assert().success();
    cmd(&bin).args(["verify", "--feature", "1"]).assert().code(1);
    enable_verify_rule(&dir.path().join("hp"));
    cmd(&bin)
        .args(["close", "--feature", "1", "--status", "done"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("AC-1"));
}

// ---------------------------------------------------------------------------
// Feature #44, la deuda que destapa: el AC-12 de la #28 declaraba ESTE nombre
// y la funcion no existia. `cargo test` con un filtro que no matchea sale 0, y
// asi el invariante mas citado de la #28 quedo "verde" sin nada detras.
// ---------------------------------------------------------------------------

/// Un backend de mentira que SIEMPRE devuelve un candidato valido. Es la parte
/// que importa: con un backend que no propone nada, el test seria tautologico
/// (no se escribe nada porque no hay nada que escribir). Aca hay algo para
/// fusionar y aun asi no se toca el arbol.
fn backend_falso(dir: &Path, candidato: &str) -> PathBuf {
    let script = dir.join("backend-falso.sh");
    std::fs::write(
        &script,
        format!("#!/bin/sh\nprintf '%s' '{candidato}'\n"),
    )
    .unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    script
}

fn escribir_leccion(dir: &Path, nombre: &str, trigger: &str) {
    let lecciones = dir.join("docs/lecciones");
    std::fs::create_dir_all(&lecciones).unwrap();
    std::fs::write(
        lecciones.join(format!("{nombre}.md")),
        format!(
            "---\nnombre: {nombre}\ndescripcion: Sobre {nombre}.\ntriggers: [{trigger}]\n\
             relacionadas: []\norigen: [1]\nusos: 0\nultimo_uso:\n\
             ultima_actualizacion: 2026-08-19\nestado: activa\n---\n\n\
             ## Cuando aplica\n\nCuando {trigger}.\n"
        ),
    )
    .unwrap();
}

/// Huella del arbol: rutas y contenido. Compara lo que un backup o una
/// reescritura cambiarian.
fn huella(dir: &Path) -> Vec<(String, String)> {
    fn recorrer(base: &Path, dir: &Path, out: &mut Vec<(String, String)>) {
        let Ok(entradas) = std::fs::read_dir(dir) else { return };
        for e in entradas.flatten() {
            let ruta = e.path();
            if ruta.is_dir() {
                recorrer(base, &ruta, out);
            } else if let Ok(texto) = std::fs::read_to_string(&ruta) {
                let rel = ruta.strip_prefix(base).unwrap_or(&ruta);
                out.push((rel.display().to_string(), texto));
            }
        }
    }
    let mut out = Vec::new();
    recorrer(dir, dir, &mut out);
    out.sort();
    out
}

#[test]
fn consolidar_without_aplicar_should_not_touch_anything() {
    // AC-12 de la feature #28, que hasta la #44 no tuvo test: sin `--aplicar`,
    // cero escrituras, cero backups, el arbol byte a byte igual.
    let (dir, bin) = sandbox_with_binary();
    let raiz = dir.path();
    // `una-cosa` nace ya en forma de paraguas (union de triggers y puntero a la
    // otra) para que el CONTROL de mas abajo pueda aplicar de verdad: lo que se
    // mide aca es si se escribe o no, no el chequeo de paraguas de la #28.
    escribir_leccion(raiz, "una-cosa", "alfa, beta");
    escribir_leccion(raiz, "otra-cosa", "beta");
    let paraguas = raiz.join("docs/lecciones/una-cosa.md");
    let texto = std::fs::read_to_string(&paraguas).unwrap();
    std::fs::write(&paraguas, format!("{texto}\nAbsorbe a [[otra-cosa]].\n")).unwrap();
    // La regla tiene que estar encendida: si no, `consolidar` se salta todo y
    // el test volveria a ser tautologico.
    let lista = raiz.join("hp/feature_list.json");
    std::fs::write(
        &lista,
        r#"{"features": [], "rules": {"consolidar_backend": "auto"}}"#,
    )
    .unwrap();
    let falso = backend_falso(
        raiz,
        r#"{"candidatos":[{"miembros":["una-cosa","otra-cosa"],"motivo":"ensenan lo mismo","confianza":0.9}]}"#,
    );

    // La huella cubre docs/ Y el directorio del arnes: los backups del curador
    // van a `<raiz>/hp/bkp` (usan `paths.root`, no la raiz del repo), asi que
    // mirar solo `<raiz>/bkp` era mirar una ruta que nunca existe.
    let instantanea = |raiz: &Path| {
        let mut v = huella(&raiz.join("docs"));
        v.extend(huella(&raiz.join("hp/bkp")));
        v
    };
    let antes = instantanea(raiz);
    cmd(&bin)
        .args(["lecciones", "consolidar"])
        .env("HARNESS_CONSOLIDAR_CMD", &falso)
        .assert()
        .success()
        // Que haya propuesto algo y que ese algo haya sido ACEPTADO. `una-cosa`
        // a secas no alcanzaba: el mensaje de descarte tambien nombra la
        // leccion, asi que el test pasaba igual si `validar` rechazaba todo.
        // "1 candidato(s) a consolidar" solo se imprime con candidatos vivos.
        .stdout(predicate::str::contains("1 candidato(s) a consolidar"))
        .stdout(predicate::str::contains("una-cosa + otra-cosa"));

    assert_eq!(antes, instantanea(raiz), "la deteccion toco el arbol");
    assert!(
        !raiz.join("hp/bkp").exists(),
        "la deteccion creo un backup (van a <raiz>/hp/bkp, no a <raiz>/bkp)"
    );

    // CONTROL: el mismo caso CON --aplicar si tiene que mover el arbol. Sin
    // esto el test no discrimina — pasaria igual si `consolidar` estuviera
    // roto y no hiciera nada nunca, que es exactamente como el AC-12 de la #28
    // llego a estar verde sin existir.
    cmd(&bin)
        .args([
            "lecciones",
            "consolidar",
            "--aplicar",
            "--en",
            "una-cosa",
            "--de",
            "una-cosa,otra-cosa",
            "--motivo",
            "control del test: aca SI tiene que escribir",
        ])
        .env("HARNESS_CONSOLIDAR_CMD", &falso)
        .assert()
        .success();
    assert_ne!(
        antes,
        instantanea(raiz),
        "con --aplicar el arbol tenia que cambiar: el test no esta midiendo nada"
    );
}

// ---------------------------------------------------------------------------
// Feature #47: aislamiento real (rama + worktree) y cierre GitFlow.
// ---------------------------------------------------------------------------

/// Sandbox donde el PROYECTO es un repo git de verdad: es la unica forma de
/// probar ramas, worktrees y merges.
fn sandbox_git() -> (tempfile::TempDir, PathBuf) {
    let (dir, bin) = sandbox_with_binary();
    let raiz = dir.path();
    for args in [
        vec!["init", "-q", "-b", "main"],
        vec!["config", "user.email", "test@example.com"],
        vec!["config", "user.name", "Test"],
    ] {
        Command::new("git").args(&args).current_dir(raiz).output().unwrap();
    }
    std::fs::write(raiz.join("README.md"), "# proyecto\n").unwrap();
    // El dir del arnes no entra al repo del proyecto (como en la vida real).
    std::fs::write(raiz.join(".gitignore"), "hp/\n").unwrap();
    Command::new("git").args(["add", "-A"]).current_dir(raiz).output().unwrap();
    Command::new("git")
        .args(["commit", "-q", "-m", "init"])
        .current_dir(raiz)
        .output()
        .unwrap();
    (dir, bin)
}

fn git_en(dir: &Path, args: &[&str]) -> String {
    let out = Command::new("git").args(args).current_dir(dir).output().unwrap();
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

#[test]
fn start_should_create_branch_and_worktree_per_feature() {
    // AC-2, AC-3, AC-4: rama GitFlow + worktree hermano, reusables.
    let (dir, bin) = sandbox_git();
    cmd(&bin).args(["add", "--name", "Cobranza"]).assert().success();
    cmd(&bin)
        .args(["start", "--feature", "1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("feature/1-cobranza"))
        .stdout(predicate::str::contains("Trabaja ahi: cd"));

    let ramas = git_en(dir.path(), &["for-each-ref", "--format=%(refname:short)", "refs/heads/"]);
    assert!(ramas.contains("feature/1-cobranza"), "{ramas}");
    // El checkout principal NO cambio de rama.
    assert_eq!(git_en(dir.path(), &["rev-parse", "--abbrev-ref", "HEAD"]), "main");
    // El worktree es hermano del repo y tiene el arbol.
    let backlog: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(dir.path().join("hp/feature_list.json")).unwrap(),
    )
    .unwrap();
    let wt = backlog["features"][0]["worktree"].as_str().unwrap();
    assert!(Path::new(wt).join("README.md").is_file(), "worktree poblado: {wt}");
    assert!(wt.contains("-wt/1-cobranza"), "hermano del repo: {wt}");
}

#[test]
fn start_should_keep_working_without_git_or_with_sin_worktree() {
    // AC-5 y AC-6: sin repo git, o pidiendo el modo clasico, no hay aislamiento
    // y el flujo sigue igual.
    let (_dir, bin) = sandbox_with_binary(); // sin git
    cmd(&bin).args(["add", "--name", "Sin Git"]).assert().success();
    cmd(&bin)
        .args(["start", "--feature", "1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("sin aislamiento"));

    let (_dir2, bin2) = sandbox_git();
    cmd(&bin2).args(["add", "--name", "Clasica"]).assert().success();
    cmd(&bin2)
        .args(["start", "--feature", "1", "--sin-worktree"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--sin-worktree"));
}

#[test]
fn close_done_should_refuse_without_to_and_then_integrate() {
    // AC-14, AC-15, AC-16, AC-19: sin --to se niega; con --to mergea, el commit
    // no lleva trailers de IA y el worktree se borra conservando la rama.
    let (dir, bin) = sandbox_git();
    cmd(&bin).args(["add", "--name", "Cobranza"]).assert().success();
    cmd(&bin).args(["start", "--feature", "1"]).assert().success();

    let backlog: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(dir.path().join("hp/feature_list.json")).unwrap(),
    )
    .unwrap();
    let wt = PathBuf::from(backlog["features"][0]["worktree"].as_str().unwrap());
    // Trabajo real dentro del worktree.
    std::fs::write(wt.join("cobranza.txt"), "hecho\n").unwrap();
    Command::new("git").args(["add", "-A"]).current_dir(&wt).output().unwrap();
    Command::new("git")
        .args(["commit", "-q", "-m", "feat: cobranza"])
        .current_dir(&wt)
        .output()
        .unwrap();

    // Sin --to: exit 2 y le pide al agente que pregunte al USUARIO.
    cmd(&bin)
        .args(["close", "--feature", "1", "--status", "done", "--note", "listo"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("PREGUNTALE AL USUARIO"))
        .stderr(predicate::str::contains("Ramas disponibles"));

    // Con --to: integra.
    cmd(&bin)
        .args([
            "close", "--feature", "1", "--status", "done", "--note", "listo", "--to", "main",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("merge hecho"));

    assert!(dir.path().join("cobranza.txt").is_file(), "el trabajo llego a main");
    let log = git_en(dir.path(), &["log", "-1", "--format=%B"]);
    assert!(!log.to_lowercase().contains("co-authored-by"), "sin trailers: {log}");
    // AC-19: worktree borrado, rama conservada.
    assert!(!wt.exists(), "el worktree se borro");
    let ramas = git_en(dir.path(), &["for-each-ref", "--format=%(refname:short)", "refs/heads/"]);
    assert!(ramas.contains("feature/1-cobranza"), "la rama se conserva: {ramas}");
}

#[test]
fn close_should_refuse_an_unknown_target_branch() {
    // AC-20: falla antes de tocar nada y lista las validas.
    let (dir, bin) = sandbox_git();
    cmd(&bin).args(["add", "--name", "Cobranza"]).assert().success();
    cmd(&bin).args(["start", "--feature", "1"]).assert().success();
    cmd(&bin)
        .args([
            "close", "--feature", "1", "--status", "done", "--note", "x", "--to", "no-existe",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("no existe"));
    assert_eq!(git_en(dir.path(), &["rev-parse", "--abbrev-ref", "HEAD"]), "main");
}

#[test]
fn close_blocked_should_keep_branch_and_worktree() {
    // AC-21: solo `done` integra; lo demas conserva todo para retomar.
    let (dir, bin) = sandbox_git();
    cmd(&bin).args(["add", "--name", "Cobranza"]).assert().success();
    cmd(&bin).args(["start", "--feature", "1"]).assert().success();
    let backlog: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(dir.path().join("hp/feature_list.json")).unwrap(),
    )
    .unwrap();
    let wt = PathBuf::from(backlog["features"][0]["worktree"].as_str().unwrap());

    cmd(&bin)
        .args(["close", "--feature", "1", "--status", "blocked", "--note", "trabada"])
        .assert()
        .success()
        .stdout(predicate::str::contains("conservada"));
    assert!(wt.is_dir(), "el worktree se conserva para retomar");
}
