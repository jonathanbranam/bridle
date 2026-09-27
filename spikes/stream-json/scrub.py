#!/usr/bin/env python3
"""Scrub account/machine-identifying strings from fixtures/ in place.
Run after any scenario re-run, before committing."""
import glob, json, os, re

home = os.path.expanduser("~")
user = os.path.basename(home)
tmp = os.path.realpath(os.environ.get("TMPDIR", "/tmp")).rstrip("/")
REPL = [
    (tmp, "$TMPDIR"),
    (tmp.replace("/private", "", 1), "$TMPDIR"),
    (re.sub(r"[^A-Za-z0-9]", "-", tmp), "-TMPDIR"),
    (re.sub(r"[^A-Za-z0-9]", "-", tmp.replace("/private", "", 1)), "-TMPDIR"),
    (home, "$HOME"),
    (home.replace("/", "-"), "-HOME"),
]
EMAIL = re.compile(r"[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}")
TOKEN = re.compile(r"(MESSAGING_TOKEN=)\S+")

def scrub_text(s: str) -> str:
    for a, b in REPL:
        s = s.replace(a, b)
    s = EMAIL.sub("<email>", s)
    s = TOKEN.sub(r"\1<redacted>", s)
    return s.replace(user, "<user>")

def scrub_value(v):
    """get_usage carries per-account usage stats; keep the shape, drop contents."""
    if isinstance(v, dict):
        return {k: ("<scrubbed>" if k == "behaviors" else scrub_value(x)) for k, x in v.items()}
    if isinstance(v, list):
        return [scrub_value(x) for x in v]
    return v

for path in glob.glob(os.path.join(os.path.dirname(__file__), "fixtures", "*")):
    out = []
    for line in open(path):
        if path.endswith(".jsonl"):
            e = json.loads(line)
            try:
                inner = json.loads(e["line"])
                e["line"] = json.dumps(scrub_value(inner), separators=(",", ":"))
            except (json.JSONDecodeError, TypeError):
                pass
            line = json.dumps(e, separators=(",", ":")) + "\n"
        out.append(scrub_text(line))
    open(path, "w").writelines(out)
    print("scrubbed", os.path.basename(path))
