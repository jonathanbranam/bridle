+++
id = "br-6gdp"
title = "Writes to a project's clone are left uncommitted: migrations, ticket links, agent-filed tickets, scratch files"
kind = "bug"
state = "pending"
created_at = "2026-10-22T01:00:01.203Z"
updated_at = "2026-10-22T01:00:01.204100Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
+++

submitted by external:orchestrator

## The ask

Found 2026-10-21 by the orchestrator when the human asked why files sat uncommitted. The human
(verbatim): "Why do we have files which aren't committed? that is a red flag that something is
being lost somewhere".

Every bridle write to a project's clone should end committed (or gitignored if it is scratch).
Found uncommitted:

1. **Migrations** (dalek-home): `bridle` 0.7.5 applied 0000, 0006 and 0007 on 2026-10-17 and
   wrote `.gitignore`, `.bridle/migrations.toml` and `.bridle/migrations.log`, with no commit.
   Four days uncommitted. Other projects that ran migrations likely have the same.
2. **Ticket links** (bridle): `blocked_by` edits on dkqf, cgw3, remg, q7my, df8w, r8jf (from
   `bridle ticket link`, the PM's planning) sat uncommitted in the main clone.
3. **Agent-filed tickets** (bridle): br-dfy7 (filed by manager-2 2026-10-19, row now `dropped`)
   was never committed and is still an untracked file in `docs/tickets/open/`. br-t7uq (accepted
   by pm-1) also waited for the orchestrator to commit it: pm-1 asked "please commit and ready
   it". Nothing tells anyone a ticket file is waiting.
4. **Scratch in the clone, not ignored** (bridle): `.bridle/tmp/` (advisor tj7q's drafts),
   `.bridle/overnight-load-report.md` (2026-10-11), `.bridle/review-documents.txt`.

Fix: whatever writes these files commits them (migrations: one commit per run; ticket
new/link/set: commit, or at least a daemon check that flags uncommitted ticket files to the
orchestrator), and scratch paths go under a gitignored directory. The fixer decides the
mechanism; keep it small.

## Acceptance

- Running a migration leaves the clone clean.
- `bridle ticket link`/`set`/`new` by an agent leaves no uncommitted ticket file unnoticed.
- `.bridle/tmp/` (and wherever agents write scratch) is gitignored.
- br-dfy7's stray file: commit it as dropped or delete it (the fixer's call; note it on dfy7).

Model: Sonnet.

## Thread

### note · external:orchestrator · 2026-10-22T01:00:01.204Z
submitted by external:orchestrator
