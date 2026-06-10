//! PgGraphStore (paridad: graph_memory.py lineas 65-171). Cada operacion
//! abre una conexion fresca, igual que `psycopg2.connect(...)` por bloque.

use anyhow::Context;
use indexmap::IndexMap;
use postgres::config::SslMode;
use serde_json::{Map, Value};

use super::tls;

pub struct PgGraphStore {
    config: postgres::Config,
    sslmode: String,
    pub nodes: IndexMap<String, Map<String, Value>>,
    pub edges: Vec<Map<String, Value>>,
}

impl PgGraphStore {
    pub fn new(
        dbname: &str,
        user: &str,
        password: &str,
        host: &str,
        port: &str,
        sslmode: &str,
    ) -> anyhow::Result<Self> {
        let mut config = postgres::Config::new();
        config
            .dbname(dbname)
            .user(user)
            .password(password)
            .host(host)
            .connect_timeout(std::time::Duration::from_secs(10));
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
        let node = self.nodes.entry(nid).or_default();
        for (k, v) in props {
            node.insert(k, v); // dict.update: reemplaza en posicion, agrega al final
        }
        node.insert("_label".to_string(), Value::String(label.to_string()));
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
            self.edges.push(edge);
        }
    }

    pub fn load(&mut self) -> anyhow::Result<()> {
        self.nodes.clear();
        self.edges.clear();
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

    pub fn save(&self) -> anyhow::Result<()> {
        let mut client = self.client()?;
        let mut txn = client.transaction()?;
        for (nid, props) in &self.nodes {
            let label = props
                .get("_label")
                .and_then(Value::as_str)
                .unwrap_or("Unknown")
                .to_string();
            let mut p = props.clone();
            p.remove("_label");
            txn.execute(
                "INSERT INTO graph_nodes (id, label, props)
                 VALUES ($1, $2, $3)
                 ON CONFLICT (id) DO UPDATE SET
                 label = EXCLUDED.label,
                 props = graph_nodes.props || EXCLUDED.props;",
                &[&nid, &label, &Value::Object(p)],
            )?;
        }
        for edge in &self.edges {
            let source = edge.get("source").and_then(Value::as_str).unwrap_or_default();
            let target = edge.get("target").and_then(Value::as_str).unwrap_or_default();
            let etype = edge.get("type").and_then(Value::as_str).unwrap_or_default();
            let mut p = edge.clone();
            p.remove("source");
            p.remove("target");
            p.remove("type");
            txn.execute(
                "INSERT INTO graph_edges (source, target, type, props)
                 VALUES ($1, $2, $3, $4)
                 ON CONFLICT (source, target, type) DO UPDATE SET
                 props = COALESCE(graph_edges.props, '{}'::jsonb) || EXCLUDED.props;",
                &[&source, &target, &etype, &Value::Object(p)],
            )?;
        }
        txn.commit()?;
        Ok(())
    }
}
