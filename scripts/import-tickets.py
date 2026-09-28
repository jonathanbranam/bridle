#!/usr/bin/env python3
"""import-tickets.py — throwaway importer for P0-6a (ticket tskm), the
verification gate for "migrate bridle's own work into bridle task
management". Reads docs/questions/open/*.md and docs/spikes/open/*.md and
creates one bridle task per ticket via the `bridle` CLI, plus a `blocks`
edge for each `needs:` reference that resolves to another ticket in the
same import batch.

Not a permanent CLI subcommand (YAGNI) — point it at a throwaway daemon:

    BRIDLE_URL=http://127.0.0.1:PORT BRIDLE_TOKEN=... \\
        scripts/import-tickets.py --bridle-bin target/debug/bridle

Ticket files are only read, never moved or edited: the folder stays the
state for tickets (docs/README.md); this is additive.
"""

import argparse
import json
import re
import subprocess
import sys
from pathlib import Path

# (glob under docs/, bridle TaskKind)
TREES = [
    ("docs/questions/open/*.md", "question"),
    ("docs/spikes/open/*.md", "research"),
]

FRONTMATTER_RE = re.compile(r"^---\n(.*?\n)---\n", re.DOTALL)


def parse_scalar(value: str) -> str:
    value = value.strip()
    if len(value) >= 2 and value[0] == value[-1] == '"':
        value = value[1:-1]
    return value


def parse_list(value: str) -> list[str]:
    value = value.strip()
    if not (value.startswith("[") and value.endswith("]")):
        return []
    inner = value[1:-1].strip()
    if not inner:
        return []
    return [item.strip() for item in inner.split(",") if item.strip()]


def parse_frontmatter(text: str, path: Path) -> dict | None:
    m = FRONTMATTER_RE.match(text)
    if not m:
        print(f"warning: {path}: no frontmatter block found, skipping", file=sys.stderr)
        return None
    fields: dict[str, str] = {}
    for line in m.group(1).splitlines():
        if not line.strip() or ":" not in line:
            continue
        key, _, value = line.partition(":")
        fields[key.strip()] = value
    if "id" not in fields or "title" not in fields:
        print(f"warning: {path}: missing id/title in frontmatter, skipping", file=sys.stderr)
        return None
    return {
        "id": parse_scalar(fields["id"]),
        "title": parse_scalar(fields["title"]),
        "needs": parse_list(fields.get("needs", "[]")),
    }


def bridle(bin_path: str, *args: str) -> dict:
    result = subprocess.run(
        [bin_path, *args, "--json"],
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        raise RuntimeError(
            f"`bridle {' '.join(args)}` failed: {result.stderr.strip()}"
        )
    return json.loads(result.stdout)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo-root", default=".", help="bridle repo root (default: cwd)")
    parser.add_argument("--bridle-bin", required=True, help="path to the bridle binary")
    args = parser.parse_args()

    repo_root = Path(args.repo_root).resolve()

    tickets = []
    malformed = 0
    for glob, kind in TREES:
        for path in sorted(repo_root.glob(glob)):
            text = path.read_text()
            fm = parse_frontmatter(text, path)
            if fm is None:
                malformed += 1
                continue
            tickets.append(
                {
                    "path": path.relative_to(repo_root),
                    "kind": kind,
                    "id": fm["id"],
                    "title": fm["title"],
                    "needs": fm["needs"],
                }
            )

    # id -> bridle task id, only for tickets in this batch (needs same-tree resolution).
    task_ids: dict[str, str] = {}
    for t in tickets:
        body = f"ticket: {t['path']}\noriginal id: {t['id']}\n"
        task = bridle(
            args.bridle_bin,
            "task", "new", t["title"],
            "-k", t["kind"],
            "--body", body,
        )
        task_ids[t["id"]] = task["id"]
        print(f"{t['id']} -> {task['id']}  ({t['kind']}) {t['title']}")

    edges = 0
    skipped_needs = []
    for t in tickets:
        for needed in t["needs"]:
            if needed not in task_ids:
                skipped_needs.append((t["id"], needed))
                continue
            bridle(
                args.bridle_bin,
                "dep", "add", task_ids[t["id"]],
                "--blocked-by", task_ids[needed],
            )
            edges += 1

    print(f"\nimported {len(tickets)} tickets, {edges} dep edges, {malformed} malformed skipped")
    if skipped_needs:
        print("skipped needs: (not resolvable within this batch — cross-tree or unknown)")
        for from_id, needed in skipped_needs:
            print(f"  {from_id} needs {needed}")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
