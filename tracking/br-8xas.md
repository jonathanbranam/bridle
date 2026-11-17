+++
id = "br-8xas"
title = "bridle ci keeps a re-run CI run red: it does not pick up a passing re-run attempt"
kind = "bug"
state = "integrated"
created_at = "2026-11-17T17:50:22.210Z"
updated_at = "2026-11-17T20:45:14.387210Z"
created_by = "external:orchestrator"
watchers = [
    "external:orchestrator",
    "agent:8xas",
]
branch = "bridle/8xas"
commit = "8d3913addcfb8723e5ee23765a0bec52d85a62ef"
summary = "The CI watcher (crates/bridle-daemon/src/ci.rs) kept a set of seen commits and never re-looked at one. A re-run keeps the run id and replaces its result, so the first failure stuck. seen is now a map of sha to reported conclusion; a changed conclusion on a finished commit is reported again (event, last status). Test: failure then success flips to success, no extra event after. No design docs described this."
ticket = "8xas"
+++

docs/tickets/open/bridle-ci-keeps-a-re-run-ci-run-red-it-does-not-pick-up-a-pa-8xas.md

## Thread

### note · external:orchestrator · 2026-11-17T17:50:43.848Z
readied by external:orchestrator

### note · external:orchestrator · 2026-11-17T17:50:44.312Z
Filed and readied by orchestrator (bug; rule ready-without-asking).

### note · agent:8xas · 2026-11-17T20:44:49.827Z
built: 147a9709c372a1afd99280852473df629f8b115c (branch bridle/8xas)

### note · agent:8xas · 2026-11-17T20:44:49.874Z
done: CI watcher re-reports a commit when its conclusion changes (passing re-run flips to success); check exit 0, 2017 tests passed; 147a9709c372a1afd99280852473df629f8b115c

### note · agent:manager-2 · 2026-11-17T20:44:58.832Z
integrated: 8d3913addcfb8723e5ee23765a0bec52d85a62ef (branch bridle/8xas)

### note · agent:manager-2 · 2026-11-17T20:45:14.387Z
cleanup: removed agent 8xas, origin branch bridle/8xas, branch bridle/8xas
