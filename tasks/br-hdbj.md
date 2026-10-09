+++
id = "br-hdbj"
title = "Only the owner's clone can push the integration branch: enforced, not a rule (build)"
kind = "feature"
state = "planned"
created_at = "2026-10-09T23:09:30.765Z"
updated_at = "2026-10-09T23:09:35.418948Z"
created_by = "external:orchestrator"
watchers = [
    "external:orchestrator",
    "external:advisor/product-manager",
]
parent = "br-8z7j"
+++

Ticket: docs/tickets/open/only-the-owner-s-clone-can-push-the-integration-branch-enfor-8z7j.md. Build Option A exactly as its "Design options" section describes (approved by the human 2026-10-09, with Q2 answered: integration branch only, not the release branch or tags). Read the whole ticket, including "What exists today", "Touches" and the rejected alternatives; do not restate them here. Model: Sonnet.

Build:
1. A marked `pre-push` hook installed in each clone bridle manages, next to the existing installer pattern in crates/bridle/src/tools_only.rs (`bridle machine tools-only-install`). Git has ONE pre-push: the existing tools-only pre-push and this owner check must chain or share one marked script, and each install stays idempotent and marker-guarded. Resolve the hooks dir with `git rev-parse --git-path hooks`.
2. A hidden helper (e.g. `bridle machine push-check`): reads ref lines from the hook's stdin; for a push to `refs/heads/<integration>` it compares this machine's name (the one `serve` writes) with `owner.toml`, preferring a fresh fetch of `origin/bridle/state`, falling back to the local file when origin is unreachable; no owner file, no state worktree or a read error means refuse (fail closed). Other branches and tags pass untouched. The refusal text is the ticket's: names the project, the owner and the date, and "To move it here: bridle serve --take-over".
3. Install call from `serve` and `bridle sync`/workspace setup (the ticket's "Touches"). Refuse or warn, and do not write, if `.git/hooks/pre-push` (or the resolved hooks path) is a symlink (the human's shared-dotfiles hooks wrote through symlinks into every repo at once; those are now removed, but never write through one). Never clobber a foreign hook (existing behaviour).
4. `claim_owner` in crates/bridle-daemon/src/state_branch.rs only queues `owner.toml` for the next flush: take-over must flush before it returns.
5. Agent deny entries, locked against project override like release's: `Bash(git push --no-verify*)`, `Bash(git push * --no-verify*)`, `Bash(git config *hooksPath*)` (config::apply_branches in crates/bridle-daemon/src/config.rs).
6. The rule "one pusher for the integration branch" (new file in workflow/base/rules/, match the neighbouring rule files' format), operating-model.md Merging step 4, docs/design/cli.md, docs/context/add-a-machine.md (one line, if hua2 landed; else a note on the thread), CHANGELOG.md.
Note for the daemon-side pushers (br-8umh push command and br-8ay6 docs push, if landed): they run on the owner's clone, so the hook passes; a rejection by this hook must be reported like any failed push.
Tests: temp bare origin plus two clones with a fake owner.toml: owner pushes the integration branch, non-owner is refused, a feature branch from the non-owner passes, missing owner file refuses, symlinked hook is not written through, chaining with the tools-only hook, take-over flushes owner.toml. No network.
Acceptance: just check passes. Migration: automatic: `serve` and `bridle sync` install the hook in existing projects' clones on the next run (idempotent); say so in CHANGELOG. This only touches clones of projects bridle already runs and respects the existing-projects rule only on the trial branches, so state in the task comment that it installs hooks only and never edits branches or settings.
Out of scope: Option C credentials (deferred until a bypass happens or xccp), the release branch and tags, a dead push URL, clones bridle never touched.

## Thread

### note · external:orchestrator · 2026-10-09T23:09:30.855Z
Orchestrator: carries br-8z7j's build. br-8z7j was 'reopened' after its design branch landed, and a reopened non-incident task can't be readied or planned (tasks.rs plan_task). The hold was released by advisor/product-manager 2026-10-09 23:08Z; Option A approved by the human (see br-8z7j's thread and the ticket).

### note · external:orchestrator · 2026-10-09T23:09:30.954Z
From orchestrator: br-8z7j can't leave 'reopened' (only incidents can be planned from it), so its build is now br-hdbj (same brief, open). Plan and queue it as you said.
