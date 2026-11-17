+++
id = "br-rqrd"
title = "Tighten the existing settle-skip wording: only when the human explicitly says so (no new rule)"
kind = "docs"
state = "built"
created_at = "2026-11-17T12:14:25.505Z"
updated_at = "2026-11-17T14:15:21.525180Z"
created_by = "external:orchestrator@remus"
watchers = [
    "external:orchestrator@remus",
    "agent:rqrd",
]
branch = "bridle/rqrd"
commit = "08c3b199d44748b9ea4ce11aba002d967774694d"
summary = "Settle-skip wording tightened in the existing rules, no new rule: project-manager.md and the skip-settle --reason help now say the orchestrator/PM skip only when the human explicitly says to skip (a deadline or 'work it now' is not that), and in bridle's own project also for an urgent critical fix the agent determines. orchestrator.md gets one bullet pointing at the PM text; manager.md unchanged (no contradiction). Also aligned: server.rs doc comment, docs/design/cli.md and coordination.md. Test: just check exit 0, 2010 run, 6 skipped, 0 failed (ran env -i). Role text is not a prime role here (bridle prime project-manager is invalid; prime orchestrator needs dalek-home), so verified by reading the files and the help output. No migration."
ticket = "rqrd"
+++

## The ask

Edit the EXISTING settle-period text in the workflow. Do not add a new rule.

The human, 2026-11-17, via advisor/flume (relayed by orchestrator@remus), verbatim: "This should already be. This is already a bridle rule. I'm still baffled here. You don't need to add a rule, really. This is already a rule and should already be a rule. We don't need to add a new rule. Don't add a new rule. If the rule that exists doesn't work, then let's modify the existing bridle rule and update bridle."

On the substance, earlier the same morning, verbatim: "Don't skip the 5-minute cooldown. Plus, you built that in 3 minutes, so it was completely unnecessary at this point. Don't skip the cooldown unless explicitly told to skip the cooldown. Or, in the case of bridle, if there are urgent, critical fixes determined by the agent, it can skip the cooldown. Otherwise, don't skip it."

Where the text is today (workflow v0.8.0, found by the advisor; the worker confirms):
- workflow/base/roles/project-manager.md: "Don't skip it unless the human asks or it's an urgent downtime fix."
- workflow/base/roles/manager.md: "Don't try to claim it earlier."
- workflow/base/roles/orchestrator.md: nothing on settle, though the orchestrator acts as PM where there is none.
- `bridle ticket skip-settle --reason` help (crates/bridle/src/cli.rs, grep `skip-settle`): "because the human asked or for an urgent downtime fix".

Why it failed: orchestrator@remus read "unless the human asks" as satisfied by "Please ready and work the ticket. You have one hour before I get to work" and skipped settle on fl-macf (flume); the ticket landed in about 3 minutes, so the skip saved nothing.

## The change

- project-manager.md line and the skip-settle help become: skip "unless the human explicitly says to skip the settle period (a deadline or 'work it now' is not that); in bridle's own project, also for an urgent, critical fix the agent determines".
- Make it reach the orchestrator when it acts as PM: one line in orchestrator.md pointing at the PM text, or the same wording there.
- manager.md needs no change unless its line contradicts.
- Grep docs/design for other descriptions of the skip and align them.

## Acceptance

The prime text for project-manager and orchestrator (`bridle prime project-manager`, `bridle prime orchestrator`) shows the new wording; the help agrees; no new file under workflow/base/rules/; `just check` with BRIDLE_* env unset (update any snapshot that quotes the old text). Workflow text reaches projects through the existing sync, so no migration. Model: Haiku (light, mechanical).

## Thread

### note · external:orchestrator@remus · 2026-11-17T12:14:25.506Z
submitted by external:orchestrator@remus

### note · external:orchestrator · 2026-11-17T12:14:52.837Z
readied by external:orchestrator

### note · agent:rqrd · 2026-11-17T13:55:10.598Z
built: 08c3b199d44748b9ea4ce11aba002d967774694d (branch bridle/rqrd)

### note · agent:rqrd · 2026-11-17T13:55:13.359Z
done: settle-skip wording tightened in the existing rules (no new rule file): project-manager.md and skip-settle --reason help, one bullet in orchestrator.md, server doc comment, docs/design cli.md and coordination.md; manager.md unchanged. Commit 08c3b199d44748b9ea4ce11aba002d967774694d. just check exit 0, 2010 tests run, 2010 passed, 6 skipped (run with env -i). Caveats: bridle prime project-manager is not a valid role in this binary, and bridle prime orchestrator needs the dalek-home project (absent here), so I verified the role files and the skip-settle help from the worktree build instead. Branch pushed and ticket marked built.

### note · agent:rqrd · 2026-11-17T14:15:21.525Z
main moved (br-4wu6 landed); merged local main into bridle/rqrd with no conflicts. New tip a1415d59abd98ab14e38c9cc3bfed76106678b32: just check exit 0, 2011 tests run, 2011 passed, 6 skipped (env -i). Supersedes 08c3b199 as the checked commit for landing. Branch pushed.
