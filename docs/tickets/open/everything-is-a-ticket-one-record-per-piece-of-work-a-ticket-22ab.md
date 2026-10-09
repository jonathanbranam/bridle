---
id: 22ab
title: "Everything is a ticket: one record per piece of work, a ticket is at most one change, and the task becomes only its tracking row"
kind: arch-revision
opened: 2026-10-09
filed_by: external:advisor/tickets
repos: [bridle]
changes: []
specs: []
needs: []
see: [k7tm, zkbb, 95mu, stx8, p2ys, hvxk, nkd9, v3dk, kzw2]
tasks: []
---

## The ask

Make the ticket the one record of every piece of work: the ask, the design, the discussion, the
reviews and the approvals. The task stops being a second record and becomes only the row that
tracks a ticket through the workflow. A ticket too big for one change gets child tickets, each
one change. Rename the ticket fields to standard tracker names (`see` and `needs` first). Then
rename "task" to "ticket" everywhere: user-facing first, then the code. The human's words are
under "The human's words" at the end.

This is a design ticket: decisions first, then the full design, then a suggested sequence of
tickets. It's a new workstream for advisor (product-manager) to plan. Nothing here gets built
until the human approves the plan.

History: [[tickets-and-tasks-why-both-k7tm|k7tm]] (why both, the decisions of 2026-10-03),
[[ticket-frontmatter-fields-have-unclear-names-rename-them-wit-zkbb|zkbb]] (field names; this
ticket settles it), [[a-change-spec-proposal-and-design-reviewed-for-risk-and-impa-95mu|95mu]]
(the change spec and its gate; this ticket answers "where it lives"),
[[a-task-s-state-says-what-s-really-happening-held-and-built-a-stx8|stx8]] (states; held for
the human's approval), [[ticket-state-without-moving-files-p2ys|p2ys]],
[[refining-a-task-with-the-human-before-it-ships-hvxk|hvxk]],
[[ids-the-human-can-say-aloud-task-and-ticket-ids-that-survive-nkd9|nkd9]],
[[tickets-get-a-kind-kinds-are-editable-tickets-and-tasks-link-v3dk|v3dk]],
[[task-data-the-system-acts-on-is-structured-fields-not-text-t-kzw2|kzw2]].

## Why

- "Ticket" and "task" are near-synonyms, and the boundary between them is blurred
  (`docs/context/naming.md`, "Same name, blurred boundary"). The human and the agents both mix
  them up.
- A ticket that spawns several tasks has nowhere to put each task's design, so the designs end up
  mixed together in one ticket.
- Discussion is split: review comments are on the ticket file, while the work thread is on the
  task. Nobody can find the whole story in one place.
- Two IDs per piece of work (`k7tm` and `br-k7tm`, or an unrelated hex ID) are confusing to
  read, say and look up.
- There's no gate on design before build (95mu). Incident 2ax5 shows the cost. A change ticket
  with required parts gives the gate something to check.
- KISS: one noun and one record.

## Decisions (the human, 2026-10-09)

1. **Everything is a ticket.** All work has a ticket, including critical fixes, incidents and
   small chores. The ticket holds the ask, the design parts, the thread, reviews and approvals.
2. **A ticket is at most one change.** If an ask is bigger than one change, it becomes a parent
   ticket (a bug report, feedback, a large design) with child tickets, each one well-scoped
   change.
3. **The task is only a tracking row.** No body, no thread of its own. It follows the ticket
   through the workflow (state, claim, priority). It's an internal thing; the human never needs
   to think about it.
4. **The thread lives on the state branch, keyed by the ticket ID, and is shown with the ticket**
   (option A). The human, verbatim: "I don't _prefer_ the split, but it's an ok compromise for
   now and is less change/churn."
5. **Commits for tickets are an acceptable cost.** Every piece of work needs a ticket file on
   `main` first.
6. **Rename the ticket fields to standard tracker names**, starting with `see` and `needs`, by
   `docs/context/naming.md`.
7. **User-facing names drop "task" and use "ticket".** The UI is a later issue, but change
   tickets must be shown differently from other tickets in the UI.
8. **The code is renamed too**, so code names match what users see: "agents will be confused (as
   would a human) when the code names don't match the external names". This can be a second
   pass, but the workstream isn't complete until it's done.
9. **Migration is fine**, including stub tickets for work that has none. Purging old tickets
   from `resolved/` is fine.
10. **The workstream is managed by advisor (product-manager).**

## Design

### 1. The model

| Thing | What it is | Where it lives |
|---|---|---|
| Ticket | the one record of a piece of work or an issue: ask, parts, links | `docs/tickets/{open,resolved}/` on `main` (the project's integration branch) |
| Thread | comments, questions and answers, notes, reviews, approvals, the landing summary | state branch, `tickets/<id>.md` |
| Tracking row | state, claim, priority, size, watchers, components, impact | SQLite index, flushed to the state branch with the thread |

One ID names all three. In code, the "task" becomes the ticket's tracking record (section 9).

**Rule of thumb for which side a field goes on**: what an author writes and reviews goes in the
file (title, type, links, the parts). What the system changes as work moves goes in the tracking
row (state, claim, priority, watchers). That keeps the many small automatic updates off `main`,
and keeps everything the human edits in vim or Obsidian in one file.

### 2. Every ticket is indexed; its type decides its workflow

The human described a row "for certain ticket types that are going to be implemented". The
advisor recommended a small extension, and the human accepted it (2026-10-09: "row for every
ticket is fine"): **every ticket gets a row**, and the type decides what the row can do:

- Every ticket needs a thread (any ticket can be discussed), and the thread needs a record to
  hang on. A row per ticket also gives the UI and `bridle ticket list`/`search` one index for
  everything.
- Research, explore and question tickets are worked by agents too, not only changes. They need
  claims and states, and they aren't code changes.
- Rows are cheap and rebuilt from the files, so a row for a discussion-only ticket costs
  nothing.

Each **type** declares, in workflow config, its states, the parts it must have, and the
approvals it needs (sections 5 and 6). Two groups:

- **Change types** produce a branch that merges. They need parts, pass gates, and the UI shows
  them differently (decision 7). Proposed: `feature`, `fix`, `docs`, `chore`, `arch-revision`.
- **Other types** have a simpler workflow. `bug` (a report), `epic` (a large initiative split into
  children), `question` and `incident` only open and close. `research` and `explore` can be
  claimed and worked, and end in findings, not a merge. Then `re-evaluate` and `postmortem`
  (cr7t).

New or changed types: `fix` (the change that fixes a `bug` report; today's `bug` tasks that are
fixes migrate to `fix`) and `epic`. **Name check (naming.md):** the human said "large initiative
design". "Epic" is the standard tracker word (Jira, GitHub, Linear) for "a big ticket split into
child tickets"; "initiative" in Jira is the level above an epic. Both dictate cleanly.
**Decided: `epic`** (the human, 2026-10-09).

### 3. Readiness and who may start work

This replaces k7tm's "a ticket without a task gets no work" with the same rule in the new
model:

- A new ticket's row starts in the first state (today's `pending`, "not ready"). Nothing works
  it.
- Marking it ready (today's `bridle task ready`) is the intentional step. It's taken only with
  the human's approval, in one of their conversations with the orchestrator or an advisor. The
  project manager schedules only ready tickets.
- Unchanged: for a critical issue, the orchestrator files a `fix` ticket, marks it ready and
  plans it itself.
- `bridle ticket task` and `ticket new --no-task` go away: there's nothing to "file a task" for.
  The row exists from `ticket new`. The check that the ticket file is committed (k7tm's "the
  task races the ticket body" fix) moves to "mark ready". It refuses a ticket with an empty
  `## The ask`, or one not committed on the local integration branch.

**Push (the human asked, "it doesn't have to be pushed, does it?")**: no. Workers branch from the
local integration branch, so a local commit is enough on the same machine. Only a daemon on
another machine needs the push, as it does today for everything else.

### 4. The ticket on disk: one file

**Decided (the human, 2026-10-09): start with one file.** A ticket is one file,
`docs/tickets/{open,resolved}/<slug>-<id>.md`, at today's path, so every `[[stem|text]]` link and
Obsidian lookup keeps working. A change ticket's parts (section 5) are in the same file.

How the parts are separated in the file is an implementation decision, and it doesn't block this
work. The human's proposal is MIME parts: each part a separately delimited section of the one
file, so the UI can show the parts as tabs (or as a folder), while vim and Obsidian show one file.
The simplest fallback is headings (`## Proposal`, `## Design`, ...). Either way:

- The gate (section 6) finds each required part by its name and checks that it isn't empty.
- Non-change tickets have no parts beyond `## The ask` and the discussion.
- `ticket resolve` moves one file, as today.

The human's earlier words on layout: "big tickets only exist on disk; in the UI they can be
rendered as a folder; they will show as a single file in a local editor, however; this is
something to consider; tickets that aren't changes might not need folders, though."

### 5. Change parts (modeled on OpenSpec)

| Part | Holds |
|---|---|
| proposal | **Why** (bullets, like this ticket's), **What changes**, **What doesn't**, **Impact** (users, other projects, running daemons, stored data) |
| design | exact names (checked against naming.md), states, wire and CLI changes, storage and migration, rollout and rollback, risks |
| specs | the spec requirements added, modified or removed (IDs from `design/specs/`), with the new or changed SHALL text. Specs are edited in place on the build branch (`docs/design/specs.md`); this part is the declared impact (today's `task impact set`) |
| tasks | the build checklist the worker follows and ticks on its branch; lands with the merge |

The proposal, design and review prompts are part of this workstream (as 95mu asks).

### 6. Gates and approvals per type

Configured in the workflow, not in role prompts. A sketch, not final syntax:

```toml
[ticket.types.feature]
change = true
parts = ["proposal", "design", "specs", "tasks"]
approve = ["human"]          # who signs off the parts before planning

[ticket.types.fix]
change = true
parts = ["proposal", "tasks"]
approve = ["reviewer"]       # an agent reviewer; critical fixes may skip

[ticket.types.question]
change = false
parts = []
approve = []
```

- An **approval** is a structured thread entry (`kind = approval`, who, when, and the commit of
  the ticket and its parts it approves; kzw2: structured, not text). Changing a part after
  approval invalidates the approval.
- The daemon refuses to plan a change ticket until every required part is present (a non-empty
  file or section) and every required approval stands. This is the real gate 95mu asks for.
- `bridle ticket approve <id>` records an approval. The UI gets an approve button.

### 7. Links: the field renames

Links live only in the frontmatter, the one source of truth. The daemon indexes them into the
existing `edges` table when it reads tickets, so readiness (`blocked_by`) keeps working.
`bridle ticket link`/`set` edit the file (a commit), replacing `task dep add`. No two-way lists:
children and "blocks" are computed by querying, never stored on both sides (today's
`tasks:`/`ticket` pair is the drift k7tm warns about).

Decided by the human (2026-10-09) except `kind`, which no one asked to change, so it stays.

| Today | New | Why |
|---|---|---|
| `needs` | `blocked_by` | the trackers' name: Jira "is blocked by", Linear "Blocked by", GitHub Issues "Blocked by" (API `blocked_by`); pairs with our `blocks` edge. `depends`/`depends_on` (beads' field, Compose, Terraform) were weighed: developer-tool names, not tracker names |
| `see` | `related` | the standard name, and already our edge kind |
| (none) | `parent` | the standard name; already our edge kind; one ID |
| `tasks` | (dropped) | one row per ticket, same ID |
| `changes` | (dropped) | agents put commit SHAs there; the landing commit goes in the thread |
| `specs` | (dropped) | replaced by the `specs` part; a resolved ticket's "where the answer landed" stays in `## Resolution` |
| `opened` | `created` | the standard name |
| `filed_by` | `created_by` | matches the row's `created_by` |
| `closed` | `resolved` | matches the `resolved/` folder and `ticket resolve` |
| `kind` | `kind` | unchanged: clear, and used throughout the code (task, message and edge kinds) |
| `repos` | `repos` | fine |

`duplicates` and `supersedes` stay edge kinds, used from the CLI; they're rare enough not to
need fields (YAGNI). Link values are bare IDs (`k7tm`), resolved by the tooling. That settles
zkbb's "bare ID vs full stem".

> [!comment] c1 human, 2026-10-09 14:56 EDT, on "That settles zkbb's "bare ID vs full stem"." [pending 2026-10-09 14:56 EDT]
> I'm pretty sure we need to use the project id here. We should be clear on the project to avoid confusion on ticket ids. It is fine to use shorthand when working on a specific project, but we want ticket names to be unique globally, as much as possible. br-<id> not <id>.

### 8. IDs

- One ID per ticket, and the row has the same ID. No `br-` prefix shown to the human inside a
  project.
- Across projects, the prefix stays as the qualified form (`br-k7tm`, `tw-k7tm`), and the CLI
  accepts both.
- Old hex task IDs that can't be ticket IDs (they contain `0` or `1`) get a fresh ticket ID. The
  old ID stays as an alias.
- nkd9 (speakable IDs) may change the alphabet; this design doesn't depend on it.

### 9. The rename ("task" -> "ticket")

**User-facing (first pass, required):**
- CLI: `bridle task <x>` becomes `bridle ticket <x>` (`comment`, `ask`, `answer`, `plan`,
  `ready`, `drop`, `done`, `claim`, `list`, `search`, `priority`, ...). `bridle task` stays as a
  hidden alias for one release, then goes.
- Roles, rules, skills (`bridle-worker`: "claim a task" becomes "claim a ticket"), `docs/`,
  messages and event names shown to people.
- Gateway API and UI: change tickets look different from the rest (state, parts, gates and
  approvals visible). The UI work is in the separate bridle-ui project; the gateway exposes what
  it needs.

**Code (second pass, required before the workstream is complete):**
- `TaskManager`, `Task`, `TaskState`, `TaskKind` (wire types in `bridle-api/src/types.rs`), the
  `tasks` table, and the state branch's `tasks/` folder.
- Wire changes go to all clients and the daemon together, and stay compatible for one release
  (accept old and new names). This is the lesson of incident 2ax5 (a breaking wire change broke
  cross-project messaging). bridle-ui reads the gateway, so the rename must be coordinated with
  that repo.
- The tracking row needs a name in code that isn't "task". The two candidates are `TicketState`
  for the row (state, claim, priority...) next to `Ticket` for the file, or one `Ticket` struct
  that holds both. Leave that to the code design, within one rule: no type is called `Task`
  afterwards.

### 10. States

**Decided (the human, 2026-10-09): merging task into ticket uses the existing states.** The row
keeps today's states (`pending`, `open`, `planned`, `claimed`, `integrated`, `dropped`,
`reopened`). New states are not part of this workstream.

stx8 is reviewed separately, later. The human: "I think we need more than just these states",
meaning more than the held and built-and-waiting-to-land states stx8 proposes. stx8 stays held
until the human approves its state design. When it lands, it designs the states for change
tickets, and other types use a subset (section 2).

### 11. Migration

Run by `bridle migrate` (xebc; auto-run at start-up is built, br-2718), per project:

1. Rename the frontmatter fields (section 7). The tools read old and new names until the
   migration has run everywhere.
2. Move each task's thread to `tickets/<id>.md` on the state branch, and drop its body (the
   ticket path).
3. Tasks without a ticket (about 40 active today: 19 planned, 11 claimed, 10 pending) each get a
   stub ticket made from the task's title and body (`ticket new --from-task` exists). Finished
   ticketless tasks (about 420) stay as read-only history and get no tickets.
4. Tickets with more than one task (11 today): each extra task becomes a child ticket with
   `parent` set.
5. `bug` tasks that are fixes become `fix`.
6. Purging `resolved/`: **deferred** until it's a problem (the human, 2026-10-09: "defer a
   decision until we have a problem. YAGNI, maybe.").

**Other projects** (track-web, meta-notes, ...): their migrations commit stub tickets and field
renames. By rule `existing-projects`, those go to the project's trial or integration branch
(rxe8), never its `main`.

**`ticket submit` (visitors, no repo access)**: a submission is a ticket with no file yet. Its
ask lives on the state branch until the project manager accepts it and commits the file.
Dropping it tells the submitter, as today.

### 12. What doesn't change

- The quiet period before work starts, claims and leases, worktrees, the merge model, and the
  orchestrator's right to fast-track critical fixes.
- Handovers, messages and schedules.
- `docs/design/` stays the durable record: a resolved ticket's `## Resolution` names where the
  answer landed.

## Suggested sequence of tickets

Each is one change ticket under this one as its `parent` (the first use of the new model).
Advisor (product-manager) sizes, orders and plans them, with the human's approval.

1. **Field names** (absorbs zkbb): `ticket new/set/check` write the new names and read both;
   `parent` added; the migration renames the fields everywhere. Needs the human's answer on
   section 7's table first.
2. **Ticket types** (`fix`, `epic`, the change flag), and readiness moved to the ticket
   (section 3): `ticket new` makes the row; `ticket ready` replaces `ticket task`.
3. **Links from frontmatter**: the daemon indexes `blocked_by`/`parent`/`related` into `edges`;
   `ticket link` replaces `task dep`.
4. **The thread moves to the ticket**: `tickets/<id>.md` on the state branch, `ticket
   comment/ask/answer/show`, and migration steps 2-4.
5. ~~States (stx8)~~: **out of this workstream** (the human, 2026-10-09). The merge uses the
   existing states; stx8 is reviewed separately, later (section 10).
6. **Change parts, gate and approvals** (with 95mu): the layout (section 4), the per-type
   config (section 6), `ticket approve`, the daemon refusing to plan, and the proposal, design
   and review prompts.
7. **User-facing rename**: CLI with a hidden `task` alias, roles, rules, skills, docs; the
   gateway API exposes change tickets for the UI.
8. **UI** (bridle-ui project): change tickets shown with parts, state, gates and approvals.
9. **Code rename**: types, storage, the state branch folder, wire compatibility for one release.
10. **Cleanup**: remove the `task` alias and the old-name readers (purging `resolved/` is deferred),
    and a final pass over the docs (`docs/README.md`, `cli.md`, `storage.md`, `agent-host/`).
    The workstream is complete only after this one.

## Plan (advisor product-manager, 2026-10-09; Q1-Q5 answered, waits on the human's approval)

**Priority.** Second to machine setup: it starts as its gates clear and doesn't take worker slots
from machine setup phase 1. The roadmap tracks it as the workstream "Everything is a ticket".

**What gates each step** (the suggested sequence above, numbered the same):

| Step | Size (guess) | After | Gated by the human |
|---|---|---|---|
| 1 Field names (zkbb) | M | - | - (Q1 answered) |
| 2 Ticket types, readiness on the ticket | M | 1 | - (Q2 epic, Q4 a row per ticket) |
| 3 Links from frontmatter | M | 2 | - |
| 4 The thread moves to the ticket | L | 3 | - |
| 5 States (stx8) | - | - | Out of this workstream (answer 6): the merge uses the existing states |
| 6 Change parts, gates, approvals (95mu) | L | 2 | 95mu review (Q3: one file, MIME parts) |
| 7 User-facing rename | L | 1-4 | - |
| 8 UI (bridle-ui) | M | 7's gateway API | - |
| 9 Code rename | L | 7 | - (run at a quiet time: it touches every crate) |
| 10 Cleanup (drop the `task` alias) | S | 9, one release later | - (Q5: purge deferred) |

**Order.** 1 to 4 one at a time: each changes the ticket tooling the next builds on, and running
them side by side would conflict. 6 can run beside 3 and 4 once Q3 is answered. 7 waits until the
names and commands of 1 to 4 are stable, so the docs and roles are renamed once. 9 waits for 7 and
runs while little else is in flight. 10 waits one release after 9, for the wire compatibility.

**Answer together.** Q2 (epic or initiative) was the same question as research ticket d9wq
(workstream or initiative): the human's answer, `epic`, settles both.

**Filing.** Once the human approves this plan, the PdM files steps 1 to 4 as child tickets now
(each with this ticket as parent; until step 1 adds `parent`, a "Parent: 22ab" line and `see`),
and later steps when their gates clear, so no brief goes stale waiting. Each child gets its task
from `bridle ticket task` and is readied when it's buildable.

**Risks.** The rename changes what every agent types daily; the hidden `task` alias (step 7)
keeps old habits working until step 10. The PdM's own roadmap generator reads `bridle task list`
and needs the same change.

## Answered by the human (2026-10-09)

1. Field names (section 7): `needs` -> `blocked_by`, `see` -> `related`, a new `parent`, `opened` ->
   `created`, `closed` -> `resolved`. `kind` stays.
2. `epic` (section 2).
3. One file to start, with the parts separated inside it (MIME parts is the human's proposal; an
   implementation decision, not blocking) (section 4).
4. A row for every ticket (section 2).
5. Purging `resolved/`: deferred until it's a problem (section 11).
6. Not asked, but decided: the merge uses the existing states. stx8 needs more states than it
   lists and is reviewed separately, later (section 10). Step 5 of the sequence is out of this
   workstream.

## The human's words

2026-10-09, to advisor (tickets), verbatim:

> I want to talk about how our task and ticket workflow and practices are handled:
>
> > I'm still slightly unsure about where designs live because a ticket can spawn multipe tasks with different designs... don't change precedent now, but it feels like a ticket is a filed issue and the task is a "change" which, to me means that the ticket is the user stories / bug report / user feedback, and the task is a corrolary to an openspec change - every task should be a well-scoped change with a proposal, design, and updated specs. Possibly including tasks.

Then, after the advisor found that a task's body and thread live on the state branch:

> where does the task thread live? Where is the body text of a task stored? I thought tasks lived on disk somewhere and only had a small row in a db. agree, though, that tasks would need to be reviewable and commentable.
>
> So, the alternative is that we leave a task the way it is. (but it needs less information on it!!) and we say "a ticket is a change" and then "If a ticket is too big for one change, make a 'child' ticket that is a single change" Then we can keep all of the proposal, design, specs, and tasks on a ticket. What about that? We can have a ticket "bug report" or "large initiative design" and then we spawn many smaller tickets from that of type "feature" or "bug-fix" or "docs-update", etc.
>
> I'm OK with that. it feels more in line with what we've built. I think then we need to slim down the task body even further to just point at the ticket. OR - maybe we move the thread ALSO to the ticket. So the task is literally just the db row that tracks a ticket. IDK I kind of like that. Then really we can ignore the task entirely.
>
> Think about this for a while and consider our goals. tickets vs. tasks is very confusing. we want to KISS. do we really need them?
>
> If everything is a ticket, then we have one place for comments, reviews, documents with multiple parts, etc. Then, for certain ticket types that are going to be implemented, we create a db row to follow that ticket through the system. And then implementation tickets have a status and gates. E.g. the workflow / rules / schema thing says need specific artifacts like proposal, design, specs, tasks. And specific approvals and review sign off. Some need human review, some don't.
>
> I think the main impact is that all work needs a ticket, even internal critical fixes and small things. :shrug: probably just file. We may need to purge old tickets from resolved/ but that's easy enough.

Then, after the advisor laid out options (A, the thread on the state branch shown with the
ticket; B, the thread in the ticket file on `main`):

> yes - as part of this we need to rename see and needs. The names are bad and confuse agents. We need to use standard names that are typical for ticket tracking and software development. Refer to @docs/context/naming.md.
>
> I accept (A) for where the thread lives on these reasons; I don't _prefer_ the split, but it's an ok compromise for now and is less change/churn.
>
> - commit cost is acceptable tradeoff, I think; it doesn't have to be pushed, does it?
> - migration no problem
> - agree; all user-facing must drop task and adopt ticket, but the UI itself becomes an issue; we can address that later, but I do want "change tickets" to be treated differently in the UI; code renaming but must be done - agents will be confused (as would a human) when the code names don't match the external names; can be second pass, but must be completed as part of this workstream before it is considered complete
> - big tickets only exist on disk; in the UI they can be rendered as a folder; they will show as a single file in a local editor, however; this is something to consider; tickets that aren't changes might not need folders, though.
>
> Yes, so, this needs to be a new workstream managed by the advisor/product-manager.
>
> I want you to write up a new ticket that focuses on the decisions that we've made. Link to other tickets for history and keep the motivation and reasoning very short. A bullet list of why (model on openspec proposal Why?). Then explain the entire design in detail. At the end, make a suggestion on a sequence of tickets that will implement it. When that doc is written, send it to advisor/product-manager as a new workstream to plan.

Then, answering the open questions (2026-10-09, verbatim):

> My proposal for additional files is to use MIME parts to separate them within the same file; but that is an implementation decision not blocking this work.
>
> 1. blocked_by or depends? depends is more generic I feel; see -> related is good; parent is good; you're suggesting renaming opened to created and closed to resolved, is that right?
>
> Ah - we need to also review this: docs/tickets/open/a-task-s-state-says-what-s-really-happening-held-and-built-a-stx8.md ; I think we need more than just these states; although we can come back to this work later. the task -> ticket merge could be done with the existing states.
>
> 2. epic
> 3. let's start with one file and the idea of MIME parts for the moment
> 4. i'm fine with either; row for every ticket is fine
> 5. defer a decision until we have a problem. YAGNI, maybe.

The advisor confirmed `opened` -> `created` and `closed` -> `resolved`. On `depends` vs
`depends_on` vs `blocked_by`, the advisor checked beads (field `depends_on`, edge `blocks`;
`workflow/research/02-beads.md` names only the edge types) and the trackers (Jira, Linear and
GitHub Issues all say "blocked by"), and recommended `blocked_by` as the tracker name. The human
(2026-10-09, verbatim): "agree; good discussion; blocked_by".
