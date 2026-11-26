---
id: br-cyzr
title: Show the installed UI version and how far it is behind bridle-ui main in bridle status
kind: feature
created: 2026-11-26
created_by: agent:pm-1
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-vdvw]
resolved: 2026-11-26T17:32:14Z
---

## The ask

`bridle status` shows which bridle-ui version is installed and how far it is behind bridle-ui main, so a stale UI is visible. Split from br-vdvw (item 3 of the proposal the human approved on its thread).

The human's approval, 2026-11-26 ~12:05 PM ET (via the bridle-ui aide), to the proposal "the stale UI is visible: bridle status and the UI footer show the installed UI version and how far it is behind bridle-ui main", verbatim: "Okay, good. Thank you. Approved." The requirement behind it, verbatim: "The solution that should be in place (and I do not understand why it is not) is that when Bridle updates on each machine, the UI updates on each machine."

## The work

- `bridle status` (and `--json`) shows the installed UI version from `~/.bridle/ui/bridle-version` (format e.g. `v0.8.0 bridle-ui b0f3dbe`), and says plainly when there is none or when the machine serves a UI installed another way (dalek installs from its own bridle-ui build with `npm run install-ui`, which must keep working; check what that writes).
- The gateway or API exposes the same value so the UI footer can show it (the footer itself is a bridle-ui ticket).
- "How far behind bridle-ui main": find the cheapest honest source. It needs bridle-ui's main, which not every machine has (a local clone on dalek; GitHub elsewhere). Do not add a new secret or a network call on every `status`; a cached, best-effort check, or "unknown" with the reason, is fine. Say on the thread what you chose.
- Update `docs/design/cli.md` and `docs/design/human-web-ui.md`. Changing the wire format means `bridle-api/src/types.rs` and all clients together.
- Migration: new fields only, optional; older UIs and daemons keep working.

## Acceptance

`just check`; tests for present, missing and unreadable version file.

## Model

Sonnet. Out of scope: moving `ui.pin` and the release check (br-vdvw), the footer (bridle-ui), dalek's landing-time install (the human rejected dropping it).
