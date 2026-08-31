//! `harness close --feature <id> --status <estado>` (paridad: cmd_close).

use std::io::Write;

use serde_json::{Value, json};

use crate::exit::Exit;
use crate::features::{feature_at, feature_mut, find_feature_index, load_features, save_features};
use crate::lecciones;
use crate::memories::update_memories;
use crate::paths::HarnessPaths;
use crate::plan::{plan_path, slugify};
use crate::prd;
use crate::progress::{log, now_stamp};
use crate::pycompat::{py_str, relpath};
use crate::spec::{close_requires_spec, spec_gate, spec_path};

/// Estado de una entrada cuyo trabajo se hizo en OTRA feature (#37). No es
/// `done` (nunca tuvo spec ni evidencia propia) ni `blocked` (no esta trabada).
pub const SUPERSEDED: &str = "superseded";

/// Estado de una entrada cuyo trabajo se hizo en OTRO REPO (#65).
///
/// Distinto de `superseded`, que exige una feature de ESTE backlog: el caso real
/// es un bug del arnes reportado en un repo de trabajo y arreglado aguas arriba.
/// Medido antes de esta feature: el unico camino que existia
/// (`--absorbida-por 60`) cerraba con rc=0 apuntando a la #60 del backlog LOCAL,
/// que era otra feature — una afirmacion falsa con exit 0.
pub const AGUAS_ARRIBA: &str = "resuelto-aguas-arriba";

/// Los estados que acepta `close`, en un solo lugar.
///
/// `cli.rs` los tomaba de un literal propio y el resto de los consumidores de
/// los suyos: agregar un estado y olvidarse de uno no rompia nada hasta que
/// alguien miraba el tablero. Ahora la lista es esta, y el test del AC-9 la
/// recorre entera contra cada consumidor.
pub const ESTADOS_DE_CIERRE: [&str; 5] = ["done", "blocked", "pending", SUPERSEDED, AGUAS_ARRIBA];

/// ¿Tiene la forma `<proyecto>/feature-<id>`?
///
/// Se comprueba la FORMA y nada mas. La existencia vive en otro repo y el arnes
/// no la puede abrir: fingir que la valido seria exactamente lo que la #64
/// prohibio ("el arnes no promete enforcement que no hace"). La sintaxis no se
/// inventa: es la que `graph/ids.rs` ya usa para lo cross-proyecto, asi que el
/// cierre aguas abajo y el evento del hub aguas arriba nombran el mismo id.
pub fn forma_de_referencia_externa(r: &str) -> bool {
    let r = r.trim();
    let Some((proyecto, resto)) = r.split_once('/') else {
        return false;
    };
    if proyecto.is_empty() || proyecto.contains(char::is_whitespace) {
        return false;
    }
    let Some(id) = resto.strip_prefix("feature-") else {
        return false;
    };
    !id.is_empty() && id.chars().all(|c| c.is_ascii_digit())
}
use crate::verificacion;

/// Todo lo que decide un cierre, junto: el estado, su justificacion y — desde
/// la feature #47 — la rama a la que se integra.
pub struct CierreOpts<'a> {
    pub status: &'a str,
    pub note: Option<&'a str>,
    pub absorbida_por: Option<&'a str>,
    /// Donde se arreglo, en otro repo (`<proyecto>/feature-<id>`), con
    /// `--status resuelto-aguas-arriba` (feature #65).
    pub resuelto_en: Option<&'a str>,
    pub leccion: Option<&'a str>,
    pub leccion_motivo: Option<&'a str>,
    /// Rama destino del `done` (GitFlow). Sin ella el arnes se niega: la
    /// decide el USUARIO.
    pub to: Option<&'a str>,
}

pub fn run(paths: &HarnessPaths, fid: &str, opts: CierreOpts<'_>) -> anyhow::Result<()> {
    let CierreOpts {
        status,
        note,
        resuelto_en,
        absorbida_por,
        leccion,
        leccion_motivo,
        to,
    } = opts;
    let mut data = load_features(paths)?;
    let idx = find_feature_index(&data, fid)?;
    // El PRD es un documento RAIZ y COMPARTIDO: no vive en la rama de ninguna
    // feature (feature #60). Se guardan las rutas de la raiz ANTES de sombrear
    // `paths` con las de la feature.
    let raiz = paths;
    // Feature #47: los docs (spec, plan, evidencia) viven en el worktree de la
    // feature, no en el directorio desde el que se corre el comando.
    let paths = &match feature_at(&data, idx).as_object() {
        Some(f) => paths.para_feature(f),
        None => paths.para_feature(&serde_json::Map::new()),
    };
    // Estado `superseded` (feature #37): el trabajo se hizo en OTRA feature.
    // Exige decir cual, y esa referencia se valida: una entrada que dice
    // "absorbida" sin decir por quien es una nota en prosa, no trazabilidad.
    // No pasa por los gates de `done` a proposito — el spec, la leccion, el
    // reporte de verify y la propuesta de documentos viven en la que absorbio.
    let absorbida = if status == SUPERSEDED {
        let Some(por) = absorbida_por.map(str::trim).filter(|s| !s.is_empty()) else {
            return Err(Exit {
                code: 2,
                message: Some(format!(
                    "--status superseded exige --absorbida-por <id>: hay que decir QUE feature\n    \
                     absorbio este trabajo, o el estado no significa nada.\n    \
                     Ejemplo: sh harness_cli close --feature {fid} --status superseded --absorbida-por 36"
                )),
            }
            .into());
        };
        if find_feature_index(&data, por).is_err() {
            return Err(Exit {
                code: 2,
                message: Some(format!(
                    "--absorbida-por {por}: esa feature no existe. Una referencia rota es peor que ninguna."
                )),
            }
            .into());
        }
        if por == fid {
            return Err(Exit {
                code: 2,
                message: Some(format!("--absorbida-por {por}: una feature no se absorbe a si misma.")),
            }
            .into());
        }
        Some(por.to_string())
    } else {
        None
    };
    // Estado `resuelto-aguas-arriba` (feature #65): el trabajo se hizo en OTRO
    // repo. Exige decir donde, y de esa referencia se comprueba la FORMA — la
    // existencia no, porque vive en un repo que el arnes no puede abrir, y el
    // mensaje lo dice en vez de fingir que la valido.
    let resuelto_en_ref = if status == AGUAS_ARRIBA {
        let Some(r) = resuelto_en.map(str::trim).filter(|s| !s.is_empty()) else {
            return Err(Exit {
                code: 2,
                message: Some(format!(
                    "--status {AGUAS_ARRIBA} exige --resuelto-en <proyecto>/feature-<id>: hay que decir\n    \
                     DONDE se arreglo, o el estado no significa nada.\n    \
                     Ejemplo: sh harness_cli close --feature {fid} --status {AGUAS_ARRIBA} \\\n      \
                     --resuelto-en harness_process/feature-60"
                )),
            }
            .into());
        };
        if !forma_de_referencia_externa(r) {
            return Err(Exit {
                code: 2,
                message: Some(format!(
                    "--resuelto-en {r}: la forma esperada es <proyecto>/feature-<id>.\n    \
                     Es la misma que el arnes ya usa para lo cross-proyecto, asi que el cierre\n    \
                     de aca y el evento de alla nombran el mismo id.\n    \
                     Ejemplo: harness_process/feature-60"
                )),
            }
            .into());
        }
        Some(r.to_string())
    } else {
        None
    };
    // Gate SDD: cerrar como done exige spec aprobado por el usuario; se valida
    // ANTES de mutar la feature. blocked/pending no gatean (valvula de escape
    // para abortar/aparcar).
    if close_requires_spec(status) {
        let Some(feature) = feature_at(&data, idx).as_object() else {
            anyhow::bail!("feature_list.json: feature invalida");
        };
        spec_gate(paths, &data, feature)?;
    }
    // Gate de verificacion (feature #23): si el spec declara comandos y la regla
    // esta activa, exige el reporte verde y fresco. LEE el reporte; cerrar nunca
    // ejecuta un comando.
    if close_requires_spec(status) {
        let Some(feature) = feature_at(&data, idx).as_object() else {
            anyhow::bail!("feature_list.json: feature invalida");
        };
        verificacion::gate(paths, &data, status, &spec_path(paths, feature), fid)?;
    }
    // Gate de documentos (feature #29): si la regla esta activa, el PRD, el SDD
    // y architecture.md tienen que reflejar lo implementado, con la propuesta
    // aprobada por el USUARIO. Solo LEE: escribir es `prd apply --yes`.
    if close_requires_spec(status) {
        let feature = feature_at(&data, idx).clone();
        crate::documentos::gate(paths, &data, status, &feature, fid)?;
    }
    // Gate de revision (feature #64): cerrar como done exige el veredicto
    // ESTAMPADO del reviewer. Va despues de verify (primero se prueba, despues
    // se revisa) y antes de declarar el aprendizaje. Solo LEE: estampar es
    // `revision --veredicto`.
    if close_requires_spec(status) {
        let Some(feature) = feature_at(&data, idx).as_object() else {
            anyhow::bail!("feature_list.json: feature invalida");
        };
        crate::revision::gate(paths, &data, status, feature, fid)?;
    }
    // Gate de aprendizaje (feature #17): cerrar como done declara que se
    // aprendio. Se valida tambien ANTES de mutar, por la misma razon.
    let declaracion = lecciones::gate(paths, &data, status, leccion, leccion_motivo)?;
    let stamp = now_stamp();
    let note_text = note.unwrap_or_default().to_string();
    // Datos derivados de la feature, SIN mutarla todavia (feature #62).
    let (plan, feature_id, feature_name, slug) = {
        let Some(feature) = feature_at(&data, idx).as_object() else {
            anyhow::bail!("feature_list.json: feature invalida");
        };
        let name = feature
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        (
            plan_path(paths, feature),
            py_str(feature.get("id")),
            py_str(feature.get("name")),
            slugify(&name),
        )
    };

    // --- FASE 0 (feature #62): todo lo que puede NEGARSE, antes de escribir ---
    //
    // La integracion se DECIDE y se VALIDA aca (falta `--to`, colisiones con
    // trabajo sin commitear), pero no se ejecuta. Antes esto vivia adentro de
    // `integrar`, que corria al final: para cuando se negaba, el arnes ya habia
    // escrito el backlog, emitido la transicion a Jira, anotado el plan,
    // archivado el estado, reescrito el indice, dejado la linea en history.md,
    // guardado la memoria en el hub y dicho "Feature #N cerrada". Nueve
    // afirmaciones sobre un trabajo que no estaba integrado.
    let integracion = planificar_integracion(paths, &data, idx, status, to, &feature_id)?;

    // --- FASE 1: los artefactos que tienen que VIAJAR EN LA RAMA ---
    //
    // Estos dos no se pueden dejar para el final: viven en el `docs/` del
    // worktree y el merge borra ese worktree. Escribirlos despues seria no
    // escribirlos nunca. Por eso son IDEMPOTENTES: si la integracion falla
    // quedan escritos, y el reintento no los duplica.
    anotar_plan(&plan, &stamp, status, &note_text)?;
    let archived_rel = archivar_estado(
        paths,
        &feature_id,
        &feature_name,
        &slug,
        &stamp,
        status,
        &note_text,
    )?;

    // --- FASE 2: integrar. Si falla, NADA del estado se escribio ---
    ejecutar_integracion(&integracion, &feature_id)?;

    // --- FASE 3: recien ahora, el estado. El cierre ya ocurrio ---
    {
        let feature = feature_mut(&mut data, idx)?;
        feature.insert("status".to_string(), json!(status));
        feature.insert("closed_at".to_string(), json!(stamp.clone()));
        if !note_text.is_empty() {
            feature.insert("note".to_string(), json!(note_text.clone()));
        }
        if let Some(por) = &absorbida {
            feature.insert("superseded_by".to_string(), json!(por));
        }
        // Campo PROPIO, distinto de `superseded_by` a proposito (feature #65):
        // `superseded_by` conserva su invariante —siempre resuelve contra este
        // backlog— y la referencia externa, que el arnes NO puede comprobar,
        // vive en su propio campo. Mezclarlas obligaria a que el mismo dato
        // signifique dos cosas segun tenga o no una barra.
        if let Some(r) = &resuelto_en_ref {
            feature.insert("resuelto_en".to_string(), json!(r));
        }
        // Campos OPCIONALES (feature #17): sin declaracion la entrada queda como
        // siempre, asi que las features ya cerradas no se migran ni se tocan.
        if let Some(decl) = &declaracion {
            feature.insert("leccion".to_string(), json!(decl.clase));
            if let Some(motivo) = &decl.motivo {
                feature.insert("leccion_motivo".to_string(), json!(motivo));
            }
        }
    }
    save_features(paths, &data)?;
    // Feature #15: transicion al estado final (o flag Impediment si quedo
    // bloqueada) y comentario con la nota de cierre (AC-8). Un intent emitido no
    // se puede deshacer: por eso sale recien cuando el cierre ocurrio.
    if let Some(feature) = feature_at(&data, idx).as_object() {
        crate::atlassian::emit::on_close(paths, feature, status, Some(&note_text));
    }
    crate::atlassian::push::push_bg(paths);
    // El estado vivo de la feature cerrada desaparece (ya quedo archivado en
    // docs/) y current.md se reescribe como indice de lo que sigue abierto.
    let _ = std::fs::remove_file(paths.current_de(&feature_id));
    crate::progress::escribir_indice(paths, &data)?;
    let leccion_log = match &declaracion {
        Some(decl) => format!(" leccion={}", decl.resumen()),
        None => String::new(),
    };
    log(
        paths,
        &format!("close feature #{feature_id} status={status}{leccion_log} note={note_text}"),
    )?;
    update_memories(
        "close",
        status,
        &format!("feature-{feature_id}"),
        &note_text,
        true,
        &paths.repo_root,
    );
    // Cierra el ciclo de checkpoints DE ESTA feature (AC-10/AC-11).
    let _ = std::fs::remove_file(paths.autocheck_stamp_de(&feature_id));
    let mut msg = format!("Feature #{feature_id} cerrada como {status}.");
    if let Some(rel) = &archived_rel {
        // Feature #63: si la integracion borro el worktree, la ruta que vale es
        // la de la raiz, que es donde el merge dejo el archivo.
        let donde = ruta_del_estado_archivado(
            rel,
            &format!("docs/estado-feature-{feature_id}-{slug}.md"),
            integracion.borra_el_worktree(),
        );
        msg.push_str(&format!(" Estado archivado en {donde}."));
    }
    println!("{msg}");
    // Vuelta al PRD: marca el hito y deja bitacora en el PRD de origen.
    //
    // Corre DESPUES de integrar y solo si integrar salio bien (feature #60): un
    // hito marcado afirma que el trabajo esta en la rama destino, y hasta que el
    // merge no ocurrio, no lo esta. Antes se escribia ANTES y dentro del docs/
    // del worktree, asi que la linea viajaba en la rama y dos cierres en
    // paralelo chocaban en el mismo final de seccion: 7 de 18 se perdieron en la
    // resolucion del conflicto. Nunca reescribe el cuerpo del documento (es del
    // USUARIO) y nunca bloquea el cierre: si no se puede, avisa fuerte y sigue.
    if status == "done"
        && let Some(feature) = feature_at(&data, idx).as_object()
    {
        echo_to_prd(raiz, feature, &stamp);
    }
    if let Some(decl) = &declaracion {
        match &decl.motivo {
            Some(motivo) => println!("  Leccion declarada: ninguna ({motivo})."),
            None => println!(
                "  Leccion declarada: {} ({}).",
                decl.clase,
                lecciones::rel_path(&decl.clase)
            ),
        }
    }
    // Contrato de aprendizaje (feature #18): si la feature cerro como done SIN
    // declarar nada, se le pone delante el metodo. Va al FINAL y a stderr, con
    // el stdout y el exit code ya fijados: emitir el contrato no puede cambiar
    // el resultado de un cierre (AC-10).
    if status == "done" && declaracion.is_none() && lecciones::dir(paths).is_dir() {
        let _ = std::io::stderr().write_all(lecciones::texto_contrato_de_cierre(paths).as_bytes());
    }
    Ok(())
}

/// FASE 1: deja constancia del cierre en el plan de la feature.
///
/// IDEMPOTENTE (feature #62): se escribe ANTES de integrar porque el plan vive
/// en el worktree que el merge borra, asi que un cierre que despues falla la
/// deja escrita. Al reintentar no se duplica: la marca se busca por `status`,
/// no por fecha — el `stamp` cambia en cada corrida y nunca coincidiria.
fn anotar_plan(plan: &std::path::Path, stamp: &str, status: &str, note: &str) -> anyhow::Result<()> {
    if !plan.exists() {
        return Ok(());
    }
    let ya = std::fs::read_to_string(plan).unwrap_or_default();
    if ya
        .lines()
        .any(|l| l.starts_with("Cerrado: ") && l.contains(&format!("status={status}")))
    {
        return Ok(());
    }
    let mut f = std::fs::OpenOptions::new().append(true).open(plan)?;
    write!(f, "\n---\nCerrado: {stamp} - status={status} - {note}\n")?;
    Ok(())
}

/// FASE 1: archiva el estado vivo de la feature en su `docs/`.
///
/// No-destructivo: si `current-<id>.md` tiene estado real escrito a mano, se
/// guarda ANTES de que el cierre lo borre. Se archiva el de ESTA feature; el de
/// las otras activas no se toca — antes habia un unico `current.md` y cerrar una
/// pisaba el de la otra (era el bug de la feature #45).
///
/// Idempotente por naturaleza: sobrescribe. Y NO borra el estado vivo: eso pasa
/// en la FASE 3, para que un cierre que no llego a integrar lo conserve
/// (feature #62).
fn archivar_estado(
    paths: &HarnessPaths,
    feature_id: &str,
    feature_name: &str,
    slug: &str,
    stamp: &str,
    status: &str,
    note_text: &str,
) -> anyhow::Result<Option<String>> {
    std::fs::create_dir_all(&paths.progress)?;
    let vivo = paths.current_de(feature_id);
    if !vivo.exists() {
        return Ok(None);
    }
    let content = std::fs::read_to_string(&vivo)?;
    if content.trim().is_empty() || content.contains("Sin feature activa") {
        return Ok(None);
    }
    std::fs::create_dir_all(&paths.plans)?;
    let archived = paths
        .plans
        .join(format!("estado-feature-{feature_id}-{slug}.md"));
    let mut body = format!("# Estado archivado - Feature #{feature_id}: {feature_name}\n");
    body.push_str(&format!(
        "Cerrada: {stamp} - status={status} - {note_text}\n\n---\n\n"
    ));
    body.push_str(&content);
    std::fs::write(&archived, body)?;
    Ok(Some(
        relpath(&archived, &paths.repo_root)
            .unwrap_or_else(|| archived.clone())
            .to_string_lossy()
            .into_owned(),
    ))
}

/// Que informar sobre la rama y el worktree cuando el cierre NO integra
/// (feature #50). Devuelve `None` cuando no queda ninguno de los dos: prometer
/// que se "conserva" algo que ya no esta es peor que no decir nada.
///
/// Funcion pura: recibe lo que se encontro, no lo consulta.
fn mensaje_conservacion(
    rama: &str,
    status: &str,
    hay_rama: bool,
    hay_worktree: bool,
) -> Option<String> {
    match (hay_rama, hay_worktree) {
        (true, true) => Some(format!(
            "  Rama {rama} y su worktree conservados (el cierre `{status}` no integra)."
        )),
        (true, false) => Some(format!(
            "  Rama {rama} conservada (el cierre `{status}` no integra); su worktree ya no esta."
        )),
        (false, true) => Some(format!(
            "  La rama {rama} ya no esta, pero queda su worktree (el cierre `{status}` no integra)."
        )),
        (false, false) => None,
    }
}

/// Que decirle al usuario cuando su trabajo sin commitear choca con el merge
/// (feature #61). Funcion pura: recibe los archivos, no los busca.
///
/// Nombra cada archivo y da las tres salidas reales, en castellano: el texto
/// crudo de git ("Please commit your changes or stash them") no dice CUAL de
/// las tres queres ni como retomar el cierre.
fn mensaje_de_colision(choques: &[String], destino: &str, feature_id: &str) -> String {
    let lista = choques
        .iter()
        .map(|f| format!("      {f}"))
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        concat!(
            "[GitFlow] No puedo integrar en {} sin pisar trabajo tuyo sin commitear.\n",
            "    Tenes estos archivos modificados en tu checkout y el merge tambien los cambia:\n",
            "{}\n",
            "    NO toque nada: la rama no se movio y la feature no se commiteo.\n",
            "    Son TUS cambios, asi que elegis vos:\n",
            "      git add -A && git commit      # los queres conservar\n",
            "      git stash                     # los queres guardar para despues\n",
            "      git checkout -- <archivo>     # no te interesan (DESCARTA lo no commiteado)\n",
            "    Y despues volve a correr el cierre:\n",
            "      sh harness_cli close --feature {} --status done --to {}"
        ),
        destino, lista, feature_id, destino
    )
}

/// Lo que el cierre va a hacer con git, ya decidido y VALIDADO.
///
/// Partir la integracion en decidir y ejecutar (feature #62) es lo que permite
/// negarse antes de escribir una sola linea de estado: lo que puede fallar por
/// algo previsible —falta `--to`, colision con trabajo sin commitear— se
/// descubre construyendo este plan, no ejecutandolo.
enum PlanDeIntegracion {
    /// No hay nada que integrar: sin rama propia (modo clasico o repo sin git),
    /// o un estado que no integra. `conservacion` es lo que hay que informar.
    Nada { conservacion: Option<String> },
    /// Integrar `rama` en `destino`, con todo ya validado.
    Integrar {
        principal: std::path::PathBuf,
        rama: String,
        destino: String,
        worktree: Option<std::path::PathBuf>,
    },
}

impl PlanDeIntegracion {
    /// True si al ejecutarse este plan el worktree de la feature deja de
    /// existir. Lo usa el mensaje del cierre para no nombrar una ruta que el
    /// propio cierre acaba de borrar (feature #63).
    fn borra_el_worktree(&self) -> bool {
        matches!(self, Self::Integrar { worktree: Some(_), .. })
    }
}

/// Donde esta el estado archivado PARA EL USUARIO, que no siempre es donde el
/// cierre lo escribio.
///
/// Se escribe en el `docs/` del worktree de la feature, y si el cierre integra,
/// ese worktree se borra: la ruta real deja de existir en el mismo comando que
/// la imprime. Despues del merge el archivo vive en la RAIZ, en su ruta
/// canonica. Es el mismo defecto que la #92 arreglo para los punteros del PRD,
/// sobreviviendo en un mensaje de consola.
///
/// Funcion PURA: recibe si el worktree va a desaparecer, no lo averigua.
fn ruta_del_estado_archivado(rel_real: &str, canonica: &str, borra_el_worktree: bool) -> String {
    if borra_el_worktree {
        canonica.to_string()
    } else {
        rel_real.to_string()
    }
}

/// FASE 0: decide la integracion GitFlow del cierre y la valida (feature #47 /
/// AC-14..AC-21). NO escribe nada.
///
/// Solo `done` integra: `blocked`, `pending` y `superseded` conservan la rama y
/// el worktree para poder retomar. El arnes NO elige la rama destino — se niega
/// sin `--to` y le ordena al agente preguntarle al USUARIO (decision OBS-1).
fn planificar_integracion(
    paths: &HarnessPaths,
    data: &Value,
    idx: usize,
    status: &str,
    to: Option<&str>,
    feature_id: &str,
) -> anyhow::Result<PlanDeIntegracion> {
    let (rama, worktree) = {
        let Some(feature) = feature_at(data, idx).as_object() else {
            return Ok(PlanDeIntegracion::Nada { conservacion: None });
        };
        (
            feature.get("branch").and_then(Value::as_str).map(str::to_string),
            feature
                .get("worktree")
                .and_then(Value::as_str)
                .map(std::path::PathBuf::from),
        )
    };
    // Sin rama propia no hay nada que integrar (modo clasico o repo sin git).
    let Some(rama) = rama else {
        return Ok(PlanDeIntegracion::Nada { conservacion: None });
    };
    if status != "done" {
        // Feature #50: se mira el repo ANTES de afirmar. El backlog dice que la
        // feature tuvo rama y worktree; que sigan existiendo es otra cosa (el
        // usuario pudo haberlos borrado a mano).
        let hay_rama = crate::git::repo_principal(&paths.repo_root)
            .is_some_and(|principal| crate::git::rama_existe(&principal, &rama));
        let hay_worktree = worktree.as_ref().is_some_and(|wt| wt.is_dir());
        return Ok(PlanDeIntegracion::Nada {
            conservacion: mensaje_conservacion(&rama, status, hay_rama, hay_worktree),
        });
    }
    let Some(principal) = crate::git::repo_principal(&paths.repo_root) else {
        return Ok(PlanDeIntegracion::Nada { conservacion: None });
    };

    // AC-14: la rama destino la decide el USUARIO, no el arnes.
    let Some(destino) = to else {
        return Err(Exit {
            code: 2,
            message: Some(format!(
                concat!(
                    "[GitFlow] La feature #{} no se puede cerrar todavia: falta decir A QUE RAMA se integra.\n",
                    "    PREGUNTALE AL USUARIO a cual va (develop, release/..., main) y despues:\n",
                    "      sh harness_cli close --feature {} --status done --to <rama>\n",
                    "    Ramas disponibles: {}"
                ),
                feature_id,
                feature_id,
                crate::git::ramas(&principal).join(", ")
            )),
        }
        .into());
    };

    // Feature #61: si el merge fuese a pisar algo que el USUARIO tiene sin
    // commitear, se dice ACA — antes de escribir nada — para que el repo quede
    // exactamente como estaba. Es el unico caso que no se puede resolver sin
    // decidir por el, y el arnes no elige entre su merge y el trabajo ajeno.
    let choques = crate::git::colisiones(&principal, destino, &rama, worktree.as_deref());
    if !choques.is_empty() {
        // code 2 como el resto de los gates del cierre (spec, verify, docs,
        // --to): esto BLOQUEA, no es un error cualquiera.
        return Err(Exit {
            code: 2,
            message: Some(mensaje_de_colision(&choques, destino, feature_id)),
        }
        .into());
    }

    Ok(PlanDeIntegracion::Integrar {
        principal,
        rama,
        destino: destino.to_string(),
        worktree,
    })
}

/// FASE 2: ejecuta el plan de integracion. Lo unico que puede fallar aca es un
/// conflicto de merge REAL, que no se puede saber sin intentarlo.
fn ejecutar_integracion(plan: &PlanDeIntegracion, feature_id: &str) -> anyhow::Result<()> {
    let PlanDeIntegracion::Integrar {
        principal,
        rama,
        destino,
        worktree,
    } = plan
    else {
        if let PlanDeIntegracion::Nada {
            conservacion: Some(linea),
        } = plan
        {
            println!("{linea}");
        }
        return Ok(());
    };

    println!("[GitFlow] integrando {rama} -> {destino}");
    // El trabajo de la feature vive en su worktree: si quedo algo sin
    // commitear, se commitea AHI (nunca en el checkout principal) para que el
    // merge se lo lleve. Sin trailers de IA (AC-16).
    if let Some(wt) = worktree.as_ref().filter(|w| w.is_dir()) {
        match crate::git::commit_todo(wt, &format!("chore(harness): cierre de la feature #{feature_id}")) {
            Ok(true) => println!("  cambios del worktree commiteados en {rama}"),
            Ok(false) => {}
            Err(err) => println!("  [i] no pude commitear el worktree: {err:#}"),
        }
    }
    if let Err(err) = crate::git::merge_en(principal, destino, rama) {
        // AC-18: el merge se abortó; nada quedó a medias.
        return Err(Exit::msg(format!(
            "[GitFlow] no se pudo integrar {rama} en {destino}: {err:#}\n    El merge se aborto y el repo quedo como estaba. Resolvelo a mano y volve a correr el cierre con --to.\n    La feature NO quedo marcada como cerrada: el backlog dice la verdad."
        ))
        .into());
    }
    println!("  merge hecho (sin trailers de IA)");

    match crate::git::push(principal, destino) {
        Ok(()) => println!("  {destino} publicada en origin"),
        Err(err) => println!("  [i] merge local hecho, pero no pude publicar {destino}: {err:#}"),
    }
    // AC-19: se borra el worktree, se conserva la rama.
    if let Some(wt) = worktree {
        match crate::git::borrar_worktree(principal, wt) {
            Ok(()) => println!("  worktree {} borrado (la rama {rama} se conserva)", wt.display()),
            Err(err) => println!("  [i] no pude borrar el worktree {}: {err:#}", wt.display()),
        }
    }
    Ok(())
}

/// La raiz donde vive el PRD: el checkout PRINCIPAL, sin importar desde donde
/// se haya invocado el comando (feature #60). Sin git, la raiz de siempre.
fn raiz_del_prd(paths: &HarnessPaths) -> std::path::PathBuf {
    crate::git::repo_principal(&paths.repo_root).unwrap_or_else(|| paths.repo_root.clone())
}

/// Los punteros que la bitacora podria llevar, cada uno resuelto contra la
/// raiz. Devolver `existe` en vez de filtrar aca es lo que deja a
/// `prd::decidir_vuelta` pura y testeable sin filesystem.
fn candidatos_de_bitacora(
    raiz: &std::path::Path,
    feature: &serde_json::Map<String, Value>,
    fid: &str,
) -> Vec<prd::Candidato> {
    // La forma CANONICA post-merge: relativa a la raiz. No se calcula contra el
    // worktree — ese arbol lo borra el propio cierre unos segundos despues.
    let spec_rel = crate::spec::spec_rel_raiz(feature);
    let impl_rel = format!("docs/impl-{fid}.md");
    vec![
        prd::Candidato::nuevo("spec", &spec_rel, raiz.join(&spec_rel).is_file()),
        prd::Candidato::nuevo("impl", &impl_rel, raiz.join(&impl_rel).is_file()),
    ]
}

/// Marca el hito y deja bitacora en el PRD de origen de la feature, en la RAIZ.
/// Best-effort por diseno: un PRD ausente o ilegible NO puede impedir cerrar una
/// feature. Pero best-effort no es mudo (feature #60): lo que no se pudo
/// escribir se dice por stderr y queda como pendiente que `prd doctor` reporta.
fn echo_to_prd(paths: &HarnessPaths, feature: &serde_json::Map<String, Value>, stamp: &str) {
    let raiz = raiz_del_prd(paths);
    let slug = prd::normalize_parent(feature.get("prd").and_then(Value::as_str));
    let file = prd::file_en_raiz(&raiz, &slug);
    let rel = prd::rel_path(&slug);
    let fid = py_str(feature.get("id"));
    if !file.is_file() {
        aviso_pendiente(&fid, &format!("falta {rel} en la raiz del repo"));
        return;
    }
    let name = feature
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let date = stamp.get(..10).unwrap_or(stamp);
    let plan = prd::decidir_vuelta(&fid, name, date, &candidatos_de_bitacora(&raiz, feature, &fid));
    // Un puntero descartado se dice: omitirlo en silencio seria el mismo bug con
    // otra cara.
    for descarte in &plan.descartes {
        println!(
            "  [i] sin puntero {}: {} ({})",
            descarte.etiqueta, descarte.rel, descarte.motivo
        );
    }
    match prd::aplicar_vuelta(&file, &plan) {
        Ok(echo) if echo.milestone_marked || echo.logged => {
            let what = if echo.milestone_marked {
                "hito marcado done + bitacora"
            } else {
                "bitacora"
            };
            println!("PRD actualizado ({what}): {rel}");
            // El PRD es una ruta protegida (feature #26) y esta escritura la
            // hizo el ARNES, no el agente: se registra para que la red de
            // seguridad no la reporte como violacion en el turno siguiente.
            crate::commands::rutas::registrar_escritura_del_arnes(paths, &rel);
        }
        Ok(_) => println!("[i] El PRD {rel} ya tenia registrada esta feature."),
        Err(err) => aviso_pendiente(&fid, &format!("no se pudo escribir {rel}: {err}")),
    }
}

/// Avisa que la vuelta al PRD quedo pendiente. Va a STDERR y al final del
/// cierre, con el exit code ya fijado: avisar no puede cambiar el resultado de
/// un cierre (mismo contrato que el nudge de lecciones, AC-8).
fn aviso_pendiente(fid: &str, motivo: &str) {
    let _ = std::io::stderr().write_all(
        format!(
            "\n[!] La feature #{fid} cerro como done pero NO quedo registrada en su PRD: {motivo}.\n    \
             El cierre es valido; el que queda pendiente es el PRD. Cuando lo resuelvas:\n      \
             sh harness_cli prd doctor            # que falta\n      \
             sh harness_cli prd doctor --reparar  # escribirlo\n"
        )
        .as_bytes(),
    );
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;

    /// AC-7 (feature #63): la funcion que decide donde decir que quedo el
    /// estado archivado es PURA. Recibe si el worktree va a desaparecer; no lo
    /// averigua, no mira el disco.
    #[test]
    fn ruta_del_estado_archivado_es_pura() {
        let dir = tempfile::tempdir().unwrap();
        let antes: Vec<_> = std::fs::read_dir(dir.path()).unwrap().collect();

        let real = "../harness_process-wt/59-cmd-smoke/docs/estado-feature-59-cmd-smoke.md";
        let canonica = "docs/estado-feature-59-cmd-smoke.md";

        // El cierre integro: ese worktree ya no existe cuando se imprime.
        assert_eq!(
            ruta_del_estado_archivado(real, canonica, true),
            canonica,
            "tras integrar vale la ruta de la raiz"
        );
        // No integro: el archivo sigue donde se escribio.
        assert_eq!(
            ruta_del_estado_archivado(real, canonica, false),
            real,
            "sin integrar vale la ruta real"
        );
        // Y ninguna de las dos toco el disco.
        let despues: Vec<_> = std::fs::read_dir(dir.path()).unwrap().collect();
        assert_eq!(antes.len(), despues.len());

        // La ruta que devuelve tras integrar nunca escapa de la raiz: es
        // copiable y pegable, que era todo el punto.
        assert!(!ruta_del_estado_archivado(real, canonica, true).contains(".."));
    }

    /// AC-7 (feature #62): anotar el plan es idempotente.
    ///
    /// La anotacion se escribe ANTES de integrar (el plan vive en el worktree
    /// que el merge borra), asi que un cierre que despues falla la deja
    /// escrita. El reintento no la puede duplicar. La marca se busca por
    /// `status` y no por fecha: el `stamp` cambia en cada corrida y una
    /// comparacion por linea completa nunca coincidiria.
    #[test]
    fn anotar_plan_es_idempotente() {
        let dir = tempfile::tempdir().unwrap();
        let plan = dir.path().join("plan-feature-62-x.md");
        std::fs::write(&plan, "# Plan\n\nLo que hay que hacer.\n").unwrap();

        anotar_plan(&plan, "2026-08-27T10:00:00Z", "done", "primera").unwrap();
        // Segundo intento, con OTRA fecha y otra nota: es el reintento.
        anotar_plan(&plan, "2026-08-27T11:30:00Z", "done", "segunda").unwrap();

        let texto = std::fs::read_to_string(&plan).unwrap();
        assert_eq!(
            texto.matches("Cerrado: ").count(),
            1,
            "una sola anotacion por mas que se reintente:\n{texto}"
        );
        assert!(texto.contains("2026-08-27T10:00:00Z"), "vale la del primer intento");
        assert!(!texto.contains("11:30:00Z"));
        assert!(texto.contains("# Plan"), "el cuerpo del plan intacto");

        // Otro estado SI se anota: cerrar como pending y despues como done son
        // dos hechos distintos.
        anotar_plan(&plan, "2026-08-28T09:00:00Z", "pending", "aparcada").unwrap();
        let texto = std::fs::read_to_string(&plan).unwrap();
        assert_eq!(texto.matches("Cerrado: ").count(), 2);
        assert!(texto.contains("status=pending"));

        // Un plan que no existe no es un error: no todas las features lo tienen.
        anotar_plan(&dir.path().join("no-existe.md"), "x", "done", "").unwrap();
    }

    /// AC-4 (feature #61): el mensaje nombra cada archivo que choca y da las
    /// tres salidas reales. El texto crudo de git ("Please commit your changes
    /// or stash them") no dice cual de las tres queres ni como retomar.
    #[test]
    fn mensaje_de_colision_nombra_archivos_y_remedio() {
        let choques = vec![
            "docs/prd/PRD-master.md".to_string(),
            "docs/lecciones/promesas-estructurales-vs-disciplina.md".to_string(),
        ];
        let msg = mensaje_de_colision(&choques, "main", "61");

        // Cada archivo, por nombre.
        for archivo in &choques {
            assert!(msg.contains(archivo.as_str()), "falta {archivo} en:\n{msg}");
        }
        // Las tres salidas, y que son del USUARIO.
        assert!(msg.contains("git add -A && git commit"));
        assert!(msg.contains("git stash"));
        assert!(msg.contains("git checkout -- <archivo>"));
        assert!(msg.contains("DESCARTA"), "la opcion destructiva se marca como tal");
        // Que NO paso nada.
        assert!(msg.contains("NO toque nada"));
        assert!(msg.contains("la rama no se movio"));
        // Y como retomar, con la feature y la rama de verdad.
        assert!(msg.contains("close --feature 61 --status done --to main"));
    }

    /// Feature #50: la tabla completa de lo que el cierre puede encontrarse.
    /// El caso que motivo la feature es el ultimo: borrar la rama y el worktree
    /// a mano y cerrar despues, que antes respondia "conservada" sobre las dos.
    #[test]
    fn mensaje_conservacion_should_only_claim_what_exists() {
        // AC-1: estan los dos.
        let ambos = mensaje_conservacion("feature/50-x", "pending", true, true);
        let Some(ambos) = ambos else {
            panic!("con rama y worktree tiene que informar");
        };
        assert!(ambos.contains("feature/50-x"));
        assert!(ambos.contains("conservados"));
        assert!(ambos.contains("pending"), "nombra el estado que no integra");

        // AC-2: quedo la rama, no el worktree.
        let Some(solo_rama) = mensaje_conservacion("feature/50-x", "blocked", true, false) else {
            panic!("con rama tiene que informar");
        };
        assert!(solo_rama.contains("conservada"));
        assert!(solo_rama.contains("worktree ya no esta"));

        // AC-3: quedo el worktree, no la rama.
        let Some(solo_wt) = mensaje_conservacion("feature/50-x", "superseded", false, true) else {
            panic!("con worktree tiene que informar");
        };
        assert!(solo_wt.contains("rama feature/50-x ya no esta"));
        assert!(solo_wt.contains("queda su worktree"));

        // AC-4: no queda nada -> silencio, no una promesa vacia.
        assert!(
            mensaje_conservacion("feature/50-x", "superseded", false, false).is_none(),
            "sin rama ni worktree no hay nada que informar"
        );
    }
}

#[cfg(test)]
mod tests_aguas_arriba {
    use super::*;

    #[test]
    fn forma_de_la_referencia_externa() {
        // Se comprueba la FORMA, nunca la existencia: el repo de al lado no se
        // abre. Lo que la forma tiene que separar es una referencia de un typo
        // suelto, no una referencia real de una inventada — eso el arnes no lo
        // puede saber y no finge que si.
        for ok in [
            "harness_process/feature-60",
            "realestate/feature-1",
            "a/feature-999",
            "  harness_process/feature-60  ",
        ] {
            assert!(forma_de_referencia_externa(ok), "deberia aceptar: {ok}");
        }
        for mal in [
            "",
            "   ",
            "60",                        // sin proyecto
            "harness_process#60",        // la sintaxis de GitHub, no la del arnes
            "harness_process/60",        // sin el prefijo feature-
            "/feature-60",               // sin proyecto
            "harness_process/feature-",  // sin id
            "harness_process/feature-x", // id no numerico
            "con espacio/feature-1",
        ] {
            assert!(!forma_de_referencia_externa(mal), "deberia rechazar: {mal:?}");
        }
    }

    #[test]
    fn todos_los_estados_tienen_su_rama() {
        // AC-9. La #37 pago este bug una vez: `superseded` caia en el brazo `_`
        // de la tabla de Jira y MOVIA el ticket a To Do, o sea lo reabria. Un
        // estado nuevo sin rama explicita nace con ese mismo defecto, y el
        // sintoma no se ve hasta que alguien mira el tablero.
        //
        // La primera version de este test asertaba que cada estado no era la
        // cadena vacia y que `AGUAS_ARRIBA == "resuelto-aguas-arriba"` —una
        // constante igual a su propio literal—. O sea: NO PODIA FALLAR por la
        // razon que el AC declara. Se descubrio en la prueba del rojo: borrar la
        // rama de produccion de Atlassian lo dejaba verde.
        //
        // Ahora la tabla es la fuente y se recorre contra cada consumidor. Un
        // estado nuevo que no se agregue aca no compila; uno que se agregue sin
        // decidir que hace cada consumidor, falla.
        use crate::atlassian::emit::{Efecto, efecto_de};
        let tabla: [(&str, Efecto, bool, bool); 5] = [
            //  estado        Atlassian            ¿cuenta en el avance?  ¿bucket propio?
            ("done", Efecto::ATerminado, true, true),
            ("blocked", Efecto::Impedimento, true, true),
            ("pending", Efecto::ALaCola, true, true),
            (SUPERSEDED, Efecto::NoTocar, false, true),
            (AGUAS_ARRIBA, Efecto::NoTocar, false, true),
        ];
        assert_eq!(
            tabla.len(),
            ESTADOS_DE_CIERRE.len(),
            "se agrego un estado de cierre sin decidir que hace cada consumidor con el"
        );
        for (estado, efecto, cuenta, bucket) in tabla {
            assert!(
                ESTADOS_DE_CIERRE.contains(&estado),
                "{estado}: el CLI no lo acepta"
            );
            assert_eq!(efecto_de(estado), efecto, "{estado}: rama de Atlassian");
            assert_eq!(
                crate::prd::cuenta_en_el_avance(Some(estado)),
                cuenta,
                "{estado}: conteo de avance del PRD"
            );
            assert_eq!(
                crate::commands::status::ESTADOS_CON_BUCKET.contains(&estado),
                bucket,
                "{estado}: bucket de la cabecera de status"
            );
        }
        // Y la comprobacion de que la tabla mide algo: un estado inventado cae
        // en el brazo por defecto de TODOS, que es justo el comportamiento
        // peligroso del que hay que salvarse explicitamente.
        let inventado = "estado-que-no-existe";
        assert!(!ESTADOS_DE_CIERRE.contains(&inventado));
        assert_eq!(efecto_de(inventado), Efecto::ALaCola, "reabriria el ticket");
        assert!(crate::prd::cuenta_en_el_avance(Some(inventado)));
        assert!(!crate::commands::status::ESTADOS_CON_BUCKET.contains(&inventado));
    }

    #[test]
    fn cierre_sin_io_de_red() {
        // AC-10, comprobacion NEGATIVA: el modulo no puede resolver una
        // referencia externa, ni queriendo. Si algun dia alguien agrega una
        // validacion que abra el otro repo, este test lo obliga a pasar por el
        // spec: cerrar no puede depender de que otro repo este clonado en esta
        // maquina (la leccion de la herramienta externa ausente).
        // Se mira SOLO el codigo de produccion: `include_str!` trae el archivo
        // entero, y en la primera corrida este test se encontro a si mismo (su
        // propia lista de prohibidos matcheaba). Un test que se mide a si mismo
        // da rojo sin que haya nada roto, que es tan inutil como el verde falso.
        let fuente = include_str!("close.rs");
        let produccion = fuente.split("#[cfg(test)]").next().unwrap_or(fuente);
        for prohibido in ["reqwest", "ureq", "TcpStream", "std::net"] {
            assert!(
                !produccion.contains(prohibido),
                "close.rs no puede hacer I/O de red: encontrado {prohibido}"
            );
        }
        // Y la forma se valida sin tocar el filesystem: una referencia a un repo
        // que no existe en esta maquina es igual de valida que una que si.
        assert!(forma_de_referencia_externa("no-existe-en-ningun-lado/feature-999"));
    }
}
