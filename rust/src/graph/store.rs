//! PgGraphStore (paridad: graph_memory.py lineas 65-171). Cada operacion
//! abre una conexion fresca, igual que `psycopg2.connect(...)` por bloque.
//!
//! Feature #14: el guardado ya NO es una sentencia por fila sobre el grafo
//! entero. Se escribe SOLO lo tocado desde el ultimo `load()` (el registro de
//! "sucios") y en lotes con `UNNEST`, porque el costo real del hub es el
//! round-trip (164 ms medidos contra el hub remoto del usuario), no el tamano
//! del mensaje. Quien mute `nodes`/`edges` sin pasar por `add_node`/`add_edge`
//! DEBE marcar la clave con `mark_node_dirty` o su cambio no se persiste (hoy
//! el unico caso es `unmark`).

use anyhow::Context;
use indexmap::{IndexMap, IndexSet};
use postgres::config::SslMode;
use serde_json::{Map, Value};

use super::tls;

/// Filas por sentencia en los upserts: convierte N sentencias en
/// `ceil(N/UPSERT_CHUNK)` sin dejar que el mensaje crezca sin limite.
const UPSERT_CHUNK: usize = 1000;

/// Corte de sentencia (ms) cuando no hay `DB_STATEMENT_TIMEOUT`. Con los lotes
/// la sentencia mas cara es un upsert de 1000 filas (sub-segundo), asi que 30 s
/// es margen de sobra para que solo salte con un hub que no responde.
pub const DEFAULT_STATEMENT_TIMEOUT_MS: u64 = 30_000;

/// Cada cuanto manda keepalives TCP: detecta la conexion muerta en silencio,
/// que es lo que `statement_timeout` (del lado del servidor) no puede cubrir.
const KEEPALIVES_IDLE_SECS: u64 = 30;

/// Clave primaria de una arista, igual que en `graph_edges`.
type EdgeKey = (String, String, String);

pub struct PgGraphStore {
    config: postgres::Config,
    sslmode: String,
    pub nodes: IndexMap<String, Map<String, Value>>,
    pub edges: Vec<Map<String, Value>>,
    /// Ids de nodo tocados desde el ultimo `load()`/`save()`.
    dirty_nodes: IndexSet<String>,
    /// Claves de arista tocadas desde el ultimo `load()`/`save()`.
    dirty_edges: IndexSet<EdgeKey>,
}

impl PgGraphStore {
    pub fn new(
        dbname: &str,
        user: &str,
        password: &str,
        host: &str,
        port: &str,
        sslmode: &str,
        statement_timeout_ms: u64,
    ) -> anyhow::Result<Self> {
        let mut config = postgres::Config::new();
        config
            .dbname(dbname)
            .user(user)
            .password(password)
            .host(host)
            .connect_timeout(std::time::Duration::from_secs(10))
            .keepalives(true)
            .keepalives_idle(std::time::Duration::from_secs(KEEPALIVES_IDLE_SECS));
        // `connect_timeout` solo cubre el saludo inicial: sin esto, un hub que
        // deja de responder cuelga el comando (y el candado) para siempre.
        if statement_timeout_ms > 0 {
            config.options(&format!("-c statement_timeout={statement_timeout_ms}"));
        }
        let port: u16 = port
            .parse()
            .with_context(|| format!("DB_PORT invalido: {port}"))?;
        config.port(port);
        config.ssl_mode(match sslmode {
            "disable" => SslMode::Disable,
            "prefer" | "allow" => SslMode::Prefer,
            _ => SslMode::Require,
        });
        let store = PgGraphStore {
            config,
            sslmode: sslmode.to_string(),
            nodes: IndexMap::new(),
            edges: Vec::new(),
            dirty_nodes: IndexSet::new(),
            dirty_edges: IndexSet::new(),
        };
        store.init_db()?;
        Ok(store)
    }

    fn client(&self) -> anyhow::Result<postgres::Client> {
        let client = if self.sslmode == "disable" {
            self.config.connect(postgres::NoTls)?
        } else {
            self.config.connect(tls::make_connector(&self.sslmode)?)?
        };
        Ok(client)
    }

    fn init_db(&self) -> anyhow::Result<()> {
        let mut client = self.client()?;
        client.batch_execute(
            "CREATE TABLE IF NOT EXISTS graph_nodes (
                id TEXT PRIMARY KEY,
                label TEXT NOT NULL,
                props JSONB NOT NULL DEFAULT '{}'::jsonb
            );
            CREATE TABLE IF NOT EXISTS graph_edges (
                source TEXT NOT NULL,
                target TEXT NOT NULL,
                type TEXT NOT NULL,
                props JSONB NOT NULL DEFAULT '{}'::jsonb,
                PRIMARY KEY (source, target, type)
            );",
        )?;
        Ok(())
    }

    /// `add_node(label, props)`: upsert en memoria; `_id` queda DENTRO de los
    /// props (igual que Python) y `_label` se agrega/actualiza al final.
    pub fn add_node(&mut self, label: &str, props: Map<String, Value>) -> anyhow::Result<()> {
        let nid = props
            .get("_id")
            .and_then(Value::as_str)
            .context("add_node: props sin _id")?
            .to_string();
        let node = self.nodes.entry(nid.clone()).or_default();
        for (k, v) in props {
            node.insert(k, v); // dict.update: reemplaza en posicion, agrega al final
        }
        node.insert("_label".to_string(), Value::String(label.to_string()));
        self.dirty_nodes.insert(nid);
        Ok(())
    }

    /// `add_edge(etype, source, target, **props)` con dedup por igualdad de dict.
    pub fn add_edge(
        &mut self,
        etype: &str,
        source: &str,
        target: &str,
        props: &[(&str, &str)],
    ) {
        let mut edge = Map::new();
        edge.insert("type".to_string(), Value::String(etype.to_string()));
        edge.insert("source".to_string(), Value::String(source.to_string()));
        edge.insert("target".to_string(), Value::String(target.to_string()));
        for (k, v) in props {
            edge.insert((*k).to_string(), Value::String((*v).to_string()));
        }
        if !self.edges.contains(&edge) {
            self.dirty_edges.insert(edge_key(&edge));
            self.edges.push(edge);
        }
    }

    /// Marca un nodo para que el proximo `save()` lo escriba. Necesario cuando
    /// se muta `nodes` a mano en vez de pasar por `add_node`.
    pub fn mark_node_dirty(&mut self, nid: &str) {
        self.dirty_nodes.insert(nid.to_string());
    }

    pub fn load(&mut self) -> anyhow::Result<()> {
        self.nodes.clear();
        self.edges.clear();
        // Lo que viene de la base ya esta en la base: nada pendiente de escribir.
        self.dirty_nodes.clear();
        self.dirty_edges.clear();
        let mut client = self.client()?;
        for row in client.query("SELECT id, label, props FROM graph_nodes;", &[])? {
            let id: String = row.get(0);
            let label: String = row.get(1);
            let props: Option<Value> = row.get(2);
            let mut map = match props {
                Some(Value::Object(m)) => m,
                _ => Map::new(),
            };
            map.insert("_label".to_string(), Value::String(label));
            self.nodes.insert(id, map);
        }
        for row in client.query("SELECT source, target, type, props FROM graph_edges;", &[])? {
            let mut edge = Map::new();
            edge.insert("source".to_string(), Value::String(row.get(0)));
            edge.insert("target".to_string(), Value::String(row.get(1)));
            edge.insert("type".to_string(), Value::String(row.get(2)));
            let props: Option<Value> = row.get(3);
            if let Some(Value::Object(m)) = props {
                for (k, v) in m {
                    edge.insert(k, v); // edge.update(props): puede pisar claves
                }
            }
            self.edges.push(edge);
        }
        Ok(())
    }

    pub fn get_node(&self, nid: &str) -> anyhow::Result<Option<Map<String, Value>>> {
        let mut client = self.client()?;
        let row = client.query_opt(
            "SELECT label, props FROM graph_nodes WHERE id = $1;",
            &[&nid],
        )?;
        Ok(row.map(|row| {
            let label: String = row.get(0);
            let props: Option<Value> = row.get(1);
            let mut map = match props {
                Some(Value::Object(m)) => m,
                _ => Map::new(),
            };
            map.insert("_label".to_string(), Value::String(label));
            map
        }))
    }

    /// Escribe SOLO lo sucio, en lotes, dentro de una unica transaccion. El
    /// upsert por lote es el mismo de siempre fila a fila: `label` se pisa y
    /// `props` se fusiona con `||`.
    pub fn save(&mut self) -> anyhow::Result<()> {
        let nodes = node_rows(&self.nodes, &self.dirty_nodes);
        let edges = edge_rows(&self.edges, &self.dirty_edges);
        if nodes.is_empty() && edges.is_empty() {
            return Ok(()); // nada tocado: ni conexion ni transaccion
        }
        let mut client = self.client()?;
        let mut txn = client.transaction()?;
        for chunk in nodes.chunks(UPSERT_CHUNK) {
            let ids: Vec<&str> = chunk.iter().map(|(id, _, _)| id.as_str()).collect();
            let labels: Vec<&str> = chunk.iter().map(|(_, label, _)| label.as_str()).collect();
            let props: Vec<Value> = chunk.iter().map(|(_, _, p)| p.clone()).collect();
            txn.execute(
                "INSERT INTO graph_nodes (id, label, props)
                 SELECT * FROM UNNEST($1::text[], $2::text[], $3::jsonb[])
                 ON CONFLICT (id) DO UPDATE SET
                 label = EXCLUDED.label,
                 props = graph_nodes.props || EXCLUDED.props;",
                &[&ids, &labels, &props],
            )?;
        }
        for chunk in edges.chunks(UPSERT_CHUNK) {
            let sources: Vec<&str> = chunk.iter().map(|(s, _, _, _)| s.as_str()).collect();
            let targets: Vec<&str> = chunk.iter().map(|(_, t, _, _)| t.as_str()).collect();
            let types: Vec<&str> = chunk.iter().map(|(_, _, ty, _)| ty.as_str()).collect();
            let props: Vec<Value> = chunk.iter().map(|(_, _, _, p)| p.clone()).collect();
            txn.execute(
                "INSERT INTO graph_edges (source, target, type, props)
                 SELECT * FROM UNNEST($1::text[], $2::text[], $3::text[], $4::jsonb[])
                 ON CONFLICT (source, target, type) DO UPDATE SET
                 props = COALESCE(graph_edges.props, '{}'::jsonb) || EXCLUDED.props;",
                &[&sources, &targets, &types, &props],
            )?;
        }
        txn.commit()?;
        self.dirty_nodes.clear();
        self.dirty_edges.clear();
        Ok(())
    }
}

/// Clave primaria de la arista, con los mismos defaults que usaba el INSERT
/// fila a fila (`unwrap_or_default`).
fn edge_key(edge: &Map<String, Value>) -> EdgeKey {
    let field = |key: &str| {
        edge.get(key)
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string()
    };
    (field("source"), field("target"), field("type"))
}

/// Filas de nodo que van al lote: `(id, label, props sin _label)`, solo las
/// sucias y en el orden en que se tocaron. Pura: testeable sin base de datos.
fn node_rows(
    nodes: &IndexMap<String, Map<String, Value>>,
    dirty: &IndexSet<String>,
) -> Vec<(String, String, Value)> {
    dirty
        .iter()
        .filter_map(|nid| {
            let props = nodes.get(nid)?;
            let label = props
                .get("_label")
                .and_then(Value::as_str)
                .unwrap_or("Unknown")
                .to_string();
            let mut p = props.clone();
            p.remove("_label");
            Some((nid.clone(), label, Value::Object(p)))
        })
        .collect()
}

/// Filas de arista que van al lote, FUSIONADAS por `(source, target, type)`:
/// dos aristas con la misma clave son legales en memoria (`add_edge` deduplica
/// por igualdad de dict completo), pero en un mismo INSERT Postgres las
/// rechazaria con "ON CONFLICT DO UPDATE command cannot affect row a second
/// time". Fusionarlas con "la ultima gana clave a clave" produce exactamente lo
/// mismo que producian los INSERT secuenciales encadenando `|| EXCLUDED.props`.
/// Pura: testeable sin base de datos.
fn edge_rows(
    edges: &[Map<String, Value>],
    dirty: &IndexSet<EdgeKey>,
) -> Vec<(String, String, String, Value)> {
    let mut merged: IndexMap<EdgeKey, Map<String, Value>> = IndexMap::new();
    for edge in edges {
        let key = edge_key(edge);
        if !dirty.contains(&key) {
            continue;
        }
        let mut props = edge.clone();
        props.remove("source");
        props.remove("target");
        props.remove("type");
        let entry = merged.entry(key).or_default();
        for (k, v) in props {
            entry.insert(k, v);
        }
    }
    merged
        .into_iter()
        .map(|((source, target, etype), props)| (source, target, etype, Value::Object(props)))
        .collect()
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use serde_json::json;

    fn node(id: &str, label: &str, extra: &[(&str, &str)]) -> Map<String, Value> {
        let mut m = Map::new();
        m.insert("_id".to_string(), json!(id));
        for (k, v) in extra {
            m.insert((*k).to_string(), json!(v));
        }
        m.insert("_label".to_string(), json!(label));
        m
    }

    fn edge(source: &str, target: &str, etype: &str, props: &[(&str, &str)]) -> Map<String, Value> {
        let mut m = Map::new();
        m.insert("type".to_string(), json!(etype));
        m.insert("source".to_string(), json!(source));
        m.insert("target".to_string(), json!(target));
        for (k, v) in props {
            m.insert((*k).to_string(), json!(v));
        }
        m
    }

    /// AC-4: el lote lleva SOLO los nodos sucios, no el grafo entero.
    #[test]
    fn node_rows_solo_lleva_lo_sucio() {
        let mut nodes = IndexMap::new();
        nodes.insert("a".to_string(), node("a", "Proyecto", &[]));
        nodes.insert("b".to_string(), node("b", "Commit", &[("x", "1")]));
        nodes.insert("c".to_string(), node("c", "Artefacto", &[]));
        let dirty: IndexSet<String> = ["b".to_string()].into_iter().collect();

        let rows = node_rows(&nodes, &dirty);

        assert_eq!(rows.len(), 1);
        let (id, label, props) = &rows[0];
        assert_eq!(id, "b");
        assert_eq!(label, "Commit");
        // `_label` viaja como columna, no dentro de props; `_id` si se queda.
        assert_eq!(props, &json!({"_id": "b", "x": "1"}));
    }

    /// Sin nada sucio no hay filas: `save()` ni siquiera abre la transaccion.
    #[test]
    fn node_rows_vacio_sin_sucios() {
        let mut nodes = IndexMap::new();
        nodes.insert("a".to_string(), node("a", "Proyecto", &[]));
        assert!(node_rows(&nodes, &IndexSet::new()).is_empty());
    }

    /// Un nodo marcado que ya no existe en memoria no rompe el lote.
    #[test]
    fn node_rows_ignora_sucio_inexistente() {
        let dirty: IndexSet<String> = ["fantasma".to_string()].into_iter().collect();
        assert!(node_rows(&IndexMap::new(), &dirty).is_empty());
    }

    /// AC-3: dos aristas con la misma clave se fusionan en UNA fila (la ultima
    /// gana clave a clave), que es lo que producian los INSERT secuenciales.
    #[test]
    fn edge_rows_fusiona_misma_clave() {
        let edges = vec![
            edge("a", "b", "DEPENDE_DE", &[("origen", "manual"), ("peso", "1")]),
            edge("a", "b", "DEPENDE_DE", &[("origen", "graphify")]),
        ];
        let dirty: IndexSet<EdgeKey> = [(
            "a".to_string(),
            "b".to_string(),
            "DEPENDE_DE".to_string(),
        )]
        .into_iter()
        .collect();

        let rows = edge_rows(&edges, &dirty);

        assert_eq!(rows.len(), 1, "una sola fila por clave: ON CONFLICT no admite repetidas");
        let (source, target, etype, props) = &rows[0];
        assert_eq!((source.as_str(), target.as_str(), etype.as_str()), ("a", "b", "DEPENDE_DE"));
        assert_eq!(props, &json!({"origen": "graphify", "peso": "1"}));
    }

    /// AC-2/AC-4: solo las sucias, y source/target/type salen de props.
    #[test]
    fn edge_rows_solo_lleva_lo_sucio() {
        let edges = vec![
            edge("a", "b", "DEPENDE_DE", &[("origen", "manual")]),
            edge("x", "y", "MODIFICO", &[]),
        ];
        let dirty: IndexSet<EdgeKey> = [("x".to_string(), "y".to_string(), "MODIFICO".to_string())]
            .into_iter()
            .collect();

        let rows = edge_rows(&edges, &dirty);

        assert_eq!(rows.len(), 1);
        let (source, target, etype, props) = &rows[0];
        assert_eq!((source.as_str(), target.as_str(), etype.as_str()), ("x", "y", "MODIFICO"));
        assert_eq!(props, &json!({}));
    }

    /// El orden de las filas es el orden en que se tocaron (determinista).
    #[test]
    fn edge_rows_conserva_orden() {
        let edges = vec![
            edge("a", "b", "T", &[]),
            edge("c", "d", "T", &[]),
        ];
        let dirty: IndexSet<EdgeKey> = [
            ("c".to_string(), "d".to_string(), "T".to_string()),
            ("a".to_string(), "b".to_string(), "T".to_string()),
        ]
        .into_iter()
        .collect();

        let rows = edge_rows(&edges, &dirty);

        let keys: Vec<&str> = rows.iter().map(|(s, _, _, _)| s.as_str()).collect();
        assert_eq!(keys, vec!["a", "c"], "el orden lo fija el vector de aristas");
    }
}
