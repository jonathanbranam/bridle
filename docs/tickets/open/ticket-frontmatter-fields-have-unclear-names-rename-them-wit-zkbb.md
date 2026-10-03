---
id: zkbb
title: "Ticket frontmatter fields have unclear names: rename them, with a migration"
kind: question
opened: 2026-10-03
repos: [bridle]
changes: []
specs: []
needs: [project-migrations-one-command-applies-pending-bridle-upgrad-xebc]
see: [tickets-and-tasks-why-both-k7tm]
tasks: []
---

## The ask

The human, verbatim (2026-10-03, via advisor (tickets), during k7tm):

> But I do want to add one more thing, which is that the way we set up tickets to begin with seems
> to not be the fields we have in the front matter seem to not be clear to the agent when writing
> the tickets. So I want to revisit the names that we've chosen, and then we're going to have a
> migration that we'll have to run to update them at some point. That's not urgent, but we could
> start using the new names possibly soon and then get the migration going. There's a ticket for
> building migrations and shipping them with the app. Binary and running them. I don't know if
> that's been built or not, but you could check on that and then create a separate ticket, I guess,
> for revisiting and renaming the fields on tickets and then make it dependent on having those that
> migration work ship. Actually, yeah, it's pretty interesting. I just said you should make a ticket
> dependent on a task, I think. Which is kind of weird, but you could just write it in the text
> probably and, and connect the two tickets.

## Today (advisor, 2026-10-03, at 232 tickets)

The fields came from `workflow-instructions/ticket-conventions.md`, written for an OpenSpec
workflow: `changes:` holds OpenSpec change slugs ("producer first"), `specs:` holds
"capability/requirement anchors the decision landed in". Bridle has no OpenSpec changes, so agents
guess:

| Field | Meant | What agents write |
|---|---|---|
| `repos` | every repo the ticket touches | `[bridle]` on 238; fine |
| `changes` | OpenSpec change slugs, in dependency order | empty on 217; **commit SHAs** on 32 |
| `specs` | where the decision landed (spec anchors) | empty on 242; design-doc paths on 7 |
| `needs` | tickets that must land first | empty on 241; a mix of bare IDs and full stems |
| `see` | related, duplicate, supersedes | bare IDs (`[cvaq]`) on many, full stems on others, though the docs say link by full stem |
| `opened`, `tasks`, `kind`, `title`, `id` | as named | fine |

## To decide

1. New names (or drops) for the unclear fields. Candidates to start from, not decided:
   `changes` -> `commits` (what agents already put there) or drop it; `specs` -> `recorded_in`
   (the design docs the answer landed in, which `ticket resolve`'s Resolution section names);
   `needs` -> `depends_on`; `see` -> `related`. Settle bare ID vs full stem in the list fields too.
2. Whether k7tm (tickets and tasks, maybe merged) changes the field set first; settle names after
   or alongside it.
3. Rollout: `bridle ticket new`/`set`/`check` write and accept the new names first (old names
   still read), then a migration renames them in every project.

## Depends on migrations shipping

Needs [[project-migrations-one-command-applies-pending-bridle-upgrad-xebc|project migrations]]
(xebc). As of 2026-10-03 the manual `bridle migrate` is built (br-e25d, integrated); running
migrations automatically at `bridle serve` start-up is task **br-2718** (planned, parked for the
human's review on Sat 10-03). The rename migration should wait for br-2718 to land so it reaches
every project without a hand-run step. The ticket-kind backfill (br-e7e2, v3dk slice B) is the
first migration in line; this would follow it.
