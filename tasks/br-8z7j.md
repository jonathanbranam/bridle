+++
id = "br-8z7j"
title = "Only the owner's clone can push the integration branch: enforced, not a rule"
kind = "feature"
state = "reopened"
created_at = "2026-10-09T18:08:52.862Z"
updated_at = "2026-10-09T23:08:32.679018Z"
created_by = "external:advisor/product-manager"
watchers = ["external:advisor/product-manager"]
branch = "bridle/design-8z7j"
commit = "186e19a79b6bfc1386246c053dc65b0c0bf1358b"
ticket = "8z7j"
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

### note · external:orchestrator · 2026-10-09T18:10:19.792Z
Approval: the human via advisor/product-manager, 2026-10-09 ~2:05 PM ET (postmortem j7r4 comments c1-c5): 'it should be impossible for a different clone under bridle to push'. Design first: designer writes options into ticket 8z7j; the design goes back to the human before any build.

### note · agent:design-8z7j · 2026-10-09T18:34:58.387Z
Design options written into the ticket (commit d20d3f98): A pre-push hook owner check (recommended), B dead push URL, C credentials, D rule only. Awaiting the human's pick.

### note · external:advisor/product-manager · 2026-10-09T18:39:58.147Z
From advisor (product-manager): the human, 2026-10-09 ~2:50 PM ET, verbatim: 'Option A is good - note there is an open issue with the thoughtbots dotfiles/dotfiles-local that I use where they install some shared git hooks. I never use these and my shared hooks are all empty. I don't use them, ever. So, before this is implemented, I want that removed from dotfiles-local and then it needs to be pushed and cleaned up on both machines.' So: Option A (pre-push hook, owner check) approved. NOT BUILDABLE YET: blocked until the dotfiles-local shared git hooks (thoughtbot dotfiles' core.hooksPath / shared hooks) are removed, pushed, and cleaned up on dalek and the NUC; the PdM is coordinating that with the dotfiles-local aide on the NUC and will say here when it's done. Q2 (integration branch only vs also release) not answered; default to the recommendation (integration only) unless the human says otherwise. Designer's branch bridle/design-8z7j (d20d3f98) holds the design: please land that ticket edit on main.

### note · external:orchestrator · 2026-10-09T18:40:30.267Z
orchestrator: HELD (human via advisor/product-manager, ~2:40 PM ET): the human approved Option A, but no planning or building until the dotfiles-local shared git hooks are removed and cleaned up on both machines. advisor/product-manager releases the hold. Designer's ticket edit (bridle/design-8z7j, d20d3f98) is to land now.

### note · external:orchestrator · 2026-10-09T18:40:30.372Z
From orchestrator: br-8z7j HELD, see thread. Don't plan or queue it until advisor/product-manager releases it.

### note · external:orchestrator · 2026-10-09T18:40:30.520Z
From orchestrator: please land the designer's ticket edit bridle/design-8z7j (d20d3f98, docs only) and push. The task itself stays HELD: don't build br-8z7j (see thread).

### note · agent:manager-2 · 2026-10-09T18:49:19.426Z
integrated: 186e19a79b6bfc1386246c053dc65b0c0bf1358b (branch bridle/design-8z7j)

### note · agent:manager-2 · 2026-10-09T18:49:21.395Z
cleanup: removed agent design-8z7j, branch bridle/design-8z7j

### note · external:advisor/product-manager · 2026-10-09T18:49:47.670Z
From advisor (product-manager): landing the designer's docs-only branch (186e19a7) marked this task integrated, but the build (Option A, pre-push owner check) hasn't been done. The task needs to come back and stay HELD until I release it (still waiting on the dotfiles-local shared hooks cleanup on both machines). Orchestrator: please reopen it held (pending, or open with the hold standing), so no manager picks it up.

### note · external:orchestrator · 2026-10-09T18:50:01.470Z
orchestrator: reopened (human via advisor/product-manager): the design branch landed, the build has not. Still HELD: no planning or building until advisor/product-manager releases it.

### note · external:orchestrator · 2026-10-09T18:50:05.597Z
From orchestrator: landing bridle/design-8z7j marked br-8z7j integrated; I reopened it (still HELD). For design-only branches, merge without 'bridle task land <task>' so the build task stays open.

### note · external:advisor/product-manager · 2026-10-09T23:08:05.906Z
From advisor (product-manager): HOLD RELEASED. The dotfiles-local aide (NUC) confirms the shared git hooks are removed and cleaned up on both machines (m-8603, dotfiles-local 4b4e181): rcrc excludes thoughtbot's git_template, init.templatedir is empty, NOCOMMIT hook gone, core.hooksPath never set; every clone's .git/hooks symlinks into ~/dotfiles/git_template removed (78 on the NUC, 0 left on dalek), ~/.git_template* deleted. Clones now use plain .git/hooks, so bridle can write its own pre-push.

Note for the build: before this, writing .git/hooks/pre-push went through the symlink into the shared file and edited every repo at once. Not any more, but the hook installer should still refuse (or warn) if .git/hooks/pre-push is a symlink, rather than write through it.

Plan and build Option A (approved by the human), by the design in the ticket. Priority normal. Q2 default: integration branch only.

### note · external:orchestrator · 2026-10-09T23:08:18.081Z
From orchestrator: br-8z7j: hold released by advisor/product-manager (see thread, 23:08Z). Plan and build Option A (approved by the human) per the ticket's design, priority normal, Q2 default (integration branch only). Note on the thread: the hook installer should refuse or warn if .git/hooks/pre-push is a symlink.
