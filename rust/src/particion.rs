//! Feature #84: `leccion partir`. La parte mecanica del paso 3 de la guia:
//! mover a `docs/lecciones/<clase>/referencias/` las secciones que cuentan UNA
//! feature o sesion, dejar un puntero de una linea, y no reescribir nada.
//!
//! Medido sobre las cuatro lecciones de realestate del 2026-09-08 (2361, 398,
//! 382 y 275 lineas): los titulos por feature vienen en cuatro formas
//! (`(feature #N)`, `(#N, fecha)`, `feature #N (fecha)`, `(fecha, front #N)`,
//! `Patch #N:`), y ninguna de las cuatro baja del tope solo con lo mecanico.
//! Por eso el comando INFORMA primero, dice cuanto falta y con que secciones
//! se podria seguir, y `--seccion` deja elegir sin reescribir.
//!
//! El informe y la aplicacion reescriben el cuerpo con LA MISMA funcion
//! (`reescribir`): lo que el informe dice que quedaria es lo que queda,
//! punteros e indice incluidos (hallazgo de la revision adversarial).
//!
//! Solo `aplicar` toca el filesystem; el resto es puro y se prueba sin disco.

use std::path::{Path, PathBuf};

use anyhow::Context;

use crate::curador;
use crate::exit::Exit;
use crate::lecciones::{self, Leccion};
use crate::paths::HarnessPaths;

/// Las secciones de la CLASE: nunca se mueven (OBS-3 de la #84). Se comparan
/// sin acentos ni mayusculas y por prefijo, asi `Referencias por tema`,
/// `Referencias (el detalle, caso por caso)` y `Procedimiento: cerrar la
/// hipotesis...` cuentan como canonicas. El indice de referencias es canonico
/// porque es donde van los punteros.
const CANONICAS: [&str; 5] = [
    "cuando aplica",
    "procedimiento",
    "pitfalls",
    "verificacion",
    "referencias",
];

const TITULO_DEL_INDICE: &str = "## Referencias (el detalle, caso por caso)";

/// Una seccion `## ` del cuerpo: del titulo (inclusive) a la linea del
/// siguiente `## ` (exclusiva). Los `###` viajan adentro.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Seccion {
    pub titulo: String,
    /// Indice de la linea `## ` en las lineas del cuerpo.
    pub inicio: usize,
    /// Exclusivo: la linea del siguiente `## `, o el total de lineas.
    pub fin: usize,
}

impl Seccion {
    pub fn lineas(&self) -> usize {
        self.fin - self.inicio
    }

    pub fn es_canonica(&self) -> bool {
        es_canonica(&self.titulo)
    }

    pub fn cuenta_una_feature(&self) -> bool {
        cuenta_una_feature(&self.titulo)
    }
}

fn normalizar(titulo: &str) -> String {
    crate::contexto::sin_acentos(titulo).to_lowercase()
}

pub fn es_canonica(titulo: &str) -> bool {
    let t = normalizar(titulo);
    let t = t.trim();
    CANONICAS.iter().any(|c| t.starts_with(c))
}

/// Un titulo que cuenta UNA feature o sesion: trae `#<numero>` o una fecha
/// `YYYY-MM-DD` (OBS-1 de la #84), y no es canonico.
pub fn cuenta_una_feature(titulo: &str) -> bool {
    !es_canonica(titulo) && (tiene_numero_de_feature(titulo) || tiene_fecha(titulo))
}

fn tiene_numero_de_feature(titulo: &str) -> bool {
    let b = titulo.as_bytes();
    b.iter()
        .enumerate()
        .any(|(i, c)| *c == b'#' && b.get(i + 1).is_some_and(u8::is_ascii_digit))
}

fn tiene_fecha(titulo: &str) -> bool {
    let b = titulo.as_bytes();
    if b.len() < 10 {
        return false;
    }
    (0..=b.len() - 10).any(|i| {
        let w = &b[i..i + 10];
        w[0..4].iter().all(u8::is_ascii_digit)
            && w[4] == b'-'
            && w[5..7].iter().all(u8::is_ascii_digit)
            && w[7] == b'-'
            && w[8..10].iter().all(u8::is_ascii_digit)
    })
}

/// Las lineas del cuerpo sin el `\n` final (para que la ultima seccion no
/// cuente una linea fantasma) y sin el `\r` de un archivo CRLF: el cuerpo se
/// vuelve a armar con el fin de linea de la leccion (`con_eol`).
fn lineas_de(body: &str) -> Vec<&str> {
    let mut lineas: Vec<&str> = body.split('\n').map(|l| l.trim_end_matches('\r')).collect();
    if lineas.last().is_some_and(|l| l.is_empty()) {
        lineas.pop();
    }
    lineas
}

/// `texto` esta armado con `\n`; si la leccion es CRLF, sale CRLF entero.
fn con_eol(texto: &str, eol: &str) -> String {
    if eol == "\n" {
        texto.to_string()
    } else {
        texto.replace('\n', eol)
    }
}

/// Indices de las lineas que son titulos `## ` de verdad: un `## ` adentro de
/// un bloque de codigo (```) es texto, no una seccion. Si un bloque nunca se
/// cierra, el documento esta roto y no hay forma de saber que es codigo: se
/// vuelve a leer sin mirar los bloques, antes que arrastrar las canonicas
/// adentro de una seccion que no termina.
fn titulos(lineas: &[&str]) -> Vec<(usize, String)> {
    let (con_bloques, cerrado) = titulos_con_bloques(lineas, true);
    if cerrado {
        con_bloques
    } else {
        titulos_con_bloques(lineas, false).0
    }
}

fn titulos_con_bloques(lineas: &[&str], respetar_bloques: bool) -> (Vec<(usize, String)>, bool) {
    let mut en_codigo = false;
    let mut out = Vec::new();
    for (i, l) in lineas.iter().enumerate() {
        if respetar_bloques && l.trim_start().starts_with("```") {
            en_codigo = !en_codigo;
            continue;
        }
        if en_codigo {
            continue;
        }
        if let Some(t) = l.strip_prefix("## ") {
            out.push((i, t.trim().to_string()));
        }
    }
    (out, !en_codigo)
}

/// Las secciones `## ` del cuerpo, en orden. El preambulo antes del primer
/// `## ` no es una seccion y no se toca; `###` no corta y un `## ` dentro de
/// un bloque de codigo tampoco.
pub fn secciones(body: &str) -> Vec<Seccion> {
    let lineas = lineas_de(body);
    let inicios = titulos(&lineas);
    inicios
        .iter()
        .enumerate()
        .map(|(k, (inicio, titulo))| Seccion {
            titulo: titulo.clone(),
            inicio: *inicio,
            fin: inicios.get(k + 1).map_or(lineas.len(), |(i, _)| *i),
        })
        .collect()
}

/// El plan, puro. `candidatas` en orden de posicion; `sugeridas` son las no
/// canonicas que no se mueven, de mayor a menor, para `--seccion`. `saldo`
/// es lo que la leccion MIDE despues de mover: se calcula reescribiendo el
/// cuerpo igual que `aplicar` (punteros e indice incluidos).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plan {
    pub lineas: usize,
    /// `0` = tope apagado.
    pub tope: usize,
    pub candidatas: Vec<Seccion>,
    pub sugeridas: Vec<Seccion>,
    saldo: usize,
}

impl Plan {
    /// Lineas que quedarian despues de mover las candidatas, con sus punteros
    /// y el indice que haga falta. Sin candidatas, las de hoy.
    pub fn saldo(&self) -> usize {
        self.saldo
    }

    /// Lineas que sobrarian aun despues de mover; `0` = queda bajo el tope, o
    /// no hay tope.
    pub fn falta(&self) -> usize {
        if self.tope == 0 {
            0
        } else {
            self.saldo.saturating_sub(self.tope)
        }
    }

    pub fn sobre_el_tope_hoy(&self) -> bool {
        self.tope > 0 && self.lineas > self.tope
    }
}

/// `lineas` es lo que mide la leccion entera (frontmatter incluido); `body`
/// su cuerpo; `elegidas` las `--seccion` ya resueltas (AC-4), que se suman a
/// las que cuentan una feature (AC-2).
pub fn planificar(
    lineas: usize,
    tope: usize,
    body: &str,
    clase: &str,
    elegidas: &[Seccion],
) -> Plan {
    let secciones = secciones(body);
    let candidatas: Vec<Seccion> = secciones
        .iter()
        .filter(|s| s.cuenta_una_feature() || elegidas.contains(s))
        .cloned()
        .collect();
    let mut sugeridas: Vec<Seccion> = secciones
        .iter()
        .filter(|s| !s.es_canonica() && !candidatas.contains(s))
        .cloned()
        .collect();
    sugeridas.sort_by(|a, b| b.lineas().cmp(&a.lineas()).then(a.inicio.cmp(&b.inicio)));
    let cuerpo = lineas_de(body);
    let saldo = if candidatas.is_empty() {
        lineas
    } else {
        // Un puntero por candidata: el slug no cambia la cuenta.
        let punteros: Vec<String> = candidatas
            .iter()
            .map(|s| puntero(&s.titulo, clase, "x"))
            .collect();
        let fuera_del_cuerpo = lineas.saturating_sub(cuerpo.len());
        fuera_del_cuerpo + reescribir(&cuerpo, &candidatas, clase, &punteros).len()
    };
    Plan {
        lineas,
        tope,
        candidatas,
        sugeridas,
        saldo,
    }
}

/// Resuelve un `--seccion`: titulo exacto, o subcadena unica sin acentos ni
/// mayusculas. Los tres fallos tienen su mensaje: ninguna, varias, canonica
/// (misma forma que `perfil replace`, feature #19).
pub fn resolver<'a>(secciones: &'a [Seccion], pedido: &str) -> Result<&'a Seccion, Exit> {
    let exactas: Vec<&Seccion> = secciones.iter().filter(|s| s.titulo == pedido).collect();
    let elegida = if exactas.len() == 1 {
        exactas[0]
    } else {
        let buscado = normalizar(pedido);
        let parciales: Vec<&Seccion> = secciones
            .iter()
            .filter(|s| normalizar(&s.titulo).contains(&buscado))
            .collect();
        match parciales.len() {
            1 => parciales[0],
            0 => {
                let mut msg =
                    format!("Ninguna seccion de la leccion contiene '{pedido}'. Secciones:");
                for s in secciones {
                    msg.push_str(&format!("\n      - {}", s.titulo));
                }
                return Err(Exit {
                    code: 2,
                    message: Some(msg),
                });
            }
            n => {
                let mut msg =
                    format!("'{pedido}' matchea {n} secciones; usa un fragmento mas especifico:");
                for s in parciales {
                    msg.push_str(&format!("\n      - {}", s.titulo));
                }
                return Err(Exit {
                    code: 2,
                    message: Some(msg),
                });
            }
        }
    };
    if elegida.es_canonica() {
        return Err(Exit {
            code: 2,
            message: Some(format!(
                "'{}' es una seccion de la CLASE ({}): no se mueve a referencias/.\n    \
                 Lo que sobra ahi se parte a mano (guia, paso 3): la clase es lo que queda.",
                elegida.titulo,
                CANONICAS.join(", ")
            )),
        });
    }
    Ok(elegida)
}

/// `El caso del martes (feature #12)` -> `el-caso-del-martes-feature-12`.
/// Sin acentos, `[a-z0-9]` y guiones, 60 caracteres como maximo (la
/// convencion de las referencias que ya existen).
pub fn slug_de_referencia(titulo: &str) -> String {
    let mut s = String::new();
    let mut guion = false;
    for c in normalizar(titulo).chars() {
        if c.is_ascii_alphanumeric() {
            s.push(c);
            guion = false;
        } else if !guion && !s.is_empty() {
            s.push('-');
            guion = true;
        }
    }
    let corto: String = s.chars().take(60).collect();
    let corto = corto.trim_end_matches('-');
    if corto.is_empty() {
        "seccion".to_string()
    } else {
        corto.to_string()
    }
}

/// El slug que no choca ni con un archivo existente ni con uno de esta misma
/// corrida: `-2`, `-3`, ... Un archivo existente que `es_el_mismo` (la misma
/// referencia, escrita por una corrida anterior que fallo antes de guardar la
/// leccion) se reutiliza en vez de duplicarse.
pub fn slug_para(
    dir: &Path,
    base: &str,
    usados: &[String],
    es_el_mismo: &dyn Fn(&Path) -> bool,
) -> String {
    let candidatos = std::iter::once(base.to_string()).chain((2..).map(|n| format!("{base}-{n}")));
    for s in candidatos {
        if usados.contains(&s) {
            continue;
        }
        let archivo = dir.join(format!("{s}.md"));
        if !archivo.exists() || es_el_mismo(&archivo) {
            return s;
        }
    }
    base.to_string()
}

/// `slug_para` sin reutilizacion (solo lo usan los tests).
#[cfg(test)]
pub fn slug_libre(dir: &Path, base: &str, usados: &[String]) -> String {
    slug_para(dir, base, usados, &|_| false)
}

pub fn cabecera_de_referencia(titulo: &str, clase: &str, hoy: &str) -> String {
    format!(
        "# {titulo}\n\n\
         Referencia de la leccion `{clase}`: el caso concreto que sostiene una de sus reglas. \
         Movida aca el {hoy} por `leccion partir` para que la leccion de clase quede dentro del \
         tope de lineas; el texto es el original, sin reescribir.\n\n"
    )
}

pub fn puntero(titulo: &str, clase: &str, slug: &str) -> String {
    format!("- [{titulo}]({clase}/referencias/{slug}.md)")
}

/// El cuerpo de la seccion sin su titulo, sin lineas vacias en los bordes,
/// verbatim adentro (los `###` incluidos).
fn cuerpo_de(lineas: &[&str], s: &Seccion) -> String {
    let mut bloque: &[&str] = &lineas[s.inicio + 1..s.fin];
    while bloque.first().is_some_and(|l| l.trim().is_empty()) {
        bloque = &bloque[1..];
    }
    while bloque.last().is_some_and(|l| l.trim().is_empty()) {
        bloque = &bloque[..bloque.len() - 1];
    }
    let mut cuerpo = bloque.join("\n");
    cuerpo.push('\n');
    cuerpo
}

/// El cuerpo sin los bloques de las candidatas y con los punteros puestos:
/// lo que `aplicar` guarda y lo que `planificar` cuenta.
fn reescribir(
    lineas: &[&str],
    candidatas: &[Seccion],
    clase: &str,
    punteros: &[String],
) -> Vec<String> {
    let mut nuevas: Vec<String> = lineas
        .iter()
        .enumerate()
        .filter(|(i, _)| !candidatas.iter().any(|s| *i >= s.inicio && *i < s.fin))
        .map(|(_, l)| (*l).to_string())
        .collect();
    insertar_punteros(&mut nuevas, clase, punteros);
    nuevas
}

/// Deja los punteros en el indice `## Referencias...` (el primero que haya,
/// al final de su contenido) o crea el indice al final de la leccion. No
/// toca lo que ya habia: las lineas en blanco que cierran la ultima seccion
/// son de esa seccion y quedan.
fn insertar_punteros(lineas: &mut Vec<String>, clase: &str, punteros: &[String]) {
    let prestadas: Vec<&str> = lineas.iter().map(String::as_str).collect();
    let cabeceras = titulos(&prestadas);
    let indice = cabeceras
        .iter()
        .position(|(_, t)| normalizar(t).trim().starts_with("referencias"));
    match indice {
        Some(k) => {
            let cabecera = cabeceras[k].0;
            let fin = cabeceras.get(k + 1).map_or(lineas.len(), |(i, _)| *i);
            let ultima_con_texto = (cabecera..fin)
                .rev()
                .find(|i| !lineas[*i].trim().is_empty())
                .unwrap_or(cabecera);
            let mut nuevas: Vec<String> = Vec::new();
            if ultima_con_texto == cabecera {
                nuevas.push(String::new());
            }
            nuevas.extend(punteros.iter().cloned());
            let pos = ultima_con_texto + 1;
            // Si despues viene otra seccion pegada, que quede la linea en blanco.
            if pos < lineas.len() && !lineas[pos].trim().is_empty() {
                nuevas.push(String::new());
            }
            lineas.splice(pos..pos, nuevas);
        }
        None => {
            if lineas.last().is_some_and(|l| !l.trim().is_empty()) {
                lineas.push(String::new());
            }
            lineas.push(TITULO_DEL_INDICE.to_string());
            lineas.push(String::new());
            lineas.push(format!(
                "Movidas a `{clase}/referencias/` (`leccion partir`). Cada una es el caso que sostiene una regla de arriba: se leen cuando hace falta el detalle, no para entender la clase."
            ));
            lineas.push(String::new());
            lineas.extend(punteros.iter().cloned());
        }
    }
}

/// Id del respaldo: `<ts>-partir-<clase>`, y `-2`, `-3`... si dos corridas
/// caen en el mismo segundo (un respaldo que se pisa no es un respaldo).
pub fn id_de_respaldo(paths: &HarnessPaths, ts: &str, clase: &str) -> String {
    let base = format!("{ts}-partir-{clase}");
    let dir = curador::backups_dir(paths);
    std::iter::once(base.clone())
        .chain((2..).map(|n| format!("{base}-{n}")))
        .find(|id| !dir.join(id).exists())
        .unwrap_or(base)
}

pub struct Movida {
    pub titulo: String,
    pub destino: PathBuf,
    pub lineas: usize,
}

pub struct Resultado {
    pub respaldo: PathBuf,
    pub movidas: Vec<Movida>,
    /// Lineas de la leccion despues de mover.
    pub lineas: usize,
}

/// Ejecuta el plan: respalda (el respaldo del curador, asi `lecciones
/// rollback` lo deshace), escribe cada referencia, quita los bloques, deja los
/// punteros, fecha `ultima_actualizacion` y guarda atomico. Exige candidatas:
/// `partir` no la llama sin ellas.
///
/// # Errors
///
/// Falla si no se puede respaldar, crear `referencias/` o escribir. Lo que ya
/// se escribio queda, y el error nombra el respaldo y como volver. Una
/// referencia que quedo escrita por una corrida fallida se reutiliza en la
/// siguiente en vez de duplicarse con `-2`.
pub fn aplicar(
    paths: &HarnessPaths,
    leccion: &mut Leccion,
    plan: &Plan,
    hoy: &str,
    ts: &str,
) -> anyhow::Result<Resultado> {
    anyhow::ensure!(
        !plan.candidatas.is_empty(),
        "nada que mover: el plan no tiene candidatas"
    );
    let clase = leccion.nombre.clone();
    let id = id_de_respaldo(paths, ts, &clase);
    let respaldo = curador::respaldar(paths, &id, &format!("leccion partir {clase}"))
        .context("no se pudo respaldar docs/lecciones/ antes de partir")?;
    escribir(paths, leccion, plan, hoy, &clase, respaldo.clone()).with_context(|| {
        format!(
            "lo ya escrito queda; el respaldo esta en {} (sh harness_cli lecciones rollback --id {id} lo deshace)",
            respaldo.display()
        )
    })
}

fn escribir(
    paths: &HarnessPaths,
    leccion: &mut Leccion,
    plan: &Plan,
    hoy: &str,
    clase: &str,
    respaldo: PathBuf,
) -> anyhow::Result<Resultado> {
    let dir = lecciones::dir(paths).join(clase).join("referencias");
    std::fs::create_dir_all(&dir).with_context(|| format!("no se pudo crear {}", dir.display()))?;
    let eol = leccion.fm.eol();
    let cuerpo_viejo = leccion.body.clone();
    let lineas = lineas_de(&cuerpo_viejo);
    let mut usados: Vec<String> = Vec::new();
    let mut movidas = Vec::new();
    let mut punteros = Vec::new();
    for s in &plan.candidatas {
        let cuerpo = cuerpo_de(&lineas, s);
        let texto = con_eol(
            &format!("{}{cuerpo}", cabecera_de_referencia(&s.titulo, clase, hoy)),
            eol,
        );
        let cola = con_eol(&cuerpo, eol);
        let cabeza = con_eol(&format!("# {}\n", s.titulo), eol);
        let es_el_mismo = |archivo: &Path| {
            std::fs::read_to_string(archivo)
                .is_ok_and(|t| t.starts_with(&cabeza) && t.ends_with(&cola))
        };
        let slug = slug_para(&dir, &slug_de_referencia(&s.titulo), &usados, &es_el_mismo);
        usados.push(slug.clone());
        let destino = dir.join(format!("{slug}.md"));
        crate::features::write_text_atomic(&destino, &texto)
            .with_context(|| format!("no se pudo escribir {}", destino.display()))?;
        punteros.push(puntero(&s.titulo, clase, &slug));
        movidas.push(Movida {
            titulo: s.titulo.clone(),
            destino,
            lineas: s.lineas(),
        });
    }
    let nuevas = reescribir(&lineas, &plan.candidatas, clase, &punteros);
    let mut body = nuevas.join("\n");
    body.push('\n');
    leccion.body = con_eol(&body, eol);
    leccion.fm.set("ultima_actualizacion", hoy);
    leccion.save()?;
    Ok(Resultado {
        respaldo,
        movidas,
        lineas: leccion.lineas(),
    })
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;

    /// Los titulos REALES de las cuatro lecciones de realestate sobre el tope
    /// (2026-09-08), mas los de este repo: el criterio se prueba contra datos,
    /// no contra ejemplos inventados (leccion probar-contra-datos-reales).
    const POR_FEATURE: [&str; 8] = [
        "Dos formas nuevas, las dos de la feature #100 (2026-08-28)",
        "Tres formas nuevas, las tres de la feature #114 (2026-08-30)",
        "El instrumento no toma como entrada el archivo que acabas de tocar (#115, 2026-08-30)",
        "Un exit code 0 que tapa un fallo total, y el arreglo que era peor (#125, 2026-09-04)",
        "Cuando el AC tiene varios Then, la DECISION es la que manda (2026-09-07, front #131)",
        "Patch #100: SECURITY DEFINER no salva si el dueno perdio BYPASSRLS",
        "El rojo que fallo antes de llegar al aserto (feature #78)",
        "Incidente del 2026-09-06: el checkout borrado",
    ];
    const DE_LA_CLASE: [&str; 9] = [
        "Cuando aplica",
        "Procedimiento",
        "Procedimiento: cerrar la hipotesis ANTES de escribir el AC",
        "Pitfalls",
        "Verificacion",
        "Verificación",
        "Referencias por tema",
        "Referencias (el detalle, caso por caso)",
        "Pitfalls de la feature #12",
    ];
    const SIN_FEATURE: [&str; 3] = [
        "Las CINCO caras del mismo defecto",
        "El lector que no encuentra el formato: no falla, CALLA",
        "El comando que no puede FALLAR: el script que no existe se lee como verde",
    ];

    #[test]
    fn secciones_por_feature_should_see_every_real_title_form() {
        for t in POR_FEATURE {
            assert!(cuenta_una_feature(t), "no ve: {t}");
        }
    }

    #[test]
    fn secciones_por_feature_should_never_take_a_canonical_section() {
        for t in DE_LA_CLASE {
            assert!(es_canonica(t), "no es canonica: {t}");
            assert!(!cuenta_una_feature(t), "se moveria: {t}");
        }
    }

    #[test]
    fn secciones_por_feature_should_leave_class_text_without_number_or_date() {
        for t in SIN_FEATURE {
            assert!(!es_canonica(t), "{t}");
            assert!(!cuenta_una_feature(t), "{t}");
        }
        // `#` sin numero y numeros sin `#` no alcanzan.
        assert!(!cuenta_una_feature("El item # de la lista con 2026 cosas"));
        assert!(!cuenta_una_feature("Los 12 pasos"));
    }

    const CUERPO: &str =
        "intro\n\n## Cuando aplica\n\nclase\n\n## Caso (feature #1)\n\nuno\n### sub\ndos\n\n## Pitfalls\n\np\n";

    #[test]
    fn secciones_should_split_on_h2_only_and_keep_h3_inside() {
        let s = secciones(CUERPO);
        let titulos: Vec<&str> = s.iter().map(|x| x.titulo.as_str()).collect();
        assert_eq!(
            titulos,
            vec!["Cuando aplica", "Caso (feature #1)", "Pitfalls"]
        );
        assert_eq!((s[1].inicio, s[1].fin, s[1].lineas()), (6, 12, 6));
        // La ultima no cuenta una linea fantasma por el `\n` final.
        assert_eq!(s[2].lineas(), 3);
        // Un cuerpo CRLF mide lo mismo y los titulos salen limpios.
        let crlf = CUERPO.replace('\n', "\r\n");
        assert_eq!(secciones(&crlf), s);
    }

    #[test]
    fn secciones_should_ignore_a_h2_inside_a_code_fence() {
        let cuerpo =
            "## Cuando aplica\n\n```md\n## Esto es un ejemplo (feature #9)\n```\n\n## Caso (feature #1)\n\nx\n";
        let s = secciones(cuerpo);
        let titulos: Vec<&str> = s.iter().map(|x| x.titulo.as_str()).collect();
        assert_eq!(titulos, vec!["Cuando aplica", "Caso (feature #1)"]);
        assert_eq!(
            s[0].lineas(),
            6,
            "el bloque de codigo queda adentro de la seccion"
        );
        // Y el indice de referencias tampoco se confunde con un ejemplo.
        let mut lineas: Vec<String> = [
            "## Cuando aplica",
            "",
            "```",
            "## Referencias falsas",
            "```",
            "",
            "## Referencias",
            "",
            "- [a](x/referencias/a.md)",
        ]
        .iter()
        .map(|x| x.to_string())
        .collect();
        insertar_punteros(&mut lineas, "x", &["- [b](x/referencias/b.md)".to_string()]);
        assert_eq!(lineas.last().unwrap(), "- [b](x/referencias/b.md)");
        assert_eq!(lineas[3], "## Referencias falsas");
    }

    #[test]
    fn secciones_should_fall_back_to_plain_titles_when_a_fence_never_closes() {
        // Hallazgo de la revision adversarial: un ``` sin cerrar dentro de una
        // candidata arrastraba Pitfalls y Verificacion a referencias/.
        let cuerpo =
            "## Cuando aplica\n\nc\n\n## Caso (feature #1)\n\n```\nx\n\n## Pitfalls\n\np\n\n## Verificacion\n\nv\n";
        let s = secciones(cuerpo);
        let titulos: Vec<&str> = s.iter().map(|x| x.titulo.as_str()).collect();
        assert_eq!(
            titulos,
            vec![
                "Cuando aplica",
                "Caso (feature #1)",
                "Pitfalls",
                "Verificacion"
            ]
        );
        assert_eq!(s[1].lineas(), 5);
    }

    #[test]
    fn planificar_should_split_candidatas_saldo_and_sugeridas() {
        let plan = planificar(30, 20, CUERPO, "x", &[]);
        assert_eq!(plan.candidatas.len(), 1);
        assert_eq!(plan.candidatas[0].titulo, "Caso (feature #1)");
        // El cuerpo tiene 15 lineas; se van 6 y entran 6 (blanco, titulo del
        // indice, blanco, parrafo, blanco, puntero): el saldo cuenta eso.
        assert_eq!(plan.saldo(), 30, "{plan:?}");
        assert_eq!(plan.falta(), 10);
        assert!(
            plan.sugeridas.is_empty(),
            "las canonicas no se sugieren: {:?}",
            plan.sugeridas
        );
        assert!(plan.sobre_el_tope_hoy());
        // Con el tope apagado nunca falta nada; con una elegida, se suma.
        assert_eq!(planificar(30, 0, CUERPO, "x", &[]).falta(), 0);
        let cuerpo2 = format!("{CUERPO}\n## Tema grande\n\nx\ny\nz\n\n## Tema chico\n\nw\n");
        let plan2 = planificar(40, 20, &cuerpo2, "x", &[]);
        let sug: Vec<&str> = plan2.sugeridas.iter().map(|x| x.titulo.as_str()).collect();
        assert_eq!(sug, vec!["Tema grande", "Tema chico"], "de mayor a menor");
        let elegida = secciones(&cuerpo2)
            .into_iter()
            .find(|x| x.titulo == "Tema chico")
            .unwrap();
        let plan3 = planificar(40, 20, &cuerpo2, "x", &[elegida]);
        assert_eq!(plan3.candidatas.len(), 2);
        assert_eq!(plan3.sugeridas.len(), 1);
    }

    #[test]
    fn planificar_should_count_the_pointer_in_an_existing_index() {
        // Con indice ya presente solo entra el puntero: 19 lineas - 6 + 1 = 14,
        // mas las 21 que estan fuera del cuerpo.
        let cuerpo = format!("{CUERPO}\n## Referencias\n\n- [a](x/referencias/a.md)\n");
        let plan = planificar(40, 20, &cuerpo, "x", &[]);
        assert_eq!(plan.saldo(), 35, "{plan:?}");
        // Sin candidatas el saldo es lo de hoy.
        let sin = planificar(40, 20, "## Cuando aplica\n\nc\n", "x", &[]);
        assert_eq!(sin.saldo(), 40);
    }

    #[test]
    fn resolver_should_prefer_exact_then_unique_substring_and_refuse_canonicas() {
        let cuerpo = "## Cuando aplica\n\nx\n\n## Un caso extra sin numero\n\ny\n\n## Otro caso extra\n\nz\n";
        let s = secciones(cuerpo);
        assert_eq!(
            resolver(&s, "Otro caso extra").unwrap().titulo,
            "Otro caso extra"
        );
        assert_eq!(
            resolver(&s, "SIN NUMERO").unwrap().titulo,
            "Un caso extra sin numero"
        );
        let e = resolver(&s, "caso extra").unwrap_err();
        assert!(e.message.unwrap().contains("matchea 2 secciones"));
        let e = resolver(&s, "nada").unwrap_err();
        assert!(e.message.unwrap().contains("Ninguna seccion"));
        let e = resolver(&s, "Cuando aplica").unwrap_err();
        assert!(e.message.unwrap().contains("es una seccion de la CLASE"));
    }

    #[test]
    fn slug_de_referencia_should_match_the_existing_convention() {
        assert_eq!(
            slug_de_referencia("El caso del martes (feature #12)"),
            "el-caso-del-martes-feature-12"
        );
        assert_eq!(
            slug_de_referencia("Otro caso (#15, 2026-09-01)"),
            "otro-caso-15-2026-09-01"
        );
        assert_eq!(
            slug_de_referencia("Verificación rápida: ñandú"),
            "verificacion-rapida-nandu"
        );
        let largo = slug_de_referencia(
            "La herramienta externa que no esta convierte el test en un placebo y algo mas",
        );
        assert!(largo.len() <= 60 && !largo.ends_with('-'), "{largo}");
        assert_eq!(slug_de_referencia("¿?¡!"), "seccion");
    }

    #[test]
    fn slug_libre_should_add_a_suffix_when_the_file_or_the_run_already_has_it() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("caso.md"), "x").unwrap();
        assert_eq!(slug_libre(tmp.path(), "caso", &[]), "caso-2");
        assert_eq!(
            slug_libre(tmp.path(), "otro", &["otro".to_string()]),
            "otro-2"
        );
        assert_eq!(
            slug_libre(
                tmp.path(),
                "otro",
                &["otro".to_string(), "otro-2".to_string()]
            ),
            "otro-3"
        );
        // Y reutiliza el archivo que ya es esta misma referencia (una corrida
        // anterior que fallo antes de guardar la leccion).
        let mismo = |p: &Path| std::fs::read_to_string(p).is_ok_and(|t| t == "x");
        assert_eq!(slug_para(tmp.path(), "caso", &[], &mismo), "caso");
        let otro = |p: &Path| std::fs::read_to_string(p).is_ok_and(|t| t == "y");
        assert_eq!(slug_para(tmp.path(), "caso", &[], &otro), "caso-2");
    }

    #[test]
    fn id_de_respaldo_should_not_reuse_a_directory_of_the_same_second() {
        let tmp = tempfile::tempdir().unwrap();
        let paths = HarnessPaths::from_root(tmp.path().to_path_buf());
        assert_eq!(
            id_de_respaldo(&paths, "20260908T1", "x"),
            "20260908T1-partir-x"
        );
        std::fs::create_dir_all(curador::backups_dir(&paths).join("20260908T1-partir-x")).unwrap();
        assert_eq!(
            id_de_respaldo(&paths, "20260908T1", "x"),
            "20260908T1-partir-x-2"
        );
    }

    #[test]
    fn cuerpo_de_should_trim_blank_edges_and_keep_h3() {
        let s = secciones(CUERPO);
        let lineas = lineas_de(CUERPO);
        assert_eq!(cuerpo_de(&lineas, &s[1]), "uno\n### sub\ndos\n");
    }

    #[test]
    fn insertar_punteros_should_append_to_an_existing_index_or_create_one() {
        // Indice existente al principio (como "Referencias por tema" de realestate).
        let mut con: Vec<String> = [
            "## Referencias por tema",
            "",
            "- [a](x/referencias/a.md)",
            "",
            "## Cuando aplica",
            "",
            "c",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();
        insertar_punteros(&mut con, "x", &["- [b](x/referencias/b.md)".to_string()]);
        assert_eq!(
            con,
            vec![
                "## Referencias por tema",
                "",
                "- [a](x/referencias/a.md)",
                "- [b](x/referencias/b.md)",
                "",
                "## Cuando aplica",
                "",
                "c"
            ]
        );
        // Sin indice: se crea al final y los blancos que cerraban la ultima
        // seccion quedan como estaban (son de esa seccion).
        let mut sin: Vec<String> = ["## Cuando aplica", "", "c", "", ""]
            .iter()
            .map(|s| s.to_string())
            .collect();
        insertar_punteros(&mut sin, "x", &["- [b](x/referencias/b.md)".to_string()]);
        assert_eq!(
            sin[2..6],
            ["c", "", "", TITULO_DEL_INDICE].map(String::from)[..]
        );
        assert_eq!(sin.last().unwrap(), "- [b](x/referencias/b.md)");
        // Sin blanco final se agrega uno solo.
        let mut pegado: Vec<String> = ["## Cuando aplica", "", "c"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        insertar_punteros(&mut pegado, "x", &["- [b](x/referencias/b.md)".to_string()]);
        assert_eq!(
            pegado[2..5],
            ["c", "", TITULO_DEL_INDICE].map(String::from)[..]
        );
        // Indice vacio (solo el titulo): linea en blanco y los punteros.
        let mut vacio: Vec<String> = ["## Cuando aplica", "", "c", "", "## Referencias"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        insertar_punteros(&mut vacio, "x", &["- [b](x/referencias/b.md)".to_string()]);
        assert_eq!(
            vacio[4..],
            ["## Referencias", "", "- [b](x/referencias/b.md)"].map(String::from)[..]
        );
    }

    #[test]
    fn con_eol_should_turn_a_lf_text_into_crlf_only_when_asked() {
        assert_eq!(con_eol("a\nb\n", "\n"), "a\nb\n");
        assert_eq!(con_eol("a\nb\n", "\r\n"), "a\r\nb\r\n");
    }
}
