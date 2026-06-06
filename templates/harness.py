#!/usr/bin/env python3
import argparse
import json
import os
import re
import shutil
import subprocess
import sys
from datetime import datetime, timezone

ROOT = os.path.dirname(os.path.abspath(__file__))
FEATURES = os.path.join(ROOT, "feature_list.json")
PROGRESS = os.path.join(ROOT, "progress")
CURRENT = os.path.join(PROGRESS, "current.md")
HISTORY = os.path.join(PROGRESS, "history.md")
GRAPH_MEM = os.path.join(ROOT, "graph_memory.py")


def _repo_root():
    """Raiz multi-repo: en layout 'subdir' es el padre del arnes (igual criterio
    que graph_memory.py). docs/ y graphify-out viven en esa raiz."""
    layout = os.path.join(ROOT, ".harness_layout")
    try:
        with open(layout, "r", encoding="utf-8") as fh:
            if fh.read().strip() == "subdir":
                return os.path.dirname(ROOT)
    except OSError:
        pass
    return ROOT


REPO_ROOT = os.environ.get("HARNESS_REPO_ROOT") or _repo_root()
# Los planes (y demas docs durables del proyecto) viven en el docs/ de la RAIZ
# multi-repo, junto a los PLAN-*.md / RUNBOOK del equipo, NO en la subcarpeta del
# arnes. progress/ queda solo para el estado vivo (current.md + history.md).
PLANS = os.path.join(REPO_ROOT, "docs")
# Linea base del checkpoint automatico (mtime = ultimo autocheck del hook).
AUTOCHECK_STAMP = os.path.join(PROGRESS, ".last_autocheck")
# Debounce del aviso "sin feature activa" (mtime = ultimo nudge emitido).
NUDGE_STAMP = os.path.join(PROGRESS, ".last_nudge")


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


def slugify(text):
    s = re.sub(r"[^a-z0-9]+", "-", (text or "").lower()).strip("-")
    return (s or "feature")[:48]


def plan_path(feature):
    return os.path.join(PLANS, f"plan-feature-{feature.get('id')}-{slugify(feature.get('name', ''))}.md")


def plan_template(feature):
    services = feature.get("microservicios", []) or ["(sin servicios)"]
    lines = [
        f"# Plan - Feature #{feature.get('id')}: {feature.get('name')}",
        "",
        "Estado: in_progress",
        "Microservicios:",
    ]
    lines += [f"- {s}" for s in services]
    lines += [
        "",
        "## Alcance",
        "",
        "## Impacto entre microservicios",
        "<!-- python3 graph_memory.py impacto --microservicio <proyecto>/<servicio> -->",
        "",
        "## Consulta al grafo (graphify)",
        '<!-- graphify query "<pregunta de la task>" -->',
        "",
        "## Delegacion (implementer)",
        "- ",
        "",
        "## Criterios de cierre (reviewer)",
        "- ",
        "",
        "## Riesgos",
        "- ",
        "",
    ]
    return "\n".join(lines)


def write_plan(feature):
    """Persiste el plan en el docs/ de la RAIZ multi-repo (archivo permanente por
    feature). No pisa un plan ya escrito por el lider."""
    os.makedirs(PLANS, exist_ok=True)
    path = plan_path(feature)
    if not os.path.exists(path):
        with open(path, "w", encoding="utf-8") as f:
            f.write(plan_template(feature))
    return path


def _graphify_refresh():
    """Refresca graphify si esta instalado y hay grafo. Best-effort, bajo el
    mismo lock que el hook post-commit para no duplicar ni corromper la salida."""
    graph_json = os.path.join(REPO_ROOT, "graphify-out", "graph.json")
    if not (shutil.which("graphify") and os.path.exists(graph_json)):
        return
    lock = os.path.join(REPO_ROOT, "graphify-out", ".update.lock")
    try:
        os.mkdir(lock)
    except OSError:
        return  # ya hay un update en curso (p.ej. el hook); no dupliques
    stale = os.path.join(REPO_ROOT, "graphify-out", ".graphify_stale")
    try:
        rc = subprocess.run(
            ["graphify", "update", REPO_ROOT],
            check=False, timeout=300,
            stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
        ).returncode
        if rc == 0:
            try:
                os.remove(stale)  # grafo fresco: limpia el marcador
            except OSError:
                pass
        else:
            open(stale, "a").close()  # update fallo: marca stale
    except Exception:
        try:
            open(stale, "a").close()  # timeout u otro error: marca stale
        except OSError:
            pass
    finally:
        try:
            os.rmdir(lock)
        except OSError:
            pass


def _hub_register(accion, estado, artefacto, meta=""):
    try:
        subprocess.run(
            [sys.executable, GRAPH_MEM, "registrar",
             "--accion", accion, "--estado", estado,
             "--artefacto", artefacto, "--meta", meta, "--agente", "harness.py"],
            check=False,
        )
    except Exception as exc:
        print(f"[memoria] hub no actualizado: {exc}")


def _graphify_refresh_bg():
    """Como _graphify_refresh pero detached: lanza el rebuild en segundo plano y
    retorna de inmediato, para no colgar el turno cuando lo dispara un hook. Usa
    el mismo lock que el hook post-commit; el proceso hijo lo libera al terminar."""
    graph_json = os.path.join(REPO_ROOT, "graphify-out", "graph.json")
    if not (shutil.which("graphify") and os.path.exists(graph_json)):
        return
    lock = os.path.join(REPO_ROOT, "graphify-out", ".update.lock")
    try:
        os.mkdir(lock)
    except OSError:
        return  # ya hay un refresh en curso
    stale = os.path.join(REPO_ROOT, "graphify-out", ".graphify_stale")
    script = (
        "trap 'rmdir \"$LOCK\" 2>/dev/null || true' EXIT; "
        "if graphify update \"$ROOT\" >/dev/null 2>&1; then rm -f \"$STALE\"; "
        "else : > \"$STALE\"; fi"
    )
    env = dict(os.environ, ROOT=REPO_ROOT, STALE=stale, LOCK=lock)
    try:
        subprocess.Popen(
            ["bash", "-c", script], env=env,
            stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
            stdin=subprocess.DEVNULL, start_new_session=True,
        )
    except Exception:
        try:
            os.rmdir(lock)
        except OSError:
            pass


def _touch_stamp():
    """Fija la linea base del checkpoint automatico (mtime = ahora)."""
    try:
        os.makedirs(PROGRESS, exist_ok=True)
        open(AUTOCHECK_STAMP, "w").close()
    except OSError:
        pass


def update_memories(accion, estado, artefacto, meta="", refresh_graphify=False):
    """Mueve las memorias junto con la task: registra el evento en el hub y, en
    cierres/avances manuales, refresca graphify (sincrono). Best-effort."""
    _hub_register(accion, estado, artefacto, meta)
    if refresh_graphify:
        _graphify_refresh()


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
    plan = write_plan(feature)
    rel_plan = os.path.relpath(plan, REPO_ROOT)
    os.makedirs(PROGRESS, exist_ok=True)
    with open(CURRENT, "w", encoding="utf-8") as f:
        f.write(f"# Feature #{feature.get('id')}: {feature.get('name')}\n\n")
        f.write("Estado: in_progress\n")
        f.write(f"Plan: {rel_plan}\n\n")
        f.write("Microservicios:\n")
        for service in feature.get("microservicios", []):
            f.write(f"- {service}\n")
        f.write("\nEvidencia:\n- \n")
    log(f"start feature #{feature.get('id')} {feature.get('name')}")
    update_memories("start", "in_progress", f"feature-{feature.get('id')}", feature.get("name", ""))
    _touch_stamp()  # linea base: el plan recien creado no dispara autocheck
    print(f"Feature #{feature.get('id')} iniciada. Plan: {rel_plan}")


def cmd_close(args):
    data = load_features()
    feature = find_feature(data, args.feature)
    stamp = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    feature["status"] = args.status
    feature["closed_at"] = stamp
    if args.note:
        feature["note"] = args.note
    save_features(data)
    plan = plan_path(feature)
    if os.path.exists(plan):
        with open(plan, "a", encoding="utf-8") as f:
            f.write(f"\n---\nCerrado: {stamp} - status={args.status} - {args.note or ''}\n")
    os.makedirs(PROGRESS, exist_ok=True)
    # No-destructivo: si current.md tiene estado real escrito a mano, archivalo en
    # docs/ ANTES de resetear -- antes este paso lo borraba y se perdia.
    archived_rel = None
    if os.path.exists(CURRENT):
        with open(CURRENT, "r", encoding="utf-8") as fh:
            content = fh.read()
        if content.strip() and "Sin feature activa" not in content:
            os.makedirs(PLANS, exist_ok=True)
            archived = os.path.join(PLANS, f"estado-feature-{feature.get('id')}-{slugify(feature.get('name', ''))}.md")
            with open(archived, "w", encoding="utf-8") as fh:
                fh.write(f"# Estado archivado - Feature #{feature.get('id')}: {feature.get('name')}\n")
                fh.write(f"Cerrada: {stamp} - status={args.status} - {args.note or ''}\n\n---\n\n")
                fh.write(content)
            archived_rel = os.path.relpath(archived, REPO_ROOT)
    with open(CURRENT, "w", encoding="utf-8") as fh:
        fh.write("# Estado Actual\n\nSin feature activa.\n\n## Evidencia\n\n-\n")
        if archived_rel:
            fh.write(f"\n_Estado de la feature #{feature.get('id')} archivado en `{archived_rel}`._\n")
    log(f"close feature #{feature.get('id')} status={args.status} note={args.note or ''}")
    update_memories("close", args.status, f"feature-{feature.get('id')}", args.note or "", refresh_graphify=True)
    try:
        os.remove(AUTOCHECK_STAMP)  # cierra el ciclo de checkpoints automaticos
    except OSError:
        pass
    msg = f"Feature #{feature.get('id')} cerrada como {args.status}."
    if archived_rel:
        msg += f" Estado archivado en {archived_rel}."
    print(msg)


def active_feature(data, fid=None):
    """La feature objetivo de un avance: la indicada por --feature, o la unica
    in_progress si se omite."""
    if fid is not None:
        return find_feature(data, fid)
    active = [f for f in data.get("features", []) if f.get("status") == "in_progress"]
    if not active:
        raise SystemExit("No hay feature in_progress. Inicia una: harness.py start --feature <id>")
    if len(active) > 1:
        ids = ", ".join(f"#{f.get('id')}" for f in active)
        raise SystemExit(f"Varias features in_progress ({ids}); especifica --feature <id>.")
    return active[0]


def cmd_advance(args):
    """Registra un hito intermedio de la feature activa SIN cerrarla: mueve plan,
    current.md, history.md y las memorias (hub + graphify) de una sola vez."""
    data = load_features()
    feature = active_feature(data, args.feature)
    if feature.get("status") != "in_progress":
        raise SystemExit(f"Feature #{feature.get('id')} no esta in_progress (status={feature.get('status')}); usa start.")
    fid = feature.get("id")
    stamp = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    # 1) Plan: deja rastro del hito en el cuerpo del plan (append, no pisa).
    plan = plan_path(feature)
    if os.path.exists(plan):
        with open(plan, "a", encoding="utf-8") as f:
            f.write(f"\n### Avance {stamp}\n{args.nota}\n")
    # 2) current.md: suma el avance a la evidencia (append, no reescribe).
    if os.path.exists(CURRENT):
        with open(CURRENT, "a", encoding="utf-8") as f:
            f.write(f"- {stamp} {args.nota}\n")
    # 3) history.md: una linea append-only.
    log(f"advance feature #{fid} {args.nota}")
    # 4) Memorias: hub (in_progress, con la nota) + graphify (best-effort).
    update_memories("advance", "in_progress", f"feature-{fid}", args.nota,
                    refresh_graphify=not args.no_graphify)
    _touch_stamp()  # un advance manual tambien resetea la linea base del auto
    extra = "" if args.no_graphify else " (hub + graphify)"
    print(f"Avance registrado en feature #{fid}{extra}.")


def cmd_autocheck(args):
    """Checkpoint automatico para los hooks (fin de turno, multi-LLM): si hay UNA
    feature in_progress y cambio current.md o algun doc del proyecto (docs/*.md)
    desde el ultimo checkpoint, registra un avance auto (hub + graphify en segundo
    plano + history.md). Silencioso, idempotente y best-effort."""
    try:
        data = load_features()
        active = [f for f in data.get("features", []) if f.get("status") == "in_progress"]
        if len(active) != 1:
            return
        feature = active[0]
        fid = feature.get("id")
        last = os.path.getmtime(AUTOCHECK_STAMP) if os.path.exists(AUTOCHECK_STAMP) else 0.0
        # Vigila lo que el agente REALMENTE mantiene: el estado vivo (current.md)
        # y CUALQUIER doc del proyecto (docs/*.md) -- plan auto, PLAN-*.md a mano,
        # impl_*/impl-*/review*, runbooks. Asi captura el flujo sin imponer nombres.
        watched = []
        if os.path.exists(CURRENT):
            watched.append(CURRENT)
        if os.path.isdir(PLANS):
            for name in os.listdir(PLANS):
                if name.endswith(".md"):
                    watched.append(os.path.join(PLANS, name))
        changed = sorted({os.path.basename(p) for p in watched if os.path.getmtime(p) > last})
        if not changed:
            return
        nota = "auto: " + ", ".join(changed)
        log(f"autocheck feature #{fid} {nota}")
        _hub_register("advance", "in_progress", f"feature-{fid}", nota)
        if not args.no_graphify:
            _graphify_refresh_bg()
        _touch_stamp()
        print(f"[autocheck] avance auto en feature #{fid}: {nota}")
    except Exception as exc:
        # Best-effort absoluto: corre al cierre de cada turno; nunca abortes.
        print(f"[autocheck] omitido: {exc}")


def cmd_nudge(_args):
    """Aviso (no bloqueante) para los hooks post-tool: si NO hay feature
    in_progress, recuerda registrar el trabajo antes de seguir editando, para que
    se active el ciclo (plan en docs/ + autocheck, que duerme sin feature activa).
    Debounced (~10 min) y best-effort: escribe a stderr y nunca falla."""
    try:
        data = load_features()
        if any(f.get("status") == "in_progress" for f in data.get("features", [])):
            return  # hay feature activa: nada que recordar
        last = os.path.getmtime(NUDGE_STAMP) if os.path.exists(NUDGE_STAMP) else 0.0
        if datetime.now(timezone.utc).timestamp() - last < 600:
            return  # ya avisamos hace poco
        os.makedirs(PROGRESS, exist_ok=True)
        open(NUDGE_STAMP, "w").close()
        sys.stderr.write(
            "[harness] Sin feature activa: el avance NO se esta capturando "
            "(autocheck duerme sin una feature in_progress). Antes de seguir, "
            "consulta graphify, corre impacto y registra el trabajo con "
            "'harness.py add' + 'harness.py start'.\n"
        )
    except Exception:
        pass


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

    adv = sub.add_parser("advance")
    adv.add_argument("--feature")
    adv.add_argument("--nota", required=True)
    adv.add_argument("--no-graphify", action="store_true")
    adv.set_defaults(func=cmd_advance)

    autochk = sub.add_parser("autocheck")
    autochk.add_argument("--no-graphify", action="store_true")
    autochk.set_defaults(func=cmd_autocheck)

    sub.add_parser("nudge").set_defaults(func=cmd_nudge)

    add = sub.add_parser("add")
    add.add_argument("--name", required=True)
    add.add_argument("--service", action="append")
    add.add_argument("--acceptance", action="append")
    add.set_defaults(func=cmd_add)

    args = parser.parse_args()
    args.func(args)


if __name__ == "__main__":
    main()
