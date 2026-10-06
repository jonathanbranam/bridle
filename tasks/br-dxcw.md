+++
id = "br-dxcw"
title = "vwqt: agents run a throwaway bridle site (daemon+gateway+UI) on a free port, log in and verify UI work"
kind = "feature"
state = "integrated"
created_at = "2026-10-06T00:07:44.945Z"
updated_at = "2026-10-06T03:17:08.159187Z"
created_by = "external:orchestrator"
watchers = [
    "external:orchestrator",
    "external:aide",
]
branch = "bridle/dev-site-doc"
commit = "511b3b7183b42f6659dd292e889b28e4667eec26"
summary = "Design only: docs/design/dev-site.md (status planned, linked from docs/README.md). Settles bridle dev site up|down|list (own BRIDLE_HOME and [gateway] login, embedded fixture git repos, one real daemon per fixture, gateway, vite via VITE_DEV_PORT/VITE_API_TARGET, free ports by binding :0, pid-and-start-time-checked stop), the bridle/bridle-ui split, playwright-cli verification with a proposed base rule, and carry-over (track-web keeps its recipe; gitignore migration per xebc). Tickets: vdu8 (bridle build), dgef (bridle-ui piece, file there), tgxy (rule, role text, migration). just check: 1258 passed (first run had one flaky slow session_test, passed alone and on rerun). bridle ticket check has 20 pre-existing problems, none mine."
+++

Ticket (the ask with the human's words; read first): docs/tickets/open/agents-can-run-a-throwaway-bridle-site-on-a-free-port-log-in-vwqt.md
This task is the DESIGN only. Output is a design doc plus follow-up tickets; build tasks come after the PM sizes them.
Goal: design how any agent runs a throwaway bridle site (daemon, gateway and UI) on a free port, logs in and clicks around to verify UI work, modelled on track-web's docs/dev-second-instance.md (read it read-only from the track-web checkout; find it via `bridle project list`/the config registry, or ask on the task; change nothing there).
The design must settle, each with the alternative rejected:
1. The command (e.g. `bridle dev site` or a just recipe): starts a gateway with its OWN config (own [gateway] username/password, not ~/.bridle/config.toml) and a fixture folder of fake projects (tickets open and resolved, design/specs, documents) plus whatever daemon data /items, /tasks and /system need; then the UI dev server on another free port proxying /api to it; prints URL and login; stops only its own processes by pid (never by name, rule no-kill-by-name); many agents at once (free ports, separate gitignored directories, cleanup).
2. The split between bridle (gateway/daemon side, fixtures, the command) and bridle-ui (a dev-server piece: VITE_DEV_PORT and VITE_API_TARGET style config, and a matching fixture check). NAME the bridle-ui piece clearly as its own ticket text the orchestrator can file in bridle-ui.
3. How agents drive it in a browser to verify a task before calling it done (what tool they use, what the worker/manager brief says; propose a rule or role-doc text for UI tasks in workflow/base).
4. How it carries to every project with a web UI (a rule or pack; track-web has its own, so say how they relate) and the migration: what existing projects get and how automatically (ticket xebc direction).
Deliverable: docs/design/dev-site.md (status line "planned"), linked from docs/README.md; feature tickets (`bridle ticket new`, see: vwqt) for the bridle build, the bridle-ui piece, and the rule/pack. No tasks, no code. `bridle ticket check` clean.
Model: Sonnet. Out of scope: building any of it, changing track-web.

## Thread

### note · external:aide · 2026-10-06T00:08:02.964Z
watching the task

### note · agent:dev-site-doc · 2026-10-06T02:32:11.402Z
done: docs/design/dev-site.md + tickets vdu8 (bridle build), dgef (bridle-ui piece, to file in bridle-ui), tgxy (rule/role text/migration). just check exit 0, 1258 tests; commit b0e31c6c. bridle ticket check has 20 pre-existing problems, none in my files.

### note · agent:manager-2 · 2026-10-06T03:17:08.159Z
integrated: 511b3b7183b42f6659dd292e889b28e4667eec26 (branch bridle/dev-site-doc)
