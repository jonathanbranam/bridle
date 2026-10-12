+++
id = "br-ep5z"
title = "K10: the MVP runbook: move a project by hand, step by step"
kind = "docs"
state = "integrated"
created_at = "2026-10-12T16:16:51.749Z"
updated_at = "2026-10-12T18:51:40.809672Z"
created_by = "external:advisor/product-manager"
watchers = [
    "external:advisor/product-manager",
    "agent:wep5z",
]
branch = "bridle/wep5z"
commit = "2fa8d353a49ff057f534b03032654f911a553f0a"
summary = "New runbook docs/context/move-a-project.md (linked from docs/README.md, CHANGELOG line): preflight, drain, stop, take-over, start, checks incl. gateway, session restart, back-out at each step, written from the code and cli.md. Gaps listed in the doc: preflight --to always FAILs 'stable project' until drain ends; drain hold is in memory; back-out after stop has no command (take-over on OLD); gateway-follows-move untested; inbox/events do not move. Not run end to end on real machines."
ticket = "ep5z"
+++

docs/tickets/open/k10-the-mvp-runbook-move-a-project-by-hand-step-by-step-ep5z.md

## Thread

### note · agent:pm-1 · 2026-10-12T16:17:42.910Z
Brief (pm-1). Model: Sonnet. Size: S (docs only). Blocked by K3, K4, K5, K6, K9; write it against the real commands (`bridle project preflight|drain|stop|take-over|start --help`), not the designs. One page under docs/ (operating docs), linked from docs/README.md; ASCII only; doc links root-relative. Include back-out at each step. Out of scope: K7 (one-command move, V1).

### note · external:advisor/product-manager · 2026-10-12T17:19:40.455Z
advisor (product-manager): add to the runbook's checks after start: the gateway (bridle-ui) lists the project at NEW. K6 (br-3v2m) built the mail half only; the gateway follows through [projects] with no code, untested.

### note · agent:wep5z · 2026-10-12T18:51:35.674Z
built: 3c1e6660 (branch bridle/wep5z)

### note · agent:wep5z · 2026-10-12T18:51:35.713Z
done: runbook docs/context/move-a-project.md (+README link, CHANGELOG); just check exit 0, 1620 tests passed, main merged; 3c1e6660. Gaps in the doc's last section: preflight --to fails 'stable project' until drain ends; drain hold in memory; back-out after stop has no command (take-over on OLD); gateway follow untested; not run on real machines.

### note · agent:manager-2 · 2026-10-12T18:51:40.809Z
integrated: 2fa8d353a49ff057f534b03032654f911a553f0a (branch bridle/wep5z)
