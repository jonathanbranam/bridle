---
id: nc7r
title: Incident notices that go to every agent and disappear when it's over
opened: 2026-09-29
repos: [bridle]
changes: []
specs: []
needs: []
see: [kp3f, ex9q]
---

## The ask

The human, verbatim (2026-09-29):

> When something big happens like SSH is down, messages should go out to agents and then
> updated when the incident is over. Ideally, the message should be removed. This kind of
> need has come up repeatedly and should be considered.
>
> Eg advisor don't know ssh was down. But let's say you sent a message when advisor was down
> then we fixed ssh then advisor came up. Ideally, advisor would never know about the incident.

## Why

On 2026-09-29, after a reboot, the SSH agent had no key and `git push` failed. The
orchestrator told manager-2 by hand; the advisor, pm-1 and the other projects' managers
never heard, and the advisor found out on its own. Earlier cases of the same shape: the
daemon restarting (every `lost` worker told by hand), budget holds, a red `main` ("don't
merge until green"), the syspolicyd hangs (`docs/context/incidents.md`).

## Sketch (not a design)

An **incident** (or notice) is a record, not a message: opened with a short body and an
audience (all agents, a role, external principals, other projects' daemons?), and closed
when it's over.

- While open, every agent in the audience sees it: delivered to running agents, and to an
  agent that starts or resumes while it's open (in its first message, or the system prompt).
- Closing it sends a short "resolved" to those who saw it. **An agent that never saw it
  never hears of it**, and an undelivered notice is dropped, not delivered late.
- Updates while open replace the body (agents who saw it get the update).
- `bridle status` lists open incidents; the watcher could wake on one opening.

Open: who opens one (the orchestrator, managers, the daemon itself for things it detects,
like a failed push, a red `main` via c8qw, or a budget hold); whether it spans projects
(one incident, three daemons); whether the budget governor's hold notices become incidents.
Held messages (`when`/pending delivery) may be most of the mechanism already: a message
that is withdrawn while still pending is never delivered.

## Decisions (design in [[docs/design/agent-host/incidents|incidents]])

- A record (`incidents` table, `messages.incident_id`), states `open`/`closed`; an update rewrites the body.
- Audience: `all` or `role:<name>`. External principals read `bridle status`/`incident list`, no push. Cross-project isn't in v1: the orchestrator opens one per daemon.
- Open/update/close: human, orchestrator, managers. The daemon opens none yet; budget holds stay as they are.
- Delivery is the message queue: `system` notes, `when idle`. Close drops `pending`/`held` notices (never seen) and sends "resolved" to the rest. Agents that start or resume while open get a notice then; not in the system prompt.
- Surfaces: `bridle incident open|update|close|list|show`, `status`, `/v1/incidents`, `incident.*` events.
- Build split: held until the orchestrator frames incidents and human to-dos as one pattern.

## The human's decision: incidents and to-dos are tasks (2026-09-29, br-c83e)

Asked by the orchestrator (br-c83e): one "open request" record for incidents (nc7r) and human
to-dos (ex9q), keep them separate, or discuss? The human, verbatim:

> An incident seems unique to me - it should broadcast everywhere, and probably orch should own
> resolving it. Most likely, orch owns incidents I think. If someone detects or suspects and
> incident, they could file a "potential incident" which goes to orch to evaluate and promote to
> active incident. All agents can check pending incidents so they search before creating a new
> one; incidents should have comments and status and updates. They are like tasks. I think many
> things are like this: they have a state, owner, a description, and comments. Incidents feel
> more like a special case of a task and so does a todo sent to me - it's a task someone is
> asking me to do.
>
> I feel better about enhancing the task solution to support incidents as a means of tracking
> them since that machinery exists. The only other machinery that is needed is the broadcast and
> persistent messsage that gets sent to any cycled workesr.

What that means (the advisor's reading):

- **Neither is a new record type; both are bridle tasks.** The "open request" record and the
  separate `incidents` table in [[docs/design/agent-host/incidents|incidents]] are superseded.
- **A to-do for the human** is a task assigned to the human (ex9q's Shape; WIP 648321e on
  `bridle/human-todos` fits this).
- **An incident** is a task of its own kind, owned by the orchestrator, with the usual body,
  thread (comments and updates) and state. Anyone who detects or suspects one files it as a
  *potential* incident, after searching open ones first (`bridle task search`/`list`); the
  orchestrator evaluates it and promotes it to *active*, or drops it, and resolves it.
- **The only new machinery** is the broadcast: while an incident is active, a persistent notice
  goes to every agent, including any that start, resume or renew (cycled workers) while it's
  open. The drop-if-undelivered and "resolved" behaviour from the incidents design carries
  over.
- `docs/design/agent-host/incidents.md` needs revising to this.

## Example: an alert that didn't retract itself (2026-09-29)

m-2136 to the human at 01:36Z: "The orchestrator has no wake command running (`bridle
wait-for-wake`) since 01:34:39 UTC." The orchestrator was busy (a burst of advisor messages) and
restarted its waiter within minutes; the notice stayed unread in the human's inbox anyway. The
human asked for the grace to double (2m to 4m, `[orchestrator] waiter_grace`); this ticket's part
is that a notice whose condition has cleared should withdraw itself.

## Example: the main clone off `main` during a build (2026-09-29)

At 22:57 ET the orchestrator ran, in the foreground in the main clone, `git checkout -q 7d29cfd;
cargo install --path crates/bridle; git checkout -q main` to install the CI-verified commit
rather than `main`'s newer tip. For the minutes the build ran under load, two things were wrong
and nothing said so except one side effect:

- **The clone was detached from `main`.** `bridle land` is safe (with `main` checked out nowhere
  it moves the ref by `update-ref`, `integrator.rs` `advance`), but a commit made in the clone by
  the orchestrator or an advisor would have gone on the detached HEAD and been dropped by the
  checkout back to `main`. Had the session died mid-command (as the orchestrator did at 08:36
  that day), the clone would have stayed detached with no one told. A self-retracting "clone not
  on `main`" notice would cover it.
- **The orchestrator was tied up.** m-2257 to the human at 03:00Z: "The orchestrator has no wake
  command running (`bridle wait-for-wake`) since 02:58:22 UTC", the moment the build started.
  That alert was the only visible sign. Both conditions cleared on their own when the build
  finished, so both notices should have withdrawn themselves.

The human, verbatim:

> that looks like a bad decision; we should not be moving this clone off of main like that, it
> could be a big problem and who knows how long a build might take under load. it's very
> possible the manager could merge something

> orch shouldn't be tied up building and if a build is done, doing it on main might be ok, but
> we've also had a merge happen during a build which broke the build! so, IDK, builds during
> heavy work time might need to be done on a worktree.

Passed to the orchestrator as a standing rule (m-2258): the main clone stays on `main`; a pinned
build goes in its own worktree, as `bridle restart --upgrade` does (`upgrade.rs`).
