//! `vincular-grafo`: deriva dependencias entre microservicios desde
//! graphify-out/graph.json (paridad: graph_memory.py derive_from_graphify).

use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::LazyLock;

use regex::Regex;
use serde_json::{Map, Value, json};

use super::GraphMemoryManager;
use super::ids::qualify;

static SERVICE_RE: LazyLock<Regex> = LazyLock::new(|| {
    #[allow(clippy::unwrap_used)] // patron constante cubierto por tests
    Regex::new(r"(ms-[a-z0-9-]+-service|[a-z0-9-]+-ui)").unwrap()
});

// OJO paridad: el patron Python es r"...|\\badr\\b" (raw string con DOBLE
// backslash), o sea matchea el TEXTO LITERAL `\badr\b`, no la palabra "adr".
// Es un bug del original que se replica a proposito: "arreglarlo" cambiaria
// que dependencias se filtran como convenciones.
static CONVENTION_RE: LazyLock<Regex> = LazyLock::new(|| {
    #[allow(clippy::unwrap_used)]
    regex::RegexBuilder::new(r"convention|policy|guideline|standard|lint|layout|\\badr\\b")
        .case_insensitive(true)
        .build()
        .unwrap()
});

static DEP_RELATIONS: [&str; 6] = [
    "references",
    "implements",
    "shares_data_with",
    "depends_on",
    "uses",
    "cites",
];

impl GraphMemoryManager {
    pub fn derive_from_graphify(&mut self) -> anyhow::Result<()> {
        let graph_path = self.env.repo_root.join("graphify-out").join("graph.json");
        if !graph_path.exists() {
            println!(
                "[graphify->hub] No existe {}; nada que derivar.",
                graph_path.display()
            );
            return Ok(());
        }
        let graph: Value = serde_json::from_str(&std::fs::read_to_string(&graph_path)?)?;
        let empty = Vec::new();
        let graph_nodes = graph.get("nodes").and_then(Value::as_array).unwrap_or(&empty);
        let nodes_by_id: HashMap<&str, &Value> = graph_nodes
            .iter()
            .filter_map(|n| n.get("id").and_then(Value::as_str).map(|id| (id, n)))
            .collect();
        let service_for = |node_id: Option<&str>| -> Option<String> {
            let node = nodes_by_id.get(node_id?)?;
            let source = node
                .get("source_file")
                .and_then(Value::as_str)
                .unwrap_or_default();
            SERVICE_RE
                .captures(source)
                .and_then(|c| c.get(1))
                .map(|m| m.as_str().to_string())
        };

        // Denylist de dependencias espurias: una linea "a->b" por par;
        // '#' para comentarios; se ignora todo el whitespace.
        let mut denylist: HashSet<String> = HashSet::new();
        let deny_path = self.env.repo_root.join("harness_deps_deny.txt");
        if deny_path.exists() {
            for line in std::fs::read_to_string(&deny_path)?.lines() {
                let line = line.split('#').next().unwrap_or_default();
                let cleaned: String = line.chars().filter(|c| !c.is_whitespace()).collect();
                if !cleaned.is_empty() {
                    denylist.insert(cleaned);
                }
            }
        }

        let mut pairs: BTreeMap<(String, String), bool> = BTreeMap::new();
        let mut consumers_by_target_node: HashMap<String, HashSet<String>> = HashMap::new();
        let mut raw: Vec<(String, String, String)> = Vec::new();
        let links = graph.get("links").and_then(Value::as_array).unwrap_or(&empty);
        for edge in links {
            let a = service_for(edge.get("source").and_then(Value::as_str));
            let b = service_for(edge.get("target").and_then(Value::as_str));
            let (Some(a), Some(b)) = (a, b) else { continue };
            if a == b {
                continue;
            }
            let relation = edge.get("relation").and_then(Value::as_str).unwrap_or_default();
            if !DEP_RELATIONS.contains(&relation) {
                continue;
            }
            if denylist.contains(&format!("{a}->{b}")) {
                continue;
            }
            let Some(target_node) = edge.get("target").and_then(Value::as_str) else {
                continue;
            };
            let label = nodes_by_id
                .get(target_node)
                .and_then(|n| n.get("label"))
                .and_then(Value::as_str)
                .unwrap_or_default();
            if CONVENTION_RE.is_match(label) {
                continue;
            }
            consumers_by_target_node
                .entry(target_node.to_string())
                .or_default()
                .insert(a.clone());
            raw.push((a, b, target_node.to_string()));
        }

        for (a, b, target_node) in &raw {
            let many_consumers = consumers_by_target_node
                .get(target_node)
                .map(|s| s.len() >= 3)
                .unwrap_or(false);
            let entry = pairs.entry((a.clone(), b.clone())).or_insert(false);
            *entry = *entry || many_consumers;
        }

        if pairs.is_empty() {
            println!("[graphify->hub] Sin dependencias derivables.");
            return Ok(());
        }

        let project = self.env.project.clone();
        let pairs_for_lock = pairs.clone();
        self.locked(move |m| {
            m.store.load()?;
            m.store.edges.retain(|e| {
                !(e.get("type").and_then(Value::as_str) == Some("DEPENDE_DE")
                    && e.get("origen").and_then(Value::as_str) == Some("graphify"))
            });
            let existing: HashSet<(String, String)> = m
                .store
                .edges
                .iter()
                .filter(|e| e.get("type").and_then(Value::as_str) == Some("DEPENDE_DE"))
                .map(|e| {
                    (
                        e.get("source").and_then(Value::as_str).unwrap_or_default().to_string(),
                        e.get("target").and_then(Value::as_str).unwrap_or_default().to_string(),
                    )
                })
                .collect();
            for ((a, b), transversal) in &pairs_for_lock {
                let c = qualify(&project, a);
                let d = qualify(&project, b);
                let mut consumer_props = Map::new();
                consumer_props.insert("_id".to_string(), json!(c));
                m.store.add_node("Microservicio", consumer_props)?;
                let mut target_props = Map::new();
                target_props.insert("_id".to_string(), json!(d));
                if *transversal {
                    target_props.insert("tipo".to_string(), json!("transversal"));
                }
                m.store.add_node("Microservicio", target_props)?;
                if !existing.contains(&(c.clone(), d.clone())) {
                    m.store.add_edge("DEPENDE_DE", &c, &d, &[("origen", "graphify")]);
                }
            }
            m.store.save()
        })?;
        let summary = pairs
            .iter()
            .map(|((a, b), tv)| format!("{a}->{b}{}", if *tv { " [tv]" } else { "" }))
            .collect::<Vec<_>>()
            .join(", ");
        println!("[graphify->hub] {} dependencia(s): {}", pairs.len(), summary);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convention_regex_should_match_literal_backslash_badr_only() {
        // Bug replicado: NO matchea la palabra "adr", solo el literal \badr\b
        assert!(!CONVENTION_RE.is_match("adr"));
        assert!(!CONVENTION_RE.is_match("decision adr 001"));
        assert!(CONVENTION_RE.is_match(r"texto \badr\b literal"));
        assert!(CONVENTION_RE.is_match("Naming Convention"));
        assert!(CONVENTION_RE.is_match("LINT rules"));
    }

    #[test]
    fn service_regex_should_extract_service_and_ui_names() {
        let caps = SERVICE_RE.captures("repo/ms-pagos-service/src/main.go");
        assert_eq!(caps.and_then(|c| c.get(1)).map(|m| m.as_str()), Some("ms-pagos-service"));
        let caps = SERVICE_RE.captures("apps/admin-ui/index.tsx");
        assert_eq!(caps.and_then(|c| c.get(1)).map(|m| m.as_str()), Some("admin-ui"));
        assert!(SERVICE_RE.captures("docs/readme.md").is_none());
    }
}
