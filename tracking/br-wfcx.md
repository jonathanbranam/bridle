+++
id = "br-wfcx"
title = "take-over refuses after a host rename: local main has commits the pre-push hook itself held back"
kind = "bug"
state = "integrated"
created_at = "2026-10-14T01:46:21.579Z"
updated_at = "2026-10-14T03:46:23.171072Z"
created_by = "external:aide@vesta"
watchers = [
    "external:aide@vesta",
    "agent:wfcx",
]
branch = "bridle/wfcx"
commit = "44ab4a48907a0d24c0e2875cdcbd4c6e8b3c98a7"
summary = """take-over no longer deadlocks after a host rename. sync_integration_for_take_over (state_branch.rs) now returns whether local main is AHEAD of origin (origin an ancestor of local): accepted, and take_over_project pushes it (never forced) after the claim is confirmed on origin; a failed push does not undo the claim and the CLI prints the `git push origin <branch>` to run (MainPush enum in TakeOverOutcome::Claimed). Only real divergence is refused; the message says "diverged" and gives the next step. project_sync::ssh (used by the `[projects]` move) gets ConnectTimeout=10 and a 60 s overall limit that kills ssh and names the machine. docs/design/cli.md documents "rename = take-over every project", CHANGELOG added. Tests: ahead accepted/diverged refused (extended existing test), fake-ssh timeout test. Not done (out of scope): third issue (start/take-over not re-creating tokens; token_pair.rs is xqwy's) and the `ticket comment --project` prefix bug; the other ssh callers (preflight_remote.rs, token_pair.rs) still lack timeouts; no rename detection; no end-to-end test of the post-claim push."""
ticket = "wfcx"
+++

docs/tickets/open/take-over-refuses-after-a-host-rename-local-main-has-commits-wfcx.md

submitted by external:aide@vesta

Submitted by the meta-notes aide on vesta. The human, verbatim: "yes file the takeover refusing on main as a bug".

## What happened (2026-10-13, bridle 0.7.1)

1. The human renamed the machine nuc -> vesta in ~/.bridle/config.toml at 8:59 PM ET. owner.toml on bridle/state still said host = nuc for all four projects on the machine, so the one-pusher pre-push hook refused every push from vesta ("owned by nuc ... bridle project take-over").
2. Agents kept committing while pushes were refused. meta-notes had 1 commit on local main (fe6e60f, the incident for this very outage), and notes had 2 (the human's Tuesday daily note).
3. With the daemons stopped, `bridle projects take-over --force` in each clone gave:
   - meta-notes: `take-over refused: local main (fe6e60f...) is not an ancestor of origin's (90fc1df...); sort it out by hand, bridle never rebases or forces`
   - notes: the same (local 7fe88cd vs origin 7e85d5e)
   - dotfiles-local (local main already pushed): succeeded.

That's a deadlock. Take-over needs local main already on origin, but the push is refused until take-over has run. The message is also wrong: local main is AHEAD of origin (origin's main is its ancestor), not diverged, so there is nothing to "sort out".

Workaround the human used, per clone: `git branch hold main; git reset --hard origin/main; bridle projects take-over --force; git merge --ff-only hold; git push origin main`.

## The ask

- When local main is a fast-forward of origin's main, take-over should accept it, then push it (or say to push) once ownership has moved. Refuse only real divergence.
- The refusal message should say which case it is (ahead vs diverged) and give the steps.
- Related: a host rename in config.toml alone breaks pushes on every project at once. Consider detecting it, or document "rename = take-over every project, push first".

## A second, smaller issue in the same run

take-over's "move [projects] on every machine" step ran `ssh -o BatchMode=yes remus ...`, and remus had Tailscale SSH in "check" mode, which waits for a browser login. BatchMode doesn't stop that wait, and there's no ConnectTimeout or overall timeout, so the take-over hung with no output until the human interrupted it. Add a timeout, and name the machine it's waiting on.

## Thread

### note · external:aide@vesta · 2026-10-14T01:46:21.580Z
submitted by external:aide@vesta

### note · agent:pm-1 · 2026-10-14T01:46:31.719Z
Accepted by pm-1. Brief. Model: Sonnet. Size: S-M.

Goal: `bridle projects take-over` must not deadlock after a host rename.
1. If local main is a fast-forward of origin's main (origin's main is an ancestor of local), take-over accepts it, moves ownership, and then pushes main (or prints the one command to push). Refuse only real divergence.
2. The refusal message names the case (ahead vs diverged) and gives the steps.
3. The "move on every machine" ssh step gets a ConnectTimeout and an overall timeout, and names the machine it is waiting on (Tailscale SSH check mode waits for a browser login despite BatchMode).
4. A host rename in config.toml alone breaks pushes everywhere: document "rename = take-over every project, push first" in the relevant design doc (docs/design/cli.md / project docs); detecting it is optional, only if small.

Files: find the take-over code with `git grep -n "not an ancestor"` and the ssh step near it; docs/design/cli.md; CHANGELOG.md entry. Tests: ahead case accepted, diverged case refused with the right message, ssh timeout (use a fake ssh). No change to the one-pusher hook itself, and never bypass it (rule one-pusher-for-the-integration-branch). No project-file migration needed.
Acceptance: `just check` passes. Out of scope: automatic rename detection beyond a message, rebasing or forcing anything.

### note · external:aide · 2026-10-14T01:51:03.568Z
Relayed by bridle's aide from the meta-notes aide on vesta (2026-10-13 ~9:50 PM ET). The human asked for it to go on this ticket, verbatim (to the vesta aide): "please incude it yes".

Third issue from the same take-over: the daemons came back with fresh databases (old ones kept in each workspace's .bridle/db-backups/2026-10-14-*), but with no project tokens except `human`. Checked by GET /v1/status with each token in ~/.bridle/credentials.toml on vesta at ~9:45 PM ET:
- meta-notes: the aide, advisor, orchestrator and mail tokens are accepted (200).
- notes, dotfiles-local, meta-notes-ui: every token is refused (401), so the orchestrator's waiters and the aides on those projects are cut off.

`bridle projects start --project notes`, run again, printed only "daemon unit bridle-notes.service enabled and started / notes: daemon is up" and still 401s. Its help says it "re-creates the project's tokens on every machine (the `token pair` code)" after a take-over; it didn't, or only on the first start, which hit the remus Tailscale SSH hang. credentials.toml was last written at 21:21, before the take-overs; daemon.log shows only "wrote human token". Next step there is `bridle token pair`.

Ask: start (or take-over) re-creates the tokens reliably and says what it did, and fails loudly when it can't (an unreachable machine, for example).

Side note from the vesta aide (a small separate bug, not asked for by the human): `bridle ticket comment br-wfcx --project bridle` refuses from the meta-notes clone ("the daemon's task prefix `br` differs from `[tasks] prefix = "mn"`") and from outside a repo ("not in a git repository"), though ticket submit works from there.

### note · agent:wfcx · 2026-10-14T03:23:37.358Z
built: ef84347c (branch bridle/wfcx)

### note · agent:wfcx · 2026-10-14T03:23:37.410Z
done: take-over accepts ahead main and pushes it, diverged message, ssh timeouts; just check exit 0, 1655 tests passed; ef84347c

### note · agent:manager-2 · 2026-10-14T03:45:58.505Z
integrated: 44ab4a48907a0d24c0e2875cdcbd4c6e8b3c98a7 (branch bridle/wfcx)

### note · agent:manager-2 · 2026-10-14T03:46:23.171Z
cleanup: removed agent wfcx, branch bridle/wfcx
