---
id: cr7t
title: "Add a postmortem ticket kind: the full write-up after an incident"
kind: feature
opened: 2026-10-04
repos: [bridle]
changes: []
specs: []
needs: []
see: [h3ar, mvtz]
tasks: [br-cr7t]
---

## The ask

The human, verbatim (2026-10-04 ~7:15 PM ET, to the advisor):

> Let's introduce a new ticket type called "postmortem" or something. Actually, making this a ticket
> might be kind of weird, but let's go with it for now.
>
> I'd like to record some of the learnings that we run into [...]. I'd like to have a full write-up
> of this that we can refer back to about exactly what happened, what the permissions were that
> allowed this, and the rules that allowed this.

The first postmortem, written before this kind exists, is
[[postmortem-sessions-killed-each-other-s-wake-waiters-with-pk-mvtz|mvtz]] (kind `research` until
this lands).

## Where it fits

There are two records of a failure today:

- the `incident` task kind (`docs/design/agent-host/incidents.md`). It is **live**: the
  orchestrator owns it, it is active while `planned`, it is never queued, and agents are told about
  it.
- the one-paragraph entries in `docs/context/incidents.md`: what happened, impact, cause, category,
  follow-up.

Neither is the full write-up the human asked for. A postmortem is that write-up, for an incident
worth learning from: the timeline, how it happened, the permissions and rules that allowed it, what
went well, the learnings, and the actions with links. It's written after the cause is known and kept
to refer back to.

## Proposals

- **P1.** A `postmortem` kind in `TaskKind` (`crates/bridle-api/src/types.rs`), in `bridle ticket
  new --kind` and `bridle task new`, and in the ticket checker. That is a wire-format change, so
  every client changes together.
- **P2.** It's never queued: no PM plans it and no worker claims it. Its task starts `pending`, as
  the human's review of the write-up. The human closes it, or approves it with `ready`, after reading
  it, and then the ticket is resolved. Its actions are separate tickets and tasks, linked from its
  Actions table.
- **P3.** Sections, in order: The ask (if someone asked for it), Summary, Impact, Timeline, How it
  happened, The permissions that allowed it, The rules that allowed it, What went well, Learnings,
  Actions. Times in UTC with ET in brackets. A short template goes in `docs/README.md`, next to the
  ticket conventions.
- **P4.** The incident log entry (`docs/context/incidents.md`) links the postmortem, and the
  postmortem links its incident ticket.
- **P5.** When to write one: when the human asks; when an incident repeats one already in the log
  (as h3ar repeated fx7x); or when the orchestrator judges the cost worth it. The orchestrator files
  it, or an advisor if the human asks.
- **P6.** Retype mvtz to `postmortem` once this lands.

## Questions

- **Q1.** Ticket, or a separate folder? The human: "making this a ticket might be kind of weird".
  Tickets are resolved and moved, while a postmortem is a dated record like a research report. The
  alternative is `docs/postmortems/<date>-<slug>.md`, never moved, with a task for the review. The
  proposals above assume a ticket, as the human asked "for now". Recommendation: keep the ticket for
  now. Moving to a folder later is a rename.
