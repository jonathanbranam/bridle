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

## Decided

The human, verbatim (2026-10-04 ~7:30 PM ET, to the advisor), on where postmortems live:

> Yeah, I don't know where postmortems live. It doesn't seem like a ticket, or it's a resolved
> ticket, so I don't really know, but I want it written down somewhere. It seems like it could be in
> a docs folder, but I do want them to follow the syntax of the ticket, I think.

So: a docs folder of their own, written in the ticket syntax.

## Proposals

- **P1. A folder, in ticket syntax.** Postmortems live in `docs/postmortems/<slug>-<id>.md`. They
  have the ticket frontmatter (`id`, `title`, `kind: postmortem`, `opened`, `repos`, `see`,
  `tasks`, and so on) and an ID from the shared ID alphabet and space, so `[[...-<id>|<id>]]` links
  and `see:` work as they do for tickets. They are never moved: unlike `open/` and `resolved/`,
  where the folder is the state, a postmortem is a dated record like a research report.
  `docs/README.md`'s layout and "How the folders work" gain the folder.
- **P2. Tooling.** A `postmortem` kind in `TaskKind` (`crates/bridle-api/src/types.rs`), in
  `bridle ticket new --kind` and `bridle task new`. `bridle ticket new --kind postmortem` writes to
  `docs/postmortems/`. `bridle ticket check` checks that folder as well: IDs unique across every
  folder, and `see:` resolves into and out of it. The kind change touches the wire format, so every
  client changes together.
- **P3. Its task is the human's review.** It's never queued: no PM plans it and no worker claims it.
  The task starts `pending` and is closed once the human has read it. The file stays where it is.
  Its actions are separate tickets and tasks, linked from its Actions table.
- **P4. Sections**, in order: The ask (if someone asked for it), Summary, Impact, Timeline, How it
  happened, The permissions that allowed it, The rules that allowed it, What went well, Learnings,
  Actions. Times in UTC with ET in brackets. A short template goes in `docs/README.md`.
- **P5.** The incident log entry (`docs/context/incidents.md`) links the postmortem, and the
  postmortem links its incident ticket.
- **P6. When to write one:** when the human asks; when an incident repeats one already in the log
  (as h3ar repeated fx7x); or when the orchestrator judges the cost worth it. The orchestrator files
  it, or an advisor if the human asks.
- **P7.** Move mvtz to `docs/postmortems/` with `kind: postmortem` once this lands. Until then it
  stays in `docs/tickets/open/` as `research`, where the checker and its task's path find it.
