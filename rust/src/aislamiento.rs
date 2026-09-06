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
    /// El usuario trae un arbol YA preparado a mano (`--worktree <ruta>`).
    ///
    /// Existe por los proyectos MULTI-REPO, donde `repo` es `None` y el arnes
    /// no tiene de donde sacar un worktree: la raiz no es un repo git, los
    /// repos de verdad cuelgan de ella. Ahi el aislamiento SI se puede
    /// conseguir —`git worktree add` funciona en cada sub-repo— pero lo tiene
    /// que armar quien conoce el layout, no el arnes. Sin esta puerta, una
    /// feature abierta en un proyecto asi veta a TODAS las demas para siempre,
    /// que es el bloqueo medido el 2026-09-06 con la #89 y la #99.
    pub worktree_externo: Option<&'a Path>,
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
             Mientras siga abierta, ninguna OTRA puede escribir ahi: sin worktree no hay forma\n      \
             de atribuir un cambio a una feature. Las que traigan su worktree arrancan igual."
        )
    }
}

/// Por que un arranque se rechaza. Cada variante lleva lo que hace falta para
/// que el mensaje diga que hacer, y no solo que algo salio mal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Rechazo {
    /// Ya hay una feature escribiendo en el checkout compartido, y esta
    /// tambien lo haria: el checkout compartido tiene capacidad UNO.
    ///
    /// Feature #76: aca habia una segunda variante, `BypassEnParalelo`, para
    /// `--sin-worktree` con CUALQUIER otra feature abierta. Era mas ancha de lo
    /// que la evidencia aguantaba —el incidente fueron cuatro features sin
    /// worktree en el MISMO arbol, no una aislada conviviendo con una que no—
    /// y dejaba a una feature sin worktree vetando a las que si lo traian.
    OcupanteSinAislar { otra: String },
    /// Dos features apuntando al MISMO worktree.
    CheckoutCompartido { otra: String, ruta: PathBuf },
    /// `git worktree add` (o lo que sea) fallo. Antes esto era un `println!`.
    FalloDeGit { detalle: String },
    /// No hay repo git en la raiz —proyecto multi-repo— y ya hay otra feature
    /// abierta. Es el MISMO hecho que `OcupanteSinAislar`, con otro mensaje:
    /// ahi la salida es "arranca con worktree" y aca esa salida NO EXISTE,
    /// porque no hay de donde sacarlo. Ofrecerla igual fue el defecto medido
    /// el 2026-09-06: el gate mandaba a correr el comando que se acababa de
    /// rechazar.
    SinGitConOcupante { otra: String },
}

impl Rechazo {
    pub fn mensaje(&self) -> String {
        match self {
            Self::OcupanteSinAislar { otra } => format!(
                "la feature {otra} ya esta escribiendo en el checkout compartido, y esta tambien lo haria.\n\
                 Dos features en el mismo arbol mezclan sus cambios sin dueno atribuible: es como se\n\
                 publico un commit que se habia acordado dejar local (diagnostico 2026-09-04, seccion 3).\n\
                 Salidas:\n\
                 \x20 1. Arranca ESTA con worktree (sin --sin-worktree): tendra su propio arbol.\n\
                 \x20 2. O cerra {otra}, o volve a arrancarla con worktree, para liberar el checkout."
            ),
            Self::CheckoutCompartido { otra, ruta } => format!(
                "ese worktree ya es de la feature {otra}: {}\n\
                 Dos features no pueden compartir arbol de trabajo.",
                ruta.display()
            ),
            Self::SinGitConOcupante { otra } => format!(
                "la raiz del proyecto no es un repo git —es multi-repo— asi que el arnes no tiene\n\
                 de donde sacar un worktree, y la feature {otra} ya esta escribiendo en el checkout\n\
                 compartido. Dos features en el mismo arbol mezclan sus cambios sin dueno atribuible.\n\
                 Salidas (NO esta la de \"arranca con worktree\": aca no hay uno que dar):\n\
                 \x20 1. Prepara vos el arbol y declaralo:\n\
                 \x20      git -C <sub-repo> worktree add <ruta> -b <rama> <base>\n\
                 \x20      harness start --feature <id> --worktree <ruta>\n\
                 \x20    En multi-repo hay que repetir el `worktree add` por cada repo que la\n\
                 \x20    feature toque; la ruta que se declara es la del arbol que las suites\n\
                 \x20    resuelven (tipicamente el hermano del repo principal de la feature).\n\
                 \x20 2. O cerra {otra} para liberar el checkout compartido."
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
    /// Crear (o reusar) rama y worktree. `conviven_sin_aislar` son las
    /// features abiertas que escriben en el checkout compartido: no impiden
    /// este arranque —esta tiene su arbol— pero se INFORMAN (feature #76).
    Aislar { conviven_sin_aislar: Vec<String> },
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
    // Feature #76: el checkout compartido es un recurso de capacidad UNO. Lo
    // unico que se rechaza es que DOS features escriban ahi. Una feature con
    // su propio worktree no comparte nada con una que no lo tiene —escriben en
    // directorios distintos— asi que nunca se estorban.
    //
    // Antes de la #76 una feature sin aislar vetaba a TODAS las demas, y el
    // usuario terminaba esperando a que "la #99 libere" para poder arrancar
    // una feature que tenia su propio arbol. Era una regla mas ancha de lo que
    // el incidente original sostenia.
    let ocupante = ctx.otras.iter().find(|o| !o.aislada());

    // Un arbol traido de afuera aisla igual que uno que el arnes hubiera
    // creado: lo que el gate protege es que dos features no escriban en el
    // MISMO sitio, y eso no depende de quien corrio `git worktree add`. Se
    // mira antes que `ctx.repo` a proposito — el caso que esto viene a
    // desbloquear es justamente el de la raiz que no es repo git.
    if let Some(externo) = ctx.worktree_externo {
        if let Some(o) = ctx
            .otras
            .iter()
            .find(|o| o.worktree.as_deref().is_some_and(|w| mismo(w, externo)))
        {
            return Decision::Rechazar(Rechazo::CheckoutCompartido {
                otra: o.etiqueta(),
                ruta: externo.to_path_buf(),
            });
        }
        return Decision::Aislar {
            conviven_sin_aislar: ctx
                .otras
                .iter()
                .filter(|o| !o.aislada())
                .map(Ocupacion::etiqueta)
                .collect(),
        };
    }

    // Sin git no hay worktrees que repartir: esta feature va a escribir en el
    // checkout compartido, y ahi TODAS las abiertas escriben. Una a la vez.
    let Some(_repo) = ctx.repo else {
        return match ctx.otras.first() {
            Some(o) => Decision::Rechazar(Rechazo::SinGitConOcupante {
                otra: o.etiqueta(),
            }),
            None => Decision::Seguir(NoAislado::SinGit),
        };
    };

    // `--sin-worktree`: esta va a ocupar el checkout compartido. Solo se
    // rechaza si YA hay otra ahi; una feature aislada al lado no molesta.
    if ctx.sin_worktree {
        return match ocupante {
            Some(o) => Decision::Rechazar(Rechazo::OcupanteSinAislar {
                otra: o.etiqueta(),
            }),
            None => Decision::Seguir(NoAislado::SerialSinWorktree),
        };
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

    // Esta tiene su arbol. Si hay una sin aislar, se INFORMA: no la bloquea,
    // pero quien arranca tiene derecho a saber que el checkout esta ocupado.
    Decision::Aislar {
        conviven_sin_aislar: ctx
            .otras
            .iter()
            .filter(|o| !o.aislada())
            .map(Ocupacion::etiqueta)
            .collect(),
    }
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
            worktree_externo: None,
        }
    }

    /// El mismo contexto, pero con un arbol traido de afuera.
    fn ctx_externo<'a>(otras: &'a [Ocupacion], externo: &'a Path) -> Contexto<'a> {
        Contexto {
            repo: None,
            destino: None,
            otras,
            sin_worktree: false,
            worktree_externo: Some(externo),
        }
    }

    const REPO: &str = "/tmp/proyecto";

    fn aislar_sola() -> Decision {
        Decision::Aislar {
            conviven_sin_aislar: Vec::new(),
        }
    }

    #[test]
    fn sin_git_y_con_otra_abierta_el_mensaje_no_ofrece_lo_imposible() {
        // El defecto medido el 2026-09-06: el rechazo decia "arranca ESTA con
        // worktree (sin --sin-worktree)", que es EXACTAMENTE el comando que
        // acababa de rechazar, porque sin repo git no hay worktree que dar.
        let otras = [ocupacion("99", None)];
        let d = decidir(&ctx(None, None, &otras, false));
        assert_eq!(
            d,
            Decision::Rechazar(Rechazo::SinGitConOcupante {
                otra: "#99 la 99".to_string()
            })
        );
        let msg = match d {
            Decision::Rechazar(r) => r.mensaje(),
            _ => unreachable!(),
        };
        assert!(
            !msg.contains("Arranca ESTA con worktree"),
            "el rechazo vuelve a ofrecer el comando que rechaza: {msg}"
        );
        assert!(msg.contains("--worktree"), "no dice la salida que SI existe: {msg}");
    }

    #[test]
    fn un_arbol_traido_de_afuera_aisla_aunque_no_haya_git_en_la_raiz() {
        // El caso multi-repo: la raiz no es repo, pero el usuario preparo el
        // arbol en un sub-repo. Que lo haya creado el no lo hace menos aislado.
        let otras = [ocupacion("99", None)];
        let externo = PathBuf::from("/tmp/proyecto-wt/89");
        assert_eq!(
            decidir(&ctx_externo(&otras, &externo)),
            Decision::Aislar {
                conviven_sin_aislar: vec!["#99 la 99".to_string()]
            }
        );
    }

    #[test]
    fn un_arbol_externo_ya_tomado_se_rechaza() {
        // La puerta nueva no puede ser una forma barata de saltearse el gate:
        // dos features declarando el MISMO arbol es el defecto que el gate
        // existe para impedir.
        let otras = [ocupacion("99", Some("/tmp/proyecto-wt/89"))];
        let externo = PathBuf::from("/tmp/proyecto-wt/89");
        assert_eq!(
            decidir(&ctx_externo(&otras, &externo)),
            Decision::Rechazar(Rechazo::CheckoutCompartido {
                otra: "#99 la 99".to_string(),
                ruta: externo
            })
        );
    }

    #[test]
    fn con_git_y_sola_se_aisla() {
        let sin_otras: [Ocupacion; 0] = [];
        let c = ctx(Some(Path::new(REPO)), Some("/tmp/p-wt/72-x"), &sin_otras, false);
        assert_eq!(decidir(&c), aislar_sola());
    }

    #[test]
    fn con_git_y_otra_aislada_se_aisla_igual() {
        // El paralelo UTIL se conserva: dos features con worktrees distintos
        // no se estorban. El spec pide acotar el paralelo, no apagarlo.
        let otras = [ocupacion("70", Some("/tmp/p-wt/70-y"))];
        let c = ctx(Some(Path::new(REPO)), Some("/tmp/p-wt/72-x"), &otras, false);
        assert_eq!(decidir(&c), aislar_sola());
    }

    /// Feature #76, EL caso reportado: una feature abierta sin worktree y otra
    /// que SI trae el suyo. Escriben en directorios distintos, asi que la
    /// segunda arranca — y se le INFORMA que la primera esta sin aislar.
    ///
    /// Antes de la #76 esto se rechazaba, y el usuario terminaba escribiendo
    /// "avisame cuando la #99 libere y arranca".
    #[test]
    fn con_worktree_propio_convive_con_una_sin_aislar() {
        let otras = [ocupacion("99", None)];
        let c = ctx(Some(Path::new(REPO)), Some("/tmp/p-wt/72-x"), &otras, false);
        assert_eq!(
            decidir(&c),
            Decision::Aislar {
                conviven_sin_aislar: vec!["#99 la 99".to_string()]
            },
            "arranca, y sabe con quien convive"
        );
    }

    #[test]
    fn sin_worktree_y_sola_sigue_declarada_no_aislada() {
        let sin_otras: [Ocupacion; 0] = [];
        let c = ctx(Some(Path::new(REPO)), None, &sin_otras, true);
        assert_eq!(decidir(&c), Decision::Seguir(NoAislado::SerialSinWorktree));
    }

    /// Feature #76: `--sin-worktree` con otra feature que SI tiene worktree.
    /// Esta ocuparia el checkout compartido SOLA: arranca, declarada no
    /// aislada.
    ///
    /// Este test se llamaba `sin_worktree_con_otra_abierta_se_rechaza` y
    /// afirmaba el rechazo. Codificaba la regla ancha de la #72.
    #[test]
    fn sin_worktree_junto_a_una_aislada_arranca_serial() {
        let otras = [ocupacion("122", Some("/tmp/p-wt/122-y"))];
        let c = ctx(Some(Path::new(REPO)), None, &otras, true);
        assert_eq!(decidir(&c), Decision::Seguir(NoAislado::SerialSinWorktree));
    }

    /// El caso exacto del diagnostico: #121, #122, #126 y #98 arrancadas con
    /// --sin-worktree sobre el MISMO checkout. Dos sin aislar: se rechaza.
    #[test]
    fn dos_sin_aislar_en_el_mismo_checkout_se_rechazan() {
        let otras = [ocupacion("122", None)];
        let c = ctx(Some(Path::new(REPO)), None, &otras, true);
        let Decision::Rechazar(r) = decidir(&c) else {
            panic!("tenia que rechazar");
        };
        assert_eq!(
            r,
            Rechazo::OcupanteSinAislar {
                otra: "#122 la 122".to_string()
            }
        );
        // El mensaje nombra a la otra y ofrece la salida que de verdad resuelve:
        // arrancar ESTA con worktree.
        let m = r.mensaje();
        assert!(m.contains("#122"), "nombra la otra: {m}");
        assert!(m.contains("Arranca ESTA con worktree"), "dice que hacer: {m}");
    }

    /// La tabla completa del spec de la #76, en un solo lugar.
    #[test]
    fn la_tabla_del_checkout_de_capacidad_uno() {
        let aislada = ocupacion("1", Some("/tmp/p-wt/1-a"));
        let sin_aislar = ocupacion("2", None);
        let repo = Some(Path::new(REPO));
        let dest = Some("/tmp/p-wt/9-z");
        // nueva CON worktree, ya hay otra sin aislar -> ARRANCA e informa
        assert!(matches!(
            decidir(&ctx(repo, dest, std::slice::from_ref(&sin_aislar), false)),
            Decision::Aislar { conviven_sin_aislar } if conviven_sin_aislar.len() == 1
        ));
        // nueva CON worktree, no hay otra sin aislar -> ARRANCA
        assert_eq!(
            decidir(&ctx(repo, dest, std::slice::from_ref(&aislada), false)),
            aislar_sola()
        );
        // nueva SIN aislar, ya hay otra sin aislar -> RECHAZO
        assert!(matches!(
            decidir(&ctx(repo, None, std::slice::from_ref(&sin_aislar), true)),
            Decision::Rechazar(Rechazo::OcupanteSinAislar { .. })
        ));
        // nueva SIN aislar, no hay otra sin aislar -> ARRANCA declarada
        assert_eq!(
            decidir(&ctx(repo, None, std::slice::from_ref(&aislada), true)),
            Decision::Seguir(NoAislado::SerialSinWorktree)
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
        // Sin git el arnes no puede aislar a NADIE por su cuenta: la segunda
        // feature se rechaza aunque no haya pedido --sin-worktree.
        //
        // El rechazo es `SinGitConOcupante` y no `OcupanteSinAislar`: el HECHO
        // es el mismo, pero las salidas no. En el caso con git la salida es
        // "arranca con worktree"; aca esa salida no existe y ofrecerla mandaba
        // a correr el comando que se acababa de rechazar. La unica que queda
        // es traer el arbol de afuera con --worktree.
        let otras = [ocupacion("5", None)];
        let c = ctx(None, None, &otras, false);
        assert_eq!(
            decidir(&c),
            Decision::Rechazar(Rechazo::SinGitConOcupante {
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
            // Feature #76: el aviso ya no puede prometer que bloquea a TODAS.
            assert!(a.contains("traigan su worktree arrancan igual"), "{a}");
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
