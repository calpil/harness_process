//! `harness journey` (feature #22): la linea de tiempo de lo aprendido.
//!
//! **Solo lectura.** Este modulo no importa nada que escriba: para corregir un
//! hueco imprime el comando del almacen que corresponde, en vez de tener puerta
//! propia (decision del usuario 2026-08-17, OBS-2 y OBS-3).

use serde_json::json;

use crate::journey::{self, Mapa, Tipo};
use crate::paths::HarnessPaths;

pub fn run(paths: &HarnessPaths, as_json: bool) -> anyhow::Result<()> {
    let mapa = journey::construir(paths);
    if as_json {
        return emitir_json(&mapa);
    }
    emitir_humano(&mapa)
}

fn emitir_json(mapa: &Mapa) -> anyhow::Result<()> {
    let nodos: Vec<_> = mapa
        .cronologico()
        .iter()
        .map(|n| {
            json!({
                "tipo": n.tipo.etiqueta(),
                "id": n.id,
                "fecha": n.fecha,
                "titulo": n.titulo,
                "detalle": n.detalle,
            })
        })
        .collect();
    let enlaces: Vec<_> = mapa
        .enlaces
        .iter()
        .map(|e| json!({"desde": e.desde, "hacia": e.hacia, "clase": e.clase.etiqueta()}))
        .collect();
    let huecos: Vec<_> = mapa
        .huecos
        .iter()
        .map(|h| {
            json!({
                "motivo": h.motivo.etiqueta(),
                "sujeto": h.sujeto,
                "detalle": h.detalle,
                "remedio": h.motivo.remedio(&h.sujeto),
            })
        })
        .collect();
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "nodos": nodos, "enlaces": enlaces, "huecos": huecos
        }))?
    );
    Ok(())
}

fn emitir_humano(mapa: &Mapa) -> anyhow::Result<()> {
    if mapa.vacio() {
        println!("Todavia no hay nada que mapear.");
        println!("  El mapa se llena solo: cerra una feature declarando su leccion");
        println!("  ('close --leccion <clase>') y agrega entradas al perfil.");
        return Ok(());
    }
    println!(
        "Mapa de aprendizaje: {} nodo(s), {} enlace(s).\n",
        mapa.nodos.len(),
        mapa.enlaces.len()
    );
    // Hijos de cada feature, para colgarlos debajo.
    let mut fecha_actual = String::new();
    for nodo in mapa.cronologico() {
        // Las lecciones y entradas ya salen colgadas de su feature; se listan
        // aparte solo si no cuelgan de ninguna.
        let cuelga = mapa.nodos.iter().any(|padre| {
            padre.tipo == Tipo::Feature && mapa.hijos(padre).iter().any(|(h, _)| h.id == nodo.id)
        });
        if cuelga {
            continue;
        }
        let fecha = if nodo.fecha.is_empty() {
            "(sin fecha)"
        } else {
            &nodo.fecha
        };
        if fecha != fecha_actual {
            println!("{fecha}");
            fecha_actual = fecha.to_string();
        }
        imprimir_nodo(nodo, "  ");
        // Hijos: lo que esta feature declaro, pario o confirmo.
        for (hijo, clase) in mapa.hijos(nodo) {
            let marca = match clase {
                journey::Clase::Declarada => "leccion declarada",
                journey::Clase::Origen => "leccion (origen)",
                journey::Clase::Cita => "perfil",
                journey::Clase::Relacionada => "relacionada",
            };
            println!("      `-- [{marca}] {}", hijo.id);
            if !hijo.titulo.is_empty() {
                println!("          {} — {}", hijo.titulo, hijo.detalle);
            }
        }
    }
    // Los huecos son lo que hace util al mapa.
    println!();
    if mapa.huecos.is_empty() {
        println!("[Ok] Sin huecos: los tres almacenes son coherentes entre si.");
        return Ok(());
    }
    println!("[!] Huecos ({}):", mapa.huecos.len());
    for h in &mapa.huecos {
        println!("  - [{}] {}", h.motivo.etiqueta(), h.detalle);
        println!("      {}", h.motivo.remedio(&h.sujeto));
    }
    println!("\n  journey no escribe nada: cada correccion pasa por el comando de su almacen.");
    Ok(())
}

fn imprimir_nodo(nodo: &journey::Nodo, sangria: &str) {
    let etiqueta = match nodo.tipo {
        Tipo::Feature => format!("{} {}", nodo.id, nodo.titulo),
        Tipo::Perfil => format!("perfil: {}", nodo.titulo),
        Tipo::Leccion => format!("leccion {}", nodo.id),
        Tipo::LeccionArchivada => format!("leccion {} [archivada]", nodo.id),
    };
    println!("{sangria}{etiqueta}");
    if !nodo.detalle.is_empty() && nodo.tipo != Tipo::Feature {
        println!("{sangria}    {}", nodo.detalle);
    }
}
