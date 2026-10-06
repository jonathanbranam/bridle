---
id: kzw2
title: "Task data the system acts on is structured fields, not text: the field list (for review)"
kind: feature
opened: 2026-10-06
repos: [bridle]
changes: []
specs: []
needs: []
see: [vk3y, v3dk]
tasks: []
---

## The ask

The human, verbatim (2026-10-05 ~8:50-9:05 PM ET, to the bridle-ui aide):

> Okay, yeah, I need to be checking things more clearly. That's not good. When I asked for things to be linked and exist like this, I don't mean text. I mean fields, actual fields. A task is a database object, right? It should have fields. For this kind of thing, are there other places where we're using plain text for things? We use front matter for tickets because front matter is essentially structured metadata fields that we validate.

> Are watchers some plain text thing also? That should be fields on the table.

> Okay, to clarify this more: if we're using front matter in a task for structured data, that's fine, but things should be structured data when possible. I guess it doesn't have to be. I thought tasks were stored in the SQL database, the entire task.
>
> They don't. These fields don't have to be in the SQLite database if we don't have to query on them, but they should be structured front matter. Give me a list of the fields. Just write up a ticket so I can look at it and review it. Don't schedule any tasks. Just write a ticket.

**For the human's review. No task until the human approves.**

## The rule

Anything the system or an agent acts on is a structured field: in the task file's front matter (validated, like a ticket's), and also a SQLite column only when something queries it. The body and the thread are for people to read; no code or agent decides anything by matching their text.

## How tasks are stored today

Each task is a file on the `bridle/state` branch: TOML front matter, then the body, then the thread (comments). SQLite (`.bridle/bridle.db`) holds most front-matter fields as columns for querying; some fields are only in the file (docs/design/storage.md ~290-300).

## Fields tasks already have (structured; no change)

| Field | Where |
|---|---|
| `id`, `title`, `kind`, `state`, `priority` | front matter + column |
| `created_at`, `updated_at`, `created_by` | front matter + column |
| `claimed_by`, `claimed_at` | column |
| `size`, `components`, `watchers` | front matter only (no column) |

`watchers` is a real list field; the "watching the task" thread notes are only a log. It has no column, so "which tasks am I watching?" can't be a query; add one only if something needs that.

## Proposed new fields (text today -> field)

| Proposed field | Today, as text | Who reads it |
|---|---|---|
| `ticket` (a ticket ID, or `no-ticket`) | body first line `original id: <ticket>`; also "(fx7x)" in titles, `Ticket: docs/...` lines | `ticket check`, links (vk3y, br-vk3y) |
| `done` (`at`, `by`, `summary`, `commit`) | a worker's thread comment starting `done:` | the stop check (crates/bridle/src/stop_check.rs:48), managers |
| `merged` (`commit`, `branch`, `at`) | thread note `integrated: <commit> (branch b)` (daemon tasks.rs:848) | people, release notes |
| `settle_skipped` (`by`, `at`) | thread entry starting `settle skipped by` (tasks.rs:427) | the queue's settle check |
| `when` (`at-restart`, `at-next-reboot`) | `[at restart]` / `[at next reboot]` in a to-do's title | the human, the aide |
| `model` (`haiku`, `sonnet`, `opus`) | `Model: Sonnet` line in the brief | the manager spawning the worker |
| `approval` (`by`, `at`, `via`, `quote`, `message`) | `Approval: the human, via aide (m-0292 ...): "..."` in the brief | everyone checking a task was approved |
| `depends_on` (task IDs) | "Depends on br-vk3y ... start only after it merges" in the brief, when no `task dep` edge was made | the queue |

And on messages: an incident update is recognised by the body starting `Incident <id> updated` (supervisor.rs:2110); messages already have a task field, so use that plus a message kind, not the text.

Stays text: the brief's description, goal, build steps, acceptance and out-of-scope; thread discussion.

## Also

- Every command that writes these (task done, task land, skip-settle, task new --for-human, ticket task, ...) sets the field; readers read the field.
- A migration moves today's text into the fields (the same way br-vk3y moves `original id:`), and `bridle task show`, the API and the UI show the fields.
- Columns: add one only for a field something filters on (probably `ticket`, `when`, `merged.commit`).

## Questions for the human

1. Is the field list right? Anything to add or drop (e.g. is `approval` worth structuring, or is the quote enough as text)?
2. `when` for to-dos: only the two values, or a date/time too ("Tue 10-06, at dalek" in br-x99a's title is the same kind of thing)?
