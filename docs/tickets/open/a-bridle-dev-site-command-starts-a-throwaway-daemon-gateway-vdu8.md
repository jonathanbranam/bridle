---
id: vdu8
title: A bridle dev site command starts a throwaway daemon, gateway and UI on free ports
kind: feature
opened: 2026-10-06
repos: [bridle]
changes: []
specs: []
needs: []
see: [vwqt]
tasks: []
---

## The ask

Build `bridle dev site up|down|list` as designed in [[docs/design/dev-site|dev-site]] sections 1 and 2 (bridle half): per-site `.agent-site/<name>/` with its own BRIDLE_HOME and `[gateway]` login, embedded fixture projects (git repos, tickets open and resolved, design, specs, documents, two projects), one real daemon per fixture project seeded through the API (no agents, no `claude`), the gateway, then the UI dev server with VITE_DEV_PORT and VITE_API_TARGET. Free ports by binding :0; stop only recorded pids after a start-time and argv check; many sites at once. Writes site.json with a fixture manifest.

Verify: two sites at once on different ports; `down` of one leaves the other; login works and `/api/v1/projects` returns the fixtures; `~/.bridle` untouched; `down` leaves a reused pid alone. Needs the bridle-ui ticket for the UI half (`--no-ui` works without it).
