+++
id = "br-wphz"
title = "A tools-only clone's refusal names the machine that owns the project"
kind = "bug"
state = "integrated"
created_at = "2026-11-11T13:02:07.653Z"
updated_at = "2026-11-11T17:16:01.369991Z"
created_by = "external:aide"
watchers = [
    "external:aide",
    "agent:wphz",
]
branch = "bridle/wphz"
commit = "53898bdf46ce75abc3d3dd19b1ae068c7bfec805"
summary = "A tools-only clone's refusal (bridle serve, machine tools-only-check) now names the owner from owner.toml on bridle/state via the existing read_owner (host and since), or says the owner could not be read. owner.toml has no workspace path, so none is shown. Hook text baked at install time is unchanged. One unit test covers both cases (formatter only)."
ticket = "wphz"
+++

docs/tickets/open/a-tools-only-clone-s-refusal-names-the-machine-that-owns-the-wphz.md

## Thread

### note · external:aide · 2026-11-11T13:02:08.373Z
readied by external:aide

### note · external:aide · 2026-11-11T13:02:08.631Z
aide: readied at filing (kind bug, rule ready-without-asking).

### note · agent:pm-1 · 2026-11-11T13:02:50.968Z
pm-1 brief: model Haiku (Sonnet if the owner lookup is not already a function). Files: crates/bridle/src/tools_only.rs (the refusal, line ~32) and wherever `owner.toml` on `bridle/state` is read (reuse that reader, per br-hw6c; do not add a second one). Every command that hits the refusal gets the owner, since they share this one message. If the local state-branch copy has no owner.toml, or origin cannot be reached, the message says the owner could not be read rather than omitting it. Acceptance: a test where a test owner.toml makes the message name the host (and the workspace path if owner.toml records it), and one where it is unreadable; `just check`. Out of scope: any change to ownership itself or to `bridle project take-over`. Note: this comment restarts the settle clock.

### note · agent:wphz · 2026-11-11T14:39:45.023Z
wphz: code, tests and CHANGELOG done on bridle/wphz (uncommitted); just check is queued behind the machine test lock (background run). Will commit, summarize, mark built and report once it exits 0. Note: owner.toml records no workspace path (Owner has host, since, released only), so the message names host and since only.

### note · agent:wphz · 2026-11-11T14:51:14.479Z
built: d8b1893e4588975941dfb5592a317725558a8678 (branch bridle/wphz)

### note · agent:wphz · 2026-11-11T14:51:14.562Z
done: tools-only refusal names owner host+since or says unreadable (no workspace path in owner.toml); just check exit 0, 1990 tests passed, ran on f6c5fec5 (pre-merge); tip d8b1893e is that plus a main merge

### note · agent:manager-2 · 2026-11-11T17:15:29.066Z
integrated: 53898bdf46ce75abc3d3dd19b1ae068c7bfec805 (branch bridle/wphz)

### note · agent:manager-2 · 2026-11-11T17:16:01.369Z
cleanup: removed agent wphz, origin branch bridle/wphz, branch bridle/wphz
