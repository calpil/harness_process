//! Metodos del GraphMemoryManager (paridad: graph_memory.py 242-474).
//! Todos los textos impresos son VERBATIM respecto del Python.

use serde_json::{Map, Value, json};

use super::GraphMemoryManager;
use super::ids::{artifact_id, is_repo_root, qualify};
use crate::pycompat::py_json_pretty;

impl GraphMemoryManager {
    pub fn discover(&mut self) -> anyhow::Result<()> {
        let project = self.env.project.clone();
        let repo_root = self.env.repo_root.clone();
        let base_dir = self.env.base_dir.clone();
        let mut found: Vec<String> = Vec::new();
        self.locked(|m| {
            m.store.load()?;
            let mut project_props = Map::new();
            project_props.insert("_id".to_string(), json!(project));
            project_props.insert(
                "path".to_string(),
                json!(repo_root.to_string_lossy().into_owned()),
            );
            let graphify_out = repo_root.join("graphify-out");
            if graphify_out.join("graph.json").exists() {
                project_props.insert(
                    "graphify_out".to_string(),
                    json!(graphify_out.to_string_lossy().into_owned()),
                );
            }
            m.store.add_node("Proyecto", project_props)?;
            let mut entries: Vec<std::ffi::OsString> = std::fs::read_dir(&repo_root)?
                .filter_map(|e| e.ok().map(|e| e.file_name()))
                .collect();
            entries.sort();
            for entry in entries {
                let path = repo_root.join(&entry);
                // el propio arnes no es un microservicio (compara realpaths)
                if let (Ok(a), Ok(b)) = (
                    std::fs::canonicalize(&path),
                    std::fs::canonicalize(&base_dir),
                ) {
                    if a == b {
                        continue;
                    }
                }
                if path.is_dir() && is_repo_root(&path) {
                    let entry_str = entry.to_string_lossy().into_owned();
                    let qid = format!("{project}/{entry_str}");
                    let mut props = Map::new();
                    props.insert("_id".to_string(), json!(qid));
                    props.insert("proyecto".to_string(), json!(project));
                    props.insert("servicio".to_string(), json!(entry_str));
                    props.insert(
                        "path".to_string(),
                        json!(path.to_string_lossy().into_owned()),
                    );
                    m.store.add_node("Microservicio", props)?;
                    m.store.add_edge("CONTIENE", &project, &qid, &[]);
                    found.push(entry_str);
                }
            }
            m.store.save()
        })?;
        let listing = if found.is_empty() {
            "(ninguno)".to_string()
        } else {
            found.join(", ")
        };
        println!(
            "[Memoria] Proyecto '{}' en PostgreSQL ({}): {} microservicio(s): {}",
            project,
            self.hub_location,
            found.len(),
            listing
        );
        Ok(())
    }

    pub fn sync_git(
        &mut self,
        commit_hash: &str,
        files: &[&str],
        microservice: &str,
    ) -> anyhow::Result<()> {
        let project = self.env.project.clone();
        let qserv = format!("{project}/{microservice}");
        let files: Vec<String> = files.iter().map(|s| s.to_string()).collect();
        let microservice = microservice.to_string();
        let commit = commit_hash.to_string();
        let qserv_closure = qserv.clone();
        self.locked(move |m| {
            m.store.load()?;
            let mut commit_props = Map::new();
            commit_props.insert("_id".to_string(), json!(commit));
            commit_props.insert("proyecto".to_string(), json!(project));
            commit_props.insert("microservicio".to_string(), json!(microservice));
            m.store.add_node("Commit", commit_props)?;
            let mut agent_props = Map::new();
            agent_props.insert("_id".to_string(), json!("Agente_Implementador"));
            m.store.add_node("Agente", agent_props)?;
            m.store.add_edge("REALIZO", "Agente_Implementador", &commit, &[]);
            for file_path in &files {
                if file_path.is_empty() {
                    continue;
                }
                let aid = artifact_id(file_path);
                let nid = format!("{qserv_closure}:{aid}");
                let mut props = Map::new();
                props.insert("_id".to_string(), json!(nid));
                props.insert("ruta".to_string(), json!(file_path));
                props.insert("estado".to_string(), json!("MODIFICADO_GIT"));
                props.insert("proyecto".to_string(), json!(project));
                props.insert("microservicio".to_string(), json!(microservice));
                m.store.add_node("Artefacto", props)?;
                m.store.add_edge("MODIFICO", &commit, &nid, &[]);
            }
            m.store.save()
        })?;
        let short: String = commit_hash.chars().take(7).collect();
        println!("[Memoria] Commit {short} sincronizado para {qserv}");
        Ok(())
    }

    pub fn link(&mut self, consumer: &str, target: &str, transversal: bool) -> anyhow::Result<()> {
        let c = qualify(&self.env.project, consumer);
        let t = qualify(&self.env.project, target);
        let (c2, t2) = (c.clone(), t.clone());
        self.locked(move |m| {
            m.store.load()?;
            let mut consumer_props = Map::new();
            consumer_props.insert("_id".to_string(), json!(c2));
            m.store.add_node("Microservicio", consumer_props)?;
            let mut target_props = Map::new();
            target_props.insert("_id".to_string(), json!(t2));
            if transversal {
                target_props.insert("tipo".to_string(), json!("transversal"));
            }
            m.store.add_node("Microservicio", target_props)?;
            m.store.add_edge("DEPENDE_DE", &c2, &t2, &[("origen", "manual")]);
            m.store.save()
        })?;
        let suffix = if transversal { " [transversal]" } else { "" };
        println!("[Memoria] {c} depende de {t}{suffix}");
        Ok(())
    }

    pub fn unmark(&mut self, service: &str) -> anyhow::Result<()> {
        let s = qualify(&self.env.project, service);
        let s2 = s.clone();
        self.locked(move |m| {
            m.store.load()?;
            let was_transversal = m
                .store
                .nodes
                .get(&s2)
                .and_then(|n| n.get("tipo"))
                .and_then(Value::as_str)
                == Some("transversal");
            if was_transversal {
                if let Some(node) = m.store.nodes.get_mut(&s2) {
                    node.remove("tipo");
                }
                m.store.save()?;
                println!("[Memoria] {s2} ya no esta marcado como transversal");
            } else {
                println!("[Memoria] {s2} no estaba marcado como transversal");
            }
            Ok(())
        })
    }

    pub fn impact(&mut self, service: &str) -> anyhow::Result<()> {
        let t = qualify(&self.env.project, service);
        self.locked(|m| m.store.load())?;
        let mut affected: Vec<&str> = self
            .store
            .edges
            .iter()
            .filter(|e| {
                e.get("type").and_then(Value::as_str) == Some("DEPENDE_DE")
                    && e.get("target").and_then(Value::as_str) == Some(t.as_str())
            })
            .filter_map(|e| e.get("source").and_then(Value::as_str))
            .collect();
        affected.sort_unstable();
        if affected.is_empty() {
            println!("[Impacto] Ningun microservicio registrado depende de '{t}'");
        } else {
            println!(
                "[Impacto] Si modificas '{t}', revisa: {}",
                affected.join(", ")
            );
        }
        Ok(())
    }

    pub fn record_event(
        &mut self,
        agent: &str,
        action: &str,
        artefacto: &str,
        estado: &str,
        metadata: Option<&str>,
    ) -> anyhow::Result<()> {
        let qart = format!("{}/{artefacto}", self.env.project);
        let project = self.env.project.clone();
        let (agent_o, action_o, estado_o, qart_o) = (
            agent.to_string(),
            action.to_string(),
            estado.to_string(),
            qart.clone(),
        );
        let metadata = metadata.filter(|m| !m.is_empty()).map(str::to_string);
        self.locked(move |m| {
            m.store.load()?;
            let mut agent_props = Map::new();
            agent_props.insert("_id".to_string(), json!(agent_o));
            m.store.add_node("Agente", agent_props)?;
            let mut node = Map::new();
            node.insert("_id".to_string(), json!(qart_o));
            node.insert("estado".to_string(), json!(estado_o));
            node.insert("proyecto".to_string(), json!(project));
            if let Some(meta) = &metadata {
                node.insert("metadata".to_string(), json!(meta));
            }
            m.store.add_node("Artefacto", node)?;
            m.store
                .add_edge(&action_o.to_uppercase(), &agent_o, &qart_o, &[]);
            m.store.save()
        })?;
        println!("[Memoria] {agent} --[{action}]--> {qart} ({estado})");
        Ok(())
    }

    pub fn query_state(&mut self, artefacto: &str, microservice: &str) -> anyhow::Result<()> {
        let node_id = if microservice == "raiz" {
            format!("{}/{artefacto}", self.env.project)
        } else {
            format!("{}:{artefacto}", qualify(&self.env.project, microservice))
        };
        match self.store.get_node(&node_id)? {
            Some(node) => {
                let mut result = node.clone();
                result.insert("_id".to_string(), json!(node_id));
                result.remove("_label");
                println!("{}", py_json_pretty(&Value::Object(result))?);
            }
            None => {
                println!(
                    "Error: Artefacto '{artefacto}' no encontrado en {}/{microservice}.",
                    self.env.project
                );
            }
        }
        Ok(())
    }

    pub fn map(&mut self) -> anyhow::Result<()> {
        self.locked(|m| m.store.load())?;
        let nodes = &self.store.nodes;
        let edges = &self.store.edges;
        let node_label = |n: &Map<String, Value>| {
            n.get("_label").and_then(Value::as_str).unwrap_or_default().to_string()
        };
        let micros: indexmap::IndexMap<String, &Map<String, Value>> = nodes
            .iter()
            .filter(|(_, n)| node_label(n) == "Microservicio")
            .map(|(k, n)| (k.clone(), n))
            .collect();
        let deps: Vec<&Map<String, Value>> = edges
            .iter()
            .filter(|e| e.get("type").and_then(Value::as_str) == Some("DEPENDE_DE"))
            .collect();
        let edge_str = |e: &Map<String, Value>, key: &str| {
            e.get(key).and_then(Value::as_str).unwrap_or_default().to_string()
        };
        let mut projects: indexmap::IndexMap<String, Vec<String>> = indexmap::IndexMap::new();
        for nid in micros.keys() {
            let project = nid.split('/').next().unwrap_or_default().to_string();
            projects.entry(project).or_default().push((*nid).clone());
        }
        for (nid, node) in nodes {
            if node_label(node) == "Proyecto" {
                projects.entry(nid.clone()).or_default();
            }
        }

        let mut dependents: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();
        for edge in &deps {
            *dependents.entry(edge_str(edge, "target")).or_default() += 1;
        }

        let commit_count = nodes
            .values()
            .filter(|n| node_label(n) == "Commit")
            .count();
        println!("== Mapa del Hub PostgreSQL ({}) ==", self.hub_location);
        println!(
            "Proyectos: {} | Microservicios: {} | Dependencias: {} | Commits: {}",
            projects.len(),
            micros.len(),
            deps.len(),
            commit_count
        );
        println!();
        let mut project_names: Vec<&String> = projects.keys().collect();
        project_names.sort();
        for project in project_names {
            let graphify_out = nodes
                .get(project)
                .and_then(|n| n.get("graphify_out"))
                .and_then(Value::as_str)
                .filter(|s| !s.is_empty());
            match graphify_out {
                Some(g) => println!("[{project}]  [graphify: {g}]"),
                None => println!("[{project}]"),
            }
            let mut services = projects.get(project).cloned().unwrap_or_default();
            services.sort();
            if services.is_empty() {
                println!("   (sin microservicios registrados)");
            }
            for sid in &services {
                let short = sid.split_once('/').map(|(_, s)| s).unwrap_or(sid);
                let mut tags: Vec<String> = Vec::new();
                if micros
                    .get(sid)
                    .and_then(|n| n.get("tipo"))
                    .and_then(Value::as_str)
                    == Some("transversal")
                {
                    tags.push("transversal".to_string());
                }
                if let Some(count) = dependents.get(sid).filter(|&&c| c > 0) {
                    tags.push(format!("{count} dependiente(s)"));
                }
                let mut outgoing: Vec<String> = deps
                    .iter()
                    .filter(|e| edge_str(e, "source") == *sid)
                    .map(|e| edge_str(e, "target"))
                    .collect();
                outgoing.sort();
                let suffix = if tags.is_empty() {
                    String::new()
                } else {
                    format!(" ({})", tags.join(", "))
                };
                let dep_text = if outgoing.is_empty() {
                    String::new()
                } else {
                    format!(" -> depende de: {}", outgoing.join(", "))
                };
                println!("   - {short}{suffix}{dep_text}");
            }
            println!();
        }

        let mut cross: Vec<(String, String)> = deps
            .iter()
            .filter(|e| {
                let s = edge_str(e, "source");
                let t = edge_str(e, "target");
                s.split('/').next() != t.split('/').next()
            })
            .map(|e| (edge_str(e, "source"), edge_str(e, "target")))
            .collect();
        if cross.is_empty() {
            println!("Dependencias entre proyectos: ninguna");
        } else {
            println!("Dependencias entre proyectos:");
            cross.sort();
            for (source, target) in cross {
                println!("   {source} --> {target}");
            }
        }
        Ok(())
    }
}
