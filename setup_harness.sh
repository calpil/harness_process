#!/bin/bash
# Harness Process - instalador canonico (best-of).
# Unifica las variantes previas (setup basico + improved) en un solo instalador:
#   - Memoria hub compartida (graph_memory.py) con mapa/impacto/vincular/
#     desmarcar/sync_git/vincular-grafo + registrar/consultar.
#   - Integracion graphify (estructura automatica, rebuild semantico, hub).
#   - Capa opcional de subagentes (lider/implementer/reviewer, harness.py).
#   - Respaldos *.bak.* archivados bajo bkp/ (HARNESS_BKP_DIR para overridear).
set -Eeuo pipefail
IFS=$'\n\t'

# Subagentes y graphify quedan activos por defecto (opt-out con --no-subagents / --no-graphify).
INSTALL_GRAPHIFY=1
WITH_SUBAGENTS=1
FORCE=0

usage() {
    cat <<'USAGE'
Uso: ./setup_harness.sh [opciones]

Opciones:
  --no-subagents       Omite la capa lider/implementer/reviewer (se instala por defecto).
  --no-graphify        No asegura graphify (por defecto se asegura, instalandolo si falta).
  --with-subagents     Ya es el default; se mantiene por compatibilidad.
  --install-graphify   Ya es el default; se mantiene por compatibilidad.
  --force              Sobrescribe archivos sin crear backup.
  -h, --help           Muestra esta ayuda.

Por defecto instala la capa de subagentes y asegura graphify (lo instala si
falta). Respalda archivos existentes bajo bkp/ (configurable con HARNESS_BKP_DIR).
USAGE
}

while [ "$#" -gt 0 ]; do
    case "$1" in
        --with-subagents) WITH_SUBAGENTS=1 ;;
        --install-graphify) INSTALL_GRAPHIFY=1 ;;
        --no-subagents) WITH_SUBAGENTS=0 ;;
        --no-graphify) INSTALL_GRAPHIFY=0 ;;
        --force) FORCE=1 ;;
        -h|--help) usage; exit 0 ;;
        *)
            echo "[!] Opcion desconocida: $1" >&2
            usage >&2
            exit 2
            ;;
    esac
    shift
done

timestamp() {
    date +%Y%m%d%H%M%S
}

BKP_DIR="bkp"

# Calcula la ruta del backup dentro de bkp/, preservando la estructura de
# subcarpetas del archivo original para evitar colisiones de nombres.
backup_path() {
    target="$1"
    rel="${target#./}"
    dest="$BKP_DIR/${rel}.bak.$(timestamp)"
    mkdir -p "$(dirname "$dest")"
    echo "$dest"
}

backup_file() {
    target="$1"
    if [ "$FORCE" -eq 0 ] && [ -e "$target" ]; then
        backup="$(backup_path "$target")"
        cp -p "$target" "$backup"
        echo "[Harness] Backup creado: $backup"
    fi
}

archive_legacy_file() {
    target="$1"
    reason="$2"
    if [ -f "$target" ]; then
        backup="$(backup_path "$target")"
        mv "$target" "$backup"
        echo "[Harness] $reason; archivado como $backup"
    fi
}

write_file_notice() {
    echo "   -> $1"
}

PROJECT_NAME="${HARNESS_PROJECT:-$(basename "$(pwd)")}"

echo "== Instalando Harness Process en: $(pwd) =="
echo "   proyecto: $PROJECT_NAME"
echo "   subagentes: $([ "$WITH_SUBAGENTS" -eq 1 ] && echo si || echo no)"
echo "   graphify:   $([ "$INSTALL_GRAPHIFY" -eq 1 ] && echo asegurar || echo no)"

mkdir -p .claude
[ "$WITH_SUBAGENTS" -eq 1 ] && mkdir -p .claude/agents docs progress

archive_legacy_file ".claudemd" ".claudemd es obsoleto; Claude Code lee CLAUDE.md"
archive_legacy_file "validate_aks.sh" "validate_aks.sh quedo obsoleto"

generated=(
    "CLAUDE.md"
    ".claude/settings.json"
    "graph_memory.py"
    "init.sh"
    "validate_ui.sh"
    "debug_ui.js"
    "commit_guard.sh"
    "harness_status.sh"
    "harness_check.sh"
    "harness.py"
)
if [ "$WITH_SUBAGENTS" -eq 1 ]; then
    generated+=(
        "AGENTS.md"
        "CHECKPOINTS.md"
        "feature_list.json"
        "progress/current.md"
        "progress/history.md"
        "docs/architecture.md"
        "docs/conventions.md"
        "docs/verification.md"
        ".claude/agents/leader.md"
        ".claude/agents/implementer.md"
        ".claude/agents/reviewer.md"
    )
fi

for f in "${generated[@]}"; do
    backup_file "$f"
done

echo "Generando CLAUDE.md..."
cat <<'CLAUDE_MD_EOF' > CLAUDE.md
# Harness Process

Estas operando en la raiz de un arnes multi-repo que coordina microservicios,
memoria transversal compartida (hub), un grafo de conocimiento del codigo
(graphify), validaciones y, si esta instalado, subagentes.

## Protocolo obligatorio

Antes de tocar codigo, arquitectura o dependencias entre servicios, en ESTE
orden (no son opcionales):

1. Revisa el mapa del hub que `init.sh` imprime en cada `SessionStart`, o:
   `python3 graph_memory.py mapa`
2. Si vas a modificar un servicio, revisa su radio de impacto:
   `python3 graph_memory.py impacto --microservicio <proyecto>/<servicio>`
3. Si existe `graphify-out/graph.json`, consulta el grafo ANTES de leer a
   ciegas: `graphify query "<pregunta de la task>"`
4. Trabaja dentro del microservicio correspondiente; NUNCA programes en la raiz
   (`cd <microservicio>` y al terminar vuelve con `cd ..`).
5. Valida los servicios afectados y deja evidencia.
6. Al cerrar, los repos afectados deben quedar limpios o con commits hechos,
   segun la politica configurada por `HARNESS_COMMIT_GUARD_MODE`.

## Hub de memoria (~/.harness-hub) vs graphify (graphify-out/)

Son sistemas SEPARADOS: graphify NO usa el hub y el hub NO usa graphify.

- **Hub** (`~/.harness-hub/graph_db.json`, configurable con `HARNESS_HUB`):
  rastrea proyectos, microservicios, commits y dependencias entre servicios.
  Es COMPARTIDO entre todos los proyectos; los ids se namespacean como
  `<proyecto>/<servicio>`. `init.sh` lo siembra solo en cada `SessionStart`.
  Si agregas un microservicio a mitad de sesion, reinicia Claude Code.
- **graphify** (`graphify-out/`): grafo del CONTENIDO del codigo (funciones,
  tipos, conceptos, comunidades). Para preguntas de arquitectura o "como
  funciona X", consulta primero `graphify query "<pregunta>"`.

## Servicios transversales y dependencias

- `impacto` ve dependencias de TODOS los proyectos del hub. Para un servicio de
  otro proyecto usa el id calificado `<proyecto>/<servicio>`.
- Declarar que un servicio depende de otro:
  `python3 graph_memory.py vincular --microservicio <consumidor> --destino <proyecto>/<servicio>`.
  Agrega `--transversal` SOLO si el destino es un servicio nucleo/compartido
  consumido por varios proyectos. Para quitar la marca:
  `python3 graph_memory.py desmarcar --microservicio <servicio>`.
- Registrar/consultar progreso de un artefacto en el hub:
  `python3 graph_memory.py registrar --accion <accion> --estado <estado> --artefacto <nombre> [--meta ...]`
  y `python3 graph_memory.py consultar --artefacto <nombre> [--microservicio <servicio>]`.
- Tras commitear, valida los proyectos afectados (tests unitarios; para
  frontends `"$CLAUDE_PROJECT_DIR/validate_ui.sh" <url-dev-server>`,
  se auto-localiza desde cualquier carpeta).

## graphify: ciclo de actualizacion

- **Estructura (automatico, sin LLM):** el hook `post-commit` de cada
  microservicio corre `graphify update` (solo AST, en segundo plano) y marca
  `graphify-out/.graphify_stale`. No gasta tokens.
- **Construccion (manual, la 1a vez):** abre Claude en esta carpeta y corre
  `/graphify`. El arnes ya NO lo construye en segundo plano.
- **Rebuild semantico tras commits:** si existe `graphify-out/.graphify_stale`,
  ejecuta `/graphify --update`, borra el marcador
  (`rm -f graphify-out/.graphify_stale`) y refresca el hub con
  `python3 graph_memory.py vincular-grafo`.

## Commits

- PROHIBIDO incluir firmas o trailers de IA (`Co-Authored-By`,
  `Generated with Claude`); un hook `commit-msg` lo refuerza.
- Usa Conventional Commits desde terminal. Al commitear, el hub se actualiza
  solo. Commitea CADA microservicio afectado antes de cerrar la task; el hook
  `Stop` te bloquea si quedan cambios sin commitear
  (`HARNESS_COMMIT_GUARD_MODE=block|warn|off`).

## Subagentes

Si existe `AGENTS.md`, usalo como mapa progresivo. Si existen agentes en
`.claude/agents/`, aplica este flujo:

- Lider: decide alcance, impacto y delegacion.
- Implementer: modifica una unidad concreta y escribe reporte en `progress/`.
- Reviewer: verifica tests, impacto, checkpoints y estado del repo.

Los subagentes deben persistir resultados en archivos de `progress/`; las
respuestas cortas en chat no reemplazan la evidencia.
CLAUDE_MD_EOF
write_file_notice "CLAUDE.md"

echo "Generando .claude/settings.json..."
if [ "$WITH_SUBAGENTS" -eq 1 ]; then
    cat <<'SETTINGS_EOF' > .claude/settings.json
{
  "attribution": {
    "commit": "",
    "pr": ""
  },
  "hooks": {
    "SessionStart": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "bash \"${CLAUDE_PROJECT_DIR}/init.sh\" && bash \"${CLAUDE_PROJECT_DIR}/harness_status.sh\""
          }
        ]
      }
    ],
    "PostToolUse": [
      {
        "matcher": "Edit|Write|MultiEdit",
        "hooks": [
          {
            "type": "command",
            "command": "bash \"${CLAUDE_PROJECT_DIR}/harness_status.sh\" --brief"
          }
        ]
      }
    ],
    "Stop": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "bash \"${CLAUDE_PROJECT_DIR}/harness_check.sh\""
          }
        ]
      }
    ]
  }
}
SETTINGS_EOF
else
    cat <<'SETTINGS_EOF' > .claude/settings.json
{
  "attribution": {
    "commit": "",
    "pr": ""
  },
  "hooks": {
    "SessionStart": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "bash \"${CLAUDE_PROJECT_DIR}/init.sh\""
          }
        ]
      }
    ],
    "Stop": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "bash \"${CLAUDE_PROJECT_DIR}/commit_guard.sh\""
          }
        ]
      }
    ]
  }
}
SETTINGS_EOF
fi
write_file_notice ".claude/settings.json"

echo "Generando graph_memory.py..."
cat <<'GM_PY_EOF' > graph_memory.py
#!/usr/bin/env python3
"""Memoria distribuida del Harness Process.

Mantiene un hub compartido por defecto en ~/.harness-hub/graph_db.json.
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
PROJECT = os.environ.get("HARNESS_PROJECT") or os.path.basename(BASE_DIR)
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
        self.graph = GraphStore()
        os.makedirs(HUB_DIR, exist_ok=True)
        os.makedirs(PROGRESS_DIR, exist_ok=True)

    def load(self):
        if os.path.exists(GRAPH_DB_FILE):
            with open(GRAPH_DB_FILE, "r", encoding="utf-8") as f:
                self.graph.load(json.load(f))
        else:
            self.graph = GraphStore()

    def save(self):
        tmp = GRAPH_DB_FILE + ".tmp"
        with open(tmp, "w", encoding="utf-8") as f:
            json.dump(self.graph.to_dict(), f, indent=2, ensure_ascii=False)
        os.replace(tmp, GRAPH_DB_FILE)

    def discover(self):
        found = []
        with hub_lock():
            self.load()
            project_props = {"_id": PROJECT, "path": BASE_DIR}
            graphify_out = os.path.join(BASE_DIR, "graphify-out")
            if os.path.exists(os.path.join(graphify_out, "graph.json")):
                project_props["graphify_out"] = graphify_out
            self.graph.add_node("Proyecto", project_props)
            for entry in sorted(os.listdir(BASE_DIR)):
                path = os.path.join(BASE_DIR, entry)
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
        graph_path = graph_path or os.path.join(BASE_DIR, "graphify-out", "graph.json")
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
        deny_path = os.path.join(BASE_DIR, "harness_deps_deny.txt")
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
    parser.add_argument("--agente", default="ClaudeCode")
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
GM_PY_EOF
chmod +x graph_memory.py
write_file_notice "graph_memory.py"

echo "Generando init.sh..."
cat <<'INIT_SH_EOF' > init.sh
#!/bin/bash
set -Eeuo pipefail
IFS=$'\n\t'

ROOT="$(cd "$(dirname "$0")" && pwd -P)"
cd "$ROOT"

# Carpeta donde se archivan los respaldos *.bak.* generados por el harness.
BKP_DIR="${HARNESS_BKP_DIR:-bkp}"

echo "== Inicializando Harness Process en $ROOT =="

if ! command -v git >/dev/null 2>&1; then
    echo "[!] git no esta disponible." >&2
    exit 1
fi

if ! command -v python3 >/dev/null 2>&1; then
    echo "[!] python3 no esta disponible; graph_memory.py lo requiere." >&2
    exit 1
fi

OS="$(uname -s)"
DESIRED_AUTOCRLF=""
case "$OS" in
    Darwin|Linux) DESIRED_AUTOCRLF="input" ;;
    MINGW*|MSYS*|CYGWIN*) DESIRED_AUTOCRLF="true" ;;
esac

echo "== Sincronizando hooks de microservicios =="
for repo in */; do
    REPO_DIR="${repo%/}"
    [ -d "$REPO_DIR" ] || continue
    if ! git -C "$REPO_DIR" rev-parse --show-toplevel >/dev/null 2>&1; then
        continue
    fi
    REPO_ABS="$(cd "$REPO_DIR" && pwd -P)"
    GIT_TOP="$(git -C "$REPO_DIR" rev-parse --show-toplevel 2>/dev/null || true)"
    [ "$GIT_TOP" = "$REPO_ABS" ] || continue

    if [ -n "$DESIRED_AUTOCRLF" ]; then
        CURRENT_AUTOCRLF="$(git -C "$REPO_DIR" config --local --get core.autocrlf || true)"
        if [ "$CURRENT_AUTOCRLF" != "$DESIRED_AUTOCRLF" ]; then
            git -C "$REPO_DIR" config core.autocrlf "$DESIRED_AUTOCRLF"
            echo "   -> [Git] $REPO_DIR core.autocrlf=$DESIRED_AUTOCRLF"
        fi
    fi

    GIT_DIR="$(git -C "$REPO_DIR" rev-parse --git-dir)"
    case "$GIT_DIR" in
        /*) ;;
        *) GIT_DIR="$REPO_DIR/$GIT_DIR" ;;
    esac
    HOOKS_DIR="$GIT_DIR/hooks"
    mkdir -p "$HOOKS_DIR"
    POST_COMMIT="$HOOKS_DIR/post-commit"
    COMMIT_MSG="$HOOKS_DIR/commit-msg"

    for hook in "$POST_COMMIT" "$COMMIT_MSG"; do
        if [ -f "$hook" ] && ! grep -q "harness-managed-hook" "$hook"; then
            hook_bkp="$BKP_DIR/git-hooks"
            mkdir -p "$hook_bkp"
            backup="$hook_bkp/$(basename "$hook").bak.$(date +%Y%m%d%H%M%S)"
            cp "$hook" "$backup"
            echo "   -> [Backup] hook previo: $backup"
        fi
    done

    cat > "$POST_COMMIT" <<HOOKEOF
#!/bin/bash
# harness-managed-hook v6
set -u
HARNESS_ROOT="$ROOT"
MICROSERVICIO=\$(basename "\$(git rev-parse --show-toplevel)")
COMMIT_HASH=\$(git rev-parse HEAD)
ARCHIVOS=\$(git diff-tree --no-commit-id --name-only -r --root "\$COMMIT_HASH" | paste -sd "," -)
python3 "\$HARNESS_ROOT/graph_memory.py" sync_git --artefacto "\$COMMIT_HASH" --meta "\$ARCHIVOS" --microservicio "\$MICROSERVICIO" \
  || echo "[Harness] Aviso: no se pudo sincronizar memoria para \$MICROSERVICIO." >&2

export PATH="\$HOME/.local/bin:\$PATH"
if command -v graphify >/dev/null 2>&1 && [ -f "\$HARNESS_ROOT/graphify-out/graph.json" ]; then
    if mkdir "\$HARNESS_ROOT/graphify-out/.update.lock" 2>/dev/null; then
        (
            trap 'rmdir "\$HARNESS_ROOT/graphify-out/.update.lock" 2>/dev/null || true' EXIT
            cd "\$HARNESS_ROOT" || exit 0
            graphify update "\$HARNESS_ROOT" >/dev/null 2>&1 || true
            if printf '%s' "\$ARCHIVOS" | grep -qiE '(^|,)(README|AGENTS|[^,]+[.]md)(,|\$)'; then
                touch "\$HARNESS_ROOT/graphify-out/.graphify_stale" 2>/dev/null || true
            fi
            python3 "\$HARNESS_ROOT/graph_memory.py" vincular-grafo >/dev/null 2>&1 || true
        ) &
    fi
fi
HOOKEOF
    chmod +x "$POST_COMMIT"

    cat > "$COMMIT_MSG" <<'CMEOF'
#!/bin/sh
# harness-managed-hook v4
set -u
msg_file="${1:?commit message file missing}"
tmp="${msg_file}.harness.$$"
sed -E '/^Co-Authored-By:.*[Cc]laude/d; /^Generated with .*Claude/d' "$msg_file" > "$tmp" && mv "$tmp" "$msg_file"
rm -f "$tmp" "$msg_file.bak"
CMEOF
    chmod +x "$COMMIT_MSG"
    echo "   -> [Ok] $REPO_DIR conectado"
done

python3 "$ROOT/graph_memory.py" descubrir

if [ -f "$ROOT/graphify-out/graph.json" ]; then
    python3 "$ROOT/graph_memory.py" vincular-grafo || true
    if [ -f "$ROOT/graphify-out/.graphify_stale" ]; then
        echo "[graphify] Grafo desactualizado: corre '/graphify --update' y luego borra graphify-out/.graphify_stale."
    else
        echo "[graphify] Grafo de conocimiento al dia."
    fi
else
    echo "[graphify] Sin graphify-out/graph.json. Primera construccion manual: /graphify"
fi

echo ""
python3 "$ROOT/graph_memory.py" mapa
echo ""
echo "== [Ok] Harness listo =="
INIT_SH_EOF
chmod +x init.sh
write_file_notice "init.sh"

echo "Generando validate_ui.sh y debug_ui.js..."
cat <<'VAL_UI_EOF' > validate_ui.sh
#!/bin/bash
set -Eeuo pipefail

ROOT="$(cd "$(dirname "$0")" && pwd -P)"
cd "$ROOT"

TARGET_URL="${1:-http://localhost:5173}"
echo "== Validando UI en $TARGET_URL =="

if ! command -v node >/dev/null 2>&1; then
    echo "[!] Node.js no esta disponible." >&2
    exit 1
fi

if [ ! -d "node_modules/playwright" ]; then
    if ! command -v npm >/dev/null 2>&1; then
        echo "[!] Playwright no esta instalado y npm no esta disponible." >&2
        exit 1
    fi
    echo "[Harness] Instalando Playwright localmente..."
    npm install playwright --no-save
    npx playwright install chromium
fi

if ! node debug_ui.js "$TARGET_URL"; then
    echo "[!] UI con errores. Captura: $ROOT/debug-ui/ui_error_state.png" >&2
    exit 1
fi

echo "[Ok] UI verificada."
VAL_UI_EOF
chmod +x validate_ui.sh

cat <<'DEBUG_JS_EOF' > debug_ui.js
const { chromium } = require('playwright');
const path = require('path');
const fs = require('fs');

(async () => {
  const targetUrl = process.argv[2] || 'http://localhost:5173';
  let hasErrors = false;
  console.log(`[Playwright] Verificando UI en ${targetUrl}...`);

  const browser = await chromium.launch({ headless: true });
  const page = await browser.newPage();

  page.on('console', msg => {
    if (msg.type() === 'error') {
      console.error(`[Console Error] ${msg.text()}`);
      hasErrors = true;
    }
  });

  page.on('pageerror', exception => {
    console.error(`[Exception] ${exception}`);
    hasErrors = true;
  });

  page.on('requestfailed', request => {
    const failure = request.failure();
    console.error(`[Request Failed] ${request.method()} ${request.url()}${failure ? `: ${failure.errorText}` : ''}`);
    hasErrors = true;
  });

  try {
    await page.goto(targetUrl, { waitUntil: 'domcontentloaded', timeout: 20000 });
    await page.waitForLoadState('networkidle', { timeout: 5000 }).catch(() => {});
    await page.waitForSelector('body', { timeout: 5000 });
  } catch (error) {
    console.error(`[Timeout] ${error.message}`);
    hasErrors = true;
  }

  if (hasErrors) {
    const outDir = path.join(__dirname, 'debug-ui');
    fs.mkdirSync(outDir, { recursive: true });
    const shot = path.join(outDir, 'ui_error_state.png');
    try {
      await page.screenshot({ path: shot });
      console.log(`[Playwright] Captura guardada en ${shot}`);
    } catch (error) {
      console.error(`[Playwright] No se pudo guardar captura: ${error.message}`);
    }
    await browser.close();
    process.exit(1);
  }

  await browser.close();
  console.log('[Playwright] Exito: sin errores visibles.');
})();
DEBUG_JS_EOF
write_file_notice "validate_ui.sh / debug_ui.js"

echo "Generando guardas y estado..."
cat <<'STATUS_EOF' > harness_status.sh
#!/bin/bash
set -Eeuo pipefail

ROOT="${CLAUDE_PROJECT_DIR:-$(cd "$(dirname "$0")" && pwd -P)}"
BRIEF=0
[ "${1:-}" = "--brief" ] && BRIEF=1

cd "$ROOT"

dirty=""
for repo in "$ROOT"/*; do
    [ -d "$repo" ] || continue
    git -C "$repo" rev-parse --show-toplevel >/dev/null 2>&1 || continue
    repo_abs="$(cd "$repo" && pwd -P)"
    git_top="$(git -C "$repo" rev-parse --show-toplevel 2>/dev/null || true)"
    [ "$git_top" = "$repo_abs" ] || continue
    if [ -n "$(git -C "$repo" status --porcelain 2>/dev/null)" ]; then
        dirty="$dirty $(basename "$repo")"
    fi
done

if [ "$BRIEF" -eq 1 ]; then
    [ -n "$dirty" ] && echo "[Harness] Repos dirty:$dirty" || echo "[Harness] Repos limpios"
    exit 0
fi

echo "== Harness Status =="
if [ -f feature_list.json ]; then
    python3 harness.py status || true
fi
if [ -n "$dirty" ]; then
    echo "Repos con cambios:$dirty"
else
    echo "Repos con cambios: ninguno"
fi
python3 graph_memory.py mapa || true
STATUS_EOF
chmod +x harness_status.sh

cat <<'GUARD_EOF' > commit_guard.sh
#!/bin/sh
# harness-managed-hook v4
INPUT=$(cat 2>/dev/null)
MODE="${HARNESS_COMMIT_GUARD_MODE:-block}" # block | warn | off

[ "$MODE" = "off" ] && exit 0

STOP_HOOK_ACTIVE=0
printf '%s' "$INPUT" | grep -q '"stop_hook_active"[[:space:]]*:[[:space:]]*true' && STOP_HOOK_ACTIVE=1

ROOT="${CLAUDE_PROJECT_DIR:-$(pwd)}"
DIRTY=""
for repo in "$ROOT"/*; do
    [ -d "$repo" ] || continue
    git -C "$repo" rev-parse --show-toplevel >/dev/null 2>&1 || continue
    repo_abs=$(cd "$repo" && pwd -P)
    git_top=$(git -C "$repo" rev-parse --show-toplevel 2>/dev/null || true)
    [ "$git_top" = "$repo_abs" ] || continue
    if [ -n "$(git -C "$repo" status --porcelain 2>/dev/null)" ]; then
        DIRTY="$DIRTY $(basename "$repo")"
    fi
done

if [ -n "$DIRTY" ]; then
    echo "Cambios sin commitear en:$DIRTY" >&2
    echo "Haz commit por microservicio con Conventional Commits o usa HARNESS_COMMIT_GUARD_MODE=warn/off." >&2
    if [ "$MODE" = "warn" ] || [ "$STOP_HOOK_ACTIVE" -eq 1 ]; then
        exit 0
    fi
    exit 2
fi
exit 0
GUARD_EOF
chmod +x commit_guard.sh

cat <<'CHECK_SH_EOF' > harness_check.sh
#!/bin/bash
set -Eeuo pipefail

ROOT="${CLAUDE_PROJECT_DIR:-$(cd "$(dirname "$0")" && pwd -P)}"
cd "$ROOT"

MODE="${HARNESS_CHECK_MODE:-block}" # block | warn | off
[ "$MODE" = "off" ] && exit 0

failures=0

echo "== Harness Check =="

if [ -f feature_list.json ]; then
    python3 harness.py status || failures=$((failures + 1))
fi

if [ -f CHECKPOINTS.md ] && [ ! -s progress/current.md ]; then
    echo "[!] progress/current.md esta vacio; registra estado antes de cerrar." >&2
    failures=$((failures + 1))
fi

if [ -f graphify-out/.graphify_stale ]; then
    echo "[!] graphify-out/.graphify_stale existe; corre /graphify --update cuando aplique." >&2
    failures=$((failures + 1))
fi

if ! bash "$ROOT/commit_guard.sh"; then
    failures=$((failures + 1))
fi

if [ "$failures" -gt 0 ]; then
    echo "[Harness] Check fallo con $failures problema(s)." >&2
    [ "$MODE" = "warn" ] && exit 0
    exit 2
fi

echo "[Ok] Harness Check limpio."
CHECK_SH_EOF
chmod +x harness_check.sh
write_file_notice "harness_status.sh / commit_guard.sh / harness_check.sh"

echo "Generando harness.py..."
cat <<'HARNESS_PY_EOF' > harness.py
#!/usr/bin/env python3
import argparse
import json
import os
from datetime import datetime, timezone

ROOT = os.path.dirname(os.path.abspath(__file__))
FEATURES = os.path.join(ROOT, "feature_list.json")
PROGRESS = os.path.join(ROOT, "progress")
CURRENT = os.path.join(PROGRESS, "current.md")
HISTORY = os.path.join(PROGRESS, "history.md")


def load_features():
    if not os.path.exists(FEATURES):
        return {"project": os.path.basename(ROOT), "features": []}
    with open(FEATURES, "r", encoding="utf-8") as f:
        return json.load(f)


def save_features(data):
    tmp = FEATURES + ".tmp"
    with open(tmp, "w", encoding="utf-8") as f:
        json.dump(data, f, indent=2, ensure_ascii=False)
    os.replace(tmp, FEATURES)


def log(line):
    os.makedirs(PROGRESS, exist_ok=True)
    stamp = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    with open(HISTORY, "a", encoding="utf-8") as f:
        f.write(f"- {stamp} {line}\n")


def find_feature(data, fid):
    for feature in data.get("features", []):
        if str(feature.get("id")) == str(fid):
            return feature
    raise SystemExit(f"Feature no encontrada: {fid}")


def cmd_status(_args):
    data = load_features()
    features = data.get("features", [])
    active = [f for f in features if f.get("status") == "in_progress"]
    pending = [f for f in features if f.get("status") == "pending"]
    blocked = [f for f in features if f.get("status") == "blocked"]
    done = [f for f in features if f.get("status") == "done"]
    print(f"Backlog: {len(features)} feature(s) | active={len(active)} pending={len(pending)} blocked={len(blocked)} done={len(done)}")
    for f in features:
        services = ", ".join(f.get("microservicios", [])) or "sin servicios"
        print(f"  #{f.get('id')} [{f.get('status')}] {f.get('name')} ({services})")
    if os.path.exists(CURRENT):
        with open(CURRENT, "r", encoding="utf-8") as f:
            content = f.read().strip()
        if content:
            print("\nprogress/current.md:")
            print(content)


def cmd_next(_args):
    data = load_features()
    for f in data.get("features", []):
        if f.get("status") == "pending":
            print(json.dumps(f, indent=2, ensure_ascii=False))
            return
    print("No hay features pending.")


def cmd_start(args):
    data = load_features()
    feature = find_feature(data, args.feature)
    active = [f for f in data.get("features", []) if f.get("status") == "in_progress" and str(f.get("id")) != str(args.feature)]
    if active:
        raise SystemExit(f"Ya hay feature in_progress: #{active[0].get('id')} {active[0].get('name')}")
    feature["status"] = "in_progress"
    feature["started_at"] = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    save_features(data)
    os.makedirs(PROGRESS, exist_ok=True)
    with open(CURRENT, "w", encoding="utf-8") as f:
        f.write(f"# Feature #{feature.get('id')}: {feature.get('name')}\n\n")
        f.write(f"Estado: in_progress\n\n")
        f.write("Microservicios:\n")
        for service in feature.get("microservicios", []):
            f.write(f"- {service}\n")
        f.write("\nEvidencia:\n- \n")
    log(f"start feature #{feature.get('id')} {feature.get('name')}")
    print(f"Feature #{feature.get('id')} iniciada.")


def cmd_close(args):
    data = load_features()
    feature = find_feature(data, args.feature)
    feature["status"] = args.status
    feature["closed_at"] = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    if args.note:
        feature["note"] = args.note
    save_features(data)
    os.makedirs(PROGRESS, exist_ok=True)
    with open(CURRENT, "w", encoding="utf-8") as f:
        f.write("# Estado Actual\n\nSin feature activa.\n\n## Evidencia\n\n-\n")
    log(f"close feature #{feature.get('id')} status={args.status} note={args.note or ''}")
    print(f"Feature #{feature.get('id')} cerrada como {args.status}.")


def cmd_add(args):
    data = load_features()
    ids = [int(f.get("id", 0)) for f in data.get("features", []) if str(f.get("id", "")).isdigit()]
    fid = max(ids, default=0) + 1
    feature = {
        "id": fid,
        "name": args.name,
        "microservicios": args.service or [],
        "acceptance": args.acceptance or [],
        "status": "pending",
    }
    data.setdefault("features", []).append(feature)
    save_features(data)
    log(f"add feature #{fid} {args.name}")
    print(f"Feature #{fid} agregada.")


def main():
    parser = argparse.ArgumentParser()
    sub = parser.add_subparsers(dest="command", required=True)
    sub.add_parser("status").set_defaults(func=cmd_status)
    sub.add_parser("next").set_defaults(func=cmd_next)

    start = sub.add_parser("start")
    start.add_argument("--feature", required=True)
    start.set_defaults(func=cmd_start)

    close = sub.add_parser("close")
    close.add_argument("--feature", required=True)
    close.add_argument("--status", choices=["done", "blocked", "pending"], required=True)
    close.add_argument("--note")
    close.set_defaults(func=cmd_close)

    add = sub.add_parser("add")
    add.add_argument("--name", required=True)
    add.add_argument("--service", action="append")
    add.add_argument("--acceptance", action="append")
    add.set_defaults(func=cmd_add)

    args = parser.parse_args()
    args.func(args)


if __name__ == "__main__":
    main()
HARNESS_PY_EOF
chmod +x harness.py
write_file_notice "harness.py"

if [ "$WITH_SUBAGENTS" -eq 1 ]; then
    echo "Generando capa de subagentes..."

    cat <<'AGENTS_EOF' > AGENTS.md
# Mapa de Agentes

Este arnes usa un mapa progresivo: lee solo lo necesario para la tarea actual.

## Orden de trabajo

1. Lider revisa `feature_list.json`, `progress/current.md`, hub e impacto.
2. Implementer trabaja en una unidad concreta y escribe evidencia en `progress/`.
3. Reviewer verifica impacto, tests, checkpoints y estado Git.
4. El cierre requiere `harness_check.sh` limpio o decision explicita de bloqueo.

## Archivos principales

- `CLAUDE.md`: protocolo minimo siempre activo.
- `CHECKPOINTS.md`: criterios de cierre.
- `feature_list.json`: backlog ejecutable.
- `progress/current.md`: estado vivo de la tarea.
- `progress/history.md`: bitacora append-only.
- `docs/architecture.md`: mapa de arquitectura.
- `docs/conventions.md`: convenciones del equipo.
- `docs/verification.md`: comandos de validacion.
- `.claude/agents/leader.md`: rol lider.
- `.claude/agents/implementer.md`: rol implementador.
- `.claude/agents/reviewer.md`: rol revisor.

## Regla anti perdida de contexto

Todo hallazgo relevante se escribe en `progress/`. Una respuesta corta en chat
no reemplaza evidencia persistida.
AGENTS_EOF

    cat <<'CHECKPOINTS_EOF' > CHECKPOINTS.md
# Checkpoints

Antes de cerrar una tarea:

- [ ] La feature activa en `feature_list.json` refleja el estado real.
- [ ] `progress/current.md` contiene alcance, servicios afectados y evidencia.
- [ ] Se ejecuto impacto para los microservicios modificados:
      `python3 graph_memory.py impacto --microservicio <proyecto>/<servicio>`
- [ ] Si existe `graphify-out/graph.json`, se consulto `graphify query`.
- [ ] Tests relevantes ejecutados por cada microservicio afectado.
- [ ] Frontends validados con `validate_ui.sh <url>` cuando aplique.
- [ ] `progress/review_<feature>.md` contiene veredicto del reviewer.
- [ ] Repos afectados limpios o commiteados segun politica.
- [ ] `harness_check.sh` pasa o el bloqueo queda documentado.
CHECKPOINTS_EOF

    cat <<FEATURES_EOF > feature_list.json
{
  "project": "$PROJECT_NAME",
  "rules": {
    "one_feature_at_a_time": true,
    "require_tests_to_close": true,
    "require_impact_check": true
  },
  "features": []
}
FEATURES_EOF

    cat <<'CURRENT_EOF' > progress/current.md
# Estado Actual

Sin feature activa.

## Evidencia

-
CURRENT_EOF

    cat <<'HISTORY_EOF' > progress/history.md
# Historial
HISTORY_EOF

    cat <<'ARCH_EOF' > docs/architecture.md
# Arquitectura

Completa este archivo con:

- Microservicios y responsabilidades.
- Dependencias internas y externas.
- Servicios transversales.
- Riesgos conocidos.
- Flujos criticos.
ARCH_EOF

    cat <<'CONV_EOF' > docs/conventions.md
# Convenciones

- Usa Conventional Commits.
- No agregues `Co-Authored-By` ni firmas generadas por IA.
- Trabaja dentro del microservicio afectado.
- Prefiere cambios pequenos, verificables y documentados.
- Registra decisiones relevantes en `progress/`.
CONV_EOF

    cat <<'VERIF_EOF' > docs/verification.md
# Verificacion

Registra aqui los comandos oficiales por tipo de proyecto.

Ejemplos:

```bash
go test ./...
npm test
npm run lint
bash "$CLAUDE_PROJECT_DIR/validate_ui.sh" http://localhost:5173
```
VERIF_EOF

    cat <<'LEADER_EOF' > .claude/agents/leader.md
# Lider

Responsabilidad: definir alcance, impacto y delegacion. No implementes codigo
si puedes delegarlo a implementer.

Antes de delegar:

1. Lee `AGENTS.md`, `feature_list.json` y `progress/current.md`.
2. Ejecuta `python3 graph_memory.py mapa`.
3. Para cada servicio afectado, ejecuta impacto.
4. Si existe graphify, consulta el grafo.
5. Escribe plan corto y archivos esperados en `progress/current.md`.

Salida esperada:

- Feature activa.
- Servicios afectados.
- Riesgos.
- Delegacion concreta para implementer.
LEADER_EOF

    cat <<'IMPLEMENTER_EOF' > .claude/agents/implementer.md
# Implementer

Responsabilidad: implementar una unidad concreta.

Reglas:

- Trabaja solo en los microservicios asignados.
- No cambies contratos compartidos sin registrar impacto.
- Ejecuta tests cercanos al cambio.
- Escribe reporte en `progress/impl_<feature>.md`.

Reporte minimo:

- Archivos modificados.
- Decisiones tomadas.
- Comandos ejecutados.
- Riesgos pendientes.
IMPLEMENTER_EOF

    cat <<'REVIEWER_EOF' > .claude/agents/reviewer.md
# Reviewer

Responsabilidad: revisar calidad, impacto y cierre.

Debes verificar:

- `graph_memory.py impacto` ejecutado para servicios modificados.
- Tests relevantes ejecutados.
- `validate_ui.sh` ejecutado para frontends cuando aplique.
- `graphify query` usado o justificacion si no existe grafo.
- Checkpoints completos.
- Repos limpios o commits hechos.

Escribe veredicto en `progress/review_<feature>.md`:

- `approved`
- `changes_requested`
- `blocked`
REVIEWER_EOF

    write_file_notice "AGENTS.md / CHECKPOINTS.md / docs / progress / .claude/agents"
fi

chmod +x init.sh validate_ui.sh commit_guard.sh harness_status.sh harness_check.sh harness.py

echo "Asegurando graphify..."
if command -v graphify >/dev/null 2>&1; then
    echo "   -> graphify ya esta disponible."
elif [ "$INSTALL_GRAPHIFY" -eq 1 ]; then
    set +e
    if command -v uv >/dev/null 2>&1; then
        uv tool install --upgrade graphifyy >/dev/null 2>&1 \
            && echo "   -> graphify instalado via uv." \
            || echo "   -> aviso: no se pudo instalar via uv."
    elif command -v pipx >/dev/null 2>&1; then
        pipx install graphifyy >/dev/null 2>&1 \
            && echo "   -> graphify instalado via pipx." \
            || echo "   -> aviso: no se pudo instalar via pipx."
    else
        python3 -m pip install --user graphifyy >/dev/null 2>&1 \
            && echo "   -> graphify instalado via pip --user." \
            || echo "   -> aviso: instala manualmente graphifyy."
    fi
    set -e
else
    echo "   -> graphify no instalado (--no-graphify activo). Quita ese flag para asegurarlo."
fi

echo ""
echo "========================================================"
echo "Harness Process instalado exitosamente."
echo ""
echo "Comandos utiles:"
echo "  bash init.sh"
echo "  bash harness_status.sh"
echo "  bash harness_check.sh"
echo "  python3 graph_memory.py mapa"
echo "  python3 harness.py status"
if [ "$WITH_SUBAGENTS" -eq 1 ]; then
    echo ""
    echo "Modo subagentes activo:"
    echo "  python3 harness.py add --name \"mi_feature\" --service \"$PROJECT_NAME/servicio\""
    echo "  python3 harness.py start --feature 1"
    echo "  python3 harness.py close --feature 1 --status done"
fi
echo "========================================================"
