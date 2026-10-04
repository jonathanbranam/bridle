---
id: gtzx
title: "Seats: every role is a named, tracked seat that outlives its sessions, with its own inbox, handover and retirement"
kind: feature
opened: 2026-10-04
repos: [bridle]
changes: []
specs: []
needs: []
see: [r8kv, jttf, xxxq, r9vh, ervd, 2vja, 7d62]
tasks: []
---

## The ask

This ticket pulls together what's still wanted from
[[seats-named-interactive-roles-splitting-the-advisor-retiring-r8kv|r8kv]] (seats, the questions),
[[interactive-sessions-context-for-all-daemon-decided-wakes-re-jttf|jttf]] (context and handover
for every session) and [[task-watchers-a-creator-field-a-watchers-list-and-wakes-that-xxxq|xxxq]]
(watchers), as one build. The human, verbatim (2026-10-03 evening ET, via the advisor):

> What I want is every role. Right now, we have roles, and roles don't have names. They just roll.
> Some roles are going to be unique, so they're not going to have a separate name, and that's
> fine. The orchestrator is unique, and that's okay. We're going to leave that unique.
>
> The aide is potentially unique, but I think we will always have one per project that's going to
> replace what we have with an advisor per project. Advisor, which is probably going to need a
> rename, is that great? Advisors can be named, and EFPMO can be named. What we want is for every
> process to be tracked the same way the orchestrators track. I want bridle to know when I create
> a new advisor, when I create a new agent, and anything that gets created should be tracked,
> compared, and managed in the same way that the orchestrator and agent are managed.
>
> Every role needs a name. If it's a unique role, it doesn't need an additional name, but it's
> possible we give them a name. We give each session a name. I think that might be a good idea.
> Every time we restart the handoff, get the new ID for each one. That just retraces the... For
> the other roles, like advisor, needs to have a proper name. What I mean is, the name of the
> advisor is going to be its own mailbox. It's going to be the name that I assign or somebody
> assigns to it. It's meaningful, and there can only be one advisor with a specific name at a
> given time as its own mailbox. The human will determine how long that lives. An advisor with a
> named advisor can live for an hour or a day or a month, and I'll keep track of those. We're
> going to be tracking the ones that have existed. If I shut down that session, I can shut down
> the code session, and that advisor can still exist. When I shut down the session, I may tell it
> to remove itself or not to. Make a plan for what happens when an advisor is removed, if a named
> advisor is removed, or if a named role is removed and then inbox messages are assigned to it or
> messages are sent to it after it's been removed (or if it was like a watcher on a ticket, I
> don't know how to call that). What happens to those notifications? Do they go nowhere, or should
> we still go delete them? We have to read them and deal with them later, so it's important
> they're not lost. They should probably just stay where they are, and we should have a note that
> this advisor, this role, this name has been shut down.
>
> In the future, we'll decide to build a second two. Maybe I say, "Oh, I'm bringing that back,"
> and I revisit that line of work and bring it back up again, or I say, "Hey, no, we're done with
> that." Look through the inbox, if there's anything actionable, and have somebody else do it. I
> think that's a large part of what... They all need to be able to do a handoff to themselves, and
> they need context management. That's all in a ticket. I just lost track of it, but everybody
> needs context. Everybody needs complex management. All complex should be watched, managed, and
> handoff should be possible. They can write a message and hand that message back to themselves
> after they restart. That would be important. This is going to solve a number of problems. Look
> at our recent incidents and tickets that are related to this. I think this is really critical
> work to get done, and the related work is going to be to have all of these agents potentially
> run in the background. Those two things are not dependent. I like to get this working before we
> go with that

Dictation: "they just roll" is "they're just a role"; "EFPMO" is likely "the PM" (product
manager, e.g. `pm-1`); "complex" is "context"; "compared" may be "contained"; "build a second
two" is unclear (perhaps "bring a second one up"). "Get the new ID for each one. That just
retraces the..." is cut off; read as: each restart or handover is a new session id within the
same seat, so the seat keeps the trail.

Then, on how to proceed:

> Sorry, I can't review all this and keep it in my head. So I think, yes, file this all as a
> ticket. I think probably create a new ticket that consolidates some of the information in the
> other tickets and makes it clear what we're asking for. Put all of your suggestions in there as
> comments, and then I'll have to go just review the ticket directly.

**Priority:** the human calls this "really critical". Running agents in the background
([[put-background-agents-to-sleep-and-wake-them-on-demand-with-r9vh|r9vh]]) is related but
doesn't depend on it; this comes first.

**Status: waiting for the human's review of the proposals below.** Its task carries an open
question so nothing builds it before then.

## What the human is asking for (summary)

1. **Every role has a name.** Unique roles (orchestrator; the aide, one per project) need no
   more than the role, but may get one. Other roles (advisor, PM, ...) have a proper name,
   given by the human or whoever creates them.
2. **The name is the mailbox.** Only one live holder of a name at a time.
3. **Bridle tracks every one it creates**, interactive or not, the way it tracks the
   orchestrator and background agents, **including the ones that have ended** (history).
4. **The seat outlives the session.** Closing the Claude Code session doesn't remove the advisor.
   The human decides how long a named role lives (an hour, a day, a month) and whether closing a
   session also removes it.
5. **Removal loses nothing.** Messages, wakes and watcher notifications for a removed name stay
   where they are, with a note that the name is shut down. Later the human either brings it back
   ("revisit that line of work") or says it's done and has someone else go through its inbox for
   anything actionable.
6. **Every role gets context management and a handover to itself:** watched context, a note it
   writes and gets back after its restart. Each session has its own id; the seat ties them
   together.

## What's already built (checked 2026-10-04 at cd2f09c)

- **Role split** (br-r8kv, br-feyk, integrated): aide talks to the human about the system; the
  orchestrator only runs it; advisors talk and research.
- **Context for every interactive session** (br-jttf, integrated): `session.context` warnings at
  150k, 200k (plan a handover), 250k (ceiling), 300k (forced handover and restart);
  `bridle session keep <id>` overrides below 300k. Warnings go to the session and the human
  through `external:aide`.
- **Restart** (br-qe4d, integrated): `bridle session restart <id> [--handover|--fresh]`; every
  interactive session listed in `bridle status`.
- **Named advisor addresses:** `external:advisor/<name>`; named advisors share the advisor token
  and the CLI adds the name (`BRIDLE_ADVISOR_NAME`); the daemon's `named_advisor` treats it as "a
  label, not proof" (`crates/bridle-daemon/src/server.rs`).
- **Background agents** (workers, managers, the PM) already have names and rows in the `agents`
  table, kept across sessions and daemon restarts.

## The gaps

- **Interactive sessions are tracked only while running.** The registry in
  `crates/bridle-daemon/src/sessions.rs` is "in memory only": no record of a named advisor that
  isn't running, none of the ones that have existed, and a daemon restart forgets them until they
  re-register.
- **A stopped named advisor's mail is moved, not kept.** jttf's decision (2026-10-01): mail to a
  named advisor that isn't running, or unread when it ends, goes to the shared `external:advisor`
  inbox marked "(originally for advisor/<name>)" (`sessions::originally_for`; test
  `named_advisor_addressing_and_delivery_fallbacks`). The human now wants it to **stay where it
  is**. This reverses that decision.
- **No seat to retire or revive**, no "shut down" note, no history: r8kv's build task br-7810 was
  dropped.
- **Watchers** (xxxq, br-519b, open) key on the principal, and say nothing about a watcher that
  has been retired.
- **Handover notes are the orchestrator's only** (`bridle handover write/list/show`); a named
  role has nowhere of its own to keep one.
- **The aide and named roles beyond advisors** (a named PM, ...) have no shared model: each role's
  naming is its own convention (`advisor-<name>-<project>` session names, `pm-1` agent names).

## Related incidents (`docs/context/incidents.md`)

- **2026-10-03 ~15:30, the main advisor took a named advisor's brief:** the orchestrator sent a
  brief for the new "tickets" advisor to the shared `external:advisor`; the main advisor's wake
  returned it and marked it read, so the named advisor never saw it. Cause: one shared inbox.
  Its follow-up names r8kv (each seat its own inbox) and
  [[read-on-delivery-can-lose-messages-marked-read-before-the-re-k8jn|k8jn]].
- **Again on 2026-10-03 ~23:40 ET:** while this ticket was being written, the main advisor's
  wake returned and marked read the orchestrator's brief "For advisor doc-review" (m-4174, ticket
  x8jt). Same cause.
- **2026-10-01, starting advisors by hand went wrong twice:** a new advisor never read its brief
  ([[a-new-advisor-never-reads-its-brief-jb4e|jb4e]]).
- **2026-10-03 01:59, a daemon restart ended the advisor's wait as a "timeout"**: the session
  registry is in memory, so a daemon restart loses the session.

## Advisor's proposals (for the human's review; nothing here is decided)

Numbered so the human can answer "yes to 3, change 5", etc.

**P1. The word "seat"** for a named, lasting position that sessions occupy over time (Yegge's
and curia's word; r8kv's naming research). Unused in bridle today. Alternatives: post, desk.

**P2. Every role gets a seat, kept in the database** (a `seats` table, not memory):

| Field | Meaning |
|---|---|
| name | `orchestrator`, `aide`, `advisor/research`, `product-manager/pm-1`, ... |
| role | the role file it runs |
| project, machine | where it lives |
| created_at, created_by | who made it (the human, the orchestrator, ...) |
| state | `active` (a session is running), `idle` (no session), `retired` |
| retired_at, retired_by, retired_note | when retired and why |

Plus a `seat_sessions` history: session id, pid, pane, started, ended, how it ended (exit,
restart, handover, crash), last context reading. A restart or handover is a new row under the
same seat ("get the new ID for each one").

**P3. Names.** Unique roles' seat name is the role (`orchestrator`, `aide`); an optional
display name is allowed but not needed. Named roles are `<role>/<name>`, unique among seats that
aren't retired, so a retired name can be reused later only by reviving it (P6) or after it's
closed (P7). The principal is the seat name: `external:advisor/research`, as today. Background
agents (workers, managers, PM) are already named rows in `agents`; P2 adds them as seats so one
list covers everyone, but their lifecycle stays the supervisor's.

**P4. A seat outlives its sessions.**

- `bridle session advisor <name>` sits in the seat, creating it if it doesn't exist, and
  **refuses** if a session is already live in it.
- When the session exits, the seat goes `idle`, not gone. Mail and wakes for it wait **in its own
  inbox** (replacing jttf's move to the shared inbox, and fixing the 15:30 incident).
- The next session in the seat starts primed with the seat's latest handover note (P8) and its
  unread mail, then its brief, if any.

**P5. Retiring a seat** (`bridle seat retire <name> [--note "..."]`, or the session runs it
when the human says "remove yourself" on the way out):

- **Messages stay in its inbox.** New messages are still delivered there, not refused or
  forwarded; the sender's reply says "advisor/research is retired (2026-10-04: <note>); your
  message is kept in its inbox".
- **Watched tasks:** the seat stays on the watcher list, marked retired; its wakes are recorded
  but nobody is woken. (Needs xxxq's br-519b to know about seat states.)
- **Nothing is deleted.**
- **The aide reports retired seats with unread mail** to the human: "advisor/research, retired 2
  days, 3 unread". The human chooses P6 or P7.

**P6. Revive:** `bridle seat revive <name>` (or just `bridle session advisor <name>` on a
retired seat, with a confirm): back to `idle`, then a session starts primed with the seat's
handover and everything that arrived while retired, oldest first.

**P7. Close out:** `bridle seat hand-off <name> --to <seat>`: the other seat (the aide, or
another advisor) gets one message listing the retired seat's unread mail and its last handover,
and deals with anything actionable. The retired seat's mail is then marked read, with a note
naming who took it. The name becomes free for a new seat. Nothing is deleted.

**P8. Handover for every seat.** Generalize `bridle handover write/list/show` from the
orchestrator to any seat (`--seat` defaults to the caller's). `bridle session restart --handover`
and the 300k forced restart use it; the restarted session gets its own note back. Today's
`session.context` warnings key on the seat, so a restart doesn't reset the trail.

**P9. Commands:** `bridle seat list [--all]` (`--all` adds retired seats; shows state, current
session, context, unread, last active), `seat show <name>` (sessions history, handovers),
`seat retire`, `seat revive`, `seat hand-off --to`, and later `seat rename` (Yegge: a rename
keeps the history). `bridle status` lists active and idle seats; retired seats with unread mail
show as one line for the aide.

**P10. Order of work:**

1. The `seats` and `seat_sessions` tables; the in-memory session registry reads and writes them
   (fixes the daemon-restart loss).
2. Mail to an idle or retired seat stays in its inbox (drops the shared-inbox move); the
   priming on start (P4).
3. Retire, revive and hand-off (P5–P7), and the aide's report.
4. Handover per seat (P8).
5. Watchers aware of seat state (with xxxq, br-519b).
6. Background agents as seats in one list (P3), last.

Background running (r9vh) after this, as the human asked.

## Open for the human

- **Q1. The advisor's new name.** The human: "Advisor, which is probably going to need a
  rename". r8kv's options for the design and research role: keep **advisor**, or
  **consultant**, researcher, designer.
- **Q2. Reversing jttf:** mail to a named advisor that isn't running stays in its own inbox
  (P4, P5) instead of going to the shared `external:advisor` inbox. Confirm.
- **Q3. Is the unnamed main advisor still needed?** The human expects one aide per project to
  replace "an advisor per project"; if so, the unnamed `external:advisor` becomes just another
  named seat, and the shared inbox goes away.
- **Q4. Do named roles also need their own tokens**, so the name is proof rather than a label
  ([[how-strong-agent-provenance-should-be-2bzw|2bzw]])? The advisor's view: not for this build.
