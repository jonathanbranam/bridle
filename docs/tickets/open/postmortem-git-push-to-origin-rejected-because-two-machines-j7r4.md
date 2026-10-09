---
id: j7r4
title: "Postmortem: git push to origin rejected because two machines wrote to main and nothing fetched (br-2y3m)"
kind: incident
opened: 2026-10-06
repos: [bridle]
changes: []
specs: []
needs: []
see: [q7mv]
tasks: []
---

## The ask

Postmortem of incident br-2y3m. The human, verbatim (2026-10-05 about 10:50 PM): "Is `git push`
failing?" and "while I'm sleeping, write up an incident report, investigate this, explain what
happened, and make a postmortem. Somebody did something wrong here." Wanted by 6:00 AM for the
human to approve. Investigated read-only: no branch, ref or remote was changed. `git fetch` was
not available to this agent, so origin's state was read with `gh api` GETs (sha 28d25f8c and its
four predecessors below); the Mac clone's `origin/main` tracking ref is stale and was left alone.

## Summary

`git push origin main` from the Mac clone (/Volumes/Data/work/bridle/bridle) is rejected as a
non-fast-forward. Two machines have been committing straight onto `main` of the same GitHub repo
and neither had the other's commits. Since the last good push (`29296901`, 2026-10-05 23:22 UTC)
the Mac clone has made 23 commits (22 ticket-and-docs commits by orchestrator, advisors and aide,
plus one landed feature, `65b9c670` br-xxw9) and none was pushed. Over the same hours origin
received 5 ticket-only commits (`2203aae1`, `0cbf30b5`, `c9950da5`, `af96bffb`, `28d25f8c`) from a
different clone, apparently the NUC's (the meta-notes orchestrator filing bridle tickets). The two
sets touch different files, so reconciling is a plain merge with no conflicts expected. Nothing is
lost yet; nothing was pushed wrongly; the cost is that origin is 23 commits behind the real main
and the NUC's tickets are not on the Mac's main.

Nobody broke a written rule. The design has a hole: it assumes one clone pushes `main`, but two
machines write to it, nothing fetches on the Mac side, and a rejected push has no owner.

## Facts (by sha)

- Merge base: `29296901` (br-3haz slice 1), committed 2026-10-05 19:02 -0400 (23:02 UTC), pushed
  23:22:14 UTC by the manager-2 session `396c2b91` (that session's `git push origin main`, and the
  clone's `origin/main` reflog "update by push" 19:22:15 -0400).
- Mac-only (local `main`, tip `a49bff2e`, 22:42 -0400): 23 commits, all authored by Jonathan Branam
  (the shared git identity, so author does not say who). First `2598494e` (ticket fx7x, 20:04 -0400 =
  00:04 UTC). 22 are `docs/tickets/` changes; the 23rd is `65b9c670` br-xxw9 (code: `cli.rs`,
  `usage.rs`, `store.rs`, `server.rs`, `types.rs`, docs, CHANGELOG), landed by manager-2 at 01:37 UTC
  (reflog: fast-forward at 21:37:41 -0400).
- Origin-only (`gh api`, branch main, tip `28d25f8c`): 5 commits on top of `29296901`, all
  `docs/tickets/open/*.md` additions (one modification), committer and author
  `jonathan.branam@gmail.com` (same identity), patch dates -0400:
  `2203aae1` ticket a4pm 00:18:28 UTC; `0cbf30b5` tickets t3vq and mtdg 00:28:52 UTC; `c9950da5`
  ticket 7d4y 01:57:23 UTC; `af96bffb` ticket t3vq edit 02:10:54 UTC; `28d25f8c` ticket n63z
  02:25:55 UTC. Unsigned, ordinary commits (not a GitHub web edit, not CI).
- Who made them: no agent transcript on the Mac shows a push in that window. The ticket bodies say
  "Found by the meta-notes orchestrator on the NUC, 2026-10-06 00:20 UTC" (`0cbf30b5`) and "Found
  on the NUC, 2026-10-06 about 02:20 UTC" (`28d25f8c`), and carry `repos: [meta-notes]`. That is
  an inference from content; the NUC clone's own reflog was not read (no access from here).
- `.github/workflows/ci.yml` runs on push and pull_request only; `release.yml` pushes nothing to
  main. No automation writes to `main`.
- Disjoint: the five origin files (ticket ids a4pm, t3vq, mtdg, 7d4y, n63z) do not exist on the Mac
  main, and no Mac-side file touches them. The 23 local commits are not patch-equivalent to the 5:
  genuinely different changes, no re-made work.
- Divergence began at `2203aae1` (00:18 UTC), 14 minutes after the Mac's first post-base commit
  (00:04 UTC). From then on neither side's push could succeed without the other's commits.

## Timeline (UTC; US Eastern in parentheses only where the human quoted a time)

- 23:02 `29296901` committed on the Mac; 23:22 pushed. Both clones in sync (the NUC then pulled it,
  per its tickets "after the human installed 29296901").
- 00:04 Mac: first unpushed commit `2598494e` (ticket fx7x). 22 docs commits follow, through
  `a49bff2e` at 02:42, made directly on local main by orchestrator, advisors and aide. No one
  pushes them: the push step is part of landing (operating-model.md step 4) and ticket commits are
  not landings.
- 00:18 NUC pushes `2203aae1` to origin: origin and the Mac main diverge. 00:28, 01:57, 02:10, 02:25
  four more NUC pushes, each a fast-forward on the NUC's own view (it was based on origin).
- 01:37 manager-2 lands br-xxw9 on local main (`65b9c670`, fast-forward).
- 01:41 manager-2 runs `git push origin main`: rejected ("fetch first"; "the remote contains work
  that you do not have locally"). It tries `git fetch origin`: denied (managers run in don't-ask mode
  with fetch denied). It sends a question to the orchestrator: `bridle send` fails
  `error: unknown:`. It reports the unpushed commit in its own text at 01:41, 01:47 and 01:57 and
  keeps working. No one acts.
- 02:45 (10:45 PM) The human: "I think I saw this several times in other messages. Is `git push`
  failing?" The orchestrator runs `git status -sb` ("[ahead 23]") and `git push --dry-run`: rejected.
  First time anyone confirms it. 02:46 incident br-2y3m filed.
- 02:48 manager-2 comments on br-2y3m that it hit the same rejection at 01:41 and could not report
  it.

## Root cause

1. **Two writers on one branch, with no stated owner of the push.** The Mac clone is the project's
   integration clone, and the operating model (docs/design/agent-host/operating-model.md, "Merging",
   step 4) has only the merger push, straight after a landing. A second machine (the NUC) also
   commits and pushes tickets straight to `origin/main`. Nothing says the NUC must not, or that
   the Mac must pull before it commits; the shared git identity hides who wrote what.
2. **The Mac side never fetches.** The operating model says workers never fetch or merge from a
   remote, and managers run with fetch denied. Only the human or orchestrator could, and no step
   or check does. So `origin/main` in the Mac clone stayed at `29296901` for hours and `git status
   -sb` reported only "[ahead 23]", which looks like "just unpushed". The divergence became
   visible only when a real push was attempted.
3. **Direct-to-main ticket commits are never pushed.** The 22 docs commits are not a landing, so
   no step pushes them. "The remote never lags the clone" (step 4) holds only for landings.
4. **A rejected push has no handler.** The landing flow does not verify the push (`land` never
   pushes), and nothing records or alerts on a failed one.

The proximate act is the NUC pushing to the integration branch of a repo whose merge point is the
Mac clone, but with the rules as written it was not forbidden. Process hole, not a slip by one
agent.

## Why it went unnoticed

- 1: The NUC's pushes succeed, so nothing on its side ever looks wrong. The Mac sees nothing
  because it does not fetch, and its stale `origin/main` makes status look fine.
- 2: The one agent that hit the rejection (manager-2, 01:41) could not fetch, could not message
  (`bridle send` failed with `error: unknown:` every time, including at 02:48; this agent saw the
  same error on a plain `bridle send` to manager-2 at about 02:55 and `bridle task comment ... --notify` worked), and
  could only mention it in its own session text, which the orchestrator does not read. The report
  never reached the orchestrator's inbox.
- 3: No check, landing step or hook verifies that `origin/main` contains `main`. `just check` and
  `bridle task land` pass with the push unrecorded. A failed push produces no event.
- 4: The human found it by asking, 1 hour 4 minutes after the first rejection and 2 hours 27
  minutes after the divergence began.

## Why wasn't this reported to the human?

The manager that hit the rejection (manager-2, at 01:41 UTC) tried to report it through `bridle send` to the orchestrator but encountered a defect: every `bridle send` call returned `error: unknown:`, blocking the message. It mentioned the rejection in its own session text, which was not escalated as a task comment (`bridle task comment`). This is a combination of:

1. A transport failure (`bridle send` returning an error that prevented messaging to the orchestrator).
2. No fallback: an agent encountering a message failure should escalate to a task comment instead of relying on session text.

The human discovered it by asking at 02:45, 1 hour 4 minutes after the first rejection.

## What each agent did

- **NUC meta-notes orchestrator** (inferred): committed bridle tickets to its clone of bridle and
  pushed straight to origin main, 5 times between 00:18 and 02:25. Followed no written rule that
  forbids it; did not know the Mac had unpushed work.
- **Mac orchestrator, advisors, aide**: committed 22 ticket/doc commits directly on local main
  (00:04 to 02:42). Never pushed them and never checked `origin/main`. At 02:45 the orchestrator
  confirmed the failure with `git push --dry-run` and filed br-2y3m.
- **manager-2 (session 07a30b48)**: landed br-xxw9 at 01:37, pushed at 01:41 (rejected), tried
  fetch (denied), tried to tell the orchestrator (`bridle send` error), then repeated "push
  outstanding" in its summaries at 01:47 and 01:57. Did not retry, force or work around; also did
  not post a task comment (comment was the one path that may have worked), which is a miss. Its
  later comment on br-2y3m (02:48) is accurate.
- **Earlier manager-2 session (396c2b91)**: the last good push, 23:22:14.
- **Workers**: none pushed (disallowed). No other agent logged a push rejection (searched the
  Mac's Claude transcripts and the event log for 23:00 to 02:50 UTC).
- **Nobody force-pushed, reset or rebased**, so no work was lost.

## Recommendations

Cheap first.

1. **Say who pushes main, and that it is one clone.** Add to operating-model.md: only the
   integration clone (the Mac) writes to `origin/<integration>`; any other machine proposes via a
   branch or a ticket and does not push main. If the NUC must write tickets, it pulls before and
   pushes after each commit and treats a rejection as an incident. Record it as a rule in
   `workflow/base/rules/` with `roles:` for every role that commits (rule: "one pusher for the
   integration branch").

> [!comment] c1 human, 2026-10-09 14:03 EDT, on "Say who pushes main, and that it is one clone" [pending 2026-10-09 14:03 EDT]
> Agree. This is correct. We have "project takeover" to transfer a project between machines / clones. We just need to enforce this mechanically - either do a read-only clone or do something with git or PAT tokens or somehow enforce this. We can write a rule, but it should be impossible for a different clone under bridle to push.

2. **Make divergence visible on the Mac.** Let managers and the orchestrator run `git fetch origin`
   (a read-only fetch, allowed in don't-ask mode), and have the daemon or `bridle doctor` run it on a
   timer and warn when `origin/main` is not an ancestor of `main` ("N ahead, M behind"). Today's
   `[ahead 23]` hid the problem.
3. **Push ticket commits too.** Either push after each direct-to-main docs commit (a small wrapper
   or hook), or accept that the remote lags and say so in the operating model; right now it says it
   never lags and that is false for docs.
4. **A failed push is an event and an alarm.** The merger's push after a landing should be a bridle
   command (or `bridle task land --push`) that records `push.failed` and notifies the orchestrator
   and the human; a bare `git push` that fails should not depend on the pushing agent's session
   text. Related: an agent that cannot send must put the blocker on the task thread
   (`bridle task comment`), not just in its own output; add that to the worker/manager roles.
5. **Investigate `bridle send` returning `error: unknown:`** (reproduced by manager-2 many times
   from 01:41 and once more by this agent). Out of scope here; file a bug. It is why the one report
   was lost.
6. **Distinct git identities per machine** (for example a committer name carrying the host: `NUC`),
   so a postmortem can tell clones apart without reading ticket text.

## Reconcile (completed)

On 2026-10-06, with the human's go ("let's fix main first"; see br-2y3m thread), the orchestrator merged origin/main into local main using `git merge --no-ff origin/main`, bringing in 8 ticket-only commits from the other machine (commit a5ac8bc6). The merge had no conflicts and all commit identities were preserved. The result was a single merge commit (05d498f2) with the message "Merge origin/main: reconcile the divergence (incident br-2y3m); 8 ticket commits from the other machine". The merge commit was pushed to origin (a5ac8bc6..05d498f2), and main and origin/main are now in sync.

The incident is resolved. All ticket work from both the Mac and NUC checkouts is integrated into main. Recommendations 1–6 remain as action items for preventing future such divergence.

## Open questions

- Confirm with the human that the NUC's meta-notes orchestrator is the pusher (read the NUC clone's
  `git reflog origin/main`); this report infers it from ticket text and -0400 patch dates.
- Does the human want the NUC to keep writing tickets into bridle's main, or to use a branch?
  Recommendation 1 assumes the first, with pull-before-commit.
