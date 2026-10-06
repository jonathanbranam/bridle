---
id: dgef
title: bridle-ui dev server takes a port and API target, and checks the fixture site
kind: feature
opened: 2026-10-06
repos: [bridle-ui]
changes: []
specs: []
needs: []
see: [vwqt]
tasks: []
---

## The ask

For the bridle-ui project (file it there). Design: bridle's docs/design/dev-site.md section 2.

The human, 2026-10-05: "I think we need to be able to run a dev server and pick a port to use, typically, or assign a random port. If it's going to be run by multiple agents while they're working on tasks, then they should pick a free port."

1. `vite.config.ts` reads `VITE_DEV_PORT` (with `strictPort: true`) and `VITE_API_TARGET`, keeping today's values as defaults, and proxies `/api` to the target. Without the target a client on another port would still hit the human's gateway.
2. `npm run check:site -- <site.json>` logs in to a running `bridle dev site` and asserts the fixture manifest's facts (ticket counts, a stem link resolving, the to-do question present). It fails when the UI and the fixture drift.
3. README and CLAUDE.md "Verification": `bridle dev site up --ui .`, and that a UI task is verified in a browser before done.

Verify: start two sites with different VITE_DEV_PORT values, each proxying to its own gateway; `check:site` passes on both.
