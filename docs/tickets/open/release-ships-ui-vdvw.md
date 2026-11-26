---
id: br-vdvw
title: "Each bridle release ships the current bridle-ui: ui.pin is stuck at 2026-10-12, so bridle updates install a six-week-old UI"
kind: bug
created: 2026-11-26
created_by: agent:pm-1
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

## The ask

Updating bridle on a machine must update its UI to the current bridle-ui. A release must never ship a stale `ui.pin`.

The human, 2026-11-26 (via the bridle-ui aide), verbatim: "The solution that should be in place (and I do not understand why it is not) is that when Bridle updates on each machine, the UI updates on each machine."

## What the aide found

- br-t3dr (resolved) is in place: `.github/workflows/release.yml` builds bridle-ui at the commit in `ui.pin`, and self-upgrade installs it into `~/.bridle/ui/`. remus did so on Nov 14 (`v0.8.0 bridle-ui b0f3dbe`).
- `ui.pin` was last moved in "Release 0.7.1" (41ac413f, 2026-10-13). It is still b0f3dbe (bridle-ui 2026-10-12), 31 commits behind bridle-ui main, so v0.8.0 shipped a six-week-old UI. remus and vesta serve it; dalek looks current only because its manager runs `npm run install-ui`.
- Bumping `ui.pin` is a manual step ("part of cutting a release", `docs/design/human-web-ui.md` ~line 145), and it was missed.

## The work (this ticket)

Make it impossible to recur. Preferred: the release step moves `ui.pin` to bridle-ui origin/main (after its CI is green) and commits it, or the release refuses (fails loudly) when the pin is behind bridle-ui main. Find how a release is cut today (release recipe/doc, `justfile`, `release.yml`, CHANGELOG convention) and choose the smallest change that cannot be skipped; a check inside `release.yml` that fails on a stale pin is the backstop that works however the release is cut. Note that release.yml may not be able to read bridle-ui main or its CI status without a token: check what its existing bridle-ui checkout uses. Do not add a new secret without saying so on the thread and asking.

Update `docs/design/human-web-ui.md` and the release instructions in step. Edit the existing text/rules; add no new rule.

## Not in this ticket

Item 1 of the submission (move `ui.pin` now and cut a release so remus and vesta get the current UI) is cutting a release: that is for whoever cuts releases (orchestrator with the human's say), not a worker.

## Acceptance

`just check` passes; the new check is shown failing on a stale pin and passing on a current one (without publishing a release or tag). No migration: nothing in projects' files changes; machines pick the UI up on their next bridle update.

## Model

Sonnet.
