+++
id = "br-vdvw"
title = "Each bridle release ships the current bridle-ui: ui.pin is stuck at 2026-10-12, so bridle updates install a six-week-old UI"
kind = "bug"
state = "claimed"
created_at = "2026-11-26T16:05:09.564Z"
updated_at = "2026-11-26T16:42:15.267037Z"
created_by = "external:aide"
watchers = [
    "external:aide",
    "agent:wvdvw",
]
priority = "high"
priority_at = "2026-11-26T16:30:02.895795Z"
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

### note · external:aide · 2026-11-26T16:07:28.459Z
The human approved the ask above (fix now + (1) releases always ship current bridle-ui main + (3) stale UI visible), 2026-11-26 ~12:05 PM ET, verbatim: "Okay, good. Thank you. Approved." Bug: ready it on triage.

### note · agent:pm-1 · 2026-11-26T16:30:02.895Z
priority: normal -> high

### note · agent:pm-1 · 2026-11-26T16:31:00.154Z
pm-1: item 3 (UI version visible in status and footer) is split to br-cyzr (bridle) and a submitted bridle-ui ticket; it stays out of this ticket. For the worker: the approved item 1 also says the release moves ui.pin after bridle-ui CI is green AND the API version matches, and stops loudly rather than shipping an old UI. dalek keeps its landing-time install-ui (the human rejected removing it).

### note · agent:manager-2 · 2026-11-26T16:31:06.608Z
pm-1 added a comment on br-vdvw, please read it (bridle ticket show br-vdvw). The human approved: the release moves ui.pin to bridle-ui main after bridle-ui CI is green AND the API version matches, and stops loudly rather than ship an old UI. Cover both conditions, or say on the thread which part cannot be checked and why (ask before adding any secret). Item 3 (showing the UI version) is br-cyzr: keep it out. dalek keeps its landing-time install-ui.

### note · agent:wvdvw · 2026-11-26T16:42:15.267Z
Handoff (wvdvw): work is committed on bridle/wvdvw with main merged in; release-script-test passes. The full 'just check' is still running in the background (log /tmp/wvdvw-check.log); summary, push, built and report follow when it exits.
