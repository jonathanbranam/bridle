#!/usr/bin/env python3
"""Where the week's subscription usage went, by kind of agent and work.

Reads every local project daemon's database (read-only) from ~/.bridle/daemons/*.json:
hosted turns (`turns`) and interactive sessions' statusline deltas (`interactive_usage`).
Cost is Claude Code's list-price estimate, used only as a relative unit: a group's share
of the week's seven_day points is its share of the cost (the same estimate as
`bridle usage --by name`; Anthropic does not publish the limit's weights).

    python3 -I scripts/usage-report.py                # since the last weekly reset
    python3 -I scripts/usage-report.py --points 60    # scale to a known weekly figure
    python3 -I scripts/usage-report.py --since 2026-10-14T07:00Z --csv /tmp/usage.csv

Stdlib only. Other machines' daemons are not read (their usage needs a run there).
"""

import argparse
import csv
import glob
import json
import os
import sqlite3
import statistics
from collections import defaultdict
from datetime import datetime, timedelta, timezone
from zoneinfo import ZoneInfo

ET = ZoneInfo("America/New_York")
INTERACTIVE_ROLES = {"orchestrator", "advisor", "aide", "unbridled"}


def ts(s):
    return datetime.fromisoformat(s.replace("Z", "+00:00"))


def iso(d):
    return d.astimezone(timezone.utc).strftime("%Y-%m-%dT%H:%M:%S.000Z")


def daemons():
    out = []
    for f in sorted(glob.glob(os.path.expanduser("~/.bridle/daemons/*.json"))):
        d = json.load(open(f))
        db = os.path.join(d["workspace"], ".bridle", "bridle.db")
        if os.path.exists(db):
            out.append((d["project"], db))
    return out


def connect(db):
    return sqlite3.connect(f"file:{db}?mode=ro", uri=True)


def week_start(dbs):
    """The last seven_day reset: the newest reading's resets_at minus seven days."""
    best = None
    for _, db in dbs:
        row = connect(db).execute(
            "SELECT resets_at FROM rate_limit_history WHERE window='seven_day' "
            "AND resets_at IS NOT NULL ORDER BY observed_at DESC LIMIT 1").fetchone()
        if row:
            r = ts(row[0])
            best = r if best is None or r > best else best
    # resets_at jitters by a few hundred ms either side; round to the minute
    start = best - timedelta(days=7) + timedelta(seconds=30)
    return start.replace(second=0, microsecond=0)


def seven_day_series(dbs, since):
    pts = []
    for _, db in dbs:
        pts += connect(db).execute(
            "SELECT observed_at, utilization FROM rate_limit_history WHERE window='seven_day' "
            "AND observed_at >= ? AND resets_at >= ? AND utilization IS NOT NULL",
            # the old week's readings keep arriving for a while after its reset
            (iso(since), iso(since + timedelta(days=7) - timedelta(minutes=10)))).fetchall()
    return sorted((ts(o), u * 100) for o, u in pts)


def load(dbs, since):
    rows = []
    for project, db in dbs:
        c = connect(db)
        for (agent_id, name, role, model, n, started, ended, cost, session, inp, out, cr, cw,
             ctx, reason) in c.execute(
                "SELECT agent_id, agent_name, role, model, n, started_at, ended_at, cost_total, "
                "session_id, input_tokens, output_tokens, cache_read, cache_write, "
                "context_tokens, terminal_reason FROM turns WHERE started_at >= ?",
                (iso(since),)):
            rows.append(dict(
                t=ts(started), project=project, kind="hosted", role=role, name=name,
                agent=agent_id, model=model, cost=max(cost or 0, 0), session=session, n=n,
                busy=(ts(ended) - ts(started)).total_seconds() if ended else 0,
                tokens=inp + out + cr + cw, cache_read=cr, ctx=ctx, reason=reason))
        for observed, ident, model, cost, session, inp, out in c.execute(
                "SELECT observed_at, identity, model, cost_delta, session_id, input_delta, "
                "output_delta FROM interactive_usage WHERE observed_at >= ? "
                "AND cost_delta IS NOT NULL", (iso(since),)):
            ident = ident or "external:unbridled"
            role = ident.removeprefix("external:").split("/")[0]
            rows.append(dict(
                t=ts(observed), project=project, kind="interactive", role=role, name=ident,
                agent=ident, model=model or "?", cost=max(cost or 0, 0), session=session,
                n=None, busy=0, tokens=(inp or 0) + (out or 0), cache_read=0, ctx=None,
                reason=None))
    return rows


def table(title, groups, total, ppd, extra=None, limit=None):
    print(f"\n## {title}")
    print(f"{'':34} {'COST $':>9} {'SHARE':>7} {'OF WEEK':>8}" + (f"  {extra[0]}" if extra else ""))
    items = sorted(groups.items(), key=lambda kv: -kv[1])
    for k, v in items[:limit]:
        tail = f"  {extra[1](k)}" if extra else ""
        print(f"{str(k)[:34]:34} {v:9.2f} {100 * v / total:6.1f}% {v * ppd:7.1f}%" + tail)
    if limit and len(items) > limit:
        rest = sum(v for _, v in items[limit:])
        print(f"{f'({len(items) - limit} more)':34} {rest:9.2f} {100 * rest / total:6.1f}% "
              f"{rest * ppd:7.1f}%")


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("--since", help="ISO start (default: the last weekly reset)")
    ap.add_argument("--points", type=float,
                    help="the week's seven_day %% to share out (default: the latest reading)")
    ap.add_argument("--csv", help="also write every row here, for a notebook or a spreadsheet")
    ap.add_argument("--top", type=int, default=12, help="names to list per table")
    a = ap.parse_args()

    dbs = daemons()
    since = ts(a.since) if a.since else week_start(dbs)
    now = datetime.now(timezone.utc)
    series = seven_day_series(dbs, since)
    points = a.points if a.points is not None else (series[-1][1] if series else 0)
    rows = load(dbs, since)
    total = sum(r["cost"] for r in rows)
    ppd = points / total if total else 0

    print(f"# Usage since {since.astimezone(ET):%a %Y-%m-%d %H:%M} ET "
          f"({(now - since).total_seconds() / 3600:.0f} h, to {now.astimezone(ET):%a %H:%M} ET)")
    print(f"seven_day: {points:.0f}% used; measured cost ${total:.2f}; "
          f"{ppd:.3f} points per $; 'OF WEEK' = share of the {points:.0f}%")
    print("daemons read: " + ", ".join(p for p, _ in dbs))

    def by(key, pred=lambda r: True):
        g = defaultdict(float)
        for r in rows:
            if pred(r):
                g[key(r)] += r["cost"]
        return g

    # Interactive = sessions the human talks to or that steer (orchestrator, advisors, aides),
    # hosted = headless agents the daemon runs.
    table("Interactive vs hosted", by(lambda r: r["kind"]), total, ppd)
    table("By role", by(lambda r: f"{r['role']} ({r['kind'][0]})"), total, ppd)
    table("By model", by(lambda r: r["model"]), total, ppd)
    table("By project", by(lambda r: r["project"]), total, ppd)
    table("By name", by(lambda r: r["name"]), total, ppd, limit=a.top)

    # Coverage: an identity that stopped reporting while its session may still run is
    # spend the table above can't see.
    print("\n## Interactive sessions: last statusline report")
    last = {}
    for r in rows:
        if r["kind"] == "interactive":
            last[(r["project"], r["name"])] = max(last.get((r["project"], r["name"]), r["t"]),
                                                  r["t"])
    for (p, n), t in sorted(last.items(), key=lambda kv: kv[1]):
        age = (now - t).total_seconds() / 3600
        flag = "  <- silent" if age > 6 else ""
        print(f"{n:40} {p:14} {t.astimezone(ET):%a %m-%d %H:%M} ET ({age:5.1f} h ago){flag}")

    # Per day: does measured spend explain the seven_day gain? A day whose points per $
    # runs well above the week's suggests spend that isn't measured.
    print("\n## Per day (ET): measured cost vs seven_day gain")
    print(f"{'DAY':12} {'INTER $':>8} {'HOSTED $':>9} {'GAIN':>6} {'PTS/$':>6}  TOP ROLES")
    day_cost = defaultdict(lambda: defaultdict(float))
    for r in rows:
        day_cost[r["t"].astimezone(ET).date()][(r["kind"], r["role"])] += r["cost"]
    day_peak = {}
    for t, u in series:
        d = t.astimezone(ET).date()
        day_peak[d] = max(day_peak.get(d, 0), u)
    prev = 0.0
    for d in sorted(set(day_cost) | set(day_peak)):
        c = day_cost[d]
        inter = sum(v for (k, _), v in c.items() if k == "interactive")
        hosted = sum(v for (k, _), v in c.items() if k == "hosted")
        gain = day_peak.get(d, prev) - prev
        prev = day_peak.get(d, prev)
        top = ", ".join(f"{role} {v:.0f}" for (_, role), v in
                        sorted(c.items(), key=lambda kv: -kv[1])[:4])
        rate = f"{gain / (inter + hosted):6.3f}" if inter + hosted else "     -"
        print(f"{d} {inter:8.2f} {hosted:9.2f} {gain:6.1f} {rate}  {top}")

    print("\n## Hour of day (ET), cost summed over the period")
    hours = defaultdict(lambda: [0.0, 0.0])
    for r in rows:
        hours[r["t"].astimezone(ET).hour][r["kind"] == "hosted"] += r["cost"]
    peak = max((sum(v) for v in hours.values()), default=1)
    for h in range(24):
        i, ho = hours[h]
        bar = "#" * round(30 * i / peak) + "+" * round(30 * ho / peak)
        print(f"{h:02}h {i:7.2f} {ho:7.2f}  {bar}")
    print("(# interactive, + hosted)")

    print("\n## Hosted agents")
    agents = defaultdict(lambda: dict(cost=0.0, turns=0, busy=0.0, role="", sessions=set()))
    for r in rows:
        if r["kind"] == "hosted":
            g = agents[(r["project"], r["name"], r["agent"])]
            g["cost"] += r["cost"]
            g["turns"] += 1
            g["busy"] += r["busy"]
            g["role"] = r["role"]
            g["sessions"].add(r["session"])
    for role in sorted({g["role"] for g in agents.values()}):
        gs = [g for g in agents.values() if g["role"] == role]
        costs = sorted(g["cost"] for g in gs)
        turns = sum(g["turns"] for g in gs)
        print(f"{role:18} {len(gs):4} agents  {turns:5} turns  ${sum(costs):7.2f}  "
              f"per agent median ${statistics.median(costs):.2f} max ${costs[-1]:.2f}  "
              f"per turn ${sum(costs) / turns:.3f}  busy {sum(g['busy'] for g in gs) / 3600:.1f} h")
    print("\nmost expensive hosted agents:")
    for (p, n, _), g in sorted(agents.items(), key=lambda kv: -kv[1]["cost"])[:a.top]:
        print(f"  {n:26} {p:12} {g['role']:16} ${g['cost']:6.2f} {g['turns']:4} turns "
              f"{len(g['sessions'])} sessions")

    print("\n## Interactive sessions")
    sess = defaultdict(lambda: [0.0, None, None])
    for r in rows:
        if r["kind"] == "interactive":
            s = sess[(r["name"], r["session"])]
            s[0] += r["cost"]
            s[1] = r["t"] if s[1] is None else min(s[1], r["t"])
            s[2] = r["t"] if s[2] is None else max(s[2], r["t"])
    per = defaultdict(list)
    for (n, _), (c, t0, t1) in sess.items():
        per[n].append((c, (t1 - t0).total_seconds() / 3600))
    for n, ss in sorted(per.items(), key=lambda kv: -sum(c for c, _ in kv[1])):
        cost = sum(c for c, _ in ss)
        hrs = sum(h for _, h in ss)
        print(f"  {n:36} {len(ss):3} sessions ${cost:7.2f}  ${cost / len(ss):6.2f}/session  "
              f"{hrs:5.1f} h live  ${cost / hrs if hrs else 0:5.2f}/h")

    if a.csv:
        with open(a.csv, "w", newline="") as f:
            w = csv.writer(f)
            keys = ["t", "project", "kind", "role", "name", "model", "cost", "session",
                    "busy", "tokens", "cache_read", "ctx", "reason"]
            w.writerow(keys)
            for r in sorted(rows, key=lambda r: r["t"]):
                w.writerow([iso(r["t"]) if k == "t" else r[k] for k in keys])
        print(f"\nrows written to {a.csv}")


if __name__ == "__main__":
    main()
