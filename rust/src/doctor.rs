//! Diagnostico de la INSTALACION (feature #25).
//!
//! La pregunta que contesta es distinta a la de `harness_check.sh`: ese mira el
//! **proceso** (spec aprobado, plan fresco, PRDs, lecciones, perfil,
//! convenciones); este mira si el arnes **esta bien instalado** (binario, hooks,
//! superficies, marker, hub, herramientas, graphify). No repiten ni un chequeo
//! (AC-14, decision del usuario 2026-08-17, OBS-2).
//!
//! Ninguna de las siete areas se invento: todas diagnostican algo que ya rompio
//! en este repo. Binario viejo tras `git pull` (hubo que parchear
//! `harness_check.sh` a mano, dos veces), marker perdido (feature #10 entera),
//! checkout fuente confundido con instalacion (feature #7), hub caido (toda una
//! sesion de trabajo).
//!
//! **`diagnosticar()` es pura**: lee el filesystem y el entorno, y devuelve los
//! hallazgos. No imprime y no escribe — el modulo no importa nada que pueda
//! escribir, asi que la promesa del AC-15 la sostiene la estructura y no la
//! disciplina (leccion `promesas-estructurales-vs-disciplina`).

use std::path::{Path, PathBuf};

use crate::paths::HarnessPaths;

/// Como quedo un area revisada.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Estado {
    Ok,
    /// Impide trabajar. Es lo unico que cambia el exit code (OBS-4).
    Falla,
    /// Funciona igual, pero conviene saberlo.
    Aviso,
    /// No corresponde en este contexto (tipicamente: el checkout fuente).
    NoAplica,
}

impl Estado {
    pub fn etiqueta(self) -> &'static str {
        match self {
            Estado::Ok => "ok",
            Estado::Falla => "falla",
            Estado::Aviso => "aviso",
            Estado::NoAplica => "no_aplica",
        }
    }

    pub fn simbolo(self) -> &'static str {
        match self {
            Estado::Ok => "[ok]",
            Estado::Falla => "[!!]",
            Estado::Aviso => "[i] ",
            Estado::NoAplica => "[--]",
        }
    }

    /// Solo `Falla` bloquea: un hub caido no puede hacer mentir al exit code.
    pub fn bloquea(self) -> bool {
        matches!(self, Estado::Falla)
    }
}

/// Las siete areas del hito 3 del PRD-master.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Area {
    Binario,
    Hooks,
    Superficies,
    Marker,
    Hub,
    Herramientas,
    Graphify,
    /// Feature #26: si la proteccion de rutas esta activa y cuantas cubre.
    /// **No** revisa violaciones: eso es de `harness_check.sh` (AC-14 de la #25).
    RutasProtegidas,
}

impl Area {
    pub fn etiqueta(self) -> &'static str {
        match self {
            Area::Binario => "binario",
            Area::Hooks => "hooks",
            Area::Superficies => "superficies",
            Area::Marker => "marker",
            Area::Hub => "hub",
            Area::Herramientas => "herramientas",
            Area::Graphify => "graphify",
            Area::RutasProtegidas => "rutas_protegidas",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Hallazgo {
    pub area: Area,
    pub estado: Estado,
    pub detalle: String,
    /// Comando ejecutable tal cual, no una frase (AC-2).
    pub remedio: Option<String>,
}

impl Hallazgo {
    fn ok(area: Area, detalle: impl Into<String>) -> Self {
        Hallazgo { area, estado: Estado::Ok, detalle: detalle.into(), remedio: None }
    }

    fn falla(area: Area, detalle: impl Into<String>, remedio: impl Into<String>) -> Self {
        Hallazgo {
            area,
            estado: Estado::Falla,
            detalle: detalle.into(),
            remedio: Some(remedio.into()),
        }
    }

    fn aviso(area: Area, detalle: impl Into<String>, remedio: impl Into<String>) -> Self {
        Hallazgo {
            area,
            estado: Estado::Aviso,
            detalle: detalle.into(),
            remedio: Some(remedio.into()),
        }
    }

    fn no_aplica(area: Area, detalle: impl Into<String>) -> Self {
        Hallazgo { area, estado: Estado::NoAplica, detalle: detalle.into(), remedio: None }
    }
}

/// El remedio que arregla casi todo: volver a correr el instalador. Es
/// idempotente, asi que un falso positivo aca cuesta poco.
const REINSTALAR: &str = "bash setup_harness.sh";

/// Revisa las siete areas. **Solo lee.**
pub fn diagnosticar(paths: &HarnessPaths) -> Vec<Hallazgo> {
    let fuente = es_checkout_fuente(paths);
    vec![
        revisar_binario(paths),
        revisar_marker(paths),
        revisar_hooks(paths, fuente),
        revisar_superficies(paths, fuente),
        revisar_hub(paths),
        revisar_herramientas(paths),
        revisar_graphify(),
        revisar_rutas_protegidas(paths),
    ]
}

/// Informa el ESTADO de la proteccion, no sus violaciones: cuantas rutas cubre
/// o si esta apagada. Las violaciones las reporta `harness_check.sh`, y
/// duplicarlas seria dos herramientas diciendo lo mismo (#25, AC-14).
fn revisar_rutas_protegidas(paths: &HarnessPaths) -> Hallazgo {
    let data = crate::features::load_features(paths).unwrap_or(serde_json::Value::Null);
    let patrones = crate::rutas::patrones(&data);
    if patrones.is_empty() {
        return Hallazgo::aviso(
            Area::RutasProtegidas,
            "apagadas: `rules.rutas_protegidas` es una lista vacia, asi que ningun documento del usuario esta protegido",
            "quita esa clave de feature_list.json para volver a los defaults",
        );
    }
    Hallazgo::ok(
        Area::RutasProtegidas,
        format!(
            "{} ruta(s) protegidas: {}. Las violaciones las reporta harness_check.sh",
            patrones.len(),
            patrones.join(", ")
        ),
    )
}

/// El checkout FUENTE del arnes: `templates/harness_cli` + `rust/` en el propio
/// dir, y el padre sin huella de instalacion. Es el mismo criterio del guardrail
/// de la feature #7; aca se reusa el resultado de la resolucion en vez de
/// recalcularlo, para que doctor y el comportamiento real nunca se separen.
pub fn es_checkout_fuente(paths: &HarnessPaths) -> bool {
    paths.root.join("templates/harness_cli").is_file()
        && paths.root.join("rust").is_dir()
        && paths.repo_root == paths.root
}

fn nombre_binario() -> &'static str {
    if cfg!(windows) { "harness.exe" } else { "harness" }
}

fn mtime(path: &Path) -> Option<std::time::SystemTime> {
    std::fs::metadata(path).and_then(|m| m.modified()).ok()
}

fn ejecutable(path: &Path) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::metadata(path)
            .map(|m| m.permissions().mode() & 0o111 != 0)
            .unwrap_or(false)
    }
    #[cfg(not(unix))]
    {
        path.is_file()
    }
}

/// El caso que ya rompio dos veces: `git pull` deja los scripts nuevos y el
/// binario viejo, y el sintoma aparece tres pasos despues con otro nombre
/// (`unrecognized subcommand 'perfil'`).
fn revisar_binario(paths: &HarnessPaths) -> Hallazgo {
    let bin = paths.root.join(nombre_binario());
    if !bin.is_file() {
        return Hallazgo::falla(
            Area::Binario,
            format!("no existe {}", bin.display()),
            REINSTALAR,
        );
    }
    if !ejecutable(&bin) {
        return Hallazgo::falla(
            Area::Binario,
            format!("{} no es ejecutable", bin.display()),
            format!("chmod +x {}", bin.display()),
        );
    }
    let Some(m_bin) = mtime(&bin) else {
        return Hallazgo::ok(Area::Binario, format!("{} presente", bin.display()));
    };
    // Contra los scripts que lo invocan: son los que `git pull` actualiza.
    let mas_nuevos: Vec<String> = ["harness_cli", "harness_check.sh", "setup_harness.sh"]
        .iter()
        .filter_map(|nombre| {
            let script = paths.root.join(nombre);
            mtime(&script).filter(|m| *m > m_bin).map(|_| (*nombre).to_string())
        })
        .collect();
    if !mas_nuevos.is_empty() {
        return Hallazgo::falla(
            Area::Binario,
            format!(
                "el binario es mas viejo que {}: tipico de `git pull` sin re-correr el instalador",
                mas_nuevos.join(", ")
            ),
            REINSTALAR,
        );
    }
    Hallazgo::ok(
        Area::Binario,
        format!("{} presente, ejecutable y al dia", bin.display()),
    )
}

/// Lo que costo la feature #10: saber QUE raiz se resolvio y POR QUE.
fn revisar_marker(paths: &HarnessPaths) -> Hallazgo {
    let marker = paths.root.join(".harness_layout");
    let raiz = format!("raiz resuelta: {}", paths.repo_root.display());
    if !marker.is_file() {
        if paths.repo_root == paths.root {
            return Hallazgo::aviso(
                Area::Marker,
                format!(".harness_layout ausente; sin huella en el padre, {raiz}"),
                REINSTALAR,
            );
        }
        return Hallazgo::aviso(
            Area::Marker,
            format!(".harness_layout ausente; layout subdir inferido por la huella del padre, {raiz}"),
            REINSTALAR,
        );
    }
    let valor = std::fs::read_to_string(&marker).unwrap_or_default().trim().to_string();
    if valor == "subdir" && paths.repo_root == paths.root {
        // Marker subdir pero la raiz quedo en el propio dir: el guardrail de la
        // feature #7 actuo. Es correcto, y decirlo evita que parezca un bug.
        return Hallazgo::ok(
            Area::Marker,
            format!("marker 'subdir' con guardrail de checkout fuente aplicado, {raiz}"),
        );
    }
    Hallazgo::ok(Area::Marker, format!("marker '{valor}', {raiz}"))
}

/// Un backend "esta instalado" si su huella esta en la raiz. Solo entonces se le
/// exige su hook: pedirle hooks de Gemini a quien no usa Gemini es ruido, y el
/// ruido hunde la herramienta (leccion `probar-contra-datos-reales`).
const BACKENDS: [(&str, &str, &str); 4] = [
    ("claude", ".claude/settings.json", "CLAUDE.md"),
    ("codex", ".codex/hooks.json", "AGENTS.md"),
    ("gemini", ".gemini/settings.json", "GEMINI.md"),
    ("grok", ".grok/hooks/harness.sh", ".grok/GROK.md"),
];

fn revisar_hooks(paths: &HarnessPaths, fuente: bool) -> Hallazgo {
    if fuente {
        return Hallazgo::no_aplica(
            Area::Hooks,
            "checkout fuente del arnes: aca no se instalan hooks (y no deben instalarse)",
        );
    }
    let instalados: Vec<&str> = BACKENDS
        .iter()
        .filter(|(_, huella, _)| paths.repo_root.join(huella).exists())
        .map(|(nombre, _, _)| *nombre)
        .collect();
    if instalados.is_empty() {
        return Hallazgo::no_aplica(Area::Hooks, "ningun backend con hooks instalado");
    }
    // Que el hook APUNTE al runtime, no solo que el runtime exista: un
    // `settings.json` que quedo apuntando a otra ruta pasaba desapercibido
    // (deuda anotada en impl-25, pagada en la #36).
    let mal_apuntados: Vec<String> = BACKENDS
        .iter()
        .filter(|(nombre, huella, _)| {
            instalados.contains(nombre) && !apunta_al_runtime(&paths.repo_root.join(huella))
        })
        .map(|(nombre, huella, _)| format!("{nombre} ({huella})"))
        .collect();
    if !mal_apuntados.is_empty() {
        return Hallazgo::falla(
            Area::Hooks,
            format!(
                "hook(s) que no apuntan a bin/harness-hook: {}",
                mal_apuntados.join(", ")
            ),
            REINSTALAR,
        );
    }
    // El runtime al que todos los hooks apuntan.
    let runtime = paths.repo_root.join("bin/harness-hook");
    if !runtime.is_file() {
        return Hallazgo::falla(
            Area::Hooks,
            format!(
                "{} instalado(s) pero falta {}",
                instalados.join(", "),
                runtime.display()
            ),
            REINSTALAR,
        );
    }
    if !ejecutable(&runtime) {
        return Hallazgo::falla(
            Area::Hooks,
            format!("{} no es ejecutable", runtime.display()),
            format!("chmod +x {}", runtime.display()),
        );
    }
    Hallazgo::ok(
        Area::Hooks,
        format!("{} -> bin/harness-hook presente y ejecutable", instalados.join(", ")),
    )
}

/// Un archivo de configuracion de hooks "apunta bien" si menciona el runtime
/// del arnes. Se mira el texto y no se parsea JSON/TOML: los cinco backends
/// usan formatos distintos y lo unico que importa es si el arnes esta cableado.
fn apunta_al_runtime(config: &Path) -> bool {
    let Ok(texto) = std::fs::read_to_string(config) else {
        return false;
    };
    texto.contains("harness-hook")
        || texto.contains("harness_cli")
        || texto.contains("harness_check.sh")
}

fn revisar_superficies(paths: &HarnessPaths, fuente: bool) -> Hallazgo {
    if fuente {
        return Hallazgo::no_aplica(
            Area::Superficies,
            "checkout fuente del arnes: las superficies se generan al instalar, no viven aca",
        );
    }
    let faltan: Vec<String> = BACKENDS
        .iter()
        .filter(|(_, huella, _)| paths.repo_root.join(huella).exists())
        .filter(|(_, _, superficie)| !paths.repo_root.join(superficie).exists())
        .map(|(nombre, _, superficie)| format!("{superficie} ({nombre})"))
        .collect();
    if !faltan.is_empty() {
        return Hallazgo::falla(
            Area::Superficies,
            format!("falta la superficie de un backend instalado: {}", faltan.join(", ")),
            REINSTALAR,
        );
    }
    let presentes: Vec<&str> = BACKENDS
        .iter()
        .filter(|(_, _, superficie)| paths.repo_root.join(superficie).exists())
        .map(|(_, _, superficie)| *superficie)
        .collect();
    if presentes.is_empty() {
        return Hallazgo::no_aplica(Area::Superficies, "ningun backend instalado");
    }
    Hallazgo::ok(Area::Superficies, format!("presentes: {}", presentes.join(", ")))
}

/// Siempre aviso, nunca falla (OBS-4): todo el aprendizaje del arnes funciona
/// con el hub caido, y una sesion entera de trabajo lo demostro.
fn revisar_hub(paths: &HarnessPaths) -> Hallazgo {
    let hub_dir = std::env::var("HARNESS_HUB")
        .ok()
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .or_else(|| dirs_home().map(|h| h.join(".harness-hub")))
        .unwrap_or_else(|| paths.root.join(".harness-hub"));
    let Some((host, port)) = hub_config(&hub_dir) else {
        return Hallazgo::aviso(
            Area::Hub,
            "sin configurar (DB_HOST/DB_USER/DB_PASSWORD): lecciones, perfil, buscar y journey funcionan igual",
            format!("edita {}/.env", hub_dir.display()),
        );
    };
    match alcanzable(&host, port) {
        // Precision deliberada, y salio de una corrida real: durante toda la
        // sesion en que se escribio esta feature el hub aceptaba TCP y aun asi
        // las operaciones morian con "Connection reset by peer". Decir
        // "alcanzable" habria sido un OK falso, que es peor que no chequear:
        // el usuario descarta el hub como causa y busca en otro lado.
        true => Hallazgo::ok(
            Area::Hub,
            format!(
                "{host}:{port} acepta conexiones TCP (doctor no valida el handshake de PostgreSQL: si un comando falla con 'connection reset' o 'timed out', el problema esta mas adentro)"
            ),
        ),
        false => Hallazgo::aviso(
            Area::Hub,
            format!("{host}:{port} no acepta conexiones; el arnes sigue funcionando sin el (lecciones, perfil, buscar y journey son archivos)"),
            format!("verifica la red o {}/.env", hub_dir.display()),
        ),
    }
}

fn dirs_home() -> Option<PathBuf> {
    std::env::var("HOME")
        .ok()
        .or_else(|| std::env::var("USERPROFILE").ok())
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
}

/// Lee la config del hub con la MISMA precedencia que `GraphMemoryManager`: el
/// entorno gana sobre el `.env`.
fn hub_config(hub_dir: &Path) -> Option<(String, u16)> {
    let archivo = hub_dir.join(".env");
    let texto = std::fs::read_to_string(&archivo).unwrap_or_default();
    let del_archivo = |clave: &str| -> Option<String> {
        texto.lines().find_map(|l| {
            let l = l.trim();
            if l.starts_with('#') {
                return None;
            }
            let (k, v) = l.split_once('=')?;
            (k.trim() == clave)
                .then(|| v.trim().trim_matches(|c| c == '\'' || c == '"').to_string())
        })
    };
    let leer = |clave: &str| -> Option<String> {
        std::env::var(clave)
            .ok()
            .filter(|v| !v.is_empty())
            .or_else(|| del_archivo(clave).filter(|v| !v.is_empty()))
    };
    let host = leer("DB_HOST")?;
    leer("DB_USER")?;
    leer("DB_PASSWORD")?;
    let port = leer("DB_PORT").and_then(|p| p.parse().ok()).unwrap_or(5432);
    Some((host, port))
}

/// TCP con timeout corto: alcanza para "responde o no" y no cuelga el comando.
/// Un doctor que tarda medio minuto porque el hub esta caido no lo corre nadie.
fn alcanzable(host: &str, port: u16) -> bool {
    use std::net::ToSocketAddrs;
    let Ok(mut addrs) = (host, port).to_socket_addrs() else {
        return false;
    };
    addrs.any(|addr| {
        std::net::TcpStream::connect_timeout(&addr, std::time::Duration::from_secs(2)).is_ok()
    })
}

/// Requeridas contra opcionales. `cargo` solo se exige donde hay `rust/`: en una
/// instalacion normal el binario ya viene compilado.
fn revisar_herramientas(paths: &HarnessPaths) -> Hallazgo {
    let mut requeridas: Vec<&str> = vec!["git"];
    if paths.root.join("rust").is_dir() {
        requeridas.push("cargo");
    }
    let faltan: Vec<&str> = requeridas.iter().copied().filter(|t| !en_path(t)).collect();
    if !faltan.is_empty() {
        return Hallazgo::falla(
            Area::Herramientas,
            format!("falta(n) en el PATH: {}", faltan.join(", ")),
            format!("instala {} y volve a correr doctor", faltan.join(" ")),
        );
    }
    let opcionales: Vec<&str> = ["curl", "kimi", "uv", "pipx"]
        .iter()
        .copied()
        .filter(|t| !en_path(t))
        .collect();
    if !opcionales.is_empty() {
        return Hallazgo::aviso(
            Area::Herramientas,
            format!(
                "requeridas ok ({}); opcionales ausentes: {}",
                requeridas.join(", "),
                opcionales.join(", ")
            ),
            "solo hacen falta para las capacidades que las usan".to_string(),
        );
    }
    Hallazgo::ok(
        Area::Herramientas,
        format!("requeridas y opcionales presentes ({})", requeridas.join(", ")),
    )
}

fn revisar_graphify() -> Hallazgo {
    if en_path("graphify") {
        return Hallazgo::ok(Area::Graphify, "graphify en el PATH");
    }
    Hallazgo::aviso(
        Area::Graphify,
        "graphify no esta en el PATH: el arnes funciona igual, sin grafo de conocimiento",
        "bash setup_harness.sh --with-graphify",
    )
}

fn en_path(programa: &str) -> bool {
    let Some(path) = std::env::var_os("PATH") else {
        return false;
    };
    std::env::split_paths(&path).any(|dir| {
        let cand = dir.join(programa);
        cand.is_file() && ejecutable(&cand)
    })
}

/// Exit code segun los hallazgos (AC-3): 2 solo si algo impide trabajar.
pub fn exit_code(hallazgos: &[Hallazgo]) -> i32 {
    if hallazgos.iter().any(|h| h.estado.bloquea()) { 2 } else { 0 }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;

    fn paths_en(dir: &Path) -> HarnessPaths {
        let harness = dir.join("hp");
        std::fs::create_dir_all(&harness).unwrap();
        HarnessPaths::from_root(harness)
    }

    fn sembrar_binario(paths: &HarnessPaths) -> PathBuf {
        let bin = paths.root.join(nombre_binario());
        std::fs::write(&bin, "#!/bin/sh\n").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&bin, std::fs::Permissions::from_mode(0o755)).unwrap();
        }
        bin
    }

    #[test]
    fn estado_should_only_block_on_falla() {
        assert!(Estado::Falla.bloquea());
        assert!(!Estado::Aviso.bloquea());
        assert!(!Estado::Ok.bloquea());
        assert!(!Estado::NoAplica.bloquea());
    }

    #[test]
    fn diagnosticar_should_cover_the_seven_areas_exactly_once() {
        let dir = tempfile::tempdir().unwrap();
        let paths = paths_en(dir.path());
        let hallazgos = diagnosticar(&paths);
        for area in [
            Area::Binario,
            Area::Hooks,
            Area::Superficies,
            Area::Marker,
            Area::Hub,
            Area::Herramientas,
            Area::Graphify,
            Area::RutasProtegidas,
        ] {
            assert_eq!(
                hallazgos.iter().filter(|h| h.area == area).count(),
                1,
                "{} no aparece exactamente una vez",
                area.etiqueta()
            );
        }
    }

    #[test]
    fn every_problem_should_carry_a_remedy() {
        // AC-2: la falla sin remedio es una queja, no un diagnostico.
        let dir = tempfile::tempdir().unwrap();
        let paths = paths_en(dir.path()); // sin binario: al menos una falla
        for h in diagnosticar(&paths) {
            match h.estado {
                Estado::Falla | Estado::Aviso => assert!(
                    h.remedio.as_deref().is_some_and(|r| !r.trim().is_empty()),
                    "{} sin remedio: {}",
                    h.area.etiqueta(),
                    h.detalle
                ),
                Estado::Ok | Estado::NoAplica => assert!(h.remedio.is_none()),
            }
        }
    }

    #[test]
    fn missing_binary_should_be_a_failure() {
        let dir = tempfile::tempdir().unwrap();
        let paths = paths_en(dir.path());
        let h = revisar_binario(&paths);
        assert_eq!(h.estado, Estado::Falla);
        assert_eq!(h.remedio.as_deref(), Some(REINSTALAR));
    }

    #[test]
    fn a_binary_older_than_the_scripts_should_be_a_failure() {
        // El caso real: `git pull` deja scripts nuevos y binario viejo.
        let dir = tempfile::tempdir().unwrap();
        let paths = paths_en(dir.path());
        let bin = sembrar_binario(&paths);
        let viejo = filetime::FileTime::from_unix_time(1_600_000_000, 0);
        filetime::set_file_mtime(&bin, viejo).unwrap();
        std::fs::write(paths.root.join("harness_cli"), "#!/bin/sh\n").unwrap();
        let h = revisar_binario(&paths);
        assert_eq!(h.estado, Estado::Falla, "{}", h.detalle);
        assert!(h.detalle.contains("git pull"), "{}", h.detalle);
        assert_eq!(h.remedio.as_deref(), Some(REINSTALAR));
    }

    #[test]
    fn a_binary_newer_than_the_scripts_should_be_ok() {
        let dir = tempfile::tempdir().unwrap();
        let paths = paths_en(dir.path());
        std::fs::write(paths.root.join("harness_cli"), "#!/bin/sh\n").unwrap();
        let bin = sembrar_binario(&paths);
        let nuevo = filetime::FileTime::from_unix_time(2_000_000_000, 0);
        filetime::set_file_mtime(&bin, nuevo).unwrap();
        assert_eq!(revisar_binario(&paths).estado, Estado::Ok);
    }

    #[test]
    fn hooks_should_fail_when_the_runtime_is_missing() {
        let dir = tempfile::tempdir().unwrap();
        let paths = paths_en(dir.path());
        std::fs::create_dir_all(paths.repo_root.join(".claude")).unwrap();
        std::fs::write(paths.repo_root.join(".claude/settings.json"), "{}").unwrap();
        let h = revisar_hooks(&paths, false);
        assert_eq!(h.estado, Estado::Falla, "{}", h.detalle);
        assert!(h.detalle.contains("claude"), "{}", h.detalle);
        assert!(h.detalle.contains("bin/harness-hook"), "{}", h.detalle);
    }

    #[test]
    fn doctor_should_detect_a_hook_pointing_to_another_path() {
        // Deuda de impl-25: hasta la #36 solo se verificaba que el runtime
        // existiera, asi que un hook apuntando a otro lado pasaba.
        let dir = tempfile::tempdir().unwrap();
        let paths = paths_en(dir.path());
        std::fs::create_dir_all(paths.repo_root.join(".claude")).unwrap();
        std::fs::write(
            paths.repo_root.join(".claude/settings.json"),
            r#"{"hooks":{"Stop":[{"hooks":[{"command":"bash /otro/lado/script.sh"}]}]}}"#,
        )
        .unwrap();
        std::fs::create_dir_all(paths.repo_root.join("bin")).unwrap();
        std::fs::write(paths.repo_root.join("bin/harness-hook"), "#!/bin/sh\n").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(
                paths.repo_root.join("bin/harness-hook"),
                std::fs::Permissions::from_mode(0o755),
            )
            .unwrap();
        }
        let h = revisar_hooks(&paths, false);
        assert_eq!(h.estado, Estado::Falla, "{}", h.detalle);
        assert!(h.detalle.contains("no apuntan"), "{}", h.detalle);
        assert!(h.detalle.contains("claude"), "{}", h.detalle);
    }

    #[test]
    fn doctor_should_stay_quiet_with_well_wired_hooks() {
        // El chequeo mas fino no puede volverse ruidoso.
        let dir = tempfile::tempdir().unwrap();
        let paths = paths_en(dir.path());
        std::fs::create_dir_all(paths.repo_root.join(".claude")).unwrap();
        std::fs::write(
            paths.repo_root.join(".claude/settings.json"),
            r#"{"hooks":{"Stop":[{"hooks":[{"command":"bash \"$HOOK_BASE/bin/harness-hook\" plain Stop"}]}]}}"#,
        )
        .unwrap();
        std::fs::create_dir_all(paths.repo_root.join("bin")).unwrap();
        std::fs::write(paths.repo_root.join("bin/harness-hook"), "#!/bin/sh\n").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(
                paths.repo_root.join("bin/harness-hook"),
                std::fs::Permissions::from_mode(0o755),
            )
            .unwrap();
        }
        assert_eq!(revisar_hooks(&paths, false).estado, Estado::Ok);
    }

    #[test]
    fn surfaces_should_only_be_demanded_for_installed_backends() {
        // Con solo Claude instalado, la falta de GEMINI.md no es un problema.
        let dir = tempfile::tempdir().unwrap();
        let paths = paths_en(dir.path());
        std::fs::create_dir_all(paths.repo_root.join(".claude")).unwrap();
        std::fs::write(paths.repo_root.join(".claude/settings.json"), "{}").unwrap();
        let h = revisar_superficies(&paths, false);
        assert_eq!(h.estado, Estado::Falla, "{}", h.detalle);
        assert!(h.detalle.contains("CLAUDE.md"), "{}", h.detalle);
        assert!(!h.detalle.contains("GEMINI.md"), "no debe pedir Gemini: {}", h.detalle);
        std::fs::write(paths.repo_root.join("CLAUDE.md"), "# surface").unwrap();
        assert_eq!(revisar_superficies(&paths, false).estado, Estado::Ok);
    }

    #[test]
    fn a_source_checkout_should_not_demand_surfaces_or_hooks() {
        // AC-12: aca su ausencia es lo CORRECTO.
        let dir = tempfile::tempdir().unwrap();
        let paths = paths_en(dir.path());
        assert_eq!(revisar_superficies(&paths, true).estado, Estado::NoAplica);
        assert_eq!(revisar_hooks(&paths, true).estado, Estado::NoAplica);
    }

    #[test]
    fn an_unconfigured_hub_should_be_a_warning() {
        let dir = tempfile::tempdir().unwrap();
        let paths = paths_en(dir.path());
        // Sin DB_* en el entorno el hub queda sin configurar y eso NO bloquea.
        let h = revisar_hub(&paths);
        assert!(!h.estado.bloquea(), "el hub nunca puede bloquear: {:?}", h);
    }

    #[test]
    fn exit_code_should_be_two_only_with_a_failure() {
        let ok = [Hallazgo::ok(Area::Hub, "x")];
        let aviso = [Hallazgo::aviso(Area::Hub, "x", "y")];
        let falla = [Hallazgo::falla(Area::Binario, "x", "y")];
        assert_eq!(exit_code(&ok), 0);
        assert_eq!(exit_code(&aviso), 0);
        assert_eq!(exit_code(&falla), 2);
    }

    #[test]
    fn required_tools_should_fail_and_optional_ones_should_warn() {
        let dir = tempfile::tempdir().unwrap();
        let paths = paths_en(dir.path());
        // `git` existe en cualquier maquina de desarrollo; el area no puede
        // fallar por las opcionales.
        let h = revisar_herramientas(&paths);
        assert_ne!(h.estado, Estado::Falla, "{}", h.detalle);
    }

    #[test]
    fn doctor_should_report_protected_paths_status() {
        // AC-16: informa el estado, nunca las violaciones.
        let dir = tempfile::tempdir().unwrap();
        let paths = paths_en(dir.path());
        let h = revisar_rutas_protegidas(&paths);
        assert_eq!(h.estado, Estado::Ok, "{}", h.detalle);
        assert!(h.detalle.contains("docs/prd/**"), "{}", h.detalle);
        assert!(h.detalle.contains("harness_check.sh"), "remite, no duplica: {}", h.detalle);
        // Y nunca bloquea: el estado de la proteccion no impide trabajar.
        assert!(!h.estado.bloquea());
    }

    #[test]
    fn graphify_should_never_block() {
        assert!(!revisar_graphify().estado.bloquea());
    }
}
