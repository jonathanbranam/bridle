+++
id = "br-wphz"
title = "A tools-only clone's refusal names the machine that owns the project"
kind = "bug"
state = "planned"
created_at = "2026-11-11T13:02:07.653Z"
updated_at = "2026-11-11T13:41:19.084266Z"
created_by = "external:aide"
watchers = ["external:aide"]
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
