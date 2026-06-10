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
fn start_should_create_plan_sign_it_and_check_plan_should_pass() {
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
        .stderr(predicate::str::contains("El Memory Hub PostgreSQL requiere: DB_HOST, DB_USER, DB_PASSWORD"));
    assert!(dir.path().join("docs/plan-feature-1-pago-qr.md").exists());
    cmd(&bin)
        .arg("check-plan")
        .assert()
        .success()
        .stdout("Plan fresco (sin cambios desde la ultima firma registrada).\n[OK] Plan fresco para implementacion.\n");
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
