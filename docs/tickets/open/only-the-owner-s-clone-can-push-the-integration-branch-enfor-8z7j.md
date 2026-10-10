---
id: 8z7j
title: "Only the owner's clone can push the integration branch: enforced, not a rule"
kind: feature
opened: 2026-10-09
filed_by: external:advisor/product-manager
repos: [bridle]
changes: []
specs: []
needs: []
see: [j7r4, kt25, xrkh, k6jd, 8ay6, xccp]
tasks: [br-8z7j, br-hdbj]
---

## The ask

From postmortem j7r4 (incident br-2y3m), recommendation 1: only one clone pushes a project's
integration branch. The human, 2026-10-09 14:03 EDT, verbatim (comment c1 on j7r4):

> Agree. This is correct. We have "project takeover" to transfer a project between machines /
> clones. We just need to enforce this mechanically - either do a read-only clone or do something
> with git or PAT tokens or somehow enforce this. We can write a rule, but it should be impossible
> for a different clone under bridle to push.

The ask: make it **impossible**, not just a rule, for any clone under bridle other than the
project's owner (the machine in `owner.toml`, changed by `serve --take-over`) to push the
integration branch to origin. Taking a project over moves the right to push with it.

Needs a design (the designer role) before it is built; the human picks. Options the human named
or that fit:

- A pre-push hook bridle installs in every clone it manages, refusing a push to
  `origin/<integration>` unless this machine is the owner (agents already can't pass
  `--no-verify` if their deny lists say so; the human can).
- Non-owner clones get a push URL that can't push (`git remote set-url --push origin no-push`),
  and take-over switches it back.
- Credentials: only the owner machine holds a key or token with write access (a deploy key or
  fine-grained PAT per machine), moved at take-over. Strongest; needs human work per machine
  (see xccp).

Also the rule ("one pusher for the integration branch", `workflow/base/rules/`) as the written
form, and operating-model.md's "Merging" step 4 says who pushes. Part of the machine setup
workstream: the Windows PC is a third clone.

## Design options

Focus: interface (what the human sees at push and take-over) with a small architecture part
(where the check lives). The problem in one sentence: today nothing but a written assumption stops
a second clone from pushing the integration branch, so make that push fail unless this machine is
the project's owner.

### What exists today

- Ownership: `owner.toml` on the `bridle/state` branch names one host; `serve` refuses to start on
  another host's project, and `serve --take-over` claims it after checking that origin was
  reached and the branches fast-forward (`docs/design/agent-host/daemon.md`, `docs/design/storage.md`,
  `StateBranch::claim_owner` / `fetch_and_check_owner` in `crates/bridle-daemon/src/state_branch.rs`).
  So "owner" is already defined and already moves at take-over; nothing ties it to `git push`.
- Agents: workers can't `git push` at all, and when `release` is set every role but the
  orchestrator gets `Bash(git push origin <release>)` denied, locked against project override
  (`config::apply_branches`, operating-model.md). Managers and the orchestrator can push the
  integration branch on any clone. Those deny lists are per-agent, not per-clone.
- A hook installer already exists: `bridle machine tools-only-install` writes marked `pre-commit`
  and `pre-push` hooks, resolves the hooks dir with `git rev-parse --git-path hooks` (so it works
  with `core.hooksPath` and is shared by every worktree of the clone), refuses to clobber a
  foreign hook, and has a hidden helper (`tools-only-check`) that the hook calls
  (`crates/bridle/src/tools_only.rs`). That is a ready pattern, not a new mechanism.

### Option A: pre-push hook, owner check (recommended)

What the user sees. On a non-owner clone:

```
$ git push origin main
bridle: refusing to push main. Project bridle is owned by nuc (since 2026-10-09T14:00:00Z).
        To move it here: bridle serve --take-over
error: failed to push some refs to 'origin'
```

On the owner's clone the push behaves as today. Pushing other branches (feature or trial
branches, tags other than the integration branch) is not touched. Take-over needs no new step:
it already rewrites `owner.toml`, and the hook reads that.

Inside. `bridle` installs a marked `pre-push` hook in each clone it manages (same installer
pattern as tools-only; both hooks must chain or share one marked script since git has one
`pre-push`). The hook reads the ref lines from stdin and, for a push to `refs/heads/<integration>`
(from project config), runs a hidden helper such as `bridle machine push-check`. The helper
compares this machine's name (the one `serve` writes, `hostname`) to `owner.toml`, preferring a
fresh fetch of `origin/bridle/state` so a stale local file can't pass a clone that lost the
project; if origin is unreachable the push can't succeed anyway, so it falls back to the local
file. No owner file, no state worktree, or a read error: refuse (fail closed). Installed by
`serve` and `bridle sync`/workspace setup (idempotent, marker-guarded); the add-a-machine guide
(hua2) gets one line.

Costs. One helper, one hook script, one install call in two places. `claim_owner` only queues
`owner.toml` for the next flush, so take-over must flush before it returns (small change).
Falls short: a human (or an agent without a deny entry) can `git push --no-verify`, edit
`core.hooksPath`, or push from a clone bridle never touched. Close the agent side by adding
`Bash(git push --no-verify*)`, `Bash(git push * --no-verify*)` and `Bash(git config *hooksPath*)`
style entries to the locked deny list, as release already does. Not defeated by a human on
purpose, which matches "impossible for a different clone under bridle to push": the human is
not a clone.

Principles. KISS and modularity: reuses the installer and a check helper, touches no daemon
runtime path. One name: no new user command; the escape is the existing `serve --take-over`.
User's side: the refusal names the owner and the exact fix.

### Option B: non-owner clones get a dead push URL

What the user sees: `git push origin main` on a non-owner fails with git's own error
(`fatal: 'no-push' does not appear to be a git repository`), which does not say why. Take-over
runs `git remote set-url --push origin <real url>`; handing the project away sets it dead again.

Inside: no hook, no helper; `serve` sets the push URL from `owner.toml` at start and take-over.
Cost: smallest code. Falls short: blunt (blocks every branch, so a non-owner can't push a
feature branch or its own trial branch either); the message is cryptic; the state is a git
config value that drifts from `owner.toml` (the clone that lost the project keeps a live URL
until its daemon next starts and sees the change, a window the hook avoids by fetching);
`git push <url>` or `git push --no-verify`-free bypass is trivial for anyone who knows the URL.
Fails "the user's side first" (error text) and is not tied to the integration branch.

### Option C: credentials: only the owner machine can write

What the user sees: on a non-owner clone, the push is rejected by GitHub (403 / protected
branch). Take-over moves the right: revoke the old machine's key or token, grant the new one.

Inside: a deploy key (write) or fine-grained PAT per machine, or a branch ruleset on the
integration branch whose bypass list is only the owner's key; `serve --take-over` would call the
GitHub API to swap it. Strongest: nothing in the clone can bypass it, not even `--no-verify`, and
it also covers clones bridle never touched. Costs: per-machine human setup (keys, tokens;
overlaps xccp and hua2), a GitHub admin token that bridle must hold to move the right, a
network call and a new failure mode at take-over, and it only works for remotes with such an
API (GitHub today). Whether a ruleset can bypass-list a deploy key per machine needs checking
before relying on it. Fails YAGNI today: the observed incident was a well-meaning agent on the
wrong clone, not a hostile one.

### Option D: do nothing, keep the written rule

Add the rule ("one pusher for the integration branch") and the operating-model "Merging" step 4
sentence only. Costs nothing, and is what failed in j7r4: the human asked for impossible, not
written. Rejected by the ask; the rule is still worth writing with A.

### Recommendation

Option A, with the rule and operating-model.md step 4 updated in the same change to say "only the
owner's clone pushes the integration branch; the hook enforces it". Accepts: a human can bypass
with `--no-verify`, and a clone bridle never touched is not covered. Add Option C later only if a
bypass actually happens, or when per-machine keys arrive anyway with xccp (they are then a
cheap second layer, not a new mechanism).

Rejected, for the next reader: B as the primary (blunt, cryptic, drifts from `owner.toml`); a
read-only clone (it would stop the owner's own merges and agents' worktree commits, and a clone
is read-only only by file permissions, which also block `git` itself); a server-side check by a
bridle service (a new component for a problem one hook solves).

### Touches

`crates/bridle/src/` (hook installer next to tools_only, one hidden subcommand), `serve` and
`sync` (install call), `claim_owner` (flush at take-over), `config::apply_branches` (deny entries
for `--no-verify` and hooksPath), `workflow/base/rules/` (one-pusher rule),
`docs/design/agent-host/operating-model.md` (Merging step 4), `docs/design/cli.md`, and
`docs/context/add-a-machine.md`. No wire type change.

### Open for the human

1. Option A, as recommended?
2. Should the hook also cover the release branch and release tags, or the integration branch
   only (as asked)? Recommendation: integration only; release is already locked per role.

## Resolution

Option A built as br-hdbj, landed 2026-10-10 (1af6f764), on the human's go that morning; br-8z7j
was the design task (it couldn't leave 'reopened', so the build moved to br-hdbj; dropped). A
marked pre-push hook, shared with the tools-only hook, runs `bridle machine push-check`: a push to
the integration branch from a machine that isn't the owner in `owner.toml` (a fresh fetch of
`origin/bridle/state`, else the local copy; unreadable means refuse) is refused with the take-over
hint; other branches and tags pass. `serve` and `bridle sync` install it (idempotent; never
through a symlink or over a foreign hook), only in projects that push state. Take-over flushes
`owner.toml` before serving. Agents get locked deny entries for `--no-verify` and `hooksPath`.
Rule `one-pusher-for-the-integration-branch`. Option C (credentials) stays deferred.
