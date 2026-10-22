+++
id = "br-6gdp"
title = "Writes to a project's clone are left uncommitted: migrations, ticket links, agent-filed tickets, scratch files"
kind = "bug"
state = "built"
created_at = "2026-10-22T01:00:01.203Z"
updated_at = "2026-10-22T02:54:53.270743Z"
created_by = "external:orchestrator"
watchers = [
    "external:orchestrator",
    "agent:g6dp",
]
branch = "bridle/g6dp"
commit = "836b8986390d2807929a8da6d9c32bdadc694e9b"
summary = "The start-up run already committed its output (br-rdag); the gap was the `bridle migrate` command, which applied and left the clone dirty. It now calls the same commit_leftovers after a non-dry-run, non-`--only` run: one commit of the migrations' files plus migrations.toml/.log, via `git commit -- paths`, never the user's other changes; on a protected branch or a git failure it prints the to-do to stderr. Branch is the project's current one, with the existing guard refusing main/dev unless declared integration. `--only` (opt-in) runs still leave the commit to the owner. Already-dirty projects: the next `bridle migrate` or daemon start commits the leftover toml/log (the catch-up). A file a past run edited (dalek-home's .gitignore) is not guessed at and needs a one-off note to that orchestrator to commit it. Test: manual apply + commit_leftovers leaves only the stray file; docs in design/migrations.md; CHANGELOG added."
ticket = "6gdp"
+++

docs/tickets/open/writes-to-a-project-s-clone-are-left-uncommitted-migrations-6gdp.md

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

### note · external:orchestrator · 2026-10-22T01:00:42.755Z
orchestrator: committed and readied (bug/chore, rule ready-without-asking). Split from br-6gdp by pm-1.

### note · agent:g6dp · 2026-10-22T02:28:48.693Z
built: 836b8986390d2807929a8da6d9c32bdadc694e9b (branch bridle/g6dp)

### note · agent:g6dp · 2026-10-22T02:28:48.729Z
done: bridle migrate now commits its run's files (like start-up); just check exit 0, 1965 tests passed, run on the commit before merging main (main only added one ticket doc); tip 836b8986390d2807929a8da6d9c32bdadc694e9b

### note · agent:g6dp · 2026-10-22T02:54:53.270Z
br-6gdp: merged main (br-xvb2) into the branch, no conflicts; just check exit 0, 1965 passed; new tip 847e662c0d9956d8c6e8a1ccad2e451633c3b765, pushed
