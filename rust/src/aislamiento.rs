//! Feature #72: si una feature esta AISLADA o no, decidido aparte de ejecutarlo.
//!
//! El diagnostico del 2026-09-04 encontro tres features (`#98`, `#122`, `#126`)
//! marcadas `in_progress` sin rama ni worktree, escribiendo las tres en el mismo
//! checkout. Ninguna fallo: `start` las dio por arrancadas igual. Dos caminos lo
//! permitian, y los dos vivian en `commands/start.rs`:
//!
//! - `--sin-worktree` devolvia `None` sin mirar si habia otra feature abierta.
//! - un fallo de `git worktree add` se imprimia con `[i]` y el arranque seguia.
//!
//! El estado se escribia ANTES de todo eso, asi que un arranque que no consiguio
//! aislamiento dejaba la feature activa lo mismo. Por eso la decision vive aca,
//! es PURA y no sabe ejecutar nada: la unica forma de que `start` no vuelva a
//! "avisar y seguir" es que lo que decide no tenga con que continuar.
//!
//! Lo que este modulo NO promete: que nadie escriba fuera de su worktree. Un
//! `cd` y un editor se lo saltean. Promete que el arnes no va a *declarar*
//! aislada una feature que no lo esta, que es lo unico comprobable desde aca.

use std::path::{Path, PathBuf};

/// Una feature `in_progress` distinta de la que arranca, y donde escribe.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ocupacion {
    pub id: String,
    pub nombre: String,
    /// Worktree declarado en el backlog, o `None` si trabaja en el checkout
    /// compartido (es decir: no esta aislada).
    pub worktree: Option<PathBuf>,
}

impl Ocupacion {
    /// Como se nombra en un mensaje de error.
    fn etiqueta(&self) -> String {
        format!("#{} {}", self.id, self.nombre)
    }

    fn aislada(&self) -> bool {
        self.worktree.is_some()
    }
}

/// Lo que hay que saber para decidir, sin tocar disco.
#[derive(Debug, Clone)]
pub struct Contexto<'a> {
    /// Raiz del repo git del proyecto, o `None` si no hay git utilizable.
    pub repo: Option<&'a Path>,
    /// Worktree que le tocaria a esta feature (solo si `repo` es `Some`).
    pub destino: Option<PathBuf>,
    /// Las otras features `in_progress`.
    pub otras: &'a [Ocupacion],
    /// El usuario pidio `--sin-worktree`.
    pub sin_worktree: bool,
}

/// Por que una feature queda sin aislar cuando igual se la deja arrancar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NoAislado {
    /// El proyecto no es un repo git: no hay worktrees que dar.
    SinGit,
    /// `--sin-worktree` y ninguna otra feature abierta.
    SerialSinWorktree,
}

impl NoAislado {
    /// La linea que `start` imprime. Dice que NO hay, no que todo esta bien.
    pub fn aviso(&self) -> String {
        let porque = match self {
            Self::SinGit => "no hay repo git utilizable",
            Self::SerialSinWorktree => "--sin-worktree",
        };
        format!(
            "  [!] Feature NO AISLADA ({porque}): se escribe en el checkout compartido.\n      \
             Mientras siga abierta, el arnes no va a permitir arrancar otra: sin worktree\n      \
             no hay forma de atribuir un cambio a una feature."
        )
    }
}

/// Por que un arranque se rechaza. Cada variante lleva lo que hace falta para
/// que el mensaje diga que hacer, y no solo que algo salio mal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Rechazo {
    /// `--sin-worktree` con otra feature abierta: el bypass inseguro del AC-1.
    BypassEnParalelo { otras: Vec<String> },
    /// Ya hay una feature escribiendo en el checkout compartido.
    OcupanteSinAislar { otra: String },
    /// Dos features apuntando al MISMO worktree.
    CheckoutCompartido { otra: String, ruta: PathBuf },
    /// `git worktree add` (o lo que sea) fallo. Antes esto era un `println!`.
    FalloDeGit { detalle: String },
}

impl Rechazo {
    pub fn mensaje(&self) -> String {
        match self {
            Self::BypassEnParalelo { otras } => format!(
                "--sin-worktree con otra feature abierta ({}).\n\
                 Dos features en el mismo checkout mezclan sus cambios: es como se publico\n\
                 un commit que se habia acordado dejar local (diagnostico 2026-09-04, seccion 3).\n\
                 Salidas:\n\
                 \x20 1. Arranca con worktree (sin el flag): cada feature en el suyo.\n\
                 \x20 2. Cerra o pausa la otra feature primero, si de verdad queres trabajar serial.",
                otras.join(", ")
            ),
            Self::OcupanteSinAislar { otra } => format!(
                "la feature {otra} esta abierta SIN worktree, escribiendo en el checkout compartido.\n\
                 Arrancar una segunda ahi deja sus cambios entremezclados y sin dueno atribuible.\n\
                 Salidas:\n\
                 \x20 1. Cerra {otra}, o\n\
                 \x20 2. Volve a arrancarla con worktree para que libere el checkout compartido."
            ),
            Self::CheckoutCompartido { otra, ruta } => format!(
                "ese worktree ya es de la feature {otra}: {}\n\
                 Dos features no pueden compartir arbol de trabajo.",
                ruta.display()
            ),
            Self::FalloDeGit { detalle } => format!(
                "no se pudo preparar el aislamiento: {detalle}\n\
                 El arranque se cancela: antes esto se avisaba y se seguia igual, y asi es como\n\
                 quedaron activas tres features sin rama ni worktree.\n\
                 Salidas:\n\
                 \x20 1. Arregla lo que git reporta y volve a correr start.\n\
                 \x20 2. Si de verdad no hay forma, arranca serial con --sin-worktree\n\
                 \x20    (queda declarada NO AISLADA y bloquea abrir otra)."
            ),
        }
    }
}

/// Lo que `start` tiene que hacer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Decision {
    /// Crear (o reusar) rama y worktree.
    Aislar,
    /// Seguir sin aislamiento, declarandolo.
    Seguir(NoAislado),
    /// No arrancar. El backlog no se toca.
    Rechazar(Rechazo),
}

/// La decision del AC-1. PURA: no consulta git ni el filesystem.
///
/// El orden de los rechazos no es casual — se comprueba primero lo que el
/// usuario acaba de pedir (`--sin-worktree`) y despues lo que ya estaba pasando
/// (una ocupante sin aislar), para que el mensaje hable de su comando y no de
/// un estado que el no eligio en esta corrida.
pub fn decidir(ctx: &Contexto) -> Decision {
    // Sin git no hay worktrees que repartir. Sigue siendo un estado valido
    // —el arnes corre en proyectos sin git—, pero uno solo a la vez.
    let Some(_repo) = ctx.repo else {
        return match primera_ocupante(ctx.otras) {
            Some(o) => Decision::Rechazar(Rechazo::OcupanteSinAislar {
                otra: o.etiqueta(),
            }),
            None => Decision::Seguir(NoAislado::SinGit),
        };
    };

    if ctx.sin_worktree {
        let otras: Vec<String> = ctx.otras.iter().map(Ocupacion::etiqueta).collect();
        if !otras.is_empty() {
            return Decision::Rechazar(Rechazo::BypassEnParalelo { otras });
        }
        return Decision::Seguir(NoAislado::SerialSinWorktree);
    }

    // Una feature abierta sin aislar tiene tomado el checkout compartido: no
    // hay paralelo de escritura contra ella, ni aunque esta si traiga worktree.
    if let Some(o) = ctx.otras.iter().find(|o| !o.aislada()) {
        return Decision::Rechazar(Rechazo::OcupanteSinAislar {
            otra: o.etiqueta(),
        });
    }

    if let Some(destino) = &ctx.destino
        && let Some(o) = ctx
            .otras
            .iter()
            .find(|o| o.worktree.as_deref().is_some_and(|w| mismo(w, destino)))
    {
        return Decision::Rechazar(Rechazo::CheckoutCompartido {
            otra: o.etiqueta(),
            ruta: destino.clone(),
        });
    }

    Decision::Aislar
}

/// La primera feature abierta, aislada o no. Sin git, cualquiera ocupa.
fn primera_ocupante(otras: &[Ocupacion]) -> Option<&Ocupacion> {
    otras.first()
}

/// Compara rutas por identidad real, con fallback lexico: dos features pueden
/// declarar la misma carpeta escrita distinto (`./x` y `x`).
fn mismo(a: &Path, b: &Path) -> bool {
    match (std::fs::canonicalize(a), std::fs::canonicalize(b)) {
        (Ok(ca), Ok(cb)) => ca == cb,
        _ => a == b,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ocupacion(id: &str, worktree: Option<&str>) -> Ocupacion {
        Ocupacion {
            id: id.to_string(),
            nombre: format!("la {id}"),
            worktree: worktree.map(PathBuf::from),
        }
    }

    fn ctx<'a>(
        repo: Option<&'a Path>,
        destino: Option<&str>,
        otras: &'a [Ocupacion],
        sin_worktree: bool,
    ) -> Contexto<'a> {
        Contexto {
            repo,
            destino: destino.map(PathBuf::from),
            otras,
            sin_worktree,
        }
    }

    const REPO: &str = "/tmp/proyecto";

    #[test]
    fn con_git_y_sola_se_aisla() {
        let sin_otras: [Ocupacion; 0] = [];
        let c = ctx(Some(Path::new(REPO)), Some("/tmp/p-wt/72-x"), &sin_otras, false);
        assert_eq!(decidir(&c), Decision::Aislar);
    }

    #[test]
    fn con_git_y_otra_aislada_se_aisla_igual() {
        // El paralelo UTIL se conserva: dos features con worktrees distintos
        // no se estorban. El spec pide acotar el paralelo, no apagarlo.
        let otras = [ocupacion("70", Some("/tmp/p-wt/70-y"))];
        let c = ctx(Some(Path::new(REPO)), Some("/tmp/p-wt/72-x"), &otras, false);
        assert_eq!(decidir(&c), Decision::Aislar);
    }

    #[test]
    fn sin_worktree_y_sola_sigue_declarada_no_aislada() {
        let sin_otras: [Ocupacion; 0] = [];
        let c = ctx(Some(Path::new(REPO)), None, &sin_otras, true);
        assert_eq!(decidir(&c), Decision::Seguir(NoAislado::SerialSinWorktree));
    }

    #[test]
    fn sin_worktree_con_otra_abierta_se_rechaza() {
        // El caso exacto del diagnostico: #121, #122, #126 y #98 arrancadas
        // con --sin-worktree sobre el mismo checkout.
        let otras = [ocupacion("122", Some("/tmp/p-wt/122-y"))];
        let c = ctx(Some(Path::new(REPO)), None, &otras, true);
        let Decision::Rechazar(r) = decidir(&c) else {
            panic!("tenia que rechazar");
        };
        assert_eq!(
            r,
            Rechazo::BypassEnParalelo {
                otras: vec!["#122 la 122".to_string()]
            }
        );
        // El mensaje nombra a la otra feature y ofrece las dos salidas.
        let m = r.mensaje();
        assert!(m.contains("#122"), "nombra la otra: {m}");
        assert!(m.contains("Arranca con worktree"), "dice que hacer: {m}");
    }

    #[test]
    fn una_ocupante_sin_aislar_bloquea_a_la_siguiente() {
        // AC-1: el uso serial sin worktree NO habilita paralelo de escritura.
        let otras = [ocupacion("98", None)];
        let c = ctx(Some(Path::new(REPO)), Some("/tmp/p-wt/72-x"), &otras, false);
        assert_eq!(
            decidir(&c),
            Decision::Rechazar(Rechazo::OcupanteSinAislar {
                otra: "#98 la 98".to_string()
            })
        );
    }

    #[test]
    fn dos_features_al_mismo_worktree_se_rechazan() {
        let otras = [ocupacion("70", Some("/tmp/p-wt/72-x"))];
        let c = ctx(Some(Path::new(REPO)), Some("/tmp/p-wt/72-x"), &otras, false);
        assert_eq!(
            decidir(&c),
            Decision::Rechazar(Rechazo::CheckoutCompartido {
                otra: "#70 la 70".to_string(),
                ruta: PathBuf::from("/tmp/p-wt/72-x"),
            })
        );
    }

    #[test]
    fn sin_git_y_sola_sigue_declarada_no_aislada() {
        let sin_otras: [Ocupacion; 0] = [];
        let c = ctx(None, None, &sin_otras, false);
        assert_eq!(decidir(&c), Decision::Seguir(NoAislado::SinGit));
    }

    #[test]
    fn sin_git_no_hay_paralelo_de_escritura() {
        // Sin git no hay como aislar a NADIE: la segunda feature se rechaza
        // aunque no haya pedido --sin-worktree.
        let otras = [ocupacion("5", None)];
        let c = ctx(None, None, &otras, false);
        assert_eq!(
            decidir(&c),
            Decision::Rechazar(Rechazo::OcupanteSinAislar {
                otra: "#5 la 5".to_string()
            })
        );
    }

    /// Los avisos y los rechazos tienen que ser distinguibles a simple vista:
    /// el aviso dice que la feature NO esta aislada, y no felicita.
    #[test]
    fn el_aviso_de_no_aislado_no_suena_a_exito() {
        for n in [NoAislado::SinGit, NoAislado::SerialSinWorktree] {
            let a = n.aviso();
            assert!(a.contains("NO AISLADA"), "{a}");
            assert!(a.contains("checkout compartido"), "{a}");
        }
    }

    /// El fallo de git tiene que decir que el arranque SE CANCELA: era el
    /// caso que antes se tragaba con un `[i]` y seguia.
    #[test]
    fn el_fallo_de_git_dice_que_cancela() {
        let r = Rechazo::FalloDeGit {
            detalle: "fatal: invalid reference: develop".to_string(),
        };
        let m = r.mensaje();
        assert!(m.contains("fatal: invalid reference"), "conserva el error: {m}");
        assert!(m.contains("El arranque se cancela"), "{m}");
        assert!(m.contains("--sin-worktree"), "ofrece la salida serial: {m}");
    }
}
