#!/usr/bin/env python3
"""Summarise bench runs (samples.csv + manifest.json + events.jsonl) as plain text.

usage: analyse-runs.py RUN_DIR [RUN_DIR ...]   (RUN_DIR as checked out of bridle/benchmarks)
"""
import collections
import csv
import datetime as dt
import json
import statistics
import sys
from pathlib import Path


def ts(s):
    return dt.datetime.strptime(s, "%Y-%m-%dT%H:%M:%SZ").replace(tzinfo=dt.timezone.utc)


def stats(xs):
    return f"min {min(xs):.1f} mean {statistics.mean(xs):.1f} max {max(xs):.1f}"


def analyse(d):
    d = Path(d)
    m = json.loads((d / "manifest.json").read_text())
    rows = list(csv.DictReader((d / "samples.csv").open()))
    t = [ts(r["ts"]) for r in rows]
    span = (t[-1] - t[0]).total_seconds()
    gaps = [(t[i + 1] - t[i]).total_seconds() for i in range(len(t) - 1)]
    num = lambda k: [float(r[k]) for r in rows]
    print(f"== {d.name}: {m['scenario']} {m['load']['class']}, daemon {m['daemon_version']}, "
          f"{m['samples']} samples over {span:.0f}s, interval {m['interval_s']}s")
    print(f"gaps: max {max(gaps):.0f}s, over 1.5x interval: {sum(g > 1.5 * m['interval_s'] for g in gaps)}")
    print(f"load1: {stats(num('load1'))}; working agents max {m['load']['working_max']} mean {m['load']['working_mean']}")
    pids = {r["daemon_pid"] for r in rows}
    print(f"daemon pids seen: {sorted(pids)}")
    for name, col in (("daemon", "daemon_cpu_s"), ("children", "children_cpu_s"), ("claude", "claude_cpu_s")):
        v = num(col)
        # a drop means a process left the tree; count only rises so it never goes negative
        rise = sum(max(0.0, b - a) for a, b in zip(v, v[1:]))
        print(f"{name} cpu: {rise:.1f}s over run = {100 * rise / span:.2f}% of one core; "
              f"net {v[-1] - v[0]:+.1f}s; drops {sum(b < a for a, b in zip(v, v[1:]))}")
    for name, col in (("daemon rss MB", "daemon_rss_kb"), ("children rss MB", "children_rss_kb"),
                      ("claude rss MB", "claude_rss_kb")):
        v = [x / 1024 for x in num(col)]
        print(f"{name}: {stats(v)}; first {v[0]:.0f} last {v[-1]:.0f}")
    print(f"children_n: {stats(num('children_n'))}; claude_n: {stats(num('claude_n'))}")
    for name, col, k in (("compressed MB", "compressed_kb", 1024), ("swap MB", "swap_used_mb", 1)):
        v = [x / k for x in num(col)]
        print(f"{name}: {stats(v)}; first {v[0]:.0f} last {v[-1]:.0f} ({v[-1] - v[0]:+.0f})")
    print("pressure level:", dict(collections.Counter(r["pressure_level"] for r in rows)))
    print("agents (first row): working {agents_working} idle {agents_idle} stopped {agents_stopped} "
          "lost {agents_lost}".format(**rows[0]))
    sc = m["sampler_cpu_s"]
    print(f"sampler own cpu: {sc:.1f}s = {100 * sc / m['duration_s']:.2f}% of one core; "
          f"vs daemon rise above")
    ev = collections.Counter()
    for line in (d / "events.jsonl").read_text().splitlines():
        ev[json.loads(line)["kind"]] += 1
    print(f"events {sum(ev.values())} ({sum(ev.values()) / m['duration_s'] * 60:.1f}/min), spawns {m['spawns']}:",
          ", ".join(f"{k} {n}" for k, n in ev.most_common(8)))
    print()


for p in sys.argv[1:]:
    analyse(p)
