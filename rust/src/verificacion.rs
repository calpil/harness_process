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

/// Tope de salida retenida por comando (decision del usuario, OBS-1 de la #46).
/// Se retiene la COLA: una suite entera entra holgada y una salida infinita no
/// puede voltear al arnes por memoria.
pub const MAX_SALIDA_BYTES: usize = 4 * 1024 * 1024;

/// Un AC del spec y, si lo declara, su comando de verificacion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Verificacion {
    /// `AC-1`, `AC-12`, ...
    pub ac: String,
    /// Los comandos que el AC declara, en el orden en que estan escritos.
    ///
    /// Feature #73: esto era `Option<String>` —UN comando— y el segundo
    /// `Comando:` de un mismo AC se descartaba sin marca. El AC-8 de la #72
    /// declaraba cuatro y `verify` corrio uno, reportando "1 verde, 0 en rojo".
    /// El modelo lo hacia inevitable: no habia donde poner el segundo.
    ///
    /// Vacio = verificacion MANUAL, a cargo del reviewer. NO es un fallo.
    pub comandos: Vec<String>,
}

impl Verificacion {
    /// True si el AC no declara ninguna verificacion ejecutable.
    pub fn es_manual(&self) -> bool {
        self.comandos.is_empty()
    }
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
    // Los bloques de codigo se saltean. Hallazgo de la primera corrida real de
    // la #23: el propio spec EXPLICA el formato con un ejemplo dentro de un
    // bloque, y `verify` ejecuto ese ejemplo. Un spec que documenta la sintaxis
    // no puede quedar verificando su documentacion.
    //
    // Feature #67: esto usaba su propio parser, que togglea solo con ```` ``` ````
    // y no conocia `~~~`, asi que el bug de la #23 seguia ABIERTO para tildes —
    // medido: un `Comando:` dentro de un bloque `~~~` se ejecutaba. Ahora usa el
    // parser unico, que conoce los dos fences y los empareja.
    // Una linea que ARRANCA como AC pero que no se puede leer cierra el AC
    // anterior. Sin esto, su `Comando:` se le colgaba al de arriba: reproducido
    // con el parser real, `- AC-1: uno` (sin comando) seguido de un AC ilegible
    // con `Comando: touch MAL.txt` dejaba a **AC-1** con ese comando, y `verify`
    // habria impreso "AC-1 verde" tras correr la prueba de otro criterio. Un
    // verde atribuido al AC equivocado es la familia de la #44.
    let mut ac_ilegible = false;
    for linea in crate::markdown::lineas_fuera_de_bloque(spec) {
        let t = linea.trim();
        if let Some(ac) = ac_de(t) {
            ac_ilegible = false;
            out.push(Verificacion {
                ac,
                comandos: Vec::new(),
            });
            continue;
        }
        if t.starts_with("- AC-") {
            ac_ilegible = true;
            continue;
        }
        // `Comando:` pertenece al ultimo AC abierto (decision del usuario
        // 2026-08-17, OBS-1: la prueba va pegada al criterio).
        // Feature #73: se acumulan TODOS. Antes habia un `ultimo.comando.is_none()`
        // que hacia que el primero ganara y los demas desaparecieran: un AC con
        // cuatro verificaciones quedaba verde por la primera. La guarda de la
        // #68 (`!ac_ilegible`) sigue igual y es la que impide que el `Comando:`
        // de un AC que no se pudo leer se le cuelgue al AC de arriba.
        if let Some(cmd) = comando_de(t)
            && !ac_ilegible
            && let Some(ultimo) = out.last_mut()
        {
            ultimo.comandos.push(cmd);
        }
    }
    out
}

/// Las lineas que ARRANCAN como AC y no se pueden leer.
///
/// Es la misma condicion que `parsear` ya evalua para no colgarle el `Comando:`
/// de una linea ilegible al AC de arriba (feature #68) — solo que ahi moria
/// adentro de la funcion y nadie la podia preguntar. La #68 lo dejo declarado
/// como limite conocido: el criterio desaparecia y nadie se enteraba.
///
/// Pura, como `parsear`: se puede correr sobre los specs reales del repo sin
/// tocar disco.
pub fn lineas_ac_ilegibles(spec: &str) -> Vec<String> {
    crate::markdown::lineas_fuera_de_bloque(spec)
        .into_iter()
        .map(str::trim)
        .filter(|t| t.starts_with("- AC-") && ac_de(t).is_none())
        .map(str::to_string)
        .collect()
}

/// `- AC-12: ...` -> `AC-12`. Tambien `- AC-4b:` -> `AC-4b` y
/// `- AC-11 (MANUAL):` -> `AC-11`.
///
/// Antes pedia los dos puntos PEGADOS a los digitos, y cualquier otra cosa en el
/// medio tiraba el AC entero. Medido sobre los 55 specs del repo: siete AC
/// desaparecidos en dos familias —cuatro `(MANUAL)` en las #64/#65/#66/#67, y
/// `AC-4b` / `AC-12b` / `AC-12c` en las #16/#51—. La anotacion `(MANUAL)`, que
/// existe justo para marcar "esto lo tiene que mirar una persona", era lo que
/// hacia que el arnes no se lo pidiera a nadie.
///
/// El sufijo de letra SI entra en el nombre (`AC-4b` es otro criterio que
/// `AC-4`); la anotacion entre parentesis NO (`- AC-11 (MANUAL):` es `AC-11`).
///
/// Se afloja lo justo: lo que no es un AC tiene que seguir sin serlo, porque un
/// parser que se come prosa es peor que uno que pierde un AC (feature #68, AC-5).
fn ac_de(linea: &str) -> Option<String> {
    let resto = linea.strip_prefix("- AC-")?;
    let numero: String = resto.chars().take_while(char::is_ascii_digit).collect();
    if numero.is_empty() {
        return None;
    }
    let resto = &resto[numero.len()..];
    // `AC-4b`, `AC-12c`: parte del NOMBRE, es un criterio distinto.
    let letras: String = resto.chars().take_while(char::is_ascii_alphabetic).collect();
    let resto = &resto[letras.len()..];
    // ` (MANUAL)` y cualquier otra anotacion: NO entra en el nombre. Se acepta
    // la que los specs ya usan, no se inventa una sintaxis nueva.
    let resto = match resto.strip_prefix(" (") {
        Some(r) => match r.find(')') {
            Some(i) => &r[i + 1..],
            // Un parentesis sin cerrar no es una anotacion: es prosa. Este
            // `return` es EXPLICITO, no cargante: se comprobo por mutacion que
            // dejar caer el caso da el mismo resultado (lo que queda tampoco
            // empieza con `:`), asi que no hay test que distinga las dos
            // versiones. Se deja escrito para que la intencion se lea, y se
            // declara aca que no es una defensa independiente.
            None => return None,
        },
        None => resto,
    };
    if !resto.starts_with(':') {
        return None;
    }
    Some(format!("AC-{numero}{letras}"))
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
    // Feature #46: los lectores arrancan ANTES de esperar al proceso. Si se
    // espera primero, el hijo se bloquea al llenar el buffer del pipe (~64 KB)
    // y nadie lo desbloquea nunca.
    let lectores = lanzar_lectores(&mut hijo);
    let estado_salida = match hijo.wait_timeout(timeout) {
        Ok(Some(s)) => Some(s),
        Ok(None) => {
            let _ = hijo.kill();
            let _ = hijo.wait();
            None
        }
        Err(err) => {
            let _ = hijo.kill();
            let _ = juntar_lectores(lectores);
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
    // Los hilos terminan solos cuando el proceso cierra sus pipes (tambien
    // cuando se lo mata por timeout), asi que este join no puede quedarse.
    let (completa, omitidos, quedo_abierto) = juntar_lectores(lectores);
    let casos = casos_corridos(&completa);
    let mut salida = recortar_salida(&completa);
    // OBS-2: si el tope recorto, se DICE, y se dice cuanto quedo afuera.
    if quedo_abierto {
        salida = format!(
            "(un proceso hijo dejo el pipe abierto: se reporta lo leido hasta el corte)\n{salida}"
        );
    }
    if omitidos > 0 {
        salida = format!(
            "(... {} KB del principio omitidos por el tope de {} MB; el estado se midio sobre lo retenido)\n{salida}",
            omitidos / 1024,
            MAX_SALIDA_BYTES / (1024 * 1024)
        );
    }
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

/// Buffer compartido entre el hilo lector y quien lo espera. Compartido y no
/// devuelto al final porque el lector puede NO terminar: si el comando deja un
/// nieto vivo con el pipe heredado, no hay EOF hasta que ese nieto muera. En
/// ese caso el gate se queda con lo leido hasta el momento y sigue.
#[derive(Default)]
struct Buf {
    datos: std::collections::VecDeque<u8>,
    /// Bytes leidos en total, que puede ser mas que los retenidos.
    total: usize,
}

type Compartido = std::sync::Arc<std::sync::Mutex<Buf>>;

/// Cuanto se espera a un lector DESPUES de que el proceso termino. Pasado esto
/// se asume un descriptor heredado por un nieto y se sigue con lo que haya.
const GRACIA_LECTOR: Duration = Duration::from_secs(2);

/// Vacia un pipe HASTA EL EOF en un hilo aparte, reteniendo como mucho
/// `MAX_SALIDA_BYTES` — y reteniendo la **cola**, no la cabeza (OBS-2): los
/// resumenes que deciden el estado (`test result:`, `FAILED`) estan al final.
///
/// Esta funcion es el corazon de la feature #46. Antes se leia DESPUES de
/// esperar al proceso, y un comando que imprimia mas que el buffer del pipe
/// (~64 KB) se bloqueaba escribiendo mientras `verify` se bloqueaba
/// esperandolo: deadlock. Medido en vivo: el instalador once minutos sin
/// avanzar, con `stdout`/`stderr` en PIPE y sin un solo hijo.
fn lector<R: std::io::Read + Send + 'static>(
    mut pipe: R,
) -> (std::thread::JoinHandle<()>, Compartido) {
    let compartido: Compartido = std::sync::Arc::new(std::sync::Mutex::new(Buf::default()));
    let mio = std::sync::Arc::clone(&compartido);
    let hilo = std::thread::spawn(move || {
        let mut chunk = [0u8; 8192];
        loop {
            match pipe.read(&mut chunk) {
                Ok(0) | Err(_) => break,
                Ok(n) => {
                    let Ok(mut buf) = mio.lock() else { break };
                    buf.total += n;
                    buf.datos.extend(&chunk[..n]);
                    // `drain` en un VecDeque cuesta lo que se tira, no lo que
                    // queda: por eso el tope no vuelve cuadratica la lectura de
                    // una salida enorme.
                    if buf.datos.len() > MAX_SALIDA_BYTES {
                        let sobra = buf.datos.len() - MAX_SALIDA_BYTES;
                        buf.datos.drain(..sobra);
                    }
                }
            }
        }
    });
    (hilo, compartido)
}

/// Lanza los dos hilos lectores. Se llama ANTES de esperar al proceso: esa es
/// toda la diferencia con la version que se colgaba.
type Lectores = Vec<(std::thread::JoinHandle<()>, Compartido)>;

fn lanzar_lectores(hijo: &mut std::process::Child) -> Lectores {
    let mut out = Vec::new();
    if let Some(p) = hijo.stdout.take() {
        out.push(lector(p));
    }
    if let Some(p) = hijo.stderr.take() {
        out.push(lector(p));
    }
    out
}

/// Junta lo que leyeron los hilos —stdout primero y stderr despues, como
/// siempre, porque la leccion de la #44 depende de que el resumen quede al
/// final— **sin quedarse esperando para siempre**.
///
/// El limite existe por un caso concreto: `(sleep 30 &) ; echo listo` termina al
/// instante, pero el nieto se queda con el pipe abierto y no hay EOF. Sin
/// gracia, el gate esperaba 30 segundos con un timeout de 3: el corte existia y
/// el join lo ignoraba.
///
/// Devuelve el texto, cuantos bytes se perdieron por el tope, y si algun lector
/// quedo abierto.
fn juntar_lectores(lectores: Lectores) -> (String, usize, bool) {
    let mut texto = String::new();
    let mut omitidos = 0usize;
    let mut quedo_abierto = false;
    for (hilo, compartido) in lectores {
        let limite = Instant::now() + GRACIA_LECTOR;
        while !hilo.is_finished() && Instant::now() < limite {
            std::thread::sleep(Duration::from_millis(20));
        }
        if !hilo.is_finished() {
            quedo_abierto = true;
        }
        // Se toma una foto del buffer: si el hilo sigue vivo, es lo leido hasta
        // aca, que es mejor que nada y que esperar para siempre.
        let Ok(buf) = compartido.lock() else { continue };
        let bytes: Vec<u8> = buf.datos.iter().copied().collect();
        omitidos += buf.total.saturating_sub(bytes.len());
        texto.push_str(&String::from_utf8_lossy(&bytes));
    }
    (texto, omitidos, quedo_abierto)
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
#[cfg(test)]
pub fn render_reporte(fid: &str, stamp: &str, resultados: &[Resultado]) -> String {
    render_reporte_desde(fid, stamp, None, resultados)
}

/// Variante del reporte para una corrida real: declara el árbol que se midió.
/// Los tests puros conservan `render_reporte` para no inventar una ruta de
/// ejecución que no tuvieron.
pub fn render_reporte_desde(
    fid: &str,
    stamp: &str,
    raiz: Option<&Path>,
    resultados: &[Resultado],
) -> String {
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
    let raiz = raiz
        .map(|r| format!("Raiz de ejecucion: {}\n", r.display()))
        .unwrap_or_default();
    let mut out = format!(
        "# Verificacion de AC - Feature #{fid}\n\n\
         Corrida: {stamp}\n\
         {raiz}\
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
/// Los nombres de AC de una lista, sin repetir y conservando el orden.
///
/// Feature #73: con una fila por comando, un AC con dos comandos rojos aparece
/// dos veces. Los DOS lugares que hablan de "AC en rojo" —el mensaje de
/// `verify` y el gate del cierre, que lo lee del reporte— tienen que dar la
/// misma lista, y por eso deduplican con la misma funcion en vez de cada uno
/// con la suya. Dos implementaciones de la misma pregunta que divergen es la
/// familia de bug mas repetida de este repo (features #64, #67, #69).
pub fn sin_repetir(acs: impl IntoIterator<Item = String>) -> Vec<String> {
    acs.into_iter().fold(Vec::new(), |mut out: Vec<String>, ac| {
        if !out.contains(&ac) {
            out.push(ac);
        }
        out
    })
}

pub fn rojos_del_reporte(texto: &str) -> Vec<String> {
    let filas = texto
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
        .collect::<Vec<String>>();
    sin_repetir(filas)
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
        .filter(|v| !v.es_manual())
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

    // -----------------------------------------------------------------
    // Feature #46: el deadlock del pipe lleno.
    //
    // Todos estos tests se COLGABAN antes del arreglo (el comando se bloquea
    // escribiendo, `ejecutar` se bloquea esperandolo). Que terminen ES la
    // verificacion: por eso cada uno mide ademas su duracion.
    // -----------------------------------------------------------------

    /// ~120 KB, casi el doble del buffer tipico de un pipe (64 KB).
    const LINEAS_GRANDES: usize = 4000;

    fn corre(comando: &str) -> (Estado, Option<i32>, u128, String) {
        ejecutar(comando, Path::new("."), Duration::from_secs(60))
    }

    #[test]
    fn verify_salida_grande_stdout() {
        let cmd = format!("for i in $(seq 1 {LINEAS_GRANDES}); do echo \"linea larga de relleno numero $i\"; done");
        let (estado, code, ms, _) = corre(&cmd);
        assert_eq!(estado, Estado::Verde, "un comando verboso que sale 0 es verde");
        assert_eq!(code, Some(0));
        assert!(ms < 60_000, "termino solo, no por timeout ({ms} ms)");
    }

    #[test]
    fn verify_salida_grande_stderr() {
        let cmd = format!("for i in $(seq 1 {LINEAS_GRANDES}); do echo \"error de relleno numero $i\" >&2; done");
        let (estado, _, ms, _) = corre(&cmd);
        assert_eq!(estado, Estado::Verde);
        assert!(ms < 60_000, "termino solo ({ms} ms)");
    }

    #[test]
    fn verify_salida_grande_ambos() {
        // El caso real: el instalador escribe por los dos a la vez. Con un solo
        // lector secuencial, el segundo pipe se llena mientras se drena el
        // primero y el hijo queda bloqueado igual.
        let cmd = format!(
            "for i in $(seq 1 {LINEAS_GRANDES}); do echo \"salida $i\"; echo \"error $i\" >&2; done"
        );
        let (estado, _, ms, _) = corre(&cmd);
        assert_eq!(estado, Estado::Verde);
        assert!(ms < 60_000, "termino solo ({ms} ms)");
    }

    #[test]
    fn verify_estado_sobre_salida_completa() {
        // Leccion de la #44: el resumen que decide el estado llega al final, y
        // detras de miles de lineas. Si el estado se midiera sobre lo recortado
        // (20 lineas) esto daria Vacio.
        let cmd = format!(
            "for i in $(seq 1 {LINEAS_GRANDES}); do echo \"compilando modulo $i\"; done; \
             echo 'test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out'"
        );
        let (estado, _, _, _) = corre(&cmd);
        assert_eq!(estado, Estado::Verde, "3 casos corridos: verde, no vacio");

        // Y el detector sigue distinguiendo el 0: exit 0 sin casos es Vacio.
        let cmd_vacio = format!(
            "for i in $(seq 1 {LINEAS_GRANDES}); do echo \"compilando modulo $i\"; done; \
             echo 'test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 12 filtered out'"
        );
        let (estado, _, _, _) = corre(&cmd_vacio);
        assert_eq!(estado, Estado::Vacio, "0 casos: el filtro no matcheo nada");
    }

    #[test]
    fn verify_nieto_que_hereda_el_pipe() {
        // El unico riesgo que introduce leer con hilos: si el comando deja un
        // NIETO vivo con el pipe heredado, el lector no ve el EOF aunque el
        // hijo haya terminado, y el join se quedaria esperando.
        //
        // Los procesos detached del propio arnes (atlassian push, graphify) ya
        // redirigen sus descriptores a null, asi que no caen aca. Este test fija
        // el comportamiento para el resto.
        let arranque = Instant::now();
        let (estado, _, _, _) = ejecutar(
            "(sleep 30 &) ; echo listo",
            Path::new("."),
            Duration::from_secs(3),
        );
        let ms = arranque.elapsed().as_millis();
        assert!(
            ms < 20_000,
            "un nieto con el pipe heredado no puede colgar el gate ({ms} ms, estado {estado:?})"
        );
    }

    #[test]
    fn verify_timeout_sigue_cortando() {
        // Un comando que de verdad no termina se sigue cortando: el arreglo no
        // puede haber cambiado esto por otro cuelgue.
        let (estado, code, _, _) = ejecutar("sleep 30", Path::new("."), Duration::from_secs(1));
        assert_eq!(estado, Estado::Timeout);
        assert_eq!(code, None, "no hay codigo de salida cuando se lo mata");
    }

    #[test]
    fn verify_salida_acotada() {
        // Mas de 4 MB: se retiene la COLA y el recorte se DECLARA (OBS-1/OBS-2).
        // El comando sale 1 a proposito, porque el camino verde no guarda salida.
        let cmd = "head -c 5000000 /dev/zero | tr '\\0' 'x' | fold -w 100; exit 1";
        let (estado, _, ms, salida) = corre(cmd);
        assert_eq!(estado, Estado::Rojo);
        assert!(ms < 60_000, "termino solo ({ms} ms)");
        assert!(
            salida.contains("omitidos por el tope"),
            "el recorte por tope se declara: {}",
            &salida[..salida.len().min(200)]
        );
        assert!(
            salida.contains("el estado se midio sobre lo retenido"),
            "y se dice sobre que se midio"
        );
    }

    #[test]
    fn parse_should_take_the_command_below_its_ac() {
        let spec = "- AC-1: Given algo, When otra, Then resultado.\n  \
                    Comando: `bash tests/smoke.sh`\n\
                    - AC-2: Otro criterio.\n";
        let v = parsear(spec);
        assert_eq!(v.len(), 2);
        assert_eq!(v[0].ac, "AC-1");
        assert_eq!(v[0].comandos, vec!["bash tests/smoke.sh".to_string()]);
        assert_eq!(v[1].ac, "AC-2");
        assert!(v[1].es_manual());
    }

    #[test]
    fn parse_should_accept_the_command_without_backticks() {
        let spec = "- AC-1: algo.\n  Comando: cargo test\n";
        assert_eq!(parsear(spec)[0].comandos, vec!["cargo test".to_string()]);
    }

    /// AC-1 de la #73: un AC se queda con TODOS sus comandos, en orden.
    ///
    /// Este test se llamaba `parse_should_keep_only_the_first_command_of_an_ac`
    /// y afirmaba lo contrario. No estaba mal escrito: describia con precision
    /// lo que el codigo hacia. Lo que faltaba era preguntarse si eso era lo que
    /// tenia que hacer — el AC-8 de la #72 declaro cuatro verificaciones y
    /// quedo verde por la primera, con este test en verde al lado.
    #[test]
    fn parse_should_keep_every_command_of_an_ac_in_order() {
        let spec = "- AC-1: algo.\n  Comando: `uno`\n  Comando: `dos`\n  Comando: `tres`\n";
        let v = parsear(spec);
        assert_eq!(v.len(), 1, "sigue siendo UN criterio");
        assert_eq!(
            v[0].comandos,
            vec!["uno".to_string(), "dos".to_string(), "tres".to_string()],
            "los tres, en el orden en que estan escritos"
        );
        assert!(!v[0].es_manual());
    }

    /// Y los comandos van a SU AC: el de abajo no se lleva los de arriba.
    #[test]
    fn parse_should_not_leak_commands_into_the_next_ac() {
        let spec = "- AC-1: uno.\n  Comando: `a`\n  Comando: `b`\n\
                    - AC-2: dos.\n  Comando: `c`\n";
        let v = parsear(spec);
        assert_eq!(v.len(), 2);
        assert_eq!(v[0].comandos, vec!["a".to_string(), "b".to_string()]);
        assert_eq!(v[1].comandos, vec!["c".to_string()]);
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
        assert_eq!(v[0].comandos, vec!["true".to_string()]);
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

    // -----------------------------------------------------------------
    // Feature #68: el arnes no pierde los AC que pide revisar a mano.
    // -----------------------------------------------------------------

    // -----------------------------------------------------------------
    // Feature #69: una linea AC ilegible no desaparece en silencio.
    // -----------------------------------------------------------------

    #[test]
    fn no_hay_falsos_ilegibles() {
        // AC-3. Avisar de mas es peor que no avisar: un aviso que salta siempre
        // se deja de leer, y entonces el dia que importa nadie lo mira.
        let spec = "- AC-1: normal\n\
            - AC-4b: sufijo\n\
            - AC-11 (MANUAL): anotado\n\
            - AC-11 (lo mira una persona): anotacion larga\n\
            - ACR-1: no es un AC\n\
            - Alcance: tampoco\n\
            - AC de la feature: prosa que arranca parecido\n\
            texto suelto\n\
            | AC-1 | fila de tabla |\n";
        assert!(
            lineas_ac_ilegibles(spec).is_empty(),
            "falsos positivos: {:?}",
            lineas_ac_ilegibles(spec)
        );
        // Y las que SI son ilegibles aparecen, con su texto entero.
        let spec = "- AC-7 Given algo, When pasa\n- AC-: sin numero\n- AC-1 (sin cerrar: x\n";
        assert_eq!(
            lineas_ac_ilegibles(spec),
            vec![
                "- AC-7 Given algo, When pasa".to_string(),
                "- AC-: sin numero".to_string(),
                "- AC-1 (sin cerrar: x".to_string(),
            ]
        );
    }

    #[test]
    fn el_bloque_de_codigo_no_dispara_el_aviso() {
        // AC-5. Un spec que DOCUMENTA la forma de un AC escribe ejemplos rotos a
        // proposito. Sale gratis porque se usa el parser unico de la #67, y se
        // fija aca para que no se pierda si alguien lo toca.
        let spec = "- AC-1: real\n```\n- AC-7 Given algo, sin dos puntos\n```\n~~~\n- AC-: rota\n~~~\n";
        assert!(lineas_ac_ilegibles(spec).is_empty());
        // Fuera del bloque, la misma linea si avisa.
        assert_eq!(lineas_ac_ilegibles("- AC-7 Given algo, sin dos puntos\n").len(), 1);
    }

    #[test]
    fn el_corpus_real_no_tiene_ilegibles() {
        // AC-4. Medido antes de escribir el spec: el arreglo no cambia nada de
        // lo que existe, se pone en medio del proximo typo. Si manana alguien
        // escribe uno, este test lo agarra en la corrida siguiente.
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../docs");
        let Ok(entradas) = std::fs::read_dir(&dir) else {
            return;
        };
        let mut specs = 0usize;
        let mut malas: Vec<String> = Vec::new();
        for entrada in entradas.flatten() {
            let path = entrada.path();
            let nombre = path.file_name().unwrap_or_default().to_string_lossy().to_string();
            if !nombre.starts_with("spec-feature-") {
                continue;
            }
            let Ok(texto) = std::fs::read_to_string(&path) else {
                continue;
            };
            specs += 1;
            for l in lineas_ac_ilegibles(&texto) {
                malas.push(format!("{nombre}: {l}"));
            }
        }
        assert!(specs >= 20, "esperaba el corpus real, lei {specs} specs");
        assert!(malas.is_empty(), "hay AC ilegibles en specs reales: {malas:#?}");
    }

    #[test]
    fn el_sufijo_de_letra_es_un_ac_propio() {
        // AC-3. `AC-4b` es OTRO criterio que `AC-4`, no una nota al pie: son
        // tres AC reales de los specs #16 y #51 que el arnes venia tirando.
        let v = parsear("- AC-4: cuatro\n- AC-4b: cuatro bis\n- AC-12c: doce ce\n");
        let nombres: Vec<&str> = v.iter().map(|x| x.ac.as_str()).collect();
        assert_eq!(nombres, vec!["AC-4", "AC-4b", "AC-12c"]);
    }

    #[test]
    fn la_anotacion_no_entra_en_el_nombre() {
        // `- AC-11 (MANUAL):` es `AC-11`, no `AC-11 (MANUAL)`: el review lo cita
        // por su numero, y el gate compara el nombre token a token.
        let v = parsear("- AC-11 (MANUAL): audita esto\n");
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].ac, "AC-11");
        assert!(v[0].es_manual(), "un AC manual no declara comando");
    }

    #[test]
    fn el_comando_no_migra_al_ac_anterior() {
        // AC-4. El daño peor, y era LATENTE: `parsear` cuelga cada `Comando:`
        // del ultimo AC abierto, asi que el comando de un AC ilegible se le
        // adjudicaba al de arriba. `verify` habria impreso "AC-1 verde" tras
        // correr la prueba de otro criterio.
        //
        // Ahora la forma `(MANUAL)` se entiende, asi que el comando queda donde
        // corresponde.
        let v = parsear("- AC-1: uno\n- AC-2 (MANUAL): dos\n  Comando: `mio`\n");
        assert_eq!(v.len(), 2);
        assert!(v[0].es_manual(), "AC-1 se quedo con el comando de AC-2");
        assert_eq!(v[1].comandos, vec!["mio".to_string()]);

        // Y con una linea que ARRANCA como AC pero es ilegible de verdad, el
        // comando no se le cuelga a nadie: vale mas perderlo que adjudicarselo
        // al criterio equivocado.
        let v = parsear("- AC-1: uno\n- AC-: rota\n  Comando: `ajeno`\n");
        assert_eq!(v.len(), 1);
        assert!(
            v[0].es_manual(),
            "el comando de una linea ilegible se le colgo a AC-1"
        );
    }

    #[test]
    fn lo_que_no_es_un_ac_sigue_sin_serlo() {
        // AC-5. Aflojar el parser no puede empezar a comerse prosa: un parser
        // que inventa un AC es peor que uno que pierde uno, porque el que
        // inventa hace fallar cierres que estaban bien.
        for linea in [
            "- AC-12 y AC-13: dos de una",
            "- AC-: sin numero",
            "- AC-1 sin dos puntos",
            "- ACR-1: otra cosa",
            "- AC-1 (sin cerrar: parentesis abierto",
            "- AC 1: con espacio",
            "-  AC-1: doble espacio",
            "- AC-1b2: letra y numero",
            "  - AC-1: sangria de lista",
            "- AC-1 (MANUAL) : espacio antes de los dos puntos",
        ] {
            assert!(
                ac_de(linea).is_none(),
                "{linea:?} no es un AC y el parser lo acepto"
            );
        }
        // Y las formas que SI son AC siguen siendolo.
        for (linea, esperado) in [
            ("- AC-1: normal", "AC-1"),
            ("- AC-12: dos digitos", "AC-12"),
            ("- AC-4b: sufijo", "AC-4b"),
            ("- AC-11 (MANUAL): anotado", "AC-11"),
            ("- AC-11 (lo mira una persona): anotacion larga", "AC-11"),
        ] {
            assert_eq!(ac_de(linea).as_deref(), Some(esperado), "{linea:?}");
        }
    }

    #[test]
    fn los_siete_que_faltaban_y_ninguno_mas() {
        // AC-6. Sobre los specs REALES: el arreglo tiene que traer exactamente
        // los AC medidos y ninguno inventado. Eran SIETE cuando se midio para
        // escribir el spec; son ocho porque el spec de esta feature agrego el
        // suyo al corpus, que es justo lo que el AC-8 dice que tiene que pasar. Se asserta la DIFERENCIA
        // nombrada, no el total: el total sube con cada spec nuevo y un assert
        // sobre el seria un detector-de-cambios.
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../docs");
        let Ok(entradas) = std::fs::read_dir(&dir) else {
            return; // sin docs/ en el sandbox de build
        };
        // `ac_de` como era antes de la #68: los dos puntos PEGADOS al numero.
        fn ac_de_viejo(linea: &str) -> Option<String> {
            let resto = linea.strip_prefix("- AC-")?;
            let numero: String = resto.chars().take_while(char::is_ascii_digit).collect();
            if numero.is_empty() || !resto[numero.len()..].starts_with(':') {
                return None;
            }
            Some(format!("AC-{numero}"))
        }
        let mut nuevos: Vec<String> = Vec::new();
        let mut specs = 0usize;
        for entrada in entradas.flatten() {
            let path = entrada.path();
            let nombre = path.file_name().unwrap_or_default().to_string_lossy().to_string();
            if !nombre.starts_with("spec-feature-") {
                continue;
            }
            let Ok(texto) = std::fs::read_to_string(&path) else {
                continue;
            };
            specs += 1;
            for linea in crate::markdown::lineas_fuera_de_bloque(&texto) {
                let t = linea.trim();
                if let Some(ac) = ac_de(t)
                    && ac_de_viejo(t).is_none()
                {
                    nuevos.push(format!("{}:{ac}", nombre.trim_start_matches("spec-feature-")));
                }
            }
        }
        assert!(specs >= 20, "esperaba el corpus real, lei {specs} specs");
        nuevos.sort();
        assert_eq!(
            nuevos,
            vec![
                "16-atlassian-auto-push.md:AC-4b",
                "51-revision-adversarial-y-modelos-por-rol.md:AC-12b",
                "51-revision-adversarial-y-modelos-por-rol.md:AC-12c",
                "64-el-arnes-no-promete-enforcement-que-no-hace.md:AC-12",
                "65-el-arnes-cierra-lo-resuelto-aguas-arriba.md:AC-11",
                "66-el-stop-hook-no-entra-en-bucle.md:AC-13",
                "67-los-dos-parsers-del-review-no-se-contradicen.md:AC-11",
                // El octavo es el AC-8 de ESTA feature, escrito a proposito con
                // la forma que desaparecia. Que aparezca aca es el AC-8
                // cumpliendose: la feature se prueba sobre si misma, y si el
                // arreglo se revierte, este AC se vuelve a perder.
                "68-el-arnes-no-pierde-los-ac-que-pide-revisar-a-man.md:AC-8",
                // Y el noveno es el AC-8 (MANUAL) de la #71, escrito con la
                // misma forma sin querer probar nada: es la evidencia de que el
                // arreglo de la #68 sigue haciendo falta en el uso normal.
                "71-el-close-archiva-el-sello-de-cierre-en-el-worktr.md:AC-8",
                "73-verify-corre-un-comando-por-ac-y-no-lo-dice-un-a.md:AC-9",
            ],
            "el arreglo trae AC distintos de los medidos en el corpus real"
        );
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
            // Feature #73: se comparan COMANDOS contra COMANDOS. Antes se
            // comparaban AC-con-comando contra lineas `Comando:`, que solo
            // coincide si ningun AC declara mas de una — justo lo que la
            // feature vino a permitir.
            let comandos: usize = hallados.iter().map(|v| v.comandos.len()).sum();
            let declarados = declaraciones_fuera_de_bloques(&texto);
            assert_eq!(
                comandos,
                declarados,
                "{}: el parser reporto {comandos} comando(s) y el spec declara {declarados}",
                path.display()
            );
            // Y ningun comando sale vacio: un `Comando:` sin nada detras seria
            // un AC que dice verificarse y no verifica nada.
            for v in &hallados {
                for c in &v.comandos {
                    assert!(!c.trim().is_empty(), "{}: {} con comando vacio", path.display(), v.ac);
                }
            }
        }
        assert!(specs >= 1, "no se leyo ningun spec real");
        assert!(acs > 100, "esperaba cientos de AC reales, encontre {acs}");
    }

    /// Cuenta las lineas `Comando:` que estan fuera de un bloque — las que el
    /// parser tiene que ver. El conteo de AC se hace por un camino distinto al
    /// de `parsear` a proposito; la CLASIFICACION de bloques, en cambio, sale
    /// del mismo lugar (feature #67).
    ///
    /// Antes este cross-check tenia su propia copia, que —como `parsear`— solo
    /// conocia ```` ``` ````. Las dos compartian el punto ciego de `~~~`, asi
    /// que su acuerdo sobre 20+ specs NO significaba lo que este comentario
    /// decia: dos instrumentos mal calibrados de la misma forma coinciden
    /// perfectamente y no miden nada.
    fn declaraciones_fuera_de_bloques(texto: &str) -> usize {
        let mut n = 0usize;
        let mut ac_abierto = false;
        for linea in crate::markdown::lineas_fuera_de_bloque(texto) {
            let t = linea.trim();
            if t.starts_with("- AC-") {
                ac_abierto = true;
            } else if t.starts_with("Comando:") && ac_abierto {
                // Feature #73: se cuentan TODOS. Esta linea decia
                // `ac_abierto = false; // solo el primero cuenta, como en parsear`
                // — o sea que el oraculo estaba escrito para imitar el bug. El
                // test prometia "el parser no inventa ni pierde comandos" y
                // contaba solo el primero, asi que pasaba en verde mientras el
                // AC-8 de la #72 perdia tres. Un oraculo que copia a la
                // implementacion no verifica: la acompaña.
                n += 1;
            }
        }
        n
    }

    /// `parsear` como era antes de la #67: togglea solo con ```` ``` ````, no
    /// conoce `~~~`. Se conserva para poder medir la diferencia sobre documentos
    /// reales, que es la unica forma de saber si el arreglo CAMBIA algo o solo
    /// cierra un agujero que nadie habia pisado todavia.
    fn acs_con_el_parser_viejo(spec: &str) -> Vec<String> {
        let mut out = Vec::new();
        let mut dentro = false;
        for linea in spec.lines() {
            if linea.trim_start().starts_with("```") { // PARSER-VIEJO-A-PROPOSITO
                dentro = !dentro;
                continue;
            }
            if dentro {
                continue;
            }
            if let Some(ac) = ac_de(linea.trim()) {
                out.push(ac);
            }
        }
        out
    }

    #[test]
    fn corpus_real_sin_cambios() {
        // AC-5: sobre los documentos REALES del repo, el parser unico da
        // exactamente los mismos AC que el viejo. O sea: el arreglo de `~~~` no
        // es un cambio de comportamiento sobre lo que ya existe, es un agujero
        // que se cierra antes de que alguien lo pise.
        //
        // Se asserta la DIFERENCIA (cero) y no el total (733 hoy): el total sube
        // con cada spec nuevo y un assert sobre el seria un detector-de-cambios,
        // que es como murio la primera version del test de al lado.
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../docs");
        let Ok(entradas) = std::fs::read_dir(&dir) else {
            return; // sin docs/ en el sandbox de build: nada que comprobar
        };
        let mut documentos = 0usize;
        let mut acs = 0usize;
        let mut difieren: Vec<String> = Vec::new();
        for entrada in entradas.flatten() {
            let path = entrada.path();
            let nombre = path.file_name().unwrap_or_default().to_string_lossy().to_string();
            if !(nombre.starts_with("spec-feature-") || nombre.starts_with("review-")) {
                continue;
            }
            let Ok(texto) = std::fs::read_to_string(&path) else {
                continue;
            };
            documentos += 1;
            let nuevos: Vec<String> = parsear(&texto).into_iter().map(|v| v.ac).collect();
            acs += nuevos.len();
            let viejos = acs_con_el_parser_viejo(&texto);
            if nuevos != viejos {
                difieren.push(format!(
                    "{nombre}: viejo={} nuevo={}",
                    viejos.len(),
                    nuevos.len()
                ));
            }
        }
        assert!(documentos >= 20, "esperaba el corpus real, lei {documentos} documentos");
        assert!(acs > 300, "esperaba cientos de AC reales, encontre {acs}");
        assert!(
            difieren.is_empty(),
            "el parser unico cambia lo que se lee de documentos reales: {difieren:?}"
        );
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
