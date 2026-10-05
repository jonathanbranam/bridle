---
id: q7mv
title: "Postmortem: the self-upgrade refused three good builds overnight because the new binary's self-check timed out, and the fix couldn't install itself (aqa7)"
kind: research
opened: 2026-10-05
repos: [bridle]
changes: []
specs: []
needs: []
see: [up82, ksz7, p88z, bek3, cs7x]
tasks: []
---

## The ask

The human, verbatim (2026-10-05 ~8:15 AM ET, to the orchestrator):

> did you file an incident for that? Please do record an incident, and possibly a postmortem on
> this would be helpful. I wasn't able to read all the details yet, but that's a critical kind of
> thing we want to be sure doesn't happen in the future.

Incident task: br-aqa7. Written from the `upgrade.*` and surrounding events, `git log`, CI runs and
`crates/bridle-daemon/src/upgrade.rs`. Earlier record: docs/context/incidents.md (e0fbe4d0).

## Summary

The daemon's automatic upgrade built green main and then refused to restart into it three times
overnight (03:51Z, 07:19Z, 09:50Z), each time with "the new binary's self-check timed out". The
self-check is `bridle serve --check` (load the config, exit) run on the freshly installed binary
with a 60 s cap; by hand, ten minutes after the first failure, the same check took under a second.
Upgrades in between (04:48Z) and after (12:08Z) went through. The fix, br-up82 (retry a timed-out
check up to three times), landed at 06:40Z but couldn't help: **the retry loop lives in the running
daemon, which is the old binary**, so it only takes effect from the first upgrade *after* one
succeeds without it. That happened at 12:08Z, by a single lucky attempt on a quiet machine, not by
the fix.

## Impact

- The daemon stayed on d1ed064d from 04:48Z to 12:08Z (~7 h 20 m) while eleven tasks landed on
  main (btx7, cyvf, ksz7, up82, tc7t, s6cj, 7sd9, a3yd, rk7k, g49c, ds96). None of their daemon
  changes were live until 12:08Z.
- Before that, 03:51Z-04:48Z, the daemon stayed on c548182b (without br-e9yu).
- **CLI/daemon skew.** `cargo install` replaces `~/.cargo/bin/bridle` before the self-check, and a
  refused check doesn't put the old file back, so after each failure the CLI ran the new build
  against the old daemon (03:51-04:48Z, 07:19-12:08Z). No harm seen; nothing guards against it.
- A human to-do, br-46me, asks for a manual upgrade. (It was filed at 03:00Z, before this incident,
  because Claude Code's auto-mode classifier refused the orchestrator's `bridle daemon restart
  --upgrade` as "Interfere With Workloads"; at 03:52Z the orchestrator noted the automatic upgrade
  had failed too.) The orchestrator could not recover by itself at any point.
- The bridle-ui pages that landed overnight (Tasks, System, links, send) showed no data, and the
  new web/mobile rule packs were reported not active. **Partly not this incident:** the running
  gateway (pid 60915) started 2026-10-04 22:05Z, before br-bek3 (01:22Z) taught the gateway to
  re-exec when the binary changes, so it has served the old code all along and still does after the
  12:08Z upgrade. It needs `bridle gateway --detach` once (step 2 of br-46me). How much of the UI
  gap is the stale daemon (missing API routes) versus the stale gateway is not checked.

## Timeline (2026-10-05, UTC)

- **01:50-01:55** automatic upgrade to c548182b succeeds.
- **03:00** orchestrator files br-46me: its own `restart --upgrade` was refused by auto mode.
- **03:42** land check for br-yfjc starts `just check` (runs to 03:54); per up82, two workers'
  `just check` runs were also going.
- **03:46:49-03:51:21** build of 0d69d617 (4.5 min), then **upgrade.failed**: "refused to restart
  into it: the new binary's self-check timed out". Checker: c548182b (one attempt, 60 s).
- **~04:00** orchestrator runs the same check by hand: rc 0 in under a second. Files up82 and ksz7
  and the incidents.md entry (e0fbe4d0, 04:02Z).
- **04:40:39-04:48:19** build of d1ed064d (7.7 min) and **upgrade.built**: success, *also under
  load* (br-cyvf's land check from 04:39:57; a-0u47a's `just check` from 04:44:01).
- **06:20-06:40** br-up82 lands (c2ac03d6): `CHECK_ATTEMPTS = 3`, retry on timeout only. CI green 06:55.
- **07:11:50-07:19:31** build of c2ac03d6 (up82 itself, 7.7 min), **upgrade.failed**, same error.
  Load: a-ufjzn's `just check` from 07:04:22; a-1rk6v's `cargo nextest` from 07:18:28.
  The error has no "(3 attempts of 60 s)" suffix: the old d1ed064d checker ran it.
- **07:48-11:02** five more commits go green on CI; no upgrade attempt is recorded until 09:42.
- **09:42:36-09:50:13** build of 1f0a5bd4 (7.6 min), **upgrade.failed**, same error, old checker.
  Load: br-rk7k's land check `just check` from 09:34:05 (it failed at 09:53 with an aborted test);
  a-ormv5 was waiting on web-packs2's `just check`.
- **09:50-12:05** three more commits go green; no attempt recorded.
- **12:05:36-12:08:49** build of 1ca6c3c4 (3.2 min), **upgrade.built**, restart. No `just check`
  was running (y25n's land check ended 11:59; the next check started 12:15). Checker: d1ed064d, one
  attempt. The daemon now carries up82.
- **12:17** orchestrator files br-aqa7 at the human's request.

## How it happened

1. **What the check does.** After `cargo install` (CARGO_TARGET_DIR `<state>/upgrade-target`) and
   an attempted re-sign, the running daemon runs `<installed bridle> serve --check --repo ..
   --workspace ..` under `tokio::time::timeout(60 s)` (upgrade.rs `check_built`). `serve --check`
   loads the config and exits; it opens no database. Its work takes well under a second.
2. **Why it took over 60 s (inference, unconfirmed).** The leading explanation is macOS assessing a
   freshly linked, ad-hoc-signed binary on its first exec (syspolicyd; cs7x), slowed by load on this
   8-core/16-thread i9 Intel Mac. Supporting: a second run is fast; dalek has **no** "bridle local
   signing" identity (`security find-identity -p codesigning`: 0 valid), so p88z's stable signature
   never applies and every build is a new ad-hoc identity; all three failures overlapped one or more
   `just check` runs. Against a pure load story: the 04:48Z success also overlapped two checks. Not
   done: `log show --predicate 'process == "syspolicyd"'` around the failures, or timing the check.
3. **The fix couldn't reach the checker (the bootstrap gap).** The timeout and retry are in the
   daemon that is running, and the binary under test is the new one. A fix to the checker only works
   once a daemon containing it is running, and getting there needs an upgrade that passes the old
   checker. up82 landed at 06:40Z; the next two attempts still ran d1ed064d's single 60 s attempt.
   The 12:08Z success was that same single attempt passing on a quiet machine.
4. **Failure is sticky per commit, and retries wait on new commits and quiet points.** A refused
   commit is never retried (in memory); the next try needs a newer green commit that touches the
   binary *and* a tick with no agent mid-turn. Green commits existed from 07:48Z and 10:09Z, yet the
   next attempts came at 09:42Z and 12:05Z. Inference: no quiet point was found in between; the
   daemon records nothing when it skips a tick for being busy, so this can't be confirmed from events.
5. **No escape hatch for the orchestrator.** The manual route, `bridle daemon restart --upgrade`, is
   denied to the orchestrator by Claude Code's auto mode, and it runs the same self-check anyway.
   Recovery depended on the human or on luck.

## What went well

- The self-check did its job in the sense that matters: it never restarted into a binary it couldn't
  vouch for, and the old daemon kept serving. No agent lost a turn.
- Each failure produced an `upgrade.failed` event with a clear message and woke the orchestrator,
  which found the first one within minutes, reproduced the check by hand and filed up82, ksz7 and the
  incidents entry the same hour.
- up82 was built and landed within three hours.

## What went badly

- Nobody noticed that up82 could not take effect by itself; the 07:19Z and 09:50Z failures were
  read as the same problem rather than as "the fix isn't running yet".
- The failure mode was silent to the human for hours: the daemon was quietly ~7 h behind main
  while the night's work (including UI the human would look at first) piled up behind it.
- The "result" of a refused check left a mixed install: new CLI, old daemon.
- The check's timing isn't recorded, so the cause is still a guess.

## Learnings

- A check that gates its own replacement must be fixable without passing it first. Anything in the
  upgrade path (timeouts, retries, the check itself) needs a way around a stale copy of itself.
- A gate that runs under load needs a budget that measures the work, not the machine's queue.
- An install that fails its check should leave the machine as it was (or say clearly it didn't).

## Recommendations

1. **Let the new binary own its own check budget.** Run a short warm-up exec of the new binary
   (`bridle --version`, generous or no cap) before the capped check, and record both durations in the
   `upgrade.built` / `upgrade.failed` event data. This also answers point 2 above the next time.
2. **Close the bootstrap gap.** On a check *timeout* (not a failure), have the daemon retry the same
   commit on a later quiet tick instead of marking it failed for good, and after N timeouts wake the
   orchestrator and the human with "upgrade stuck: N refused builds since <commit>". Possibly read the
   check budget from config, so it can be raised without a new binary.
3. **A human-approved override.** `bridle daemon restart --upgrade --skip-check` (or similar),
   human-run or human-approved, for when the check is the thing that's broken. The rollback marker
   (`upgrade-pending.json`) already covers a binary that really can't start.
4. **Fix the signing setup on dalek.** Run `just sign-setup` so p88z's stable identity applies;
   if first-exec assessment is the cause, a stable signature should make it rare. Then confirm or rule
   out syspolicyd from its log at the next slow check.
5. **Restore the installed binary on a refused check** from `.bridle/bridle.prev`, so the CLI and the
   daemon don't run different builds.
6. **Record why an automatic upgrade didn't start** (busy agents) at most once per commit, so a
   long stall like 09:50-12:05Z shows in `bridle events`.
7. **Now:** the human runs `bridle gateway --detach` once (br-46me step 2); the gateway predates bek3
   and is still serving the old code.
