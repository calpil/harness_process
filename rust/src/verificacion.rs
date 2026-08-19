//! AC ejecutables: que un criterio declare como se prueba (feature #23).
//!
//! Esta es la primera vez que el binario ejecuta un comando arbitrario, y eso
//! condiciona el modulo entero. Tres barreras, las tres en los AC del spec:
//!
//! - **Spec aprobado.** `verify` se niega en `draft`: aprobar significa que el
//!   USUARIO leyo el texto, y por lo tanto vio los comandos. Es lo que impide que
//!   un comando escrito por un agente se ejecute sin que nadie lo mire.
//! - **Invocacion manual.** Ningun hook ni comando del arnes llama a `verify`.
//! - **Cada comando se imprime antes de correr.** Nada a ciegas.
//!
//! Y el cierre **no ejecuta**: lee el reporte. Cerrar no puede disparar shell.
//!
//! El parseo esta separado de la ejecucion a proposito: asi se puede probar
//! contra los 310 AC reales del repo sin correr un solo comando.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use crate::paths::HarnessPaths;

/// Segundos por comando antes de cortarlo (`rules.verify_timeout_segundos`).
pub const TIMEOUT_DEFAULT: u64 = 300;
/// Lineas de salida que se guardan de un fallo.
pub const LINEAS_SALIDA: usize = 20;

/// Un AC del spec y, si lo declara, su comando de verificacion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Verificacion {
    /// `AC-1`, `AC-12`, ...
    pub ac: String,
    /// `None` = verificacion manual, a cargo del reviewer. NO es un fallo.
    pub comando: Option<String>,
}

/// Como quedo un AC tras la corrida.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Estado {
    Verde,
    Rojo,
    Timeout,
    /// Sin comando declarado: lo verifica el reviewer, como siempre.
    Manual,
    /// El comando salio 0 pero no ejecuto ningun caso: `cargo test` con un
    /// filtro que no matchea imprime `running 0 tests`, dice `ok` y sale 0.
    /// Un AC asi no esta verificado, esta sin medir. Feature #44.
    Vacio,
}

/// Todas las variantes, para los recorridos que tienen que cubrirlas sin
/// depender de que alguien se acuerde de agregar la nueva a mano.
pub const ESTADOS: [Estado; 5] = [
    Estado::Verde,
    Estado::Rojo,
    Estado::Timeout,
    Estado::Manual,
    Estado::Vacio,
];

impl Estado {
    pub fn etiqueta(self) -> &'static str {
        match self {
            Estado::Verde => "verde",
            Estado::Rojo => "rojo",
            Estado::Timeout => "timeout",
            Estado::Manual => "manual",
            Estado::Vacio => "vacio",
        }
    }

    /// La vuelta de `etiqueta`. Existe para que el lector del reporte
    /// (`rojos_del_reporte`, que es lo unico que `close` usa) salga del enum en
    /// vez de comparar contra cadenas sueltas: asi agregar un estado sexto se
    /// cubre en un solo lugar y no se filtra por el cierre. Feature #44, que es
    /// la misma forma del defecto que la #37 encontro en el emisor de Jira.
    pub fn desde_etiqueta(texto: &str) -> Option<Estado> {
        ESTADOS.into_iter().find(|e| e.etiqueta() == texto)
    }

    pub fn simbolo(self) -> &'static str {
        match self {
            Estado::Verde => "[ok]",
            Estado::Rojo => "[!!]",
            Estado::Timeout => "[..]",
            Estado::Manual => "[--]",
            Estado::Vacio => "[??]",
        }
    }

    /// Bloquean rojo, timeout y vacio: un AC manual sigue siendo valido, pero
    /// uno que no midio nada no es evidencia de nada.
    pub fn bloquea(self) -> bool {
        matches!(self, Estado::Rojo | Estado::Timeout | Estado::Vacio)
    }
}

/// Cuantos casos ejecuto realmente un comando, leido de su salida.
///
/// `None` = "no opino": la salida no tiene la forma de una corrida de libtest,
/// asi que puede ser un `grep`, un `bash`, un compilador o cualquier otra cosa
/// y no hay nada que contar. Ese contrato es lo que evita que el detector
/// ponga en rojo trabajo sano.
///
/// Se mira la SALIDA y no el texto del comando a proposito: un `cargo test`
/// adentro de un script de shell tambien queda cubierto, y un comando que se
/// llama "test" pero no lo es, no.
pub fn casos_corridos(salida: &str) -> Option<usize> {
    let mut total = 0usize;
    let mut hubo_linea = false;
    for linea in salida.lines() {
        let Some(resto) = linea.trim_start().strip_prefix("test result:") else {
            continue;
        };
        hubo_linea = true;
        // `ok. 12 passed; 0 failed; 1 ignored; 0 measured; 3 filtered out`.
        // El primer tramo trae el veredicto adelante (`ok. 12 passed`), asi que
        // se busca la palabra `passed` y se lee el numero que la precede en vez
        // de asumir una posicion fija.
        let palabras: Vec<&str> = resto.split_whitespace().collect();
        for (i, palabra) in palabras.iter().enumerate() {
            if palabra.trim_end_matches(';') != "passed" || i == 0 {
                continue;
            }
            if let Ok(n) = palabras[i - 1].parse::<usize>() {
                total += n;
            }
        }
    }
    hubo_linea.then_some(total)
}

#[derive(Debug, Clone)]
pub struct Resultado {
    pub ac: String,
    pub comando: Option<String>,
    pub estado: Estado,
    pub exit: Option<i32>,
    pub duracion_ms: u128,
    /// Ultimas lineas de la salida, solo cuando fallo.
    pub salida: String,
}

/// Extrae los AC del spec y su `Comando:` si lo declaran.
///
/// Funcion **pura**: no toca el filesystem ni ejecuta nada, asi que se puede
/// correr sobre los 310 AC reales del repo en un test.
pub fn parsear(spec: &str) -> Vec<Verificacion> {
    let mut out: Vec<Verificacion> = Vec::new();
    let mut en_bloque = false;
    for linea in spec.lines() {
        let t = linea.trim();
        // Los bloques ``` se saltean. Hallazgo de la primera corrida real: el
        // propio spec de la #23 EXPLICA el formato con un ejemplo dentro de un
        // bloque, y `verify` ejecuto ese ejemplo. Un spec que documenta la
        // sintaxis no puede quedar verificando su documentacion.
        if t.starts_with("```") {
            en_bloque = !en_bloque;
            continue;
        }
        if en_bloque {
            continue;
        }
        if let Some(ac) = ac_de(t) {
            out.push(Verificacion { ac, comando: None });
            continue;
        }
        // `Comando:` pertenece al ultimo AC abierto (decision del usuario
        // 2026-08-17, OBS-1: la prueba va pegada al criterio).
        if let Some(cmd) = comando_de(t)
            && let Some(ultimo) = out.last_mut()
            && ultimo.comando.is_none()
        {
            ultimo.comando = Some(cmd);
        }
    }
    out
}

/// `- AC-12: ...` -> `AC-12`.
fn ac_de(linea: &str) -> Option<String> {
    let resto = linea.strip_prefix("- AC-")?;
    let numero: String = resto.chars().take_while(char::is_ascii_digit).collect();
    if numero.is_empty() || !resto[numero.len()..].starts_with(':') {
        return None;
    }
    Some(format!("AC-{numero}"))
}

/// `Comando: `algo`` -> `algo` (con o sin backticks).
fn comando_de(linea: &str) -> Option<String> {
    let resto = linea.strip_prefix("Comando:")?.trim();
    let limpio = resto.trim_matches('`').trim();
    (!limpio.is_empty()).then(|| limpio.to_string())
}

/// Umbral de timeout desde `rules`.
pub fn timeout_segundos(data: &serde_json::Value) -> u64 {
    data.get("rules")
        .and_then(|r| r.get("verify_timeout_segundos"))
        .and_then(serde_json::Value::as_u64)
        .filter(|v| *v > 0)
        .unwrap_or(TIMEOUT_DEFAULT)
}

/// Ejecuta UN comando desde la raiz del proyecto, con timeout.
///
/// El comando se pasa tal cual al shell: no se interpola nada del entorno ni de
/// la consulta de nadie. Un comando inexistente o no ejecutable sale **rojo**
/// con su error (OBS-4): un criterio que no se puede correr no esta verificado.
pub fn ejecutar(comando: &str, cwd: &Path, timeout: Duration) -> (Estado, Option<i32>, u128, String) {
    use std::process::{Command, Stdio};
    use wait_timeout::ChildExt;

    let arranque = Instant::now();
    let hijo = if cfg!(windows) {
        Command::new("cmd")
            .args(["/C", comando])
            .current_dir(cwd)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    } else {
        Command::new("sh")
            .args(["-c", comando])
            .current_dir(cwd)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    };
    let mut hijo = match hijo {
        Ok(h) => h,
        Err(err) => {
            return (
                Estado::Rojo,
                None,
                arranque.elapsed().as_millis(),
                format!("no se pudo ejecutar: {err}"),
            );
        }
    };
    let estado_salida = match hijo.wait_timeout(timeout) {
        Ok(Some(s)) => Some(s),
        Ok(None) => {
            let _ = hijo.kill();
            let _ = hijo.wait();
            None
        }
        Err(err) => {
            return (
                Estado::Rojo,
                None,
                arranque.elapsed().as_millis(),
                format!("fallo esperando al proceso: {err}"),
            );
        }
    };
    let duracion = arranque.elapsed().as_millis();
    // Se MIDE sobre la salida completa y se RECORTA solo para el reporte. Al
    // reves —que es como nacio la #44— el resumen de libtest se cae de las
    // ultimas LINEAS_SALIDA en cuanto el comando imprime algo despues, y el
    // detector se apaga solo justo cuando mas hace falta: `cargo test` manda los
    // diagnosticos de compilacion por stderr, y stderr va al final.
    let completa = leer_salida(&mut hijo);
    let casos = casos_corridos(&completa);
    let salida = recortar_salida(&completa);
    match estado_salida {
        None => (Estado::Timeout, None, duracion, salida),
        // Feature #44: el camino feliz ya no descarta la salida. Un exit 0 dice
        // que el comando anduvo, no que haya medido algo, y esa diferencia solo
        // esta en lo que imprimio.
        Some(s) if s.success() => match casos {
            Some(0) => (Estado::Vacio, s.code(), duracion, salida),
            _ => (Estado::Verde, s.code(), duracion, String::new()),
        },
        Some(s) => (Estado::Rojo, s.code(), duracion, salida),
    }
}

fn leer_salida(hijo: &mut std::process::Child) -> String {
    use std::io::Read;
    let mut texto = String::new();
    if let Some(mut out) = hijo.stdout.take() {
        let _ = out.read_to_string(&mut texto);
    }
    if let Some(mut err) = hijo.stderr.take() {
        let mut e = String::new();
        let _ = err.read_to_string(&mut e);
        texto.push_str(&e);
    }
    texto
}

/// Ultimas `LINEAS_SALIDA` lineas: lo suficiente para diagnosticar sin volcar
/// una suite entera en el reporte.
pub fn recortar_salida(texto: &str) -> String {
    let lineas: Vec<&str> = texto.lines().collect();
    if lineas.len() <= LINEAS_SALIDA {
        return texto.trim_end().to_string();
    }
    let ultimas = &lineas[lineas.len() - LINEAS_SALIDA..];
    format!(
        "(... {} lineas omitidas)\n{}",
        lineas.len() - LINEAS_SALIDA,
        ultimas.join("\n")
    )
}

pub fn reporte_path(paths: &HarnessPaths, fid: &str) -> PathBuf {
    paths.plans.join(format!("verify-{fid}.md"))
}

pub fn reporte_rel(fid: &str) -> String {
    format!("docs/verify-{fid}.md")
}

/// Cuerpo del reporte. Se separa del comando para poder testear el formato.
pub fn render_reporte(fid: &str, stamp: &str, resultados: &[Resultado]) -> String {
    let cuenta = |e: Estado| resultados.iter().filter(|r| r.estado == e).count();
    let vacios = cuenta(Estado::Vacio);
    // Los vacios bloquean, pero contarlos dentro de "en rojo" volveria a
    // esconder justo lo que esta feature vino a mostrar.
    let rojos = resultados.iter().filter(|r| r.estado.bloquea()).count() - vacios;
    let verdes = cuenta(Estado::Verde);
    let manuales = cuenta(Estado::Manual);
    let sin_casos = if vacios > 0 {
        format!(", {vacios} sin casos")
    } else {
        String::new()
    };
    let mut out = format!(
        "# Verificacion de AC - Feature #{fid}\n\n\
         Corrida: {stamp}\n\
         Resultado: {verdes} verde(s), {rojos} en rojo, {manuales} manual(es){sin_casos}.\n\n\
         | AC | Estado | Comando | Exit | ms |\n| --- | --- | --- | --- | --- |\n"
    );
    for r in resultados {
        let comando = r.comando.as_deref().unwrap_or("(verificacion manual)");
        let exit = r
            .exit
            .map(|c| c.to_string())
            .unwrap_or_else(|| "-".to_string());
        out.push_str(&format!(
            "| {} | {} | `{}` | {} | {} |\n",
            r.ac,
            r.estado.etiqueta(),
            comando.replace('|', "\\|"),
            exit,
            r.duracion_ms
        ));
    }
    let fallidos: Vec<&Resultado> = resultados.iter().filter(|r| r.estado.bloquea()).collect();
    if !fallidos.is_empty() {
        out.push_str("\n## Salida de los que fallaron\n");
        for r in fallidos {
            out.push_str(&format!(
                "\n### {} ({})\n\n```\n{}\n```\n",
                r.ac,
                r.estado.etiqueta(),
                if r.salida.is_empty() { "(sin salida)" } else { &r.salida }
            ));
        }
    }
    if manuales > 0 {
        out.push_str(
            "\n---\n\nLos AC marcados `manual` no declaran comando: los verifica el\n\
             reviewer, como siempre. No cuentan como fallo.\n",
        );
    }
    out
}

/// Lee un reporte ya escrito y dice que AC quedaron bloqueando. Es lo UNICO que
/// usa el cierre: `close` nunca ejecuta un comando (AC-16).
pub fn rojos_del_reporte(texto: &str) -> Vec<String> {
    texto
        .lines()
        .filter(|l| l.starts_with("| AC-"))
        .filter_map(|l| {
            let celdas: Vec<&str> = l.split('|').map(str::trim).collect();
            let ac = celdas.get(1)?;
            // Falla CERRADO: una etiqueta que no se reconoce bloquea. Antes se
            // descartaba en silencio, asi que un estado nuevo que alguien se
            // olvidara de agregar a `ESTADOS` se filtraba por el cierre sin que
            // nada fallara — la misma forma del defecto que la #37 encontro en
            // el emisor de Jira, y la que este mismo AC prometia cerrar.
            let bloquea = Estado::desde_etiqueta(celdas.get(2)?).is_none_or(Estado::bloquea);
            bloquea.then(|| (*ac).to_string())
        })
        .collect()
}

/// Lee `rules.require_verify_green` (default false: la regla nace apagada, como
/// las otras dos, para no romper instalaciones existentes).
pub fn require_verify_green(data: &serde_json::Value) -> bool {
    data.get("rules")
        .and_then(|r| r.get("require_verify_green"))
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false)
}

/// Gate de cierre (AC-12..AC-16). **Solo lee**: ningun camino de aqui ejecuta un
/// comando. Cerrar una feature no puede disparar shell — esa es la razon de que
/// el reporte se versione en `docs/` en vez de recalcularse al cerrar.
///
/// Se sostiene por estructura y no por disciplina (leccion
/// `promesas-estructurales-vs-disciplina`): esta funcion no llama a `ejecutar`,
/// y lo unico que hace con el spec es leerlo para saber si declara comandos.
pub fn gate(
    paths: &HarnessPaths,
    data: &serde_json::Value,
    status: &str,
    spec: &Path,
    fid: &str,
) -> Result<(), crate::exit::Exit> {
    use crate::exit::Exit;
    // Igual que los otros dos gates: blocked/pending son la valvula de escape.
    if status != "done" || !require_verify_green(data) {
        return Ok(());
    }
    let Ok(texto_spec) = std::fs::read_to_string(spec) else {
        return Ok(()); // sin spec ya gatea spec_gate; no duplicamos el mensaje
    };
    let declarados = parsear(&texto_spec)
        .into_iter()
        .filter(|v| v.comando.is_some())
        .count();
    if declarados == 0 {
        // AC-13: la regla activa no rompe features cuyos AC no declaran nada.
        return Ok(());
    }
    let reporte = reporte_path(paths, fid);
    let Ok(texto) = std::fs::read_to_string(&reporte) else {
        return Err(Exit {
            code: 2,
            message: Some(format!(
            "[GATE] Falta el reporte de verificacion: {}.\n    \
             La regla require_verify_green esta activa y el spec declara {declarados} comando(s).\n    \
             Corre: sh harness_cli verify --feature {fid}",
            reporte_rel(fid)
        ))});
    };
    // AC-15: fresco. Si el spec se edito despues de la corrida, lo verificado ya
    // no es lo que dice el spec.
    if let (Ok(m_rep), Ok(m_spec)) = (
        std::fs::metadata(&reporte).and_then(|m| m.modified()),
        std::fs::metadata(spec).and_then(|m| m.modified()),
    ) && m_rep < m_spec
    {
        return Err(Exit {
            code: 2,
            message: Some(format!(
            "[GATE] El reporte {} es mas viejo que el spec.\n    \
             El spec cambio despues de la ultima corrida: lo verificado ya no es\n    \
             lo que el spec pide. Corre: sh harness_cli verify --feature {fid}",
            reporte_rel(fid)
        ))});
    }
    let rojos = rojos_del_reporte(&texto);
    if rojos.is_empty() {
        return Ok(());
    }
    Err(Exit {
        code: 2,
        message: Some(format!(
        "[GATE] Hay AC en rojo: {}.\n    \
         Reporte: {}. Arreglalos y volve a correr:\n      \
         sh harness_cli verify --feature {fid}",
        rojos.join(", "),
        reporte_rel(fid)
    ))})
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;
    use serde_json::json;

    #[test]
    fn parse_should_take_the_command_below_its_ac() {
        let spec = "- AC-1: Given algo, When otra, Then resultado.\n  \
                    Comando: `bash tests/smoke.sh`\n\
                    - AC-2: Otro criterio.\n";
        let v = parsear(spec);
        assert_eq!(v.len(), 2);
        assert_eq!(v[0].ac, "AC-1");
        assert_eq!(v[0].comando.as_deref(), Some("bash tests/smoke.sh"));
        assert_eq!(v[1].ac, "AC-2");
        assert_eq!(v[1].comando, None);
    }

    #[test]
    fn parse_should_accept_the_command_without_backticks() {
        let spec = "- AC-1: algo.\n  Comando: cargo test\n";
        assert_eq!(parsear(spec)[0].comando.as_deref(), Some("cargo test"));
    }

    #[test]
    fn parse_should_keep_only_the_first_command_of_an_ac() {
        let spec = "- AC-1: algo.\n  Comando: `uno`\n  Comando: `dos`\n";
        let v = parsear(spec);
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].comando.as_deref(), Some("uno"));
    }

    #[test]
    fn parse_should_ignore_lines_that_are_not_acs() {
        let spec = "## Criterios\n\n- Un item cualquiera.\n- AC-3: real.\n  Comando: `x`\n\
                    Texto suelto con AC-9: que no es un item.\n";
        let v = parsear(spec);
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].ac, "AC-3");
    }

    #[test]
    fn parse_should_ignore_examples_inside_fenced_blocks() {
        // El bug que encontro la primera corrida real sobre el spec de la #23:
        // el ejemplo que ENSENA el formato terminaba ejecutandose.
        let spec = "DESPUES: un AC puede declarar como se prueba:\n\n\
                    ```\n\
                    - AC-1: Given un ejemplo, Then se ve el formato.\n  \
                    Comando: `bash tests/setup_smoke.sh`\n\
                    ```\n\n\
                    ## Criterios\n\n\
                    - AC-1: Given lo real, Then esto si.\n  Comando: `true`\n";
        let v = parsear(spec);
        assert_eq!(v.len(), 1, "el ejemplo del bloque se colo: {v:?}");
        assert_eq!(v[0].comando.as_deref(), Some("true"));
    }

    #[test]
    fn parse_should_find_nothing_in_a_spec_without_acs() {
        assert!(parsear("# Spec\n\nProsa sin criterios.\n").is_empty());
    }

    #[test]
    fn manual_should_never_block() {
        assert!(!Estado::Manual.bloquea());
        assert!(Estado::Rojo.bloquea());
        assert!(Estado::Timeout.bloquea());
        assert!(!Estado::Verde.bloquea());
    }

    #[test]
    fn timeout_should_come_from_rules_with_a_default() {
        assert_eq!(timeout_segundos(&json!({})), TIMEOUT_DEFAULT);
        assert_eq!(
            timeout_segundos(&json!({"rules": {"verify_timeout_segundos": 30}})),
            30
        );
        // 0 no puede apagar el timeout: seria un comando colgado para siempre.
        assert_eq!(
            timeout_segundos(&json!({"rules": {"verify_timeout_segundos": 0}})),
            TIMEOUT_DEFAULT
        );
    }

    #[test]
    fn ejecutar_should_report_green_on_success() {
        let dir = tempfile::tempdir().unwrap();
        let (estado, exit, _, salida) =
            ejecutar("exit 0", dir.path(), Duration::from_secs(10));
        assert_eq!(estado, Estado::Verde);
        assert_eq!(exit, Some(0));
        assert!(salida.is_empty());
    }

    #[test]
    fn ejecutar_should_report_red_with_output_on_failure() {
        let dir = tempfile::tempdir().unwrap();
        let (estado, exit, _, salida) = ejecutar(
            "echo 'algo salio mal' >&2; exit 3",
            dir.path(),
            Duration::from_secs(10),
        );
        assert_eq!(estado, Estado::Rojo);
        assert_eq!(exit, Some(3));
        assert!(salida.contains("algo salio mal"), "{salida}");
    }

    #[test]
    fn ejecutar_should_report_red_when_the_command_does_not_exist() {
        // OBS-4: un criterio que no se puede correr no esta verificado.
        let dir = tempfile::tempdir().unwrap();
        let (estado, _, _, _) = ejecutar(
            "comando-que-no-existe-en-ningun-lado",
            dir.path(),
            Duration::from_secs(10),
        );
        assert_eq!(estado, Estado::Rojo);
    }

    #[test]
    fn ejecutar_should_time_out_a_hung_command() {
        let dir = tempfile::tempdir().unwrap();
        let (estado, _, _, _) = ejecutar("sleep 30", dir.path(), Duration::from_millis(200));
        assert_eq!(estado, Estado::Timeout);
    }

    #[test]
    fn ejecutar_should_run_from_the_given_directory() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("marcador.txt"), "x").unwrap();
        let (estado, _, _, _) = ejecutar("test -f marcador.txt", dir.path(), Duration::from_secs(10));
        assert_eq!(estado, Estado::Verde);
    }

    #[test]
    fn recortar_salida_should_keep_the_tail() {
        let largo: String = (1..=50).map(|i| format!("linea {i}\n")).collect();
        let corto = recortar_salida(&largo);
        assert!(corto.contains("lineas omitidas"));
        assert!(corto.contains("linea 50"));
        assert!(!corto.contains("linea 1\n"));
        assert_eq!(recortar_salida("corta"), "corta");
    }

    fn resultado(ac: &str, estado: Estado, comando: Option<&str>) -> Resultado {
        Resultado {
            ac: ac.to_string(),
            comando: comando.map(str::to_string),
            estado,
            exit: Some(if estado == Estado::Verde { 0 } else { 1 }),
            duracion_ms: 12,
            salida: if estado.bloquea() { "detalle".to_string() } else { String::new() },
        }
    }

    #[test]
    fn render_should_summarize_and_detail_failures() {
        let r = [
            resultado("AC-1", Estado::Verde, Some("cargo test")),
            resultado("AC-2", Estado::Rojo, Some("false")),
            resultado("AC-3", Estado::Manual, None),
        ];
        let texto = render_reporte("23", "2026-08-17T00:00:00Z", &r);
        assert!(texto.contains("1 verde(s), 1 en rojo, 1 manual(es)"), "{texto}");
        assert!(texto.contains("| AC-1 | verde | `cargo test` | 0 | 12 |"), "{texto}");
        assert!(texto.contains("(verificacion manual)"), "{texto}");
        assert!(texto.contains("## Salida de los que fallaron"), "{texto}");
        assert!(texto.contains("### AC-2"), "{texto}");
        assert!(texto.contains("No cuentan como fallo"), "{texto}");
    }

    #[test]
    fn render_should_not_add_a_failure_section_when_all_green() {
        let r = [resultado("AC-1", Estado::Verde, Some("true"))];
        let texto = render_reporte("23", "ts", &r);
        assert!(!texto.contains("Salida de los que fallaron"));
    }

    #[test]
    fn rojos_del_reporte_should_read_back_what_render_wrote() {
        // El cierre LEE esto; nunca ejecuta (AC-16). Round trip render -> lectura.
        let r = [
            resultado("AC-1", Estado::Verde, Some("true")),
            resultado("AC-2", Estado::Rojo, Some("false")),
            resultado("AC-5", Estado::Timeout, Some("sleep 999")),
            resultado("AC-9", Estado::Manual, None),
        ];
        let texto = render_reporte("23", "ts", &r);
        assert_eq!(rojos_del_reporte(&texto), ["AC-2", "AC-5"]);
    }

    #[test]
    fn rojos_del_reporte_should_be_empty_for_a_green_report() {
        let texto = render_reporte("23", "ts", &[resultado("AC-1", Estado::Verde, Some("true"))]);
        assert!(rojos_del_reporte(&texto).is_empty());
    }

    #[test]
    fn parse_should_only_report_commands_the_spec_actually_declares() {
        // Los specs del repo son DATO DE ENTRADA de este parser: leerlos no es
        // leer el fuente (docs/conventions.md, regla 2). El test seguiria
        // valiendo si `parsear` se reescribiera entera.
        //
        // La primera version de este test asertaba "ningun spec salvo el de la
        // #23 declara comandos". Era un detector-de-cambios: se rompio en cuanto
        // la #24 declaro los suyos, sin que nada estuviera mal. Ahora asserta el
        // INVARIANTE: el parser no inventa ni pierde comandos, cuente el repo lo
        // que cuente.
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../docs");
        let Ok(entradas) = std::fs::read_dir(&dir) else {
            return; // sin docs/ en el sandbox de build: nada que comprobar
        };
        let mut acs = 0usize;
        let mut specs = 0usize;
        for entrada in entradas.flatten() {
            let path = entrada.path();
            let es_spec = path
                .file_name()
                .is_some_and(|n| n.to_string_lossy().starts_with("spec-feature-"));
            if !es_spec {
                continue;
            }
            let Ok(texto) = std::fs::read_to_string(&path) else {
                continue;
            };
            specs += 1;
            let hallados = parsear(&texto);
            acs += hallados.len();
            let con_comando = hallados.iter().filter(|v| v.comando.is_some()).count();
            let declarados = declaraciones_fuera_de_bloques(&texto);
            assert_eq!(
                con_comando,
                declarados,
                "{}: el parser reporto {con_comando} comando(s) y el spec declara {declarados}",
                path.display()
            );
            // Y ningun comando sale vacio: un `Comando:` sin nada detras seria
            // un AC que dice verificarse y no verifica nada.
            for v in &hallados {
                if let Some(c) = &v.comando {
                    assert!(!c.trim().is_empty(), "{}: {} con comando vacio", path.display(), v.ac);
                }
            }
        }
        assert!(specs >= 1, "no se leyo ningun spec real");
        assert!(acs > 100, "esperaba cientos de AC reales, encontre {acs}");
    }

    /// Cuenta las lineas `Comando:` que estan fuera de un bloque ``` — las que
    /// el parser tiene que ver. Se calcula por un camino distinto al de
    /// `parsear` a proposito: si las dos implementaciones coinciden sobre 20+
    /// specs reales, el acuerdo significa algo.
    fn declaraciones_fuera_de_bloques(texto: &str) -> usize {
        let mut en_bloque = false;
        let mut n = 0usize;
        let mut ac_abierto = false;
        for linea in texto.lines() {
            let t = linea.trim();
            if t.starts_with("```") {
                en_bloque = !en_bloque;
                continue;
            }
            if en_bloque {
                continue;
            }
            if t.starts_with("- AC-") {
                ac_abierto = true;
            } else if t.starts_with("Comando:") && ac_abierto {
                n += 1;
                ac_abierto = false; // solo el primero cuenta, como en `parsear`
            }
        }
        n
    }
}

/// Feature #44: el instrumento que dice "verde" sin haber medido nada.
///
/// La salida de los tests es dato REAL capturado del repo, no inventada: es
/// lo que imprimio `cargo test consolidar_without_aplicar_should_not_touch_anything`
/// el 2026-08-18, que es el falso verde que dio origen a esta feature.
#[cfg(test)]
mod tests_vacio {
    use super::*;

    /// Los dos binarios de test de este repo, con el filtro que no matchea nada.
    const FILTRO_VACIO_REAL: &str = "\
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 322 filtered out; finished in 0.00s

running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 161 filtered out; finished in 0.00s";

    #[test]
    fn casos_corridos_should_not_opine_about_non_libtest_output() {
        // Un grep, un script de shell, un compilador, nada. Ninguno es una
        // corrida de tests, y opinar sobre ellos es lo unico que podria poner
        // en rojo trabajo sano.
        for salida in [
            "",
            "docs/architecture.md:12:superseded",
            "[Ok] paridad: los ocho modos verdes",
            "    Finished `release` profile [optimized] target(s) in 7.57s",
            "warning: el resultado del test no importa aca",
        ] {
            assert_eq!(
                casos_corridos(salida),
                None,
                "no deberia opinar sobre: {salida:?}"
            );
        }
    }

    #[test]
    fn casos_corridos_should_count_zero_on_the_real_empty_filter() {
        assert_eq!(casos_corridos(FILTRO_VACIO_REAL), Some(0));
    }

    #[test]
    fn casos_corridos_should_sum_across_test_binaries() {
        // El caso normal de `cargo test <nombre>`: matchea en un binario y no
        // en los otros. Eso SI es evidencia.
        let salida = "\
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 322 filtered out; finished in 0.00s

running 3 tests
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 158 filtered out; finished in 0.70s";
        assert_eq!(casos_corridos(salida), Some(3));
    }

    #[test]
    fn casos_corridos_should_count_ignored_tests_as_no_evidence() {
        let salida = "test result: ok. 0 passed; 0 failed; 4 ignored; 0 measured; 0 filtered out; \
                      finished in 0.00s";
        assert_eq!(casos_corridos(salida), Some(0));
    }

    #[test]
    fn vacio_should_block_without_pretending_to_be_red() {
        assert!(Estado::Vacio.bloquea());
        assert_ne!(Estado::Vacio.etiqueta(), Estado::Rojo.etiqueta());
        assert_ne!(Estado::Vacio.simbolo(), Estado::Rojo.simbolo());
        assert_ne!(Estado::Vacio, Estado::Verde);
    }

    #[test]
    fn etiqueta_should_round_trip_for_every_estado() {
        // El invariante que reemplaza a las cadenas sueltas de
        // `rojos_del_reporte`: lo que el reporte escribe, el reporte lo lee.
        // Si manana aparece un estado sexto, este test lo obliga a cerrar el
        // circuito en vez de filtrarse por el cierre.
        for estado in ESTADOS {
            assert_eq!(
                Estado::desde_etiqueta(estado.etiqueta()),
                Some(estado),
                "no vuelve del reporte: {estado:?}"
            );
        }
        assert_eq!(Estado::desde_etiqueta("inventado"), None);
    }

    #[test]
    fn ejecutar_should_mark_an_empty_test_run_as_vacio() {
        let dir = std::env::temp_dir();
        // Reproduce la salida real de libtest con exit 0, que es exactamente lo
        // que hace `cargo test <nombre-inexistente>`.
        // Las comillas simples de sh conservan los saltos de linea tal cual, asi
        // que la salida real entra entera sin escapes que la desfiguren.
        let comando = format!("printf '%s' '{FILTRO_VACIO_REAL}'");
        let (estado, exit, _, salida) =
            ejecutar(&comando, &dir, std::time::Duration::from_secs(30));
        assert_eq!(estado, Estado::Vacio);
        assert_eq!(exit, Some(0));
        assert!(
            salida.contains("0 passed"),
            "la salida tiene que quedar como evidencia, y quedo: {salida:?}"
        );
    }

    #[test]
    fn ejecutar_should_measure_before_trimming_the_output() {
        // El detector tiene que medir sobre la salida COMPLETA. Si mide sobre la
        // recortada (ultimas LINEAS_SALIDA), cualquier comando que imprima algo
        // despues del resumen de libtest lo empuja fuera de la ventana y el AC
        // vuelve a salir verde sin haber medido nada.
        //
        // No es hipotetico: `cargo test` manda los diagnosticos de compilacion
        // por stderr, y `leer_salida` pega stderr DESPUES de stdout, asi que el
        // ruido queda siempre en la cola.
        let dir = std::env::temp_dir();
        let relleno: String = (0..LINEAS_SALIDA + 5)
            .map(|i| format!("warning: ruido numero {i}\n"))
            .collect();
        let comando = format!("printf '%s' '{FILTRO_VACIO_REAL}'; printf '%s' '{relleno}'");
        let (estado, _, _, _) = ejecutar(&comando, &dir, std::time::Duration::from_secs(30));
        assert_eq!(
            estado,
            Estado::Vacio,
            "el resumen de libtest quedo fuera de las ultimas {LINEAS_SALIDA} lineas y el \
             detector se apago solo"
        );
    }

    #[test]
    fn ejecutar_should_keep_a_real_test_run_green() {
        let dir = std::env::temp_dir();
        let comando = "printf 'running 1 test\\ntest result: ok. 1 passed; 0 failed; 0 ignored; \
                       0 measured; 0 filtered out; finished in 0.01s\\n'";
        let (estado, _, _, _) = ejecutar(comando, &dir, std::time::Duration::from_secs(30));
        assert_eq!(estado, Estado::Verde);
    }

    #[test]
    fn ejecutar_should_not_mark_a_non_test_command_as_vacio() {
        let dir = std::env::temp_dir();
        for comando in ["true", "echo hola", "printf ''"] {
            let (estado, _, _, _) = ejecutar(comando, &dir, std::time::Duration::from_secs(30));
            assert_eq!(estado, Estado::Verde, "no es un test: {comando}");
        }
    }

    #[test]
    fn render_should_count_empty_runs_apart_from_red() {
        let r = |ac: &str, estado: Estado| Resultado {
            ac: ac.to_string(),
            comando: Some("cargo test lo_que_sea".to_string()),
            estado,
            exit: Some(0),
            duracion_ms: 1,
            salida: if estado == Estado::Verde {
                String::new()
            } else {
                "test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 9 filtered out"
                    .to_string()
            },
        };
        let texto = render_reporte(
            "44",
            "ts",
            &[
                r("AC-1", Estado::Verde),
                r("AC-2", Estado::Vacio),
                r("AC-3", Estado::Rojo),
            ],
        );
        assert!(
            texto.contains("1 verde(s), 1 en rojo, 0 manual(es), 1 sin casos."),
            "el resumen esconde los vacios dentro de los rojos:\n{texto}"
        );
        assert!(texto.contains("| AC-2 | vacio |"));
        // Y su salida tiene que estar, que es lo que deja ver POR QUE no midio.
        assert!(texto.contains("### AC-2 (vacio)"));
        assert!(texto.contains("0 passed"));
    }

    #[test]
    fn rojos_del_reporte_should_fail_closed_on_an_unknown_estado() {
        // La garantia que el AC-11 promete no la puede dar `ESTADOS`, que es un
        // array escrito a mano: una variante nueva que nadie agregue ahi
        // compila y pasa la suite. Lo que la da es que el LECTOR no deje pasar
        // lo que no entiende.
        let texto = "\
# Verificacion de AC - Feature #1

| AC | Estado | Comando | Exit | ms |
| --- | --- | --- | --- | --- |
| AC-1 | verde | `true` | 0 | 1 |
| AC-2 | sospechoso | `algo` | 0 | 1 |
";
        assert_eq!(rojos_del_reporte(texto), vec!["AC-2".to_string()]);
    }

    #[test]
    fn rojos_del_reporte_should_include_empty_runs() {
        let texto = render_reporte(
            "44",
            "ts",
            &[Resultado {
                ac: "AC-7".to_string(),
                comando: Some("cargo test no_existe".to_string()),
                estado: Estado::Vacio,
                exit: Some(0),
                duracion_ms: 1,
                salida: "test result: ok. 0 passed".to_string(),
            }],
        );
        assert_eq!(rojos_del_reporte(&texto), vec!["AC-7".to_string()]);
    }
}
