//! Memorias de la task (paridad: harness.py _hub_register/update_memories).
//! En Python esto era un subprocess a graph_memory.py con check=False: el
//! hijo imprime su salida normal a stdout y sus errores (SystemExit) a
//! stderr, y harness.py SIGUE adelante pase lo que pase. Aqui es in-process
//! con la misma superficie observable.

use std::path::Path;

use crate::exit::Exit;
use crate::graph::GraphMemoryManager;
use crate::graphify;

/// `_hub_register`: registra el evento en el hub, best-effort.
/// El agente queda como "harness.py" para no bifurcar los datos historicos.
pub fn hub_register(accion: &str, estado: &str, artefacto: &str, meta: &str) {
    let result = GraphMemoryManager::new().and_then(|mut manager| {
        let metadata = if meta.is_empty() { None } else { Some(meta) };
        manager.record_event("harness.py", accion, artefacto, estado, metadata)
    });
    if let Err(err) = result {
        // El hijo Python imprimia su SystemExit a stderr; replicamos.
        match err.downcast_ref::<Exit>() {
            Some(exit) => {
                if let Some(msg) = &exit.message {
                    eprintln!("{msg}");
                }
            }
            None => eprintln!("{err:#}"),
        }
    }
}

/// `update_memories`: hub + (en cierres/avances manuales) graphify sincrono.
pub fn update_memories(
    accion: &str,
    estado: &str,
    artefacto: &str,
    meta: &str,
    refresh_graphify: bool,
    repo_root: &Path,
) {
    hub_register(accion, estado, artefacto, meta);
    if refresh_graphify {
        graphify::refresh_sync(repo_root);
    }
}
