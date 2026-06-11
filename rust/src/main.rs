//! Binario `harness`: port en Rust de harness.py + graph_memory.py.
//! El shim harness_cli lo prefiere sobre el fallback Python; ambas
//! implementaciones deben mantenerse en paridad (ver UPDATING.md).

mod cli;
mod commands;
mod exit;
mod features;
mod graph;
mod graphify;
mod memories;
mod paths;
mod plan;
mod progress;
mod pycompat;

use std::process::ExitCode;

/// Restaura el comportamiento Unix clasico ante SIGPIPE (morir en silencio,
/// como Python o grep). Rust lo ignora por defecto y `println!` entraria en
/// panico con "Broken pipe" cuando el lector del pipe cierra temprano
/// (p.ej. `harness status | head`).
#[cfg(unix)]
fn restore_sigpipe() {
    // SAFETY: cambiar la disposicion de SIGPIPE antes de cualquier I/O es
    // seguro; no hay otros hilos todavia.
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
    }
}

#[cfg(not(unix))]
fn restore_sigpipe() {}

fn main() -> ExitCode {
    restore_sigpipe();
    match cli::run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            if let Some(exit) = err.downcast_ref::<exit::Exit>() {
                // Semantica SystemExit: mensaje a stderr sin prefijo + code.
                if let Some(msg) = &exit.message {
                    eprintln!("{msg}");
                }
                ExitCode::from(exit.code.clamp(0, 255) as u8)
            } else {
                eprintln!("{err:#}");
                ExitCode::FAILURE
            }
        }
    }
}
