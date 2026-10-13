+++
id = "br-vdu8"
title = "A bridle dev site command starts a throwaway daemon, gateway and UI on free ports"
kind = "feature"
state = "integrated"
created_at = "2026-10-13T02:00:28.102Z"
updated_at = "2026-10-13T02:32:59.251204Z"
created_by = "external:orchestrator"
watchers = [
    "external:orchestrator",
    "agent:wvdu8",
]
branch = "bridle/wvdu8"
commit = "70db43fa37181026da9e42364426a4cd5be992b1"
summary = "Added bridle dev site up|down|list (crates/bridle/src/dev_site.rs, cli.rs, fixtures in crates/bridle/fixtures/dev-site/ embedded via include_str; new fixture files need an entry in FIXTURE_FILES). Site dir, own BRIDLE_HOME with [gateway] login, two fixture git repos (task prefixes sh/ad, needed so the shared registry accepts both), one real daemon each seeded via the CLI with --url/--token from the repo cwd, gateway, down/list by recorded pid + ps start time + argv. Deviations: gateway port is probed (it cannot report :0); no [projects] entries (registry finds daemons); no usage figure seeded; no [dev_site] ui_dir key; a missing default ../bridle-ui is a warning and gateway-only. UI step (5) is coded but untested: no bridle-ui half yet. Uses curl for health polling. Classified 'dev' as NoProject in project.rs and the resolution test."
ticket = "vdu8"
+++

docs/tickets/open/a-bridle-dev-site-command-starts-a-throwaway-daemon-gateway-vdu8.md

## Thread

### note · external:orchestrator · 2026-10-13T02:00:47.299Z
Readied by orchestrator on the human's go, via advisor product-manager (m-11853), 2026-10-12 ~9:15 PM ET: "Yes that would be good to get done": schedule vdu8, dgef (UI half, on bridle-ui) and tgxy right after usage epic rdkn. Ask: br-vwqt.

### note · agent:pm-1 · 2026-10-13T02:01:22.738Z
Brief (pm-1). Model: Sonnet. Size: L: the largest of the pair, watch the context. The ask above and docs/design/dev-site.md sections 1 and 2 are the spec (read those, 178 lines, and nothing else of the design). Build order, each step testable: (1) site dir, BRIDLE_HOME and fixture projects (git repos with tickets/design/specs/documents), embedded in the binary; (2) one real daemon per fixture project seeded through the API, no agents, no `claude`; (3) the gateway with its own [gateway] login; (4) `down`/`list` with recorded pids and the start-time plus argv check before any kill (rule no-kill-by-name: kill only recorded pids you started); (5) the UI dev server step with VITE_DEV_PORT and VITE_API_TARGET, skipped by `--no-ui` and when the bridle-ui half (dgef, in bridle-ui) is absent. If it passes a sane context budget after step 4, mark built for steps 1-4 and tell the manager so step 5 is split off. Files likely: crates/bridle/src (new `dev site` subcommand module), crates/bridle-daemon and bridle-gateway only as needed for free-port binding (:0), docs/design/cli.md, docs/design/dev-site.md (status line), CHANGELOG.md. Acceptance: the Verify list in the ask (two sites at once, `down` of one leaves the other, login works and /api/v1/projects returns the fixtures, ~/.bridle untouched, `down` leaves a reused pid alone) as tests where they can run offline; `just check` passes. Migration: none (no project files). Out of scope: the rule and role text (br-tgxy), the bridle-ui half.

### note · agent:wvdu8 · 2026-10-13T02:32:53.030Z
built: 711c0d12 (branch bridle/wvdu8)

### note · agent:wvdu8 · 2026-10-13T02:32:53.076Z
done: bridle dev site up|down|list, steps 1-5 coded; 1-4 tested (integration test: two sites, login, /projects returns both fixtures, down of one leaves the other, HOME untouched; unit tests incl. reused pid). Step 5 (Vite with VITE_DEV_PORT/VITE_API_TARGET) is written but UNTESTED, no bridle-ui half to run: split off a verify-step-5 ticket for after dgef. just check exit 0, 1636 tests passed, sha 711c0d12 (main already merged)

### note · agent:manager-2 · 2026-10-13T02:32:59.251Z
integrated: 70db43fa37181026da9e42364426a4cd5be992b1 (branch bridle/wvdu8)
