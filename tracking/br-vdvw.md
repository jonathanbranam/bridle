+++
id = "br-vdvw"
title = "Each bridle release ships the current bridle-ui: ui.pin is stuck at 2026-10-12, so bridle updates install a six-week-old UI"
kind = "bug"
state = "open"
created_at = "2026-11-26T16:05:09.564Z"
updated_at = "2026-11-26T16:06:57.675489Z"
created_by = "external:aide"
watchers = ["external:aide"]
ticket = "vdvw"
+++

docs/tickets/open/release-ships-ui-vdvw.md

submitted by external:aide

Filed by the bridle-ui aide, 2026-11-26 ~11:45 AM ET.

The human, verbatim: "The solution that should be in place (and I do not understand why it is not) is that when Bridle updates on each machine, the UI updates on each machine."

## What the aide found

- br-t3dr (Oct 11) is in place: release.yml builds bridle-ui at `ui.pin` and self-upgrade installs it into `~/.bridle/ui/`. Remus did exactly that on Nov 14 07:04 when it upgraded to v0.8.0 (`~/.bridle/ui/bridle-version`: `v0.8.0 bridle-ui b0f3dbe`).
- But `ui.pin` was last moved in "Release 0.7.1" (41ac413f, 2026-10-13). It is still b0f3dbe (bridle-ui, 2026-10-12), 31 commits behind bridle-ui main. So v0.8.0 shipped a six-week-old UI to every machine. Remus and vesta serve it; only dalek is current because its manager runs `npm run install-ui` after each landing.
- The human never sees the stale pin on dalek, so nobody noticed.

## The ask

1. Now: move `ui.pin` to bridle-ui origin/main and cut a bridle release, so remus and vesta get the current UI by their normal bridle update.
2. Make it impossible to recur: a release always ships the current bridle-ui main (e.g. the release step moves `ui.pin` to bridle-ui origin/main, after its CI is green, and commits it), or refuses to release with a pin behind main. A machine's bridle update then always brings the UI current as of that release.
3. The human's words above are the requirement: updating bridle on a machine updates its UI.

## Thread

### note · external:aide · 2026-11-26T16:05:09.568Z
submitted by external:aide

### note · external:orchestrator · 2026-11-26T16:05:40.130Z
Readied by orchestrator (dalek): bug, self-evident fix (rule ready-without-asking). Item 1 (bump ui.pin + cut a release now) is a release decision; taken to the human.

### note · external:orchestrator · 2026-11-26T16:05:51.271Z
readied by external:orchestrator

### note · external:aide · 2026-11-26T16:06:57.675Z
From the human, via bridle-ui aide (2026-11-26 ~12:00 PM ET). Asked "How are we going to prevent this from ever happening again?" The aide proposed: (1) the release step moves ui.pin to bridle-ui main itself (after its CI is green and the API version matches) and stops loudly rather than shipping an old UI; (2) dalek drops its landing-time install-ui; (3) the stale UI is visible: bridle status and the UI footer show the installed UI version and how far it is behind bridle-ui main. The human rejected (2), verbatim: "I don't agree with two. I think Dalek is unique since it's the host that builds the bridle-ui. It also doesn't install releases from GitHub." So the ask is (1) and (3); dalek keeps its landing-time install.
