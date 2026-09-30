---
id: 24mj
title: A clean handover of a project between machines
opened: 2026-09-30
repos: [bridle]
changes: []
specs: []
needs: []
see: [one-machine-owns-a-project-hw6c, push-the-state-branch-we2r, finding-remote-daemons-from-the-laptop-xqvg]
---

## The ask

The human, verbatim (2026-09-30, via the advisor):

> Possible enhancement: does take-over do a git pull or fetch and check origin? If not let's file
> that to be done. Ideally future is that assuming I have bridle running on two machines with a
> project set up and tokens, I can tell either machine and authorized a project transfer between
> machines. Orch would wind down and work and agents, finish things up and then push everything
> stop work and approve ready to take over and then go ahead either do the transfer itself and
> push or authorize the receiving machine to take over. In any case they should talk and be clear
> on branches and SHAs and everything so it's a clean move. Worst thing I want is git issues and
> someone trying to rebase and losing history.

## Today (at 9e8ecb2)

`bridle serve` fetches `origin/bridle/state` on every start, `--take-over` included
(`StateBranch::fetch_and_check_owner`, `crates/bridle-daemon/src/state_branch.rs`). It creates or
fast-forwards the local branch when that loses nothing. But `--take-over` then goes ahead in cases
it shouldn't:

- **origin unreachable** (`FetchOutcome::Failed`) or **no remote branch**: it claims ownership
  without having seen origin;
- **diverged** (`FetchOutcome::Diverged`): it only logs it and runs on a local fork. The next push
  is refused as a non-fast-forward (we2r never forces), so nothing is lost, but the two histories
  have to be sorted out by hand, which is the rebase risk the human wants gone;
- **only `bridle/state` is checked.** The integration branch (`main`, `dev`, `bridle-adopt`) and
  the workers' `bridle/*` branches aren't fetched or compared;
- **nothing says the old machine finished.** `owner.toml` has `host` and `since`, with no record
  that the old daemon stopped, landed or parked its work and pushed.

## Part 1: harden `--take-over` (small, now)

- Refuse unless origin was reached and the local `bridle/state` is created, fast-forwarded or up to
  date. Diverged or unreachable is an error naming both SHAs. No flag overrides it; the human sorts
  it out by hand.
- Fetch the integration branch. Refuse if the local one isn't an ancestor of origin's; otherwise
  fast-forward it (only with a clean checkout).
- Never rebase, never force-push, fast-forward only, anywhere in this path.

## Part 2: a coordinated transfer (later)

The human (2026-09-30): "I agree on hardening take over now. The transfer process can be manual for
now. I prompt the owning orch to shut down, wait and then check. It doesn't have to be enforced all
yet but it's useful future work to have."

Until then the move is manual: the human tells the owning orchestrator to wind down and push, waits,
checks, and runs `bridle serve --take-over` on the new machine (with part 1's checks).


The human tells either machine to move a project, and authorizes it. Sketch:

1. **Release on the old machine.** Its orchestrator winds down: no new work; running agents finish
   or park, with their branches pushed; the queue and handover notes are flushed. The daemon pushes
   the integration branch and `bridle/state`, then writes `owner.toml` with `released = true` and
   the SHAs it pushed (the integration branch, `bridle/state` and each open `bridle/*` branch). Then
   it stops.
2. **Take over on the new machine.** `--take-over` requires `released = true`. It checks that
   origin's SHAs match the ones the release recorded, fast-forwards, claims ownership and starts.
   Parked agents resume from their pushed branches.
3. **The two talk.** With cross-machine messaging (the machine-config ticket: remote principals
   named `<name>@<machine>`), the old orchestrator sends the new one the release record, and the
   new one confirms the SHAs before it starts. Either side can stop the move if anything doesn't
   match.

Open: whether the old machine can start the new daemon itself (over SSH) or only authorize it;
what happens to a worktree with uncommitted work (commit it to its branch as WIP, or refuse to
release).
