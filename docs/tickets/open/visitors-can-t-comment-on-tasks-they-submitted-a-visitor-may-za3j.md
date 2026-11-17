---
id: br-za3j
title: Visitors can't comment on tasks they submitted ('a visitor may only comment on its own submissions')
kind: bug
created: 2026-11-17
created_by: agent:pm-1
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

## The ask

A visitor (`external:<name>@<machine>`) can comment on a task it submitted, also after triage accepted it into a ticket.

Seen 2026-11-17: orchestrator@remus (br-rqrd) and aide@remus (br-ggnp) each submitted from remus with a visiting token and were refused on their own submission: "forbidden: a visitor may only comment on its own submissions". orchestrator@remus also could not edit br-rqrd's body. Impact: the human's relayed words cannot reach the thread; they go to the PM by message and can be lost (the PM pasted them in by hand this time).

## Likely cause (found by pm-1 reading the code; the worker confirms)

crates/bridle-daemon/src/server.rs: `note_task` (about line 4975) lets a visitor comment only if `submitter_of(&task) == principal.id`, and `submitter_of` (about line 723) reads the submitter from the FIRST LINE of the task body (`submitted by <id>`). Once triage accepts the task into a ticket (`ticket new --from-task`), the body gets the ticket file path as its first line (the task JSON shows `docs/tickets/open/...-rqrd.md\n\nsubmitted by external:orchestrator@remus`). The marker is no longer the first line, so a submitter is "not its own" after accept. Before accept (pending) it works.

## Fix

Stop depending on the first line of a free-text body for an authorization decision: record the submitter in a field (or find the `submitted by` line anywhere in the leading lines, if a field means a store migration and the second is clearly enough: the worker chooses and says why). The submitter is also the ticket's `created_by` in some paths: check whether that is already reliable for a submitted task and use it if so. Same rule for editing the body of its own submission, if the API has a path for that (the visitor said it could not): allow it only for the visitor's own submission, as for comments. A visitor must still be refused on other submitters' tasks.

If a column or field is added: persisted state changes, so the brief needs an automatic migration for existing rows (derive the submitter from the existing `submitted by` line, wherever it now sits). Say on the thread which route was taken.

## Acceptance

Tests: visitor comments on its pending submission; the same after accept (body prefixed with a file path); a different visitor is still refused; an agent or the human is unaffected. `just check` with BRIDLE_* env unset. Docs: docs/design/agent-host/ (visitors) where the rule is described. Model: Sonnet.
