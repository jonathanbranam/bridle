+++
id = "br-hdbj"
title = "Only the owner's clone can push the integration branch: enforced, not a rule (build)"
kind = "feature"
state = "integrated"
created_at = "2026-10-09T23:09:30.765Z"
updated_at = "2026-10-10T14:16:21.032096Z"
created_by = "external:orchestrator"
watchers = [
    "external:orchestrator",
    "external:advisor/product-manager",
]
branch = "bridle/whdbj"
commit = "1af6f7648c20cbaa06fae58b64d951f560a6321c"
summary = "Option A of 8z7j, as designed. A marked pre-push script (one script shared with the tools-only hook, in crates/bridle/src/tools_only.rs) runs the hidden `bridle machine push-check`: for a push to refs/heads/<integration> it compares `hostname` with owner.toml (fresh fetch of origin/bridle/state into FETCH_HEAD with a 10s bound, else the local bridle/state branch; no readable owner refuses); other branches and tags pass. `serve` and `bridle sync` install it (idempotent, rewritten when the binary or tools-only listing changes, not written through a symlink or over a foreign hook; those only warn). Only projects with state push on get it (no owner.toml otherwise). serve now flush_now()s after claim_owner so take-over commits owner.toml before serving. apply_branches adds the three locked deny entries to every role. Rule one-pusher-for-the-integration-branch, operating-model step 4, cli.md, add-a-machine.md, CHANGELOG done. Tests: tests/owner_push_test.rs (7, temp bare origin and clones) plus a config deny test. Installs hooks only; never edits branches or settings. Caveat: governor_test working_agent_is_notified_then_stopped_when_its_turn_ends is timing-flaky under the machine's load (30-40); it failed twice in full runs, passed alone, and the final check passed with NEXTEST_RETRIES=2."
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

### note · agent:whdbj · 2026-10-10T04:42:27.781Z
Installs hooks only; never edits branches or settings of any project. just check exit 0, 1440 tests, with NEXTEST_RETRIES=2 because governor_test working_agent_is_notified_then_stopped_when_its_turn_ends timed out twice under load (passes alone).

### note · agent:whdbj · 2026-10-10T04:42:34.451Z
done: owner-only pre-push hook (Option A) built; just check exit 0, 1440 tests (NEXTEST_RETRIES=2: one unrelated governor_test is load-flaky); 06112b58

### note · external:advisor/product-manager · 2026-10-10T07:45:47.949Z
advisor/product-manager: worker reported done 04:42Z (06112b58), still 'planned'. Is it waiting on something (load, n96z, the human)? Roadmap: machine setup is rank 1 and blocks the v0.6.0 cut; if it's held for the human like 88d4/751e, say so here and I'll put it on their morning list.

### note · agent:manager-2 · 2026-10-10T07:45:51.120Z
manager-2: built and checked, NOT landed. bridle/whdbj tip 06112b58, just check exit 0 (1440 tests; the worker ran it with NEXTEST_RETRIES=2 because governor_test is load-flaky, br-n96z), main merged, worktree clean. Parked for the human's go (credential-adjacent hook install in clones); the orchestrator said no landings/new work during the br-v6kr baseline sample. I land on the go.

### note · external:orchestrator · 2026-10-10T07:46:03.306Z
orchestrator: waiting on the human, by design. Built and checked (bridle/whdbj 06112b58), parked with br-88d4 and br-751e for the human's review this morning: it adds locked agent deny entries and a push gate on every managed clone (containment, operating-model's 'significant'). It also won't land during the br-v6kr baseline (starts 08:00Z), since a landing runs a full check. Morning order: 88d4, 751e, hdbj after review, CI green, then v0.6.0.

### note · external:aide · 2026-10-10T13:37:42.353Z
From the human, via aide (2026-10-10 ~8:15 AM ET), verbatim: "yes, land those three tickets and keep work moving according to the PdM" (br-88d4, br-751e, br-hdbj).

### note · agent:manager-2 · 2026-10-10T14:02:36.997Z
br-88d4 and br-37r9 have landed on main. Please merge main into your branch, run just check once (tell me before retrying if only a known flake fails), commit, and message manager-2 with the tip sha and exit status.

### note · agent:whdbj · 2026-10-10T14:15:35.178Z
main merged cleanly; just check exit 0, 1451 passed, no retries, no flakes; tip e4087d53

### note · agent:manager-2 · 2026-10-10T14:15:43.636Z
integrated: 1af6f7648c20cbaa06fae58b64d951f560a6321c (branch bridle/whdbj)

### note · agent:manager-2 · 2026-10-10T14:16:21.032Z
cleanup: removed agent whdbj, branch bridle/whdbj
