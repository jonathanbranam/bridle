---
id: jf9u
title: Context wakes never reach the orchestrator on a client machine (NUC)
opened: 2026-09-30
repos: [bridle]
changes: []
specs: []
needs: []
see: [7d62, chvf]
---

## The ask


Reported by the NUC's orchestrator (m-2877, 2026-09-30): the meta-notes orchestrator on the NUC
reached ~182K context and got no `context` wake. It was launched properly
(`bridle session orchestrator --project meta-notes`), and `~/.bridle/orchestrator.pid`,
`orchestrator.session` and `~/.bridle/context/<session>` (182340) were all current. But
`bridle events --kind orchestrator.context` is empty and `bridle status` shows no context. The
daemon's workspace `.bridle/` (`/srv/shared/work/meta-notes-work/.bridle/`) has none of those files.

## What the code says (at 30b2b61)

The launcher (`crates/bridle/src/session.rs`), the note-session hook and the statusline write
under `discovery::bridle_home()`: `$BRIDLE_HOME`, else `~/.bridle`. The daemon's supervisor
(`crates/bridle-daemon/src/lib.rs`, `orchestrator_task`) reads from `overrides.bridle_home`,
else the same `bridle_home()`, and only runs when `[orchestrator] enabled`. So the likely causes,
to check on the NUC in this order:

1. `[orchestrator] enabled` is off for meta-notes. Then no supervisor runs at all. The log line
   "no orchestrator.pid: the orchestrator supervisor does nothing" would be absent too.
2. The daemon runs with a different `BRIDLE_HOME` (or `--bridle-home` override) than the
   orchestrator session, e.g. from how it was started on the NUC, so the two sides look in
   different directories.
3. The NUC's binary (0.3.0, built 19:06Z) predates a fix (chvf: client binaries lag).

## Also

- `~/.bridle/daemon.json` on the NUC names a `dotfiles-local` daemon (workspace `/home/jbranam`,
  port 37607), possibly stale.
- One `orchestrator.pid` per home can't serve two orchestrators on one machine; 7d62 (per-project
  pid files) covers that.

## Done when

The cause is found and fixed, and a client-machine orchestrator gets `context` wakes, with a test
for whichever split caused it.
