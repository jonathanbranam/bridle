---
id: p372
title: "Self-upgrade: wake only on what needs attention, record every step as an event"
kind: feature
opened: 2026-10-03
repos: [bridle]
changes: []
specs: []
needs: []
see: [d3wq, q7rx, chvf]
tasks: [br-e5f1]
closed: 2026-10-09T23:11:02Z
---

## The ask


The human, verbatim (2026-10-03, via the advisor), first asking about self-upgrade:

> Yeah, I'm interested in the self-update question. Is that a no-op? Is anyone notified? I feel
> like the orchestrator has sent me messages about that. If that needs some sort of automation fix
> so that it's not using up an agent's focus to deal with, let's consider that.
>
> Is it causing a wake or a restart? I just actually don't even know what's going on with that,
> but that's an issue in bridle, I think, only because we're hosting. Bridle development hosts
> bridle itself, so if that's constrained to this repo, it might not be a big problem. You could
> run a diff stat on any pushes, and the daemon could determine if the changes were only to docs
> files and then be smart about restarting. I don't know what else is built there or how much
> more complicated that would be, but if we can automate that decision, we should do it.

Then, agreeing to the fix below:

> Yes, I agree with the self-update fixes. What we're looking for is to reduce the wakes on the
> orchestrator and reduce unnecessary messaging to an agent. A message that says, "Everything's
> normal, nothing happened," is not something that should wake up the orchestrator. Another thing
> to check is whether these are logged anywhere, like those events, because that would probably
> be useful to have stored as events so that we can come back and diagnose what's been going on.

## Today (advisor, checked at 3edc5b9)

- **Docs-only commits already skip the build and restart** (d3wq, br-ba5c, e682bd7):
  `git diff --name-only <built>..<candidate>`; nothing under `crates/`, `.cargo/`, `Cargo.toml`,
  `Cargo.lock` or `rust-toolchain*` means record it as built, no restart
  (`docs/design/agent-host/daemon.md`, "Docs-only commits skip the build").
- **But every step wakes the orchestrator** (`upgrade_in_background`, `server.rs`): `upgrade`
  "skipped …" for each docs-only green commit, `upgrade` "building …" for each code commit, then
  `restart`. Most of our commits are docs, so most of these wakes say nothing happened.
- **"No quiet point" is treated as a failure.** If agents stay busy for the 600 s quiet-point
  wait, the restart gives up: `upgrade_failed` wake to the orchestrator, plus a `system` note in
  the human's inbox ("built X but did not restart: not restarted: no quiet point within 600s;
  still busy: …"). Four of these since 2026-09-30 (m-2573, m-2661, m-2849, m-3345), none a real
  failure. The commit is then not retried (`note_failed`) until a newer green commit arrives.
- **Nothing of this is in the event log.** Upgrade wakes live only in the in-memory wake queue
  (`wake.rs` `push`) and `daemon.log` via `tracing`; the only related events are
  `daemon.stopping` and `daemon.started`. Afterwards there's no record of what was skipped,
  built, given up on, or why.
- Only bridle's own daemon has `self_upgrade` on; the NUC's daemons don't (chvf).

## Proposal

1. **Wake only on what needs attention.** No orchestrator wake for `skipped` or `building`. The
   `restart` wake after a successful upgrade stays (agents and the orchestrator learn the new
   commit). Nothing goes to an agent that just says all is normal.
2. **"No quiet point" is "try later", not a failure.** Keep the build, keep waiting for the next
   quiet point on later ticks (or rebuild when a newer commit arrives), silently. Escalate (one
   `upgrade_failed` wake, no human note) only if it keeps giving up for a long time, e.g. 3 hours.
3. **Real failures stay loud:** a failed build, a failed self-check, a rollback: wake
   `upgrade_failed` and tell the human, as today.
4. **Record every step as an event** (`upgrade.skipped` with the commit and the changed-paths
   reason, `upgrade.building`, `upgrade.built`, `upgrade.waiting` / `upgrade.gave_up` with the busy
   agents, `upgrade.failed` with the error, `upgrade.rolled_back`), so `bridle events --kind
   upgrade.` shows what happened. Event kinds go in `bridle-api/src/types.rs`.

Docs: `daemon.md` (restart in place), `api.md` (wake reasons), `storage.md` if event kinds are
listed there, CHANGELOG. Acceptance: `just check` passes; tests cover a skipped commit making an
event and no wake, a no-quiet-point give-up making no human note and a later retry.

## Resolution

Resolved 2026-10-09 by advisor (product-manager). Done: the work landed in this ticket's tasks (see `tasks:`). Resolved in the PdM's sweep of tickets whose tasks were all finished.
