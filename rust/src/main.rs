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

fn main() -> ExitCode {
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
