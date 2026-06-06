#!/usr/bin/env python3
"""Memoria distribuida del Harness Process.

Mantiene un hub compartido en PostgreSQL por defecto. El modo JSON local usa
~/.harness-hub/graph_db.json.
Ids de microservicio: <proyecto>/<servicio>.
"""
import argparse
import hashlib
import json
import os
import subprocess
from contextlib import contextmanager

try:
    import fcntl
    HAVE_FCNTL = True
except ImportError:
    HAVE_FCNTL = False

BASE_DIR = os.path.dirname(os.path.abspath(__file__))


def _repo_root():
    """Raiz multi-repo: en layout 'subdir' es el padre de BASE_DIR."""
    env = os.environ.get("HARNESS_REPO_ROOT")
    if env:
        return os.path.abspath(env)
    try:
        with open(os.path.join(BASE_DIR, ".harness_layout"), encoding="utf-8") as fh:
            if fh.read().strip() == "subdir":
                return os.path.dirname(BASE_DIR)
    except OSError:
        pass
    return BASE_DIR


REPO_ROOT = _repo_root()
PROJECT = os.environ.get("HARNESS_PROJECT") or os.path.basename(REPO_ROOT)
HUB_DIR = os.environ.get("HARNESS_HUB") or os.path.join(os.path.expanduser("~"), ".harness-hub")
GRAPH_DB_FILE = os.path.join(HUB_DIR, "graph_db.json")
PROGRESS_DIR = os.path.join(HUB_DIR, "progress")
LOCK_FILE = os.path.join(HUB_DIR, ".lock")


@contextmanager
def hub_lock():
    os.makedirs(HUB_DIR, exist_ok=True)
    with open(LOCK_FILE, "w", encoding="utf-8") as f:
        if HAVE_FCNTL:
            fcntl.flock(f.fileno(), fcntl.LOCK_EX)
        try:
            yield
        finally:
            if HAVE_FCNTL:
                fcntl.flock(f.fileno(), fcntl.LOCK_UN)



try:
    import psycopg2
    import psycopg2.extensions
    from psycopg2.extras import Json
    HAVE_PSYCOPG2 = True
except ImportError:
    HAVE_PSYCOPG2 = False

class PgGraphStore:
    def __init__(self, connection):
        self.connection = connection
        self.nodes = {}
        self.edges = []
        self._init_db()

    def _init_db(self):
        with psycopg2.connect(**self.connection) as conn:
            with conn.cursor() as cur:
                cur.execute('''
                    CREATE TABLE IF NOT EXISTS graph_nodes (
                        id VARCHAR PRIMARY KEY,
                        label VARCHAR,
                        props JSONB
                    );
                ''')
                cur.execute('''
                    CREATE TABLE IF NOT EXISTS graph_edges (
                        source VARCHAR,
                        target VARCHAR,
                        type VARCHAR,
                        props JSONB,
                        PRIMARY KEY (source, target, type)
                    );
                ''')
            conn.commit()

    def add_node(self, label, props):
        nid = props["_id"]
        node = self.nodes.get(nid, {})
        node.update(props)
        node["_label"] = label
        self.nodes[nid] = node

    def add_edge(self, etype, source, target, **props):
        edge = {"type": etype, "source": source, "target": target}
        edge.update(props)
        if edge not in self.edges:
            self.edges.append(edge)

    def load(self, data=None):
        self.nodes = {}
        self.edges = []
        with psycopg2.connect(**self.connection) as conn:
            with conn.cursor() as cur:
                cur.execute("SELECT id, label, props FROM graph_nodes;")
                for row in cur.fetchall():
                    props = row[2] or {}
                    props["_label"] = row[1]
                    self.nodes[row[0]] = props
                    
                cur.execute("SELECT source, target, type, props FROM graph_edges;")
                for row in cur.fetchall():
                    edge = {"source": row[0], "target": row[1], "type": row[2]}
                    if row[3]:
                        edge.update(row[3])
                    self.edges.append(edge)

    def to_dict(self):
        return {"nodes": self.nodes, "edges": self.edges}

    def save(self):
        with psycopg2.connect(**self.connection) as conn:
            with conn.cursor() as cur:
                for nid, props in self.nodes.items():
                    label = props.get("_label", "Unknown")
                    p = dict(props)
                    p.pop("_label", None)
                    cur.execute('''
                        INSERT INTO graph_nodes (id, label, props)
                        VALUES (%s, %s, %s)
                        ON CONFLICT (id) DO UPDATE SET
                        label = EXCLUDED.label,
                        props = graph_nodes.props || EXCLUDED.props;
                    ''', (nid, label, Json(p)))
                
                for edge in self.edges:
                    source = edge["source"]
                    target = edge["target"]
                    etype = edge["type"]
                    p = dict(edge)
                    p.pop("source", None)
                    p.pop("target", None)
                    p.pop("type", None)
                    cur.execute('''
                        INSERT INTO graph_edges (source, target, type, props)
                        VALUES (%s, %s, %s, %s)
                        ON CONFLICT (source, target, type) DO UPDATE SET
                        props = COALESCE(graph_edges.props, '{}'::jsonb) || EXCLUDED.props;
                    ''', (source, target, etype, Json(p)))
            conn.commit()

class GraphStore:
    def __init__(self):
        self.nodes = {}
        self.edges = []

    def add_node(self, label, props):
        nid = props["_id"]
        node = self.nodes.get(nid, {})
        node.update(props)
        node["_label"] = label
        self.nodes[nid] = node

    def add_edge(self, etype, source, target, **props):
        edge = {"type": etype, "source": source, "target": target}
        edge.update(props)
        if edge not in self.edges:
            self.edges.append(edge)

    def load(self, data):
        self.nodes = data.get("nodes", {})
        self.edges = data.get("edges", [])

    def to_dict(self):
        return {"nodes": self.nodes, "edges": self.edges}


def qualify(name):
    return name if "/" in name else f"{PROJECT}/{name}"


def artifact_id(path):
    safe = path.replace(os.sep, "__").replace("/", "__").replace(".", "_")
    digest = hashlib.sha1(path.encode("utf-8")).hexdigest()[:8]
    return f"{safe}_{digest}"


def is_repo_root(path):
    try:
        result = subprocess.run(
            ["git", "-C", path, "rev-parse", "--show-toplevel"],
            stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL,
            text=True,
            check=False,
        )
        return result.returncode == 0 and os.path.realpath(result.stdout.strip()) == os.path.realpath(path)
    except OSError:
        return os.path.isdir(os.path.join(path, ".git"))


class GraphMemoryManager:

    def __init__(self):
        env_file = os.path.join(HUB_DIR, ".env")
        if os.path.exists(env_file):
            with open(env_file, "r", encoding="utf-8") as f:
                for line in f:
                    line = line.strip()
                    if line and not line.startswith("#") and "=" in line:
                        key, _, val = line.partition("=")
                        os.environ.setdefault(key.strip(), val.strip().strip("'\""))

        backend_file = os.path.join(BASE_DIR, ".harness_backend")
        backend_default = "postgres"
        try:
            with open(backend_file, encoding="utf-8") as fh:
                backend_default = fh.read().strip() or backend_default
        except OSError:
            pass
        use_postgres = os.environ.get(
            "USE_POSTGRES", "1" if backend_default == "postgres" else "0"
        ).strip().lower()
        postgres_requested = use_postgres not in {"0", "false", "no", "off"}
        if postgres_requested and not HAVE_PSYCOPG2:
            raise SystemExit(
                "USE_POSTGRES esta activo pero psycopg2 no esta instalado. "
                "Vuelve a ejecutar setup_harness.sh para instalarlo."
            )
        self.use_postgres = postgres_requested

        if self.use_postgres:
            required = ("DB_HOST", "DB_USER", "DB_PASSWORD")
            missing = [name for name in required if not os.environ.get(name)]
            if missing:
                raise SystemExit(
                    "USE_POSTGRES esta activo pero faltan variables: "
                    + ", ".join(missing)
                )
            connection = {
                "dbname": os.environ.get("DB_NAME", "postgres"),
                "user": os.environ["DB_USER"],
                "password": os.environ["DB_PASSWORD"],
                "host": os.environ["DB_HOST"],
                "port": os.environ.get("DB_PORT", "5432"),
                "sslmode": os.environ.get("DB_SSL_MODE", "require"),
                "connect_timeout": 10,
            }
            self.graph = PgGraphStore(connection)
        else:
            self.graph = GraphStore()
            
        os.makedirs(HUB_DIR, exist_ok=True)
        os.makedirs(PROGRESS_DIR, exist_ok=True)



    def load(self):
        if self.use_postgres:
            self.graph.load()
        else:
            if os.path.exists(GRAPH_DB_FILE):
                with open(GRAPH_DB_FILE, "r", encoding="utf-8") as f:
                    self.graph.load(json.load(f))
            else:
                self.graph = GraphStore()



    def save(self):
        if self.use_postgres:
            self.graph.save()
        else:
            tmp = GRAPH_DB_FILE + ".tmp"
            with open(tmp, "w", encoding="utf-8") as f:
                json.dump(self.graph.to_dict(), f, indent=2, ensure_ascii=False)
            os.replace(tmp, GRAPH_DB_FILE)


    def discover(self):
        found = []
        with hub_lock():
            self.load()
            project_props = {"_id": PROJECT, "path": REPO_ROOT}
            graphify_out = os.path.join(REPO_ROOT, "graphify-out")
            if os.path.exists(os.path.join(graphify_out, "graph.json")):
                project_props["graphify_out"] = graphify_out
            self.graph.add_node("Proyecto", project_props)
            for entry in sorted(os.listdir(REPO_ROOT)):
                path = os.path.join(REPO_ROOT, entry)
                if os.path.realpath(path) == os.path.realpath(BASE_DIR):
                    continue  # el propio arnes no es un microservicio
                if os.path.isdir(path) and is_repo_root(path):
                    qid = f"{PROJECT}/{entry}"
                    self.graph.add_node("Microservicio", {"_id": qid, "proyecto": PROJECT, "servicio": entry, "path": path})
                    self.graph.add_edge("CONTIENE", PROJECT, qid)
                    found.append(entry)
            self.save()
        listing = ", ".join(found) if found else "(ninguno)"
        print(f"[Memoria] Proyecto '{PROJECT}' en {HUB_DIR}: {len(found)} microservicio(s): {listing}")

    def sync_git(self, commit_hash, files, microservice):
        qserv = f"{PROJECT}/{microservice}"
        with hub_lock():
            self.load()
            self.graph.add_node("Commit", {"_id": commit_hash, "proyecto": PROJECT, "microservicio": microservice})
            self.graph.add_node("Agente", {"_id": "Agente_Implementador"})
            self.graph.add_edge("REALIZO", "Agente_Implementador", commit_hash)
            progress_path = os.path.join(PROGRESS_DIR, PROJECT, microservice)
            os.makedirs(progress_path, exist_ok=True)
            for file_path in files:
                if not file_path:
                    continue
                aid = artifact_id(file_path)
                nid = f"{qserv}:{aid}"
                self.graph.add_node(
                    "Artefacto",
                    {"_id": nid, "ruta": file_path, "estado": "MODIFICADO_GIT", "proyecto": PROJECT, "microservicio": microservice},
                )
                self.graph.add_edge("MODIFICO", commit_hash, nid)
                with open(os.path.join(progress_path, f"{aid}.json"), "w", encoding="utf-8") as f:
                    json.dump({"_id": nid, "ruta": file_path, "estado": "MODIFICADO_GIT", "commit": commit_hash}, f, indent=2, ensure_ascii=False)
            self.save()
        print(f"[Memoria] Commit {commit_hash[:7]} sincronizado para {qserv}")

    def link(self, consumer, target, transversal=False, origin="manual"):
        c = qualify(consumer)
        t = qualify(target)
        with hub_lock():
            self.load()
            self.graph.add_node("Microservicio", {"_id": c})
            props = {"_id": t}
            if transversal:
                props["tipo"] = "transversal"
            self.graph.add_node("Microservicio", props)
            self.graph.add_edge("DEPENDE_DE", c, t, origen=origin)
            self.save()
        suffix = " [transversal]" if transversal else ""
        print(f"[Memoria] {c} depende de {t}{suffix}")

    def unmark(self, service):
        s = qualify(service)
        with hub_lock():
            self.load()
            node = self.graph.nodes.get(s)
            if node and node.get("tipo") == "transversal":
                del node["tipo"]
                self.save()
                print(f"[Memoria] {s} ya no esta marcado como transversal")
            else:
                print(f"[Memoria] {s} no estaba marcado como transversal")

    def impact(self, service):
        t = qualify(service)
        with hub_lock():
            self.load()
        affected = sorted(e["source"] for e in self.graph.edges if e.get("type") == "DEPENDE_DE" and e.get("target") == t)
        if affected:
            print(f"[Impacto] Si modificas '{t}', revisa: {', '.join(affected)}")
        else:
            print(f"[Impacto] Ningun microservicio registrado depende de '{t}'")

    def record_event(self, agent, action, artefacto, estado, metadata=None):
        qart = f"{PROJECT}/{artefacto}"
        with hub_lock():
            self.load()
            self.graph.add_node("Agente", {"_id": agent})
            node = {"_id": qart, "estado": estado, "proyecto": PROJECT}
            if metadata:
                node["metadata"] = metadata
            self.graph.add_node("Artefacto", node)
            self.graph.add_edge(action.upper(), agent, qart)
            ruta = os.path.join(PROGRESS_DIR, PROJECT)
            os.makedirs(ruta, exist_ok=True)
            with open(os.path.join(ruta, f"{artefacto}.json"), "w", encoding="utf-8") as f:
                json.dump(node, f, indent=2, ensure_ascii=False)
            self.save()
        print(f"[Memoria] {agent} --[{action}]--> {qart} ({estado})")

    def query_state(self, artefacto, microservice="raiz"):
        if microservice == "raiz":
            ruta = os.path.join(PROGRESS_DIR, PROJECT, f"{artefacto}.json")
        else:
            ruta = os.path.join(PROGRESS_DIR, PROJECT, microservice, f"{artefacto}.json")
        if os.path.exists(ruta):
            with open(ruta, "r", encoding="utf-8") as f:
                print(f.read())
        else:
            print(f"Error: Artefacto '{artefacto}' no encontrado en {PROJECT}/{microservice}.")

    def derive_from_graphify(self, graph_path=None):
        import re
        graph_path = graph_path or os.path.join(REPO_ROOT, "graphify-out", "graph.json")
        if not os.path.exists(graph_path):
            print(f"[graphify->hub] No existe {graph_path}; nada que derivar.")
            return
        with open(graph_path, "r", encoding="utf-8") as f:
            graph = json.load(f)
        nodes = {n.get("id"): n for n in graph.get("nodes", [])}
        dep_relations = {"references", "implements", "shares_data_with", "depends_on", "uses", "cites"}
        convention = re.compile(r"convention|policy|guideline|standard|lint|layout|\\badr\\b", re.I)

        def service_for(node_id):
            source = (nodes.get(node_id) or {}).get("source_file", "") or ""
            match = re.search(r"(ms-[a-z0-9-]+-service|[a-z0-9-]+-ui)", source)
            return match.group(1) if match else None

        # Denylist de dependencias espurias (falsos positivos de la extraccion
        # semantica: contratos compartidos JWT/audit/infra leidos como deps).
        # Una linea "a->b" por par a suprimir; '#' para comentarios.
        denylist = set()
        deny_path = os.path.join(REPO_ROOT, "harness_deps_deny.txt")
        if os.path.exists(deny_path):
            with open(deny_path, "r", encoding="utf-8") as fh:
                for line in fh:
                    line = line.split("#", 1)[0]
                    line = re.sub(r"\s+", "", line)
                    if line:
                        denylist.add(line)

        pairs = {}
        consumers_by_target_node = {}
        raw = []
        for edge in graph.get("links", []):
            a = service_for(edge.get("source"))
            b = service_for(edge.get("target"))
            if not (a and b and a != b):
                continue
            if edge.get("relation") not in dep_relations:
                continue
            if f"{a}->{b}" in denylist:
                continue
            target_node = edge.get("target")
            if convention.search((nodes.get(target_node) or {}).get("label", "")):
                continue
            consumers_by_target_node.setdefault(target_node, set()).add(a)
            raw.append((a, b, target_node))

        for a, b, target_node in raw:
            pairs[(a, b)] = pairs.get((a, b), False) or len(consumers_by_target_node.get(target_node, set())) >= 3

        if not pairs:
            print("[graphify->hub] Sin dependencias derivables.")
            return

        with hub_lock():
            self.load()
            self.graph.edges = [
                e for e in self.graph.edges
                if not (e.get("type") == "DEPENDE_DE" and e.get("origen") == "graphify")
            ]
            existing = {(e.get("source"), e.get("target")) for e in self.graph.edges if e.get("type") == "DEPENDE_DE"}
            for (a, b), transversal in sorted(pairs.items()):
                c, d = qualify(a), qualify(b)
                self.graph.add_node("Microservicio", {"_id": c})
                props = {"_id": d}
                if transversal:
                    props["tipo"] = "transversal"
                self.graph.add_node("Microservicio", props)
                if (c, d) not in existing:
                    self.graph.add_edge("DEPENDE_DE", c, d, origen="graphify")
            self.save()
        summary = ", ".join(f"{a}->{b}{' [tv]' if tv else ''}" for (a, b), tv in sorted(pairs.items()))
        print(f"[graphify->hub] {len(pairs)} dependencia(s): {summary}")

    def map(self):
        with hub_lock():
            self.load()
        nodes = self.graph.nodes
        edges = self.graph.edges
        micros = {nid: n for nid, n in nodes.items() if n.get("_label") == "Microservicio"}
        deps = [e for e in edges if e.get("type") == "DEPENDE_DE"]
        projects = {}
        for nid in micros:
            projects.setdefault(nid.split("/", 1)[0], []).append(nid)
        for nid, node in nodes.items():
            if node.get("_label") == "Proyecto":
                projects.setdefault(nid, [])

        dependents = {}
        for edge in deps:
            dependents.setdefault(edge["target"], []).append(edge["source"])

        commit_count = sum(1 for node in nodes.values() if node.get("_label") == "Commit")
        print(f"== Mapa del Hub ({HUB_DIR}) ==")
        print(f"Proyectos: {len(projects)} | Microservicios: {len(micros)} | Dependencias: {len(deps)} | Commits: {commit_count}")
        print()
        for project in sorted(projects):
            graphify_out = (nodes.get(project) or {}).get("graphify_out")
            print(f"[{project}]" + (f"  [graphify: {graphify_out}]" if graphify_out else ""))
            services = sorted(projects[project])
            if not services:
                print("   (sin microservicios registrados)")
            for sid in services:
                short = sid.split("/", 1)[1] if "/" in sid else sid
                tags = []
                if (micros.get(sid) or {}).get("tipo") == "transversal":
                    tags.append("transversal")
                if dependents.get(sid):
                    tags.append(f"{len(dependents[sid])} dependiente(s)")
                outgoing = sorted(e["target"] for e in deps if e["source"] == sid)
                suffix = f" ({', '.join(tags)})" if tags else ""
                dep_text = " -> depende de: " + ", ".join(outgoing) if outgoing else ""
                print(f"   - {short}{suffix}{dep_text}")
            print()

        cross = [e for e in deps if e["source"].split("/", 1)[0] != e["target"].split("/", 1)[0]]
        if cross:
            print("Dependencias entre proyectos:")
            for edge in sorted(cross, key=lambda x: (x["source"], x["target"])):
                print(f"   {edge['source']} --> {edge['target']}")
        else:
            print("Dependencias entre proyectos: ninguna")


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("command", choices=["descubrir", "mapa", "impacto", "vincular", "desmarcar", "sync_git", "vincular-grafo", "registrar", "consultar"])
    parser.add_argument("--microservicio", default="raiz")
    parser.add_argument("--destino")
    parser.add_argument("--transversal", action="store_true")
    parser.add_argument("--artefacto")
    parser.add_argument("--meta")
    parser.add_argument("--agente", default="AgentCLI")
    parser.add_argument("--accion")
    parser.add_argument("--estado")
    args = parser.parse_args()
    manager = GraphMemoryManager()

    if args.command == "registrar":
        if not (args.accion and args.estado and args.artefacto):
            parser.error("registrar requiere --accion, --estado y --artefacto")
        manager.record_event(args.agente, args.accion, args.artefacto, args.estado, args.meta)
    elif args.command == "consultar":
        if not args.artefacto:
            parser.error("--artefacto es requerido para consultar")
        manager.query_state(args.artefacto, args.microservicio)
    elif args.command == "descubrir":
        manager.discover()
    elif args.command == "mapa":
        manager.map()
    elif args.command == "impacto":
        manager.impact(args.microservicio)
    elif args.command == "vincular":
        if not args.destino:
            parser.error("--destino es requerido para vincular")
        manager.link(args.microservicio, args.destino, args.transversal)
    elif args.command == "desmarcar":
        manager.unmark(args.microservicio)
    elif args.command == "sync_git":
        if not args.artefacto:
            parser.error("--artefacto es requerido para sync_git")
        files = args.meta.split(",") if args.meta else []
        manager.sync_git(args.artefacto, files, args.microservicio)
    elif args.command == "vincular-grafo":
        manager.derive_from_graphify()


if __name__ == "__main__":
    main()
