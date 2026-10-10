#!/usr/bin/env python3
"""Passive resource sampler for bridle: the benchmark in
docs/design/benchmarks.md (design: ticket v6kr, "Design options").

  passive-sample.py [--out BASE]       run the 30-minute sample
  passive-sample.py publish RUN_DIR    copy a finished run to the bridle/benchmarks branch

It only reads (ps, sysctl, vm_stat, `bridle status`, `bridle events`) and drives nothing. Constants
are fixed here on purpose: the same script, every time. Run it from the main clone, outside any
agent or worktree, in its own tmux window.
"""
import csv
import json
import logging
import os
import platform
import re
import resource
import shutil
import signal
import subprocess
import sys
import tempfile
import time
from datetime import datetime, timezone
from pathlib import Path

DURATION_S = 30 * 60
INTERVAL_S = 15
SCENARIO = "passive"
BRANCH = "bridle/benchmarks"
EVENTS_PAGE = 500  # the events API's page size
STATES = ("working", "idle", "stopped", "lost")

COLUMNS = [
    "ts", "load1", "daemon_pid", "daemon_cpu_s", "daemon_rss_kb",
    "children_n", "children_cpu_s", "children_rss_kb",
    "claude_n", "claude_cpu_s", "claude_rss_kb",
    *(f"agents_{s}" for s in STATES), "agents_other",
    "pressure_level", "compressed_kb", "swap_used_mb",
]

log = logging.getLogger("sampler")


def now_iso():
    return datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def run(cmd, cwd=None, timeout=30):
    """stdout of a command, or None on any failure: a missing reading is a blank, not a crash."""
    try:
        r = subprocess.run(cmd, cwd=cwd, capture_output=True, text=True, timeout=timeout)
    except (OSError, subprocess.SubprocessError):
        return None
    return r.stdout if r.returncode == 0 else None


def parse_cpu(s):
    """ps `time` field: [D-][H:]M:SS[.ss] -> seconds."""
    days = 0
    if "-" in s:
        d, s = s.split("-", 1)
        days = int(d)
    secs = 0.0
    for part in s.split(":"):
        secs = secs * 60 + float(part)
    return days * 86400 + secs


def parse_ps(text):
    """`ps -axo pid=,ppid=,time=,rss=,command=` -> list of dicts."""
    rows = []
    for line in text.splitlines():
        f = line.split(None, 4)
        if len(f) < 5:
            continue
        try:
            rows.append({"pid": int(f[0]), "ppid": int(f[1]), "cpu": parse_cpu(f[2]),
                         "rss": int(f[3]), "cmd": f[4]})
        except ValueError:
            continue
    return rows


def is_claude(row):
    return os.path.basename(row["cmd"].split(None, 1)[0]) == "claude"


def sum_procs(rows):
    return len(rows), round(sum(r["cpu"] for r in rows), 2), sum(r["rss"] for r in rows)


def process_columns(rows, daemon_pid):
    """Daemon self, its direct non-claude children, and every claude process, as separate columns
    so a `ps`-style child shows up as the daemon's cost (n4w4). Blank while the daemon is down."""
    claude = sum_procs([r for r in rows if is_claude(r)])
    out = {"claude_n": claude[0], "claude_cpu_s": claude[1], "claude_rss_kb": claude[2]}
    me = next((r for r in rows if r["pid"] == daemon_pid), None) if daemon_pid else None
    if me is None:
        return out
    kids = sum_procs([r for r in rows if r["ppid"] == daemon_pid and not is_claude(r)])
    out.update(daemon_pid=daemon_pid, daemon_cpu_s=me["cpu"], daemon_rss_kb=me["rss"],
               children_n=kids[0], children_cpu_s=kids[1], children_rss_kb=kids[2])
    return out


def parse_swap_used_mb(text):
    m = re.search(r"used = ([\d.]+)M", text or "")
    return float(m.group(1)) if m else None


def parse_compressed_kb(text):
    if not text:
        return None
    page = re.search(r"page size of (\d+) bytes", text)
    comp = re.search(r"Pages occupied by compressor:\s+(\d+)", text)
    if not (page and comp):
        return None
    return int(comp.group(1)) * int(page.group(1)) // 1024


def memory_columns():
    # macOS measures per the v6kr "Memory: which measure" section; never top's used/free.
    level = (run(["sysctl", "-n", "kern.memorystatus_vm_pressure_level"]) or "").strip()
    return {"pressure_level": level or None,
            "compressed_kb": parse_compressed_kb(run(["vm_stat"])),
            "swap_used_mb": parse_swap_used_mb(run(["sysctl", "-n", "vm.swapusage"]))}


def agent_columns(status):
    by_state = (status or {}).get("agents_by_state")
    if by_state is None:
        return {}
    cols = {f"agents_{s}": by_state.get(s, 0) for s in STATES}
    cols["agents_other"] = sum(n for s, n in by_state.items() if s not in STATES)
    return cols


def bridle_json(args):
    out = run(["bridle", *args, "--json"])
    try:
        return json.loads(out) if out else None
    except ValueError:
        return None


def daemon_pid_from_file(workspace):
    try:
        return int(json.loads((workspace / ".bridle" / "daemon.json").read_text())["pid"])
    except (OSError, ValueError, KeyError):
        return None


def classify(rows):
    """'idle' when no sample saw a working agent; 'busy' otherwise. Only like is compared with like."""
    seen = [r["agents_working"] for r in rows if r.get("agents_working") not in (None, "")]
    if not seen:
        return "unknown"
    return "idle" if max(seen) == 0 else "busy"


def load_context(rows):
    w = [r["agents_working"] for r in rows if r.get("agents_working") not in (None, "")]
    return {"working_mean": round(sum(w) / len(w), 2) if w else None,
            "working_max": max(w) if w else None, "class": classify(rows)}


def ps_columns(daemon_pid):
    return process_columns(parse_ps(run(["ps", "-axo", "pid=,ppid=,time=,rss=,command="]) or ""),
                           daemon_pid)


def sample(workspace):
    status = bridle_json(["status"])
    pid = ((status or {}).get("daemon") or {}).get("pid") or daemon_pid_from_file(workspace)
    row = {"ts": now_iso(), "load1": round(os.getloadavg()[0], 2)}
    row.update(ps_columns(pid))
    row.update(agent_columns(status))
    row.update(memory_columns())
    return row


# --- events -------------------------------------------------------------------------------

def events_after(seq):
    page = bridle_json(["events", "--since", str(seq)])
    return page if isinstance(page, list) else None


def latest_seq():
    """Highest event seq now, found by probing (the API pages 500 at a time from a seq)."""
    hi = 1
    while events_after(hi):
        hi *= 2
    lo = hi // 2
    while hi - lo > 1:
        mid = (lo + hi) // 2
        if events_after(mid):
            lo = mid
        else:
            hi = mid
    page = events_after(lo - 1) if lo > 0 else None
    return page[-1]["seq"] if page else 0


def export_events(start_seq, path, tries=3):
    for attempt in range(tries):
        events, seq, ok = [], start_seq, True
        while True:
            page = events_after(seq)
            if page is None:
                ok = False
                break
            events.extend(page)
            if len(page) < EVENTS_PAGE:
                break
            seq = page[-1]["seq"]
        if ok:
            path.write_text("".join(json.dumps(e, separators=(",", ":")) + "\n" for e in events))
            return len(events)
        time.sleep(5)
    return None


# --- git and manifest ------------------------------------------------------------------------

def git(repo, *args):
    return run(["git", "-C", str(repo), *args])


def preflight(repo):
    """Refuse unless this checkout is main, clean and not behind origin/main (design point 5:
    the script is merged first). Returns an error string, or None."""
    branch = (git(repo, "rev-parse", "--abbrev-ref", "HEAD") or "").strip()
    if branch != "main":
        return f"checkout is on {branch or '?'}, not main"
    dirty = git(repo, "status", "--porcelain")
    if dirty is None or dirty.strip():
        return "checkout has uncommitted changes"
    behind = (git(repo, "rev-list", "--count", "HEAD..origin/main") or "").strip()
    if behind != "0":
        return f"checkout is behind origin/main ({behind or 'unknown'})"
    return None


def write_json(path, obj):
    tmp = path.with_suffix(".tmp")
    tmp.write_text(json.dumps(obj, indent=2, sort_keys=True) + "\n")
    tmp.replace(path)


def cpu_self_s():
    a, b = resource.getrusage(resource.RUSAGE_SELF), resource.getrusage(resource.RUSAGE_CHILDREN)
    return round(a.ru_utime + a.ru_stime + b.ru_utime + b.ru_stime, 2)


def base_manifest(repo, script):
    status = bridle_json(["status"]) or {}
    daemon = status.get("daemon") or {}
    return {
        "scenario": SCENARIO, "duration_s": DURATION_S, "interval_s": INTERVAL_S,
        "started": now_iso(), "status": "running",
        "hostname": platform.node(), "uname": " ".join(platform.uname()),
        "daemon_version": daemon.get("version"),
        "script_sha": (git(repo, "rev-parse", "HEAD") or "").strip(),
        "script_blob": (run(["git", "hash-object", str(script)]) or "").strip(),
    }


def sample_run(repo, workspace, run_dir, duration=DURATION_S, interval=INTERVAL_S):
    script = Path(__file__).resolve()
    manifest = base_manifest(repo, script)
    manifest["daemon_pid_start"] = daemon_pid_from_file(workspace)
    write_json(run_dir / "manifest.json", manifest)
    stop = []
    for sig in (signal.SIGINT, signal.SIGTERM):
        signal.signal(sig, lambda *_: stop.append(sig))
    start_seq = latest_seq()
    rows = []
    t0 = time.monotonic()
    with open(run_dir / "samples.csv", "w", newline="") as fh:
        w = csv.DictWriter(fh, COLUMNS, extrasaction="ignore")
        w.writeheader()
        n = 0
        while not stop and time.monotonic() - t0 < duration:
            row = sample(workspace)
            rows.append(row)
            w.writerow(row)
            fh.flush()  # a crash or a landing keeps what is written
            n += 1
            # sleep to the next slot, in short steps so a signal ends the run promptly
            until = t0 + n * interval
            while not stop and time.monotonic() < until:
                time.sleep(min(1, max(0, until - time.monotonic())))
    count = export_events(start_seq, run_dir / "events.jsonl")
    manifest.update(
        status="interrupted" if stop else "complete", ended=now_iso(), samples=len(rows),
        start_seq=start_seq, events_exported=count,
        events_export="failed" if count is None else "ok",
        spawns=None if count is None else sum(
            1 for line in (run_dir / "events.jsonl").read_text().splitlines()
            if json.loads(line)["kind"].startswith("agent.")),
        daemon_pids=sorted({r["daemon_pid"] for r in rows if r.get("daemon_pid")}),
        load=load_context(rows), sampler_cpu_s=cpu_self_s())
    write_json(run_dir / "manifest.json", manifest)
    return manifest


# --- publish ----------------------------------------------------------------------------------

def publish(repo, run_dir):
    """Copy a finished run onto the orphan branch bridle/benchmarks and push it. Redoable."""
    run_dir = Path(run_dir).resolve()
    if not (run_dir / "manifest.json").is_file():
        sys.exit(f"not a run directory: {run_dir}")
    tmp = Path(tempfile.mkdtemp(prefix="bench-publish-"))
    wt = tmp / "wt"
    try:
        exists = git(repo, "rev-parse", "--verify", "--quiet", f"refs/heads/{BRANCH}") is not None
        if exists:
            ok = git(repo, "worktree", "add", str(wt), BRANCH) is not None
        else:
            ok = (git(repo, "worktree", "add", "--detach", str(wt)) is not None
                  and git(wt, "checkout", "--orphan", BRANCH) is not None)
            if ok:
                git(wt, "rm", "-rf", "--quiet", ".")
        if not ok:
            sys.exit("could not create the publish worktree")
        dest = wt / run_dir.name
        shutil.rmtree(dest, ignore_errors=True)
        shutil.copytree(run_dir, dest)
        git(wt, "add", run_dir.name)
        if git(wt, "commit", "-q", "-m", f"benchmark run {run_dir.name}") is None:
            log.info("nothing new to commit")
        if git(wt, "push", "origin", BRANCH) is None:
            sys.exit(f"push of {BRANCH} failed; the run is committed locally, rerun publish to retry")
        print(f"published {run_dir.name} to {BRANCH}")
    finally:
        git(repo, "worktree", "remove", "--force", str(wt))
        shutil.rmtree(tmp, ignore_errors=True)


def main(argv):
    repo = Path(__file__).resolve().parents[2]
    if argv[:1] == ["publish"] and len(argv) == 2:
        return publish(repo, argv[1])
    base = None
    if argv[:1] == ["--out"] and len(argv) == 2:
        base = Path(argv[1])
    elif argv:
        sys.exit(__doc__)
    err = preflight(repo)
    if err:
        sys.exit(f"refusing to start: {err}. Run from a clean, current main clone.")
    workspace = repo.parent
    base = base or workspace.parent / "benchmarks"
    run_dir = base / datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    run_dir.mkdir(parents=True)
    logging.basicConfig(level=logging.INFO, format="%(asctime)s %(message)s", handlers=[
        logging.FileHandler(run_dir / "sampler.log"), logging.StreamHandler()])
    try:
        os.nice(10)
    except OSError:
        pass
    log.info("sampling %ss every %ss into %s", DURATION_S, INTERVAL_S, run_dir)
    m = sample_run(repo, workspace, run_dir)
    log.info("done: %s, %s samples, load %s", m["status"], m["samples"], m["load"]["class"])
    print(f"run directory: {run_dir}\nthen: {sys.argv[0]} publish {run_dir}")


if __name__ == "__main__":
    main(sys.argv[1:])
