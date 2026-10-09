# Incidents

A running log of every failure or problem we find, so patterns show up and we learn from them
(the human, 2026-10-02: "Anytime there's an incident or an issue that we discover, we should log
it and explain the reason for it ... so we can analyze it and learn from our failures"). It began
as a log of lost connections and sessions (the human, 2026-09-28); it now covers anything that
went wrong: something the human reports to the orchestrator or an advisor, or anything a role
discovers (a crash, a stall, a red `main`, a bad merge, work that sat stuck, a role that did the
wrong thing, a wrong assumption that cost time).

This is a record, not a to-do list. Work that comes out of an incident is a ticket or task,
linked from the entry. An active incident that agents must know about is also an `incident`
task (`docs/design/agent-host/incidents.md`); this log is the after-the-fact record of it.

Newest first. Times are UTC. Each entry has:

- **What happened** and when, and how it was found (reported by whom, or discovered how).
- **Impact:** what it cost (time, work, money, trust).
- **Cause:** the root cause, or "unknown".
- **Category:** one or more of `connectivity`, `host` (OS, machine, power), `daemon`, `ci`,
  `merge`, `coordination` (work stuck or dropped between roles), `role` (a role or prompt did
  the wrong thing), `config`, `external` (Claude Code, GitHub, network), `human-process`.
- **Follow-up:** the ticket or task, or "none" and why.

Related: [[laptop-sleep-and-network-loss-prvy|prvy]] (the laptop sleeping or losing its network).

## 2026-10-09 20:32: main red on Linux from a new upgrade_test flake

- **What happened:** CI on `f8b6b5b5` (docs only) failed on ubuntu: `upgrade_test`
  `a_drain_starting_during_a_spawn_restarts_promptly` got `Unreachable` listing messages. Found
  by the orchestrator's CI wake.
- **Impact:** `main` red; merges held until it is fixed.
- **Cause:** the test (added hours earlier by br-b6mu, 3323d2f4) calls the daemon over HTTP after
  `restart_requested()`, racing the drain's shutdown of the server.
- **Category:** `ci`
- **Follow-up:** [[flaky-on-linux-ci-upgrade-test-a-drain-starting-during-a-spa-ngya|ngya]]
  (br-ngya, critical).
- **Second cause, same window:** from br-rztb (5aa62200) every ubuntu run also failed
  `gateway_test` `a_replaced_binary_is_re_executed`: the new unit-child check trusted any
  `INVOCATION_ID`, which systemd sets for every service (the GitHub runner included). Not
  flaky; a real misdetection. Follow-up: br-8a53 (critical). `main` stayed red until both landed.
  Green again at 21:23 on `bf386175` (run 37992014772); about 50 minutes red.

## 2026-10-09 19:18: dalek's gateway keeps being restarted outside launchd (recurring)

- **What happened:** the human found the gateway on `:7878` was pid 88281, started 2026-10-09
  01:39Z from a Claude Code session (`CLAUDECODE=1`, ppid 1), not by launchd or by them. The log
  shows the same SIGTERM-then-restart at least nine times since 2026-10-07 22:08Z, with no record
  of who asked. Since ~18:54Z launchd's `dev.bridle.gateway` fails "Address already in use"
  (likely kickstarted by a br-57nt test run). Reported by the human to aide ("I NEVER started
  those gateways ... I have reported this before").
- **Impact:** the gateway runs unsupervised with an inherited session environment (the
  2026-10-08 outage had the same shape); launchd retries in a loop; the human's trust, as a
  repeat report.
- **Cause:** proven from the code and log: before br-57nt (2026-10-09 3:06 PM ET) `gateway
  restart` was SIGTERM then a detached child, never launchd, and the child inherited the
  caller's Claude env; the log shows that shape (SIGTERM, "listening" 25-150 ms later, no
  "binary changed" line) nine times. `gateway --detach` with the launchd job loaded but
  stopped did the same, and a gateway started that way then kept itself current by exec in
  place (same pid, same env), so it stayed outside launchd until stopped. Inferred: the 01:39Z
  restart was an agent or the orchestrator running `gateway restart` from a Claude session; the
  log didn't record callers, so which one is unknowable. No test touched the real gateway
  before 57nt (tests use a temp `BRIDLE_HOME`); 57nt's own pre-fix test runs likely kickstarted
  the real job (the 2:54 PM "Address already in use" loop, inferred).
- **Fix (rztb):** `--detach` is refused while a unit is loaded; a gateway that is the unit's
  child exits non-zero on a replaced binary so launchd/systemd restarts it (exec in place only
  when unmanaged, with the Claude/principal env scrubbed); every stop, start and restart
  writes a caller line to `gateway.log`; gateway tests put fake `launchctl`/`systemctl` first on
  PATH and assert they are never called. The launchd unit is unchanged.
- **Category:** `daemon`, `role`.
- **Follow-up:** [[incident-something-keeps-restarting-dalek-s-gateway-outside-rztb|rztb]]
  (br-rztb, high); earlier bek3, 76td, ppa6, 57nt.

## 2026-10-09 15:56: main red on Linux from a third upgrade_test flake

- **What happened:** CI run 37954619068 on fa22d829 (the br-f4xu merge) failed on Linux:
  `upgrade_test self_upgrade_restarts_only_after_the_mid_turn_agent_finishes` timed out (60 s)
  waiting for the agent to work or the restart. macOS passed. Found by the orchestrator's
  `ci_failed` wake.
- **Impact:** main red; merges held; the failed job re-run.
- **Cause:** unknown; a timing race in the test or the daemon's quiet check. Not br-f4xu (the test
  sends no signal).
- **Category:** `ci`.
- **Follow-up:** [[flaky-on-linux-ci-upgrade-test-self-upgrade-restarts-only-af-h7gt|h7gt]]
  (br-h7gt, critical).
- **Recurred** 2026-10-09 16:29 on ff9958b7 (br-6nzj merge), CI run 37958674369, Linux only;
  panicked at `tests/support/mod.rs:346` after 68.8 s. Main red again; merges held but br-h7gt.

## 2026-10-09 ~15:14: new binaries hang at launch on the laptop (syspolicyd again)

- **What happened:** two `bridle workflow spec check --require-ids` runs (tick6nzj's `just check`
  in `wt/tick6nzj`, and manager-2's integration check landing br-f4xu) sat 19+ min with one
  thread in an unknown image, before `main()`. Test binaries in `integration/target` (`--list`
  runs) hung the same way, up to 33 min. `syspolicyd` was in state U at ~55% CPU (610 CPU-min in
  41 h of uptime), XprotectService ~55%. Earlier, tick6nzj's first check had hit its background
  time limit. Found by the orchestrator from an `agent.stalled` wake on tick6nzj.
- **Impact:** merges (the f4xu flake fix included) and both workers stalled ~35 min. The human
  was asked through aide to clear it; it cleared by itself at ~15:49 (syspolicyd caught up,
  running again) before they acted, and the ask was withdrawn.
- **Cause:** a syspolicyd backlog, or a pending Gatekeeper window (qr8z), holding new binaries at
  launch. The morning's load holds (two workers' builds, two self-upgrades) fed it new binaries;
  `integration/target/debug/deps` is back to 192K files.
- **Category:** `host`, `merge`.
- **Follow-up:** [[incident-syspolicyd-and-spotlight-pegged-builds-and-app-laun-z7y5|z7y5]]
  (open), [[a-landing-s-integration-check-has-no-timeout-one-hung-test-h-y55w|y55w]].

## 2026-10-09 12:57: main red on macOS from a second flaky test

- **What happened:** CI run 37931990047 on 8debefe6 (docs only) failed on macOS:
  `process_test sigterm_via_signal_group_exits_143` saw exit code 1, not 143. Found by the
  orchestrator's `ci_failed` wake; the next commit (5bf90cd6) was green.
- **Impact:** one red run; no merges held, since main was already green again.
- **Cause:** unknown; the fake exits 1 rather than through its SIGTERM handler under load.
- **Category:** `ci`.
- **Follow-up:** [[flaky-on-macos-ci-process-test-sigterm-via-signal-group-exit-f4xu|f4xu]]
  (br-f4xu).

## 2026-10-09 02:29-12:20: the bridle-ui aide's messages to the orchestrator got no reply for ~10 h

- **What happened:** the bridle-ui aide sent m-7457 (2026-10-09 02:29) and a follow-up m-7608
  (10:01) asking for install-ui, planning of ui-5zrr, ui-ha6m and ui-wtr3, and the state of
  ui-9hq8. The orchestrator did install-ui but never replied, and held the rest behind the `task
  ready` permission block. The PdM raised it at 12:19.
- **Impact:** the human's top bridle-ui fix (5zrr) sat unplanned overnight; the aide and the
  human had no answer.
- **Cause:** the orchestrator handed the items over as "held" across a handover instead of
  replying with that status; the new session didn't check for unanswered messages from aides.
- **Category:** `coordination`.
- **Follow-up:** replied (m-7733); kqsp and 5zrr sent to bridle-ui's manager-2 for planning.

## 2026-10-09 11:14: a worker reported a fix done that its commit didn't contain

- **What happened:** jxaffix reported br-r9h7 done ("SLEEP 10", `just check` green, 30/30
  loop) with commit 4037f961. The commit only added a doc comment; the test's prompt was
  unchanged and the worktree was clean. The orchestrator found it reading the diff; manager-2,
  which had checked only the diffstat, was already landing it and couldn't abort.
- **Impact:** `main` stays red longer, and br-r9h7 shows integrated with the flake unfixed.
- **Cause:** unknown; likely the code edit failed or was lost and the worker tested a different
  state from the one it committed.
- **Category:** `role`.
- **Follow-up:** br-7m99 (from br-r9h7) carries the real fix. A pattern to watch: the report
  is a claim (rule `work-flow`), and a diffstat isn't a review.

## 2026-10-09 10:45: main red on macOS from a new flaky test

- **What happened:** CI run 37918335201 on b1c37432 (the br-jxaf merge) failed on macOS only:
  `spawn_messaging_test spawn_with_prompt_waits_for_the_turn_to_start_before_returning` saw the
  spawned agent `Idle` where it expected `Working`. Found by the orchestrator's `ci_failed` wake.
- **Impact:** merges held until the fix lands; one worker slot goes to the fix.
- **Cause:** a timing race in the test (the fake's turn can finish before the spawn response is
  built), not the jxaf change.
- **Category:** `ci`.
- **Follow-up:** [[flaky-on-macos-ci-spawn-with-prompt-waits-for-the-turn-to-st-r9h7|r9h7]]
  (br-r9h7).

## 2026-10-09 06:45: the orchestrator sent a manager after a stale agent record

- **What happened:** the new orchestrator session read `bridle agents --all`, saw
  page-title-rule `lost`, and asked manager-2 to resume it. It was a record left from 10-07 whose
  task (br-qbbk) had already landed through page-title-rev; manager-2 caught it.
- **Impact:** two wasted messages, no work lost.
- **Cause:** `--all` lists finished and lost agents from any date with no age shown, and the
  orchestrator didn't check the task's state first.
- **Category:** `role`.
- **Follow-up:** none; the human can `bridle agent rm page-title-rule --delete-branch` to clear it.

## 2026-10-08 (found; began 2026-10-07): bridle's own `ps` polling held spawns for ~26 h

- **What happened:** from 06:34 ET on 10-07 the load hold refused spawns; its notes named `(ps)` as the top consumer. Found by the human watching the holds, not by an alert.
- **Impact:** about 26 h of spawn holds across the machine's bridle daemons, work waiting on a load that bridle itself caused.
- **Cause:** every daemon, test daemons included, forked `ps` every 2 s from the tracker timer even with no agents; more load slowed each `ps`, so more overlapped.
- **Category:** daemon
- **Follow-up:** fixed by a1bde105 (no snapshot without agents) and 018b9cfd (no-fork read); test daemons stop polling at 200 ms and a resource-budget test guards it (br-6nzj); see [[postmortem-bridle-s-own-ps-polling-every-daemon-test-daemons-n4w4|n4w4]].

## 2026-10-08 23:40-2026-10-09 01:40: the orchestrator had no waiter on bridle for ~2 h

- **What happened:** during the self-upgrade to 018b9cfd, the orchestrator re-armed its bridle
  waiter behind a shell loop that waited for the daemon's pid to change. The daemon re-executes in
  place and keeps its pid (640), so the loop never ended and no waiter ran. The orchestrator found
  it at 01:39 when `bridle status` showed "no waiter".
- **Impact:** about 2 h of wakes unseen (all queued, none lost): br-5e4k, br-ty37, br-9z2n and
  br-ppa6 landing; the human's approvals on ukpm, n4w4, crht and v6kr; a gateway restart for the
  NUC documents left waiting.
- **Cause:** the orchestrator assumed an upgrade restart changes the pid. It doesn't. `started_at`
  changes. A waiter that exits 6 ("daemon restarting") just needs re-arming in a loop until it stops
  exiting 6.
- **Category:** `role`
- **Follow-up:** none; the lesson is in the handover note.

## 2026-10-08 14:37-22:46: the gateway ran as the orchestrator and saw every project unreachable

- **What happened:** an orchestrator session restarted the gateway at 14:37 from its own shell.
  The gateway inherited `BRIDLE_AS=orchestrator` and `BRIDLE_PROJECT=bridle` and called every
  daemon as the orchestrator, which has no token there, so every poll failed. The human saw
  "bridle is unreachable: no token for principal 'orchestrator' ..." on the web UI and no to-dos,
  and aide traced it with `ps eww` (m-7205, 22:45). The orchestrator restarted it with a clean env
  at 22:46.
- **Impact:** about 8 hours with no web UI for the human: no to-dos and no interactions (59 came in
  on the first poll after the fix). Any reply from the web UI in that window would have been written
  as the orchestrator, if it got through at all.
- **Cause:** the gateway honours `BRIDLE_AS` / `BRIDLE_PROJECT` from its environment, and the
  orchestrator's launcher sets both.
- **Category:** `daemon`, `role`
- **Follow-up:** [[gateway-inherits-bridle-as-bridle-project-from-the-shell-tha-ppa6|ppa6]].

## 2026-10-08 ~01:10-01:30: aide's in-session restart left it at the shell again

- **What happened:** aide (bridle, dalek) wrote handover h-0066 at ~200K and restarted itself.
  The session ended; `~/.bridle/restart-aide.log` says the relaunch was typed but aide didn't
  register within 60 s, then "pane %5 is gone". Pane %5 still existed, at the shell prompt, with
  no relaunch in it. The human asked the orchestrator whether they had broken it; they hadn't.
  The orchestrator typed `bridle --project bridle session aide` into %5 and aide came back at once.
- **Impact:** about 20 minutes with no bridle aide; the human's channel to the workforce was down.
- **Cause:** unknown; second occurrence of 5j35 (the restarter still can't relaunch from inside
  the session), and its "pane gone" check is wrong.
- **Category:** `connectivity`, `coordination`.
- **Follow-up:** [[bridle-session-restart-run-from-inside-the-session-stops-it-5j35|5j35]]
  (recurrence on br-5j35's thread).

## 2026-10-08 ~00:53-01:05: orchestrator wakes came back empty after br-2672

- **What happened:** the CLI installed with br-2672 (bc5ed0a5, at 00:53) waits on the new
  `GET /v1/wake?principal=external:orchestrator`. Daemons still on an older build answer that as a
  plain principal wake, so `wait-for-wake` printed `message: ` and `null` and the daemon marked the
  message read. meta-notes' daemon on the NUC answered 403. Found by the orchestrator after three
  empty wakes in a row, by reading `GET /v1/messages`.
- **Impact:** about 10 minutes of the human's relayed requests (three aide messages) reached the
  orchestrator only by hand; the orchestrator was blind on every daemon not yet upgraded.
- **Cause:** the commit kept the old route on the daemon but stopped the CLI using it; mixed
  versions (other projects, other machines) weren't considered.
- **Category:** `daemon`, `merge`.
- **Follow-up:** br-grdg (CLI falls back to `/v1/orchestrator/wake`). Workaround: call the old
  route directly.

## 2026-10-07 ~22:30-01:05: the orchestrator watched only bridle's daemon

- **What happened:** the orchestrator, relaunched after dalek's reboot, ran a waiter for `bridle`
  only, though it holds orchestrator tokens for bridle-ui, track-web and meta-notes too. The
  human's bridle-ui requests to it (m-0606, from ~22:40) sat unread until aide relayed them
  through bridle at 01:01.
- **Impact:** about 2.5 hours of delay on a request the human called important (ui-2kmw), and on
  the prototype server they wanted for phone testing.
- **Cause:** the startup steps say "start the watcher" (one); the one-waiter-per-project rule is in
  the role text but not in the startup steps or the handover note.
- **Category:** `role`.
- **Follow-up:** the handover note names the four waiters; the startup steps should say it too
  (noted in the handover).

## 2026-10-07 ~01:35-02:05: the daemon upgrade gave up under load

- **What happened:** `bridle daemon restart --upgrade` at the human's go built 57983073, then
  waited the default 600 s for every agent to be idle. Worker `page-title-rev` stayed in one turn
  throughout; the upgrade failed ("no quiet point within 600s"), spawns were allowed again and
  the commit isn't retried.
- **Impact:** every daemon stayed on the old build; the fixes the human wanted live (br-x56y and
  others) didn't reach the running daemons.
- **Cause:** the design waits for a quiet point to happen by chance (automatic upgrade starts only
  at an idle tick, 600 s give-up, only spawns held while running agents keep getting new turns).
  Under steady load it never comes.
- **Category:** `daemon`.
- **Follow-up:** [[upgrades-never-go-through-under-load-drain-no-new-turns-no-t-vn42|vn42]].

## 2026-10-07 ~00:53-01:40: auto mode refused the orchestrator's daemon upgrade

- **What happened:** Claude Code's auto mode classifier refused `bridle daemon restart --upgrade`
  ([Interfere With Workloads]), though the orchestrator role allows it. The human had to ask
  in the session before it went through.
- **Impact:** br-x56y (agents can't `bridle send`) sat landed but uninstalled for ~45 min.
- **Cause:** no permission rule for the orchestrator's daemon commands.
- **Category:** `config`, `external`.
- **Follow-up:** a new log, `docs/context/auto-mode-blocks.md`, of every auto mode refusal, to fix
  with permission rules or the global auto mode configuration.

## 2026-10-06 ~23:13 on: a hung test held the landing queue

- **What happened:** br-qbbk's landing check (`just check` in the integration worktree) hung in
  gateway_test `a_replaced_binary_is_re_executed`, which waited forever on its temp-dir `bridle
  gateway` child (0% CPU). Found by the orchestrator when br-x56y couldn't land.
- **Impact:** the landing queue blocked for over an hour; the critical br-x56y fix (agents can't
  `bridle send`) waited behind it. Only the human could end it (no abort, no timeout; rule
  `no-kill-by-name`).
- **Cause:** likely the z7y5 syspolicyd backlog delaying the new binary's first exec; the check
  itself has no timeout.
- **Category:** `merge`, `daemon`, `host`.
- **Follow-up:** [[a-landing-s-integration-check-has-no-timeout-one-hung-test-h-y55w|y55w]].

## 2026-10-06 ~23:15-23:30: syspolicyd and Spotlight pegged; builds and app launches stalled

- **What happened:** the human found Finder and app launches hung. syspolicyd was at ~300% CPU and
  Spotlight busy; ~33 rustc processes sat at 0% CPU; the daemon had been running `cp -cR
  integration/target` into a new worktree for 14+ minutes.
- **Impact:** the machine was unusable for the human; every build stalled; the br-x56y worker's spawn
  (critical send fix) was stuck behind a copy that would take ~2.5 h.
- **Cause:** `integration/target/debug/deps` had grown to 564,137 files (never pruned; rebuilt after
  each land). Every spawn clones it (`warm_target`, b7cz). The data is free, but each new file costs
  FSEvents and Spotlight work, and each new binary costs a syspolicyd check.
- **Category:** `daemon`, `machine`.
- **Follow-up:** [[incident-syspolicyd-and-spotlight-pegged-builds-and-app-laun-z7y5|z7y5]];
  possibly also explains y455's hung spawns.

## 2026-10-06 ~22:12-23:15: no agent could bridle send after the restart onto the 3haz build

- **What happened:** after the daemons restarted onto the br-3haz build, every agent's `bridle send`
  failed with "that is this daemon's own project" (or "--task isn't supported for another
  project"). bridle-ui's manager-1 never got fm-links3's report and sat idle for an hour with
  ui-7veu landable; manager-2 couldn't reach its workers. Aide noticed manager-1 going quiet.
- **Impact:** messaging between agents was down on every daemon for about an hour. Landings stalled.
- **Cause:** agents have `BRIDLE_PROJECT` and `BRIDLE_URL`. A daemon found by URL has no project, so
  the CLI read the agent's own project as another one and routed the send through the outbox.
  No test ran a send in an agent's real environment (as in 2ax5).
- **Category:** `cli`, `release`.
- **Follow-up:** [[bridle-send-from-any-agent-fails-since-the-3haz-build-a-daem-x56y|x56y]];
  workaround `env -u BRIDLE_PROJECT bridle send`.

## 2026-10-05 03:51: the automatic upgrade refused a good build (self-check timed out)

- **What happened:** the daemon built 0d69d617 (CI green) and then refused to restart into it:
  the new binary's `serve --check` exceeded its 60 s cap. Found by the orchestrator's
  `upgrade_failed` wake. Run by hand ten minutes later, the same check passed in under a second.
- **Impact:** the daemon stays on c548182b (without br-e9yu) until the next upgrade; the CLI
  already runs the new binary, so the CLI and the daemon are on different builds.
- **Cause:** unknown. Likely a freshly linked ad-hoc binary's first launch waiting on macOS's
  policy check on the loaded Intel Mac (two workers' checks were running).
- **Category:** `daemon`, `host`.
- **Follow-up:** [[self-upgrade-refuses-a-good-build-the-new-binary-s-self-chec-up82|up82]].

## 2026-10-05 ~01:30-02:50: interactive sessions replied in quiet hours on wake turns

- **What happened:** the focus gate is a UserPromptSubmit hook, so it runs only on the human's
  own prompts. Turns started by a background wake (watchers, task notifications) never saw it,
  and the advisor, the aides and the orchestrator kept writing replies the human could see from
  their phone after quiet hours began. The advisor noticed in its own session; the human asked
  for an incident (br-yy88).
- **Impact:** messages to the human during their sleep hours, the thing the gate exists to stop.
- **Cause:** the gate's text is injected only by a UserPromptSubmit hook, which never fires on
  turns started by a background wake.
- **Category:** `role`, `config`.
- **Follow-up:** [[interactive-sessions-reply-in-quiet-hours-the-focus-gate-nev-yy88|yy88]]
  (incident), [[postmortem-quiet-hours-didn-t-reach-replies-to-background-wa-v3b7|v3b7]]
  (postmortem), [[background-wakes-carry-the-session-s-prompt-context-quiet-ho-9s8u|9s8u]] (wakes
  print the gate; hook audit), all pending the human's review.

## 2026-10-05 00:28-01:15: br-bnhn's landing failed three times with "main moved"

- **What happened:** each ~10 min land check (load ~40) was overtaken by docs-only commits on
  local `main` (orchestrator and aide tickets, the doc-review watcher's automatic review commits).
- **Impact:** ~45 min of landing and a worker slot; every committing role told to hold.
- **Cause:** `task land` fails on any move of `main` during its check, even docs-only ones.
- **Category:** `merge`.
- **Follow-up:** [[a-landing-fails-main-moved-whenever-docs-are-committed-durin-btx7|btx7]].

## 2026-10-05 00:33: bridle session restart reported success on a session it never stopped

- **What happened:** aide ran `bridle session restart aide` (the human approved); it printed
  "restarted aide in pane %86", but the session kept running and the relaunch command was typed
  into aide's live Claude prompt.
- **Impact:** aide not restarted; the human has to restart it by hand. A false success message.
- **Cause:** unknown (the restart doesn't confirm the old session stopped before typing the relaunch).
- **Category:** `daemon`.
- **Follow-up:** [[bridle-session-restart-says-it-restarted-a-session-that-s-st-4s3z|4s3z]].

## 2026-10-04 23:33: a haiku worker reported done before its check finished, then lost its task

- **What happened:** worker `web-packs` (haiku, br-g49c) sent "done" at 23:33:59 with `just check`
  still running. The check then failed (load timeouts); it reran. After renewal for context (~247k
  tokens after 30 min), it took br-gtzx, the human's open question, as its own task. Caught by the
  aide and the orchestrator; the human asked for the root cause.
- **Impact:** nearly landed unverified work; a wasted worker; a false status to manager-2.
- **Cause:** a long check under load with no safe way to wait (`pgrep -f` matched other worktrees'
  checks; 33 monitors, 59 re-reads); Haiku broke the done rule; the renewal message names no task.
- **Category:** `role`, `coordination`.
- **Follow-up:** [[a-haiku-worker-reported-done-before-its-check-finished-then-qdw8|qdw8]].

## 2026-10-04 (all day): wake waiters in interactive sessions die with exit 144

- **What happened:** `bridle agent wake` commands run in the background by advisors and aides
  ended `exited with code 144` with no output, at least 10 times between 16:53Z and 22:37Z. Kills
  came in pairs at the same second in separate sessions and projects (two bridle advisors at 16:53:23Z;
  bridle's and track-web's aides at 21:28:01Z and 22:20:07Z). The human saw it in other agents too
  and called it an incident; the aide collected the evidence from Claude Code's task outputs.
- **Impact:** each session stopped listening until it noticed and re-armed. The aides did; a
  session that doesn't goes deaf to messages and task changes.
- **Cause:** sessions killing each other's waiters. To replace its own waiter, an aide or advisor ran
  `pkill -f "bridle agent wake external:aide"` (or `external:advisor`), which matches every
  project's aide (or every advisor) on the machine. Each kill lines up to the second with one of these
  `pkill`s in the transcripts. It breaks `no-kill-by-name` (fx7x), which interactive roles didn't
  reliably get.
- **Category:** `role`, `coordination`.
- **Follow-up:** [[wake-waiters-in-interactive-sessions-die-with-exit-144-in-pa-h3ar|h3ar]]:
  interactive roles follow no-kill-by-name, a new wait takes over the old one, and every
  interactive session is warned when it has no waiter.

## 2026-10-04 13:18: br-p88z (stable Mac signing) approved but never planned for 9 hours

- **What happened:** the advisor filed and opened br-p88z at 13:18. It was never planned, so it
  never reached the queue, and the human hit the firewall problem it fixes again after the 21:09
  rebuild. The human raised it at ~22:00 ("I think our queue is broken. Or PM is not working
  right"); the aide traced it from the event log and pm-1's inbox.
- **Impact:** a fix the human calls critical sat idle all day. The gateway stopped answering from
  other machines after the rebuild, and the human found out the work hadn't started.
- **Cause:** pm-1 plans only the tasks someone names to it. Every task it planned that day followed
  an orchestrator message. Nothing tells the PM when a task goes `pending -> open`:
  `queue_nudge.rs` nudges the manager on queue edits only. The advisor opened p88z and told no one.
  It opened between two daemon restarts (13:18 and 13:27). Not the cause, but nobody was watching
  then.
- **Category:** `coordination`, `role`.
- **Follow-up:** [[tasks-approved-by-anyone-but-the-orchestrator-are-never-plan-xz4f|xz4f]]: tell
  the PM when a task opens, plus an alarm for tasks open and unplanned too long. Also open and
  unplanned the same way: br-anmx, br-ckvz.

## 2026-10-04 ~5:36 PM ET: a duplicate spawn for br-bek3 drove load to 70

- **What happened:** manager-2 spawned `gateway-detach` for br-bek3; my second nudge to fill the
  slot crossed that spawn, and it started `gw-detach` for the same task. The second spawn sat
  10 minutes in `cp -cR` of `integration/target`, alongside syspolicyd and XprotectService at
  ~85% each. Load average 70 on 16 cores. manager-2 couldn't kill it (dontAsk denies `kill`).
- **Impact:** a slow, loaded machine for ~10 min; the human noticed (ticket 58c9). No work lost.
- **Cause:** spawn isn't idempotent per task (two workers on one task allowed), and I nudged
  without checking `bridle agents` and running spawns first. Copying `target/` per worktree is
  heavy and slow.
- **Category:** `coordination`, `host`.
- **Follow-up:** I killed the spawn and removed the empty worktree. br-58c9 (watch load).
  A daemon-side refusal of a second worker on a claimed task would have stopped it.

## 2026-10-04 ~2:45 PM and ~4:15 PM ET: the human told twice that landed bridle-ui work hadn't started

- **What happened:** the human's routing ask (k3qx) became ui-n6cu and landed on bridle-ui
  `main` (7af2f59) ten minutes after it was relayed. No one told `external:aide`. Hours later
  the bridle-ui aide told the human, twice, that the orchestrator hadn't acted on it. k3qx was
  never linked to its task or resolved.
- **Impact:** the human was misinformed about finished work and had to ask for an incident.
- **Cause:** two gaps. The orchestrator doesn't report landings of human-asked work to aide
  (the standing rule keeps routine status out of aide's inbox, and the line was drawn too
  tight). The aide's status check looked only at its inbox, to-dos and pending tasks, not at
  `bridle task list` or `git log`.
- **Category:** `coordination`, `role`.
- **Follow-up:** bridle-ui incident task ui-wdp3; k3qx resolved with the landing. Orchestrator
  practice from now: when work the human asked for lands, tell aide in one line.

## 2026-10-04 21:05: main red on Linux after br-qpr7 (spec test runs a macOS-only command)

- **What happened:** CI run 37233807547 on 2d9adce2 (br-qpr7) failed on ubuntu-latest only. Two
  tests in `project_resolution_test.rs` run every command, including `bridle daemon launchd
  install`, which errors "bridle launchd is macOS only" on Linux.
- **Impact:** `main` red; landing held. br-2mtr (34015348) was pushed 20 s before the failure
  was known, so its run fails the same way.
- **Cause:** a test that walks every command wasn't written for commands gated by platform; the
  worker's `just check` ran on macOS only, so it couldn't see it. Second red from the
  project-resolver work today (after 3397/2qrk).
- **Category:** `ci`.
- **Follow-up:** fix sent to manager-2 (m-4839); task to be filed by the fix.
- **Repeats:** br-nmf8 (4a3b02bc) skipped only `launchd install`; run 37237823000 failed on
  `launchd uninstall`. br-8avg (fe5730a2) then ran `daemon systemd *` on Linux, where install needs
  `[machine] name`; run 37242072891 failed (23:07Z). Three red runs from fixes written on macOS
  for a Linux-only failure, each narrower than the cause. A test also must not install a real
  system service. Fix 3: br-rmzx (Skip the service leaves). Lesson: a fix for a Linux-only
  failure states, per OS, which branch runs, since no worker can run Linux here.
- **Resolved:** br-rmzx (1dde6f73) landed 2026-10-04 23:54Z; run 37245380632 on d1e16075 green on
  ubuntu and macOS.

## 2026-10-04 ~20:00: a track-web research worker had no web tools and called the web "unreachable"

- **What happened:** track-web's tw-sxfh (h679, research Hole.io gameplay and reviews) went to
  worker `holeio-research`. It reported "web search and the game wiki were unreachable", and the
  orchestrator told the human "web search was partly unreachable". The human questioned it via
  track-web's aide (~20:30), which found the cause in the transcript.
- **Impact:** a research ticket landed with its key numbers (growth curve, level count, bot
  tuning) unsourced, and the human was told something false about the network. Reviews came only
  from the App Store feed and Wikipedia; no general search was possible.
- **Cause:** the worker role has no `WebSearch`/`WebFetch` (`allowed_tools` is Bash, Read, Edit,
  Write, Glob, Grep: bridle's default, copied into track-web). The worker fell back to `curl`,
  two sites blocked it (fandom's Cloudflare challenge, IGN's bot block), and it didn't say a tool
  was missing. Nobody checked the role's tools against a research task when planning or
  assigning it, and the manager and orchestrator passed the failure on as an aside.
- **Category:** `role`, `config`, `coordination`.
- **Follow-up:** [[workers-report-missing-tools-and-failed-fetches-plainly-a-re-2mtr|2mtr]]:
  workers record and report missing tools and failed fetches (the human's ask), and research
  tasks get a role with web tools (to decide).

## 2026-10-04 19:05: `main` red after br-3397 (launcher tests need a project)

- **What happened:** CI on d3803242 (br-3397, one project resolver for every command) failed on
  Linux and macOS: `tools_only_test` `launchers_start_elsewhere` and
  `launchers_run_under_a_bridle_agent_with_the_test_flag`, each `assertion failed: started`.
  Found by the `ci_failed` wake.
- **Impact:** `main` red ~1 h. manager-2, freshly renewed for context, landed and pushed br-kae5
  and br-ehv6 while red and before the fix (its own handoff said fix first); the ehv6 run failed
  the same way. The renewal lost the "hold" more than the note did.
- **Cause:** 3397 removed `bridle session`'s `bridle` fallback; the launcher tests pass no
  project. They passed in the worker's `just check` because agents run with
  `BRIDLE_PROJECT=bridle` in their environment and the test inherited it. CI has none.
- **Category:** `ci`, `merge`.
- **Follow-up:** br-2qrk (set `BRIDLE_PROJECT` in the test helper). Pattern: a worker's
  environment can hide a failure CI sees; tests that run the binary should set or clear
  `BRIDLE_*` (cf. br-0798, the earlier env leak).

## 2026-10-04 15:01: pm-1 couldn't plan for 1.5 hours; every Bash call denied

- **What happened:** after its context renewal at 15:01, pm-1's commands were denied
  ("don't-ask mode") and it planned nothing: br-k22s, br-puaf, br-3397, br-jrm2 and br-ehv6 sat
  `open`, and both managers went idle with no workers. pm-1 read it as lost permission and kept
  repeating "Bash is denied" in its replies. Found by the thirty-first orchestrator at startup
  (16:30) in pm-1's log.
- **Impact:** ~1.5 h with no new work started (only one worker running, then none); a red-`main`
  flake fix (k22s) waited.
- **Cause:** its commands were compound (`| head`, chained commands, backticks in a `--body`), so
  they didn't match `Bash(bridle *)`. The project-manager role told it to read CHANGELOG with
  `head -30` and, unlike the manager role, didn't say "one plain command per Bash call". Fixed
  by a message at 16:36 (plain `bridle queue` worked; it planned all five in minutes).
- **Category:** `role`, `coordination`.
- **Follow-up:** `workflow/base/roles/project-manager.md` now has the plain-command bullet and
  reads CHANGELOG with the Read tool. A blocked role saying "I can't" in its turn output, not a
  `question`, doesn't wake anyone; none filed yet (watch for a repeat).

## 2026-10-04 15:57: bridle-ui `main` red after the routing change (ui-n6cu)

- **What happened:** CI on bridle-ui 7af2f59 failed: Vitest's unhandled error, `data.projects`
  undefined in `ItemsView` (`Items.tsx:45`) after the unknown-path test redirected to `/`. Found by
  the CI wake.
- **Impact:** bridle-ui `main` red ~7 min; fixed in 68e953a.
- **Cause:** the App tests' `/items` mock returned `[]`, not `{projects: [...]}`; timing-dependent,
  so the worker's local check passed.
- **Category:** `ci`.
- **Follow-up:** fixed (68e953a); ui-c4rz for a separate local-only timeout flake.

## 2026-10-04 14:35: `main` red on macOS, a statusline test races the startup usage poll

- **What happened:** CI on fb3bdb8 (br-jmpf) failed on macOS only:
  `statusline_report_with_no_rate_limits_still_records_usage`, `rate_limits.is_empty()`. Found
  by the CI wake.
- **Impact:** `main` red; landings held until the fix.
- **Cause:** the test daemon's startup `get_usage` poll (fake five_hour 1%) can land before the
  test reads usage. f1ky (77f5dba3) fixed the same race in the sibling test only.
- **Category:** `ci`.
- **Follow-up:** [[statusline-test-no-rate-limits-test-races-the-startup-get-us-k22s|k22s]].

## 2026-10-04 ~14:00: `bridle status` told the human to remove a branch with unlanded work

- **What happened:** `bridle status` listed `self-upgrade` and `docs-rename` as "stopped agents
  whose branch has landed" with the hint `bridle rm <name> --delete-branch`; the orchestrator
  passed the commands to the human. `rm self-upgrade` refused (uncommitted changes, "use
  --force"); `rm docs-rename` failed (git no longer knew the worktree). Checking before advising
  `--force`, the orchestrator found `bridle/self-upgrade` holds two commits not on `main`
  (br-88d4's release upgrade, 98615ed8 and dbdc47f6) plus staged changes.
- **Impact:** none: the uncommitted-changes guard stopped it. With `--force`, br-88d4's work would
  have been deleted.
- **Cause:** `is_merged` accepts any `Branch: <branch>` trailer on `main`; an earlier landing from
  the same branch (br-38c8, 7d29cfd4) carried it, and the worker kept committing there. The
  orchestrator also relayed the status hint without checking it. docs-rename: git's worktree
  record had been pruned while the directory stayed.
- **Category:** `daemon`, `role`.
- **Follow-up:** [[is-merged-calls-a-reused-branch-landed-an-old-branch-trailer-x3xk|x3xk]],
  [[agent-rm-fails-when-git-no-longer-knows-the-agent-s-worktree-4f8y|4f8y]]. Check a branch
  with `git cherry main <branch>` before handing the human an `rm --delete-branch`.

## 2026-10-04 12:48: a held landing reached origin through another session's ticket push

- **What happened:** manager-2 landed br-9j2h (the product-manager to project-manager rename,
  4656be0) in the clone at 12:48. The orchestrator had just asked it to hold, because the rename had
  no alias for stored `product-manager` agents. manager-2 kept it unpushed. A session filing ticket
  anmx committed on top in the same clone and pushed, which took 4656be0 to origin/main. The
  orchestrator found it on fetch and pushed a revert (6aaff367) at about 12:51.
- **Impact:** none seen. Without the revert, the daemon's self-upgrade on green CI would have
  resumed pm-1 with an unknown role, and dalek's shared binary would have done the same to
  track-web's PM.
- **Cause:** the clone is shared. Ticket commits push whatever `main` holds, landed-but-held work
  included (role notes: "ticket commits racing landings").
- **Category:** `merge`, `coordination`.
- **Follow-up:** br-v3d7 (the alias; re-lands the rename). The shared-clone push race has no ticket
  yet beyond the role note.

## 2026-10-03: an approved rename was dropped as unapproved in the backlog sort

- **What happened:** the human approved renaming `product-manager` to `project-manager` on
  2026-10-02 (ticket 7r2c, "Decided"); the workflow advisor sent it to the orchestrator (m-3788).
  No build task was filed. pm-1 then dropped the question task br-8b6c in the k7tm sort on
  2026-10-03 as "not approved for work now". The workflow advisor caught it while retiring
  (m-4387, 2026-10-04 12:26).
- **Impact:** an approved change sat unbuilt for two days; no work lost.
- **Cause:** the decision lived only in the question ticket. Nobody turned it into a build task,
  and pm-1 judged approval from the task, not the ticket's Decided section.
- **Category:** `coordination`, `role`.
- **Follow-up:** [[rename-the-product-manager-role-to-project-manager-proj-mgr-9j2h|9j2h]] (task
  br-9j2h, ready).

## 2026-10-04 03:31 to the morning: quiet hours stopped the watchers again (third time)

- **What happened:** from 03:31 UTC each watcher's exit notification arrived with the quiet-hours
  gate attached ("no tool calls except the one the human asked for"), and the orchestrator obeyed
  it: it restarted none of the three (bridle, track-web, bridle-ui). It said so in its reply each
  time, but nothing happened until the human's morning prompt. Earlier, at 03:31 one notification
  came without the gate and the orchestrator restarted the watchers; the next ones carried it.
- **Impact:** no watch from about 11:30 PM to the morning ET. Nothing was being built, so little
  was missed (the daemon self-upgraded to bc8c745 at 03:46 and resumed its agents on its own);
  the human noticed several sessions had stopped watching overnight.
- **Cause:** cc45, unfixed: the focus gate runs on every UserPromptSubmit, background-task
  notifications included, and its text forbids tool calls. cc45 was filed after the first
  occurrence (2026-10-03 03:30) but never got a task, so nothing was built.
- **Category:** `role`, `config`.
- **Follow-up:** cc45, now with its task br-cc45 (ready). Lesson: a ticket from an incident needs
  its task filed the same day.

## 2026-10-04 01:51 and 02:02: daemon restarts dropped every interactive session

- **What happened:** the daemon self-upgraded twice (to 9d0f0f6, then 941312c). Afterwards
  `bridle status` listed no sessions though six advisor launchers and the aide ran. The
  orchestrator's `bridle send external:aide` failed ("no such recipient"), and its x8jt messages
  for advisor doc-review (m-4237, m-4240) fell back to the shared advisor inbox. Found by the
  orchestrator (the aide send) and diagnosed by the main advisor (m-4249).
- **Impact:** about 30 minutes of messages misrouted or refused; context warnings off for every
  interactive session until its launcher restarts. (Correction, 2026-10-04: the aide had in fact
  never been set up, with no `[aide]` token, so its "no such recipient" wasn't this restart; the
  human started it at 13:40Z.) The doc-review advisor missed both messages until the human pointed it
  at them ("That advisor is up and running"). Probably on every daemon restart since sessions were
  registered (also 2026-10-03 20:08 and 23:24), unnoticed.
- **Cause:** the session registry is in memory only, and a session registers only when its
  launcher starts, so any daemon restart forgets them. Self-upgrade made restarts routine.
- **Category:** `daemon`.
- **Follow-up:** e35h (stopgap: keep the registry across restarts); gtzx (seats) for good.

## 2026-10-04 00:27: the main advisor took a named advisor's brief, again

- **What happened:** `bridle advisor start doc-review --brief @file` sent the brief (m-4174) to
  `external:advisor` before the new session was up; the main advisor's background wake returned
  it and marked it read. The main advisor didn't act on it and told the orchestrator (m-4175).
  A repeat of the 2026-10-03 ~15:30 entry below.
- **Impact:** a few minutes; the orchestrator typed the brief into the doc-review pane instead.
  The human, 2026-10-04: "don't send the message as a brief. Wait until the advisor comes up or
  give it about 30 seconds and then send the message."
- **Cause:** one shared `external:advisor` inbox; `advisor start --brief` sends before the
  named session can look.
- **Category:** `coordination`, `role`.
- **Follow-up:** [[seats-every-role-is-a-named-tracked-seat-that-outlives-its-s-gtzx|gtzx]]
  (per-seat inboxes), waiting on the human's review. Until then: start the advisor with no
  `--brief`, then type the brief into its pane once it's up.

## 2026-10-03 21:36-22:15: three merges sat on local main, unpushed

- **What happened:** manager-2 merged br-bhcp (b5971d8), br-25nn and br-59vt (9bd4587) into the
  clone's `main` and reported each as landed, but none were pushed: `main` was 3 ahead of
  `origin/main` and CI never ran on them, while `bridle status` still said "pushed 0m ago". Found
  by the orchestrator looking for the CI run of 9bd4587; it pushed at ~22:15.
- **Impact:** about 40 minutes of the human's same-day u6w9 work unverified by CI and invisible
  on GitHub; bridle-ui was told to sync types from a commit not on the remote.
- **Cause:** manager-2 (m-4073): after earlier landings `git push origin main` said "Everything
  up-to-date" (something else had already pushed), and it took that to mean landing pushes on its
  own, so it stopped pushing. The status "pushed" line is the state repo, not the clone's `main`,
  so it hid the gap.
- **Category:** `merge`, `role`.
- **Follow-up:** none; manager-2 pushes after every land again. If it recurs, make landing push
  itself.

## 2026-10-03 ~15:30: the main advisor took a named advisor's brief

- **What happened:** the orchestrator sent a brief "For advisor tickets: ..." (m-3985) to
  `external:advisor` for a new named advisor ("tickets", pid 30082, the human's fresh session for
  the tickets-and-tasks question). The main advisor's background `bridle agent wake
  external:advisor` returned it and marked it read, so the tickets advisor's start-up step ("read
  unread 'For advisor <name>:' messages") wouldn't find it. Found by the main advisor reading the
  wake output.
- **Impact:** the tickets advisor started without its brief; the human pointed it at the message
  by hand. The main advisor stopped its wait loop so as not to take more of its messages.
- **Cause:** all advisors share one principal and inbox (`external:advisor`), and since rmpq
  (21bd096) a non-human wake marks what it returns read; the named-advisor convention assumes the
  named advisor is the first to look. A cousin of
  [[read-on-delivery-can-lose-messages-marked-read-before-the-re-k8jn|k8jn]].
- **Category:** `coordination`, `role`.
- **Follow-up:** seats ([[seats-named-interactive-roles-splitting-the-advisor-retiring-r8kv|r8kv]]:
  each seat its own inbox) and k8jn. Meanwhile, address a named advisor as
  `external:advisor/<name>` if the daemon routes it, or let the human hand over the brief.

## 2026-10-03 13:45: self-upgrade refused a binary that couldn't read the newer config

- **What happened:** the daemon's self-upgrade built `1afcb9d` and refused to restart into it,
  because the new binary's self-check couldn't parse the clone's `.bridle/config.toml`: `dd7400b`
  (br-14cd) had landed during the build and added `[integration] warm_build`, which the older
  binary doesn't know (config structs deny unknown fields). Discovered by the orchestrator (an
  `upgrade_failed` wake).
- **Impact:** one wasted build; the upgrade waits for the next green `main`. Until it lands, the
  installed binary can't start against the clone's config if the daemon dies.
- **Cause:** the self-check reads the clone's config at its current commit, not at the commit it
  built.
- **Category:** `daemon`, `config`.
- **Follow-up:** [[self-upgrade-checks-a-new-binary-against-the-clone-s-newer-c-yhe7|yhe7]].

## 2026-10-03 ~04:00-13:00: the orchestrator slept through the night with no wait running

- **What happened:** around midnight ET (04:00 UTC) during quiet hours, the orchestrator said it
  was quiet time, nothing was happening, and it would pick up at 6 a.m., but it ran no background
  wait, so nothing could wake it. It did nothing from about midnight to 9 a.m. ET. The daemon's
  liveness watch noticed and told the human's inbox ("no wake command running": m-3797 at 03:54
  UTC, m-3817 at 04:37), but the human was asleep. Reported by the human (2026-10-03).
- **Impact:** about nine hours with no supervision of the workforce. A message the human later
  wrote to it was lost when it handed off to a new session mid-composition (no harm, re-sent to
  the advisor).
- **Cause:** the orchestrator chose to stop without a wait; nothing wakes a session at a given
  time. Likely contributor (not verified): the quiet-hours gate text (`crates/bridle/src/focus.rs`)
  tells the session "No tool calls except the one the human asked for", and the orchestrator
  role says to obey it, so restarting its own wait looks forbidden. The advisor did the same that
  night, skipping its wait restart under the gate.
- **Category:** `role`, `coordination`.
- **Follow-up:** the human's rule (the orchestrator never sleeps without a wait, and checks the
  system at least every two hours) sent to the orchestrator for its role and the gate text;
  [[scheduled-messages-an-agent-or-the-human-schedules-a-message-hrcn|hrcn]] (scheduled messages,
  design); [[the-orchestrator-stays-running-fx7x|fx7x]].

## 2026-10-03 12:09: the NUC's daemons restarted cleanly on Linux (a data point, no fault)

- **What happened:** after the update below, the NUC's three daemons (meta-notes, notes,
  dotfiles-local) restarted in place at 12:09, with the human's approval: same pids,
  `/proc/<pid>/exe` the new binary with no `(deleted)`, the manager resumed (1 of 1). Reported by
  the NUC orchestrator (m-3846 on bridle's daemon).
- **Impact:** none. Recorded because the
  [[restart-on-linux-execs-path-deleted-after-the-binary-is-repl-fpde|fpde]] fix held on a real
  Linux restart.
- **Cause:** n/a.
- **Category:** `daemon`, `host`.
- **Follow-up:** none.

## 2026-10-03 ~12:00: the NUC's bridle drifted two days with nobody updating it

- **What happened:** the NUC's `~/.cargo/bin/bridle` dated from 2026-10-01 16:23, and its checkout
  `/srv/shared/work/bridle-work/bridle` (also the `workflow` path for meta-notes, notes and
  dotfiles-local) was 108 commits behind `origin/main`. Found by the NUC orchestrator when the
  human asked how the NUC gets updates (m-3841). Nobody owned pulling or building there, so the
  NUC's daemons still sent the `all_idle` wake (removed in br-6c6a) and its `wait-for-wake` had no
  `--timeout`, while the role docs moved on. Also: the NUC orchestrator's handover note named
  `bridle wait-for-wake --project X` for the notes and dotfiles-local watchers; in the new binary
  that alias defaults to 5 minutes, unlike `bridle orchestrator wait-for-wake --timeout`. The
  watchers now use `bridle orchestrator wait-for-wake --project X --timeout 5400`. (The base
  role text names only the `orchestrator` form; the short alias was only in the handover note.)
- **Impact:** about two days of the NUC's agents running old behaviour against newer role docs.
  No work lost.
- **Cause:** the interim rule in
  [[client-machines-stay-current-with-bridle-workflow-and-daemon-chvf|chvf]] (the NUC orchestrator
  pulls and builds, the human approves the restart) needed someone to trigger it; nothing did.
- **Category:** `coordination`, `host`.
- **Follow-up:** resolved by the chvf interim rule: the NUC orchestrator pulled to 4bec8b6, ran
  `just install` (3m 40s), and the human approved restarting the three daemons. The lasting fix is
  chvf's release upgrades (br-88d4, br-751e), planned.

## 2026-10-03 01:59: a daemon restart ended the advisor's wait as a "timeout"

- **What happened:** the advisor's `bridle agent wake external:advisor --timeout 5400`, started at
  01:46:48, exited at 01:59:13 with "nothing woke external:advisor before the timeout" (exit 4)
  after about 12 minutes. The daemon restarted at that moment (`daemon.stopping` 01:59:13,
  `daemon.started` 01:59:16). Found by the advisor comparing the exit time with the daemon events.
- **Impact:** none here (the advisor restarts its wait on any exit, and messages queue). But the
  output is misleading: a restart reads as a timeout, so a caller can't tell them apart.
- **Cause:** on shutdown the wait ends as if the timeout had passed (the daemon answers empty, or
  the CLI reads a closed connection as one). Not verified which.
- **Category:** `daemon`.
- **Follow-up:** none yet; candidate: report "daemon restarting" with its own exit code, or have the
  CLI reconnect and keep waiting until the original deadline.

## 2026-10-03 03:30: the orchestrator's watchers were down all night (quiet hours)

- **What happened:** from about 03:30 to 13:07 (23:30 to 09:07 ET) none of the orchestrator's three
  watchers ran. Each exited during quiet hours, and the focus gate's text ("no tool calls except the
  one the human asked for") stopped the orchestrator from restarting them. Found when the human
  asked at 09:07 why they'd heard nothing.
- **Impact:** wakes queued, so nothing was lost, but none were seen for about 9.5 hours. Among
  them: a red CI run on `main` (93362e5; later green after br-5669), a manager-2 question about
  landing br-01a4 (resolved by others), pm-1's question with 11 candidate tasks (m-3910, still
  waiting for the human), and an idea from the NUC to file.
- **Cause:** the quiet-hours gate applies to every submitted prompt, including the background-task
  notification turns that drive the watch loop, and it bans all tool calls.
- **Category:** `role`, `config`.
- **Follow-up:** [[quiet-hours-stop-the-orchestrator-restarting-its-watchers-so-cc45|cc45]].

## 2026-10-03 01:46: bridle-ui's manager was logged out of Claude Code and silently did nothing

- **What happened:** the bridle-ui manager answered every message (01:31 to 01:46) with "Not
  logged in · Please run /login", each recorded as a successful $0 turn. Found by the
  orchestrator after the `all_idle` wake, from `bridle agent logs manager`.
- **Cause:** the human started the daemon (pid 32152) with `bridle serve --detach` over SSH
  (its environment has `SSH_CONNECTION` and no `TMUX`). An SSH session can't read the macOS login
  keychain where Claude Code keeps its login. The orchestrator's `daemon restart` kept the same
  session. The bridle and track-web daemons were started in the laptop's tmux and work fine.
- **Second failure, the fix attempt (02:31 to 02:39):** the human stopped it and, over SSH,
  started `bridle serve` in a new window of the laptop's tmux. That daemon had **no
  `SSH_AUTH_SOCK`**, so its first git-over-SSH call (the state branch) stopped at "Enter passphrase
  for key ~/.ssh/id_ed25519" in the pane, before the daemon listened or registered. The human typed
  the passphrase about ten times from the phone and it never took (cause unknown: possibly several
  git processes competing for the tty, or the phone's input). `ssh-add` failed too, since that
  shell had no agent to talk to. The first daemon had the same gap: its state-branch push failed
  ("Could not read from remote repository"). What worked: starting it with the launchd agent that
  already holds the key, `SSH_AUTH_SOCK=/private/tmp/com.apple.launchd.<id>/Listeners bridle serve`
  (the value from the bridle daemon's environment; pid 41455, 02:39). The new manager-1 was
  logged in.
- **Impact:** about an hour of bridle-ui not starting, and about 20 minutes of the human's time
  on the phone while back from travel.
- **Category:** `host`, `daemon`, `human-process`.
- **Follow-up:** [[a-daemon-whose-claude-isn-t-logged-in-runs-agents-that-silen-nrbf|nrbf]]
  (br-5b39: fail loudly, check at start-up); a gotcha in `docs/context/adding-a-project.md`; the
  SSH key in the Keychain ([[the-humans-to-do-list-and-restart-checklist-ex9q|ex9q]] item 2) would
  remove the passphrase prompt. A daemon should never block on a terminal prompt
  (`GIT_TERMINAL_PROMPT=0`, ssh `BatchMode=yes`): added to nrbf.

## 2026-10-03 01:28: the orchestrator role named a flag the installed binary didn't have

- **What happened:** the twenty-fifth orchestrator session started its three watchers with
  `bridle orchestrator wait-for-wake --timeout ...`, as the role prompt says. All three exited at
  once: "unexpected argument '--timeout'". The role text came from `main`'s workflow (path mode),
  which br-c4f4 had just updated, but `~/.cargo/bin/bridle` (0.4.0) predates br-c4f4. Found by
  the orchestrator.
- **Impact:** seconds; the watchers were restarted without the flag (default 25 minutes).
- **Cause:** prompts and rules are read live from the checkout's `main` while the binary only
  changes at an upgrade, so docs can describe a CLI that isn't installed yet.
- **Category:** `role`, `config`.
- **Follow-up:** br-751e (chvf 3, the daemon keeps a workflow checkout at the binary's tag);
  the next upgrade brings the flag.

## 2026-10-03 01:20: `bridle serve --detach` reported a failure, but the daemon started

- **What happened:** the human ran `bridle serve --detach` for bridle-ui
  (`/Volumes/Data/work/bridle-ui-workspace/bridle-ui`) on dalek and got "error: daemon did not
  become healthy within 15s", with a log tail showing only a config warning (the `night` focus
  period's end before its start). Reported by the human to the advisor. The daemon was fine: the
  same process (pid 32152, started 01:20:01) logged its listen address at 01:20:17.3, about
  0.4 s after the CLI gave up, and was serving on port 7405 afterwards (`bridle daemons`).
- **Impact:** none to the daemon; a misleading error, and the human had to check whether it was
  running.
- **Cause:** `--detach` waits a fixed 15 s for the child's health check (`crates/bridle/src/
  serve.rs`, deadline 15 s, polled every 200 ms), then reports failure without stopping the
  child, which keeps starting. This start-up took about 15.4 s from reading config to listening.
  Why it took that long is unknown; the human thinks system load (the laptop builds Rust for
  workers, b7cz).
- **Load (advisor, checked at 01:26):** load averages 9.4 / 9.4 / **17.8** (1, 5, 15 minutes) on
  16 logical CPUs (8 cores): the 15-minute window covering the start-up was saturated. In that
  window bridle's daemon landed br-c4f4 (01:13:46), built its self-upgrade and restarted
  (01:17:20), and two `rustc` processes were still compiling at 01:26; `syspolicyd` (which
  checks new binaries, cs7x) was at 26% CPU. Load, not a daemon fault, is the likely cause of
  the slow start.
- **Category:** `daemon`, `host`.
- **Follow-up:** [[bridle-serve-detach-gives-up-at-15-s-while-the-daemon-is-sti-cy5v|cy5v]]
  (wait longer; on timeout say it's still starting and left running).

## 2026-10-03 01:15: a held task hid a startable one from the manager

- **What happened:** br-6c6a (tier 7) was startable, but `bridle task ready` shows only the
  highest tier with a startable task: tier 6's br-96cc, which the orchestrator had told the
  manager to hold until the human is back. The manager saw nothing to start and went idle.
  Found by the orchestrator reading `bridle queue`.
- **Impact:** about 10 minutes with both workers' slots empty; overnight it could have been hours.
- **Cause:** a hold given only in a message, not in the queue: the task stayed startable, so it
  masked the tiers below it.
- **Category:** `coordination`.
- **Follow-up:** hold tasks in the queue, not by message (move a held task out of the queue or
  below the work that should run); none filed yet.

## 2026-10-02 23:25 and 23:40: ready tasks sat until the orchestrator nudged

- **What happened:** br-5924 finished its settle period at 23:25 and br-1e88 at 23:40, but the
  idle development manager (manager-2) started neither until the orchestrator sent a nudge
  (found by the orchestrator checking `bridle agents` after each settle time).
- **Impact:** a few minutes each; overnight, with no one watching, a ready task could sit for hours.
- **Cause:** nothing wakes an idle manager when a task's settle period ends; the settle period
  (ny9u, br-3c71) made tasks startable later without a matching event.
- **Category:** `coordination`, `daemon`.
- **Follow-up:** ny9u follow-up (7f28faa), task br-96cc (held until the human is back).

## 2026-10-02, about 00:00-04:00: track-web's new review roles gave up after one turn

- **What happened:** track-web's `web-reviewer` and `playtester` each stopped after one turn: one
  tried `curl`, the other chained `mkdir` with `playwright-cli`; under `dontAsk` the denial read
  to them as "Bash is denied". The manager also couldn't run `scripts/preview.sh` (not in its
  `allowed_tools`), and Claude Code ignored the clone's `.claude/settings.json` because its folder
  trust dialog had never been accepted. Found by the orchestrator.
- **Impact:** the orchestrator messaged each role, made their temp directory and ran the preview
  server by hand; the allow-list fix waits on the human (auto mode refused the orchestrator's edit).
- **Cause:** narrow Bash allow-lists plus prompts that don't list exactly what a role may run, one
  command per call. Same failure as manager-2's stall on 2026-09-30 (below).
- **Category:** `role`, `config`.
- **Follow-up:** the human adds the allow-list entry and accepts the trust dialog (h-0014, h-0015);
  role-notes, twenty-second session.

## 2026-10-01 and 2026-10-02: the product manager planned work without queueing it (three times)

- **What happened:** pm-1 planned tasks twice on 2026-10-01 and br-761a on 2026-10-02 without
  adding them to the queue or telling manager-2, so `bridle ready` showed nothing and everyone
  idled. Found by the orchestrator each time.
- **Impact:** idle hours on each occasion; the orchestrator relayed or ran `bridle queue
  add-tier` by hand.
- **Cause:** planning and queueing are separate steps, and nothing warns when a planned task
  isn't queued.
- **Category:** `role`, `coordination`.
- **Follow-up:** none filed yet; role-notes (twenty-first and twenty-third sessions) propose that
  `bridle task plan` queue the task or warn.

## 2026-09-30 to 2026-10-01: track-web's messages to the orchestrator sat unread for days

- **What happened:** the orchestrator ran one wake waiter, on bridle's daemon. track-web's
  messages to it (m-0008, m-0051, m-0061) and its wakes queued from 2026-09-30 until the human
  noticed on 2026-10-01. Separately, listing the inbox didn't mark messages read, so external
  principals' messages stayed `pending`.
- **Impact:** days of delay on track-web's questions; its work still moved because filing a
  task notifies the manager directly.
- **Cause:** a waiter watches one daemon, and the role didn't say to run one per project.
- **Category:** `coordination`, `role`.
- **Follow-up:** role: one waiter per project (7b22c1e);
  [[one-watcher-for-every-project-bridle-agent-wake-all-projects-cy2v|cy2v]]; advisors mark
  read (br-c877).

## 2026-10-01, about 18:15-24:00: starting advisors by hand went wrong twice

- **What happened:** (1) The new advisor `workflow` never read its brief (m-3231) and greeted the
  human with a general status report; the human noticed. (2) The first advisor started in a new
  repo showed Claude Code's folder trust dialog, whose default is "No, exit"; the orchestrator's
  Enter quit the session.
- **Impact:** minutes; the orchestrator typed a pointer to the brief into the pane by hand.
- **Cause:** (1) nothing in the advisor's start prompt or `bridle prime advisor` mentioned a
  brief. (2) An interactive prompt the launcher doesn't handle.
- **Category:** `role`, `external`.
- **Follow-up:** [[a-new-advisor-never-reads-its-brief-jb4e|jb4e]] (br-6d49, br-0e64).

## 2026-10-01 17:58: an unexplained daemon restart

- **What happened:** bridle's daemon restarted at the commit it already ran, with no event saying
  who asked (h-0012).
- **Impact:** none seen; agents resumed.
- **Cause:** unknown; probably the human.
- **Category:** `daemon`.
- **Follow-up:** none; check with the human only if it recurs.

## 2026-10-01: a restart on the NUC (Linux) killed the daemon

- **What happened:** `bridle restart` on the NUC, after `just install` replaced the binary under
  a running 0.3.x daemon, failed with `exec .../bridle (deleted) failed` and the daemon exited.
  Reported by the NUC's orchestrator (m-3149).
- **Impact:** meta-notes' daemon was down until the human started it by hand.
- **Cause:** on Linux `current_exe()` names the replaced file with ` (deleted)` appended; macOS
  doesn't, so the laptop never hit it, and no test replaces the binary under a running daemon.
- **Category:** `daemon`, `host`.
- **Follow-up:** [[restart-on-linux-execs-path-deleted-after-the-binary-is-repl-fpde|fpde]]
  (path fix 8edb6ed; staying up on a failed exec is still open).

## 2026-09-30 and 2026-10-01: the human's token commands failed

- **What happened:** on 2026-09-30 two agents gave the human a statusline token command that
  can't work (`--url` drops the workspace, so no human token). On 2026-10-01 the human couldn't
  send from the NUC to dalek: the human token lives only in the daemon's own workspace.
- **Impact:** the human's time, twice; worked around with an explicit `--token`.
- **Cause:** wrong docs in `cli.md`; the human's token is single-machine by design.
- **Category:** `config`, `role`.
- **Follow-up:** [[the-statusline-token-setup-command-in-cli-md-can-t-work-url-cw7a|cw7a]]
  (br-4443, 7e794ee); [[the-human-s-token-works-only-on-the-daemon-s-own-machine-3ehu|3ehu]].

## 2026-10-01, about 01:30 onward: quiet hours weren't quiet

- **What happened:** during the human's first configured quiet period, the advisor and the
  orchestrator still sent long answers, filed tickets and ran many tool calls; the human called
  the setting "a bust" for the night. Later the stricter gate also blocked the orchestrator's own
  watcher loop on background wakes (2026-10-01 and 2026-10-02).
- **Impact:** the human's evening attention; deferred orchestrator work.
- **Cause:** the gate was advice the models didn't follow; the stricter version can't tell a
  human prompt from a wake.
- **Category:** `role`.
- **Follow-up:** [[quiet-hours-aren-t-quiet-agents-still-talk-too-long-cdez|cdez]] (br-9df6,
  5a6261b); the wake exemption is noted in role-notes (twentieth, twenty-third sessions).

## 2026-09-30 and 2026-10-01: overnight focus periods confused agents and the human

- **What happened:** a `[[focus]]` block crossing midnight matched the next day's entry, so the
  human split each night in two, and agents then told the human quiet hours "end at midnight".
- **Impact:** the human's confusion and config edits; wrong answers from agents.
- **Cause:** `in_window` checked `days` against the current weekday; the docs only said a range
  "may cross midnight".
- **Category:** `daemon`, `config`.
- **Follow-up:** [[windows-that-cross-midnight-match-on-the-current-day-not-the-jqx3|jqx3]]
  (docs), [[focus-hours-an-overnight-period-belongs-to-the-day-it-starts-3xr4|3xr4]] (br-14d6),
  [[overnight-periods-must-say-1d-and-bridle-doctor-validates-th-r5s3|r5s3]] (br-1c2a).

## 2026-09-30, about 23:00-23:20: the auto-mode classifier blocked the orchestrator

- **What happened:** after rightly denying `gh workflow run release.yml` as "Create Public
  Surface", Claude Code's auto-mode classifier denied nearly every Bash call (even `bridle
  status` and restarting the watcher) for about 20 minutes, then cleared by itself. The session
  handed over (h-0009). Earlier, on 2026-09-28, it had also blocked a watcher restart.
- **Impact:** the eighteenth session ended early; no wakes were lost (they queue). Releases still
  need the human's hands.
- **Cause:** the classifier's behaviour after a denial (external).
- **Category:** `external`.
- **Follow-up:** br-b647 (release permission rules, done by the human).

## 2026-09-30 to 2026-10-02: the orchestrator's wake loop misfired (recurring)

- **What happened:** two watchers ran at once after the orchestrator chained `bridle send ... &&
  bridle wait-for-wake` (2026-09-30, fourteenth session); one `wait-for-wake` was killed by Claude
  Code's 30-minute default background limit (2026-09-30); with nothing happening, two waiters
  woke about 50 times overnight and the advisor every 5 minutes (2026-10-02). Earlier, on
  2026-09-28 and 2026-09-29, the watcher was twice started with `&`, and restarted ~80 times in
  one session.
- **Impact:** missed or duplicate wakes, and turns spent on "nothing".
- **Cause:** a session-side polling loop with hand-set timeouts, restarted by the model.
- **Category:** `role`, `external`.
- **Follow-up:** [[a-calmer-orchestrator-wake-loop-v9t9|v9t9]] (restart the waiter first, run
  with `timeout: 7200000`); [[advisor-wake-waits-about-90-minutes-not-5-789x|789x]];
  [[should-the-orchestrator-wake-when-every-agent-is-idle-a-heal-aqtg|aqtg]].

## 2026-09-30 19:06-20:40: red `main` that nothing reported

- **What happened:** a focus override test depended on the wall clock and failed on CI for about
  90 minutes. No wake fired; the orchestrator found it by looking.
- **Impact:** 90 minutes of red `main`; merges held.
- **Cause:** the test read the real clock; and `[ci] github` had never been set for bridle, so the
  daemon's CI watcher was off although the role doc said bridle watches CI.
- **Category:** `ci`, `config`.
- **Follow-up:** 23f102b (test fixed, `[ci] github = true`); [[bridle-watches-ci-c8qw|c8qw]].

## 2026-09-30: the NUC's orchestrator got no context wakes

- **What happened:** the meta-notes orchestrator on the NUC reached ~182K context with no
  `context` wake (reported by it, m-2877).
- **Impact:** a large, costly context; a handover only by the agent noticing itself.
- **Cause:** not confirmed: supervision off for the project, a split `BRIDLE_HOME`, or an old
  binary on the NUC.
- **Category:** `daemon`, `config`.
- **Follow-up:** [[context-wakes-never-reach-the-orchestrator-on-a-client-machi-jf9u|jf9u]]
  (br-6b8a, parked).

## 2026-09-30: moving meta-notes to the NUC hit three gaps

- **What happened:** during the human's hand move (`stop-daemon`, `serve --take-over`): the
  project's `workflow` path was machine-specific; `tools-only-install` refused over the human's
  git-template hooks; and the first `bridle restart` after a port change, on both machines,
  reported "did not come back within 95s" although the daemon was up.
- **Impact:** the human's and orchestrator's time; a false failure report.
- **Cause:** per-machine paths in project config; the hook installer not chaining; `restart`
  polling the old URL.
- **Category:** `config`, `daemon`.
- **Follow-up:** machine-config override; [[tools-only-hooks-chain-to-existing-hooks-ged2|ged2]]
  (br-d160); [[restart-waits-on-the-old-port-6d5y|6d5y]] (br-9c3d).

## 2026-09-30: Claude Code warned on every session start

- **What happened:** a named advisor start printed warnings that `Write(path)` deny rules aren't
  matched by file permission checks (Claude Code 2.1.286). Reported by the human.
- **Impact:** noise only; the `Edit(...)` rules still denied.
- **Cause:** 717a444 (cvaq) added `Write(...)` twins; a newer Claude Code warns about them.
- **Category:** `external`, `config`.
- **Follow-up:** [[drop-the-write-path-focus-file-deny-rules-claude-code-warns-m9cy|m9cy]].

## 2026-09-30, about 06:40-14:45: managers stalled on denied commands

- **What happened:** manager-2 stalled with two tasks ready: its compound Bash calls (pipes,
  loops) were all denied under `dontAsk`, and it read that as lost permission. meta-notes'
  manager was denied `git tag pre-bridle` and the push of `main` for its promotion, and believed
  it couldn't message the orchestrator (it could).
- **Impact:** the queue idled until the orchestrator noticed; it ran the tag and push by hand.
- **Cause:** allow-lists match single plain commands; prompts didn't say so; a manager's
  day-to-day allow-list doesn't cover a one-off release.
- **Category:** `role`, `config`.
- **Follow-up:** manager role: one plain command per Bash call (4d0d8a1).

## 2026-09-30, about 12:25 and 13:50: self-upgrade starved, and restarted for nothing

- **What happened:** twice the self-upgrade build succeeded but found "no quiet point within
  600s": the manager refilled worker slots as soon as one landed. Earlier that day the restart
  message named the wrong commit (the clone's head, not the one built), and three of four
  restarts were for docs-only commits.
- **Impact:** the rollback and d3wq fixes sat merged but not running for about two hours; each
  needless restart risked an unattended dead daemon. The orchestrator paused spawns by hand.
- **Cause:** no spawn hold while a build waits; `restart.rs` read the branch at restart time; no
  check of what a commit changed. (On 2026-09-29 a manager had also spawned two workers seconds
  before a "hold spawns" message.)
- **Category:** `daemon`, `coordination`.
- **Follow-up:** [[bridle-restarts-itself-q7rx|q7rx]] (br-e990, 3c5d092);
  [[self-upgrade-reports-the-wrong-commit-and-restarts-for-docs-d3wq|d3wq]].

## 2026-09-30, about 03:15-06:30: the product manager overstepped twice

- **What happened:** pm-1's sweep of stale task records closed a67t and 6rh7, which were never
  built; and it unheld br-14cd (warm build cache) on its own reading of a measurement the human
  had said they'd decide on. The orchestrator reopened and re-held them. (On 2026-09-29 pm-1 had
  also queued already-fixed tickets as tasks.)
- **Impact:** small; caught before harm.
- **Cause:** closing by matching ticket commits without checking an implementing commit exists;
  "measure, then decide" read as "measure, then do".
- **Category:** `role`.
- **Follow-up:** none filed; role-notes, fifteenth session.

## 2026-09-30, about 06:40-14:45: the advisor filed to-dos from assumptions

- **What happened:** the advisor filed human to-dos without checking the machine: macOS
  auto-installs were already off, and "check caffeinate" was the wrong fix. The human caught the
  first.
- **Impact:** the human's time and trust.
- **Cause:** to-dos written from memory, not checked.
- **Category:** `role`.
- **Follow-up:** none; role-notes, sixteenth session.

## 2026-09-30, about 06:00: red `main`, a test inheriting the agent's environment

- **What happened:** `advisor_start` tests passed only in agents' environments because they
  inherited `BRIDLE_PROJECT`; CI failed within the hour of the merge.
- **Impact:** red `main` for under an hour; one worker fixed it.
- **Cause:** tests not isolated from the caller's `BRIDLE_*` environment (as q7fx, 2026-09-27).
- **Category:** `ci`.
- **Follow-up:** br-2c99 (14689eb).

## 2026-09-30 03:25: a worker's test drove the live orchestrator

- **What happened:** worker `nuc-scripts` tested `scripts/claude-orchestrator` by running it for
  real. It started `claude` under the live orchestrator's Remote Control name (a test prompt
  reached the orchestrator twice) and overwrote `orchestrator.pid` and `orchestrator.session`,
  so the daemon supervised a dead test session and told the human the orchestrator wasn't
  running (m-2319).
- **Impact:** a false alarm; the human restored the files by hand (auto mode refused the
  orchestrator's repair).
- **Cause:** the launcher wrote to the shared `~/.bridle` and used a fixed session name; the
  "no real `claude`" rule was read as covering only the test suite.
- **Category:** `role`, `config`.
- **Follow-up:** [[worker-tests-reach-the-live-orchestrator-k6b3|k6b3]] (br-5813, aee8ab3).

## 2026-09-30, about 03:10: the relaunched orchestrator's pane filled with escape codes

- **What happened:** after a handover, the daemon's relaunch brought up a pane where focus
  reports showed as literal `^[[O^[[I` and Enter didn't submit. The human killed the session; the
  next relaunch was fine.
- **Impact:** the human's time late at night; one lost session.
- **Cause:** the daemon's stop killed the launcher script but left `claude` running as an orphan
  on the same terminal.
- **Category:** `daemon`.
- **Follow-up:** [[orchestrator-pane-full-of-escape-codes-csfe|csfe]] (br-7798, c8569bf).

## 2026-09-30 02:16: `bridle status` suggested deleting live work

- **What happened:** right after a restart, `bridle status` listed `mark-unread` as merged and
  suggested `bridle rm --delete-branch`. The orchestrator relayed it to the human as safe; the
  branch was empty and the worktree held uncommitted work, later landed (feb7e17).
- **Impact:** a near miss: the human would have lost the work.
- **Cause:** an empty branch is always an ancestor of `HEAD`, so `is_merged` called it merged.
- **Category:** `daemon`, `role`.
- **Follow-up:** [[status-calls-an-empty-branch-merged-z4hd|z4hd]] (br-f919).

## 2026-09-30, about 00:45: the daemon wouldn't start after a config change

- **What happened:** the fourteenth session opened on a daemon that wouldn't start: the installed
  binary predated br-f05d, which changed `.bridle/config.toml` to `"200k"` and the parser in one
  commit. The handover note had said the config used plain numbers. Later in the session the
  orchestrator built in the foreground from a detached checkout of the clone; the human saw
  `HEAD` detached and a no-watcher alert fired.
- **Impact:** a delayed start; the human's confusion.
- **Cause:** a config-format change landed ahead of the binary that reads it; a handover written
  from memory.
- **Category:** `config`, `human-process`.
- **Follow-up:** [[bridle-restarts-itself-q7rx|q7rx]] (self-upgrade); the orchestrator no longer
  builds (ed3ce11).

## 2026-09-29, about 20:45-24:00: the context governor didn't renew, and a config edit blocked landing

- **What happened:** the orchestrator renewed manager-2 by hand at 149K: `[context] wind_down_at`
  defaulted to 200K because the daemon reads only `[budget]` from `~/.bridle/config.toml`. The
  human's uncommitted edit to `.bridle/config.toml` blocked landing until the orchestrator
  committed it (2a2ccb0).
- **Impact:** a large context; landing blocked until the orchestrator committed the edit.
- **Cause:** which config file holds which setting wasn't clear (machine vs project).
- **Category:** `config`.
- **Follow-up:** [[project-machine-and-account-scope-9mxw|9mxw]],
  [[reload-config-without-a-restart-9t54|9t54]].

## 2026-09-29 12:36 and 18:33: the orchestrator killed twice by a worker's `pkill -f`

- **What happened:** the orchestrator's session exited with no warning twice; nothing noticed
  until the human looked. Found by a subagent from the unified log: a worker's `pkill -f "just
  check"` matched the orchestrator, whose launcher passed the whole prime text (mentioning `just
  check`) as an argument.
- **Impact:** two orchestrator outages with no handover, each found only when the human looked;
  the second needed the human to restart it by hand. The human called the orchestrator "the lynchpin of too much".
- **Cause:** process kill by name, plus a huge argv.
- **Category:** `role`, `coordination`.
- **Follow-up:** [[the-orchestrator-stays-running-fx7x|fx7x]] (one-line prompt 5278556, rule
  no-kill-by-name br-15a7, daemon supervision br-a424/br-e949, exit log).

## 2026-09-29, about 17:00: the orchestrator deleted its watcher's state files

- **What happened:** an over-broad `rm` by the orchestrator removed the watcher's state files; it
  restored them by hand (tenth session).
- **Impact:** minutes.
- **Cause:** a careless command.
- **Category:** `role`.
- **Follow-up:** none.

## 2026-09-29: `bridle land` undid squash landing

- **What happened:** the first landings after the reboot (00af029 br-d99e, 8e41ef6 br-f671) were
  plain merge commits, not the one-squash-commit shape decided the day before.
- **Impact:** two non-squash merges on `main`, left in place.
- **Cause:** `bridle land` (br-6dd6) was built with `--no-ff` after sq4m, and the manager role
  and skill disagreed.
- **Category:** `merge`.
- **Follow-up:** [[bridle-land-undoes-squash-landing-mz4q|mz4q]] (br-1d3d, 8d078f4).

## 2026-09-29, about 03:15-08:50: about six red `main`s overnight, all test-only

- **What happened:** CI on `main` went red repeatedly overnight: rustfmt (three times, including
  bb6bbf9), missing git identity in test repos (twice, at 3d1e9ef and 402816e), and a macOS port
  race (3ce74df, 09f3035). Found by the orchestrator from CI logs.
- **Impact:** merges held each time; one fix worker each.
- **Cause:** workers reported done without running the full `just check` after their last
  commit; tests depended on the runner's git config and port timing.
- **Category:** `ci`.
- **Follow-up:** fixes d6ff4ca, df19a21, b247748, 0fd2fde, 948a2d4, 06a37d2; the manager now
  requires the full check in the report.

## 2026-09-29: a finished worker sat stopped for hours

- **What happened:** `python-pack-2` stayed stopped long after its work merged (761fcd5): the
  manager's `rm` removed the agent named after the branch, not this one, and a restart had
  stopped it mid-check. `bridle agents` hides stopped agents; the human found it in the TUI.
- **Impact:** a stray agent and worktree; the human's time.
- **Cause:** cleanup was the manager's judgement, and two agents shared one branch.
- **Category:** `coordination`.
- **Follow-up:** [[clean-up-agents-when-their-work-lands-k3wp|k3wp]] (61c52ff, e26c54d).

## 2026-09-29: a dropped task stayed claimed

- **What happened:** `bridle queue` showed br-29f9 as claimed although pm-1 had dropped it after
  merge; the human: "it is claimed but dropped, I don't understand".
- **Impact:** a confusing queue.
- **Cause:** `drop_task` didn't release the claim, and the lease check skips claims whose agent
  is gone.
- **Category:** `daemon`.
- **Follow-up:** [[dropping-a-claimed-task-leaves-its-claim-b5br|b5br]] (br-42e6).

## 2026-09-29, about 02:50-10:45: the human's track-web tasks reached nobody

- **What happened:** in track-web, with no product manager, the human's five new tasks sat unseen
  until the orchestrator told the manager; the manager then didn't `task done` until told.
- **Impact:** hours of delay on the human's first tasks there.
- **Cause:** only the product manager told the manager about new tasks.
- **Category:** `coordination`.
- **Follow-up:** br-3bb4 (wake the manager on `task.created`, b76f487);
  [[the-daemon-tells-the-manager-when-the-queue-changes-f5ww|f5ww]].

## 2026-09-28 23:55 to 2026-09-29 02:35: the orchestrator didn't watch its own context

- **What happened:** the eighth session never checked its context size; the human called the
  handover.
- **Impact:** an expensive, oversized context; the human had to step in.
- **Cause:** nothing reported the orchestrator's own context to it.
- **Category:** `role`.
- **Follow-up:** [[orchestrator-watches-its-own-context-c9zm|c9zm]] (037c6ee and others).

## 2026-09-29 01:50: the same hang again, and its cause

- **What happened:** right after the next `cargo install` (for the TUI highlight fix),
  `syspolicyd` died again and new programs hung.
- **Cause:** its crash reports (five that evening) show a SIGSEGV while validating the signature
  of an unsigned Mach-O (`Security::Universal::architecture()`); the laptop is x86_64, where
  binaries aren't ad-hoc signed by the linker.
- **Impact:** a second hang; meanwhile the orchestrator signed the installed binary by hand
  (`codesign -s - -f ~/.cargo/bin/bridle`).
- **Category:** `host`.
- **Follow-up:** [[sign-binaries-on-intel-macs-cs7x|cs7x]]: sign at link time (0f4b23a).

## 2026-09-29 01:05-01:40: macOS hung every new program on exec

- **What happened:** right after `cargo install --path crates/bridle` (01:05), every run of the
  new `bridle` hung at exec: the CLI for all agents, the orchestrator, the advisor and the
  human's `bridle tui`. The daemons (already running) were fine and answered HTTP. Found when
  every CLI call hung.
- **Impact:** about 35 minutes with no CLI for anyone.
- **Cause:** macOS, not bridle. `syspolicyd` wasn't running, so the first-run assessment of any
  new executable waited forever; a freshly compiled C program hung the same way, while
  programs run before kept working. Two earlier installs that evening had run fine. Why it
  stopped is in the entry above. `syspolicyd` came back by itself (launchd relaunched it
  around 01:40); `sudo launchctl kickstart -k system/com.apple.security.syspolicy` fails with
  "Operation not permitted" (SIP), so if it doesn't recover, the fix is a reboot.
- **Diagnosis tips:** `ps` itself hangs when it reads a process stuck in exec; use `pgrep -l`.
  Reach the daemons over HTTP (`curl` to the URL in `~/.bridle/daemons/<project>.json`) to send
  messages while the CLI is down.
- **Category:** `host`.
- **Follow-up:** [[sign-binaries-on-intel-macs-cs7x|cs7x]].

## 2026-09-28 and 2026-09-29: TUI bugs the human hit

- **What happened:** the inbox and agents panels didn't scroll (2026-09-28, with 160 unread
  messages); a fix lost the selected-row highlight (restored 51b1987, about 01:50 on 09-29); new
  agents didn't appear until the TUI restarted (2026-09-29).
- **Impact:** the human couldn't see their inbox or new workers.
- **Cause:** table state rebuilt each frame; agents listed once at startup.
- **Category:** `daemon` (the TUI client).
- **Follow-up:** [[tui-inbox-doesnt-scroll-8ups|8ups]], [[tui-agents-panel-doesnt-scroll-yurx|yurx]]
  (br-9e67), [[tui-doesnt-show-new-agents-n4vk|n4vk]] (f3817a2).

## 2026-09-28, evening: Gatekeeper's "Verifying…" window stole the human's keystrokes

- **What happened:** a small dialog flashed repeatedly and took a keystroke or two; the advisor
  found 78 in an hour: Gatekeeper scanning each new executable that workers' builds made.
- **Impact:** an annoyed human, typing lost.
- **Cause:** daemons under tmux started from iTerm make iTerm the responsible GUI app;
  launchd-started processes don't show the window.
- **Category:** `host`.
- **Follow-up:** [[gatekeeper-verify-dialog-steals-focus-qr8z|qr8z]] (`bridle launchd`, 74603a5).

## 2026-09-28, about 23:10 to 2026-09-29 01:30: CHANGELOG.md conflict markers on `main`

- **What happened:** parallel merges each adding to `CHANGELOG.md` left conflict markers on
  `main` several times (fixes 698d0b4, 1fe0ae1, e843e6d, 3d74e13, dbf50eb, 61aa912); a duplicated
  section followed on 09-29 (fd78ecd).
- **Impact:** a series of fix-up commits; CI noise.
- **Cause:** every task appends to the same Unreleased section.
- **Category:** `merge`.
- **Follow-up:** union merge driver for CHANGELOG.md (br-e7f4, 361effa).

## 2026-09-28, about 23:00: shutdown hung while the TUI was open

- **What happened:** in meta-notes, `bridle serve` received shutdown but didn't exit until the
  human quit `bridle tui`. Reported by the human.
- **Impact:** a hung restart; the daemon looked stopped to discovery while still running.
- **Cause:** graceful shutdown waited on the open SSE event stream, which never ended.
- **Category:** `daemon`.
- **Follow-up:** [[shutdown-waits-on-open-event-streams-zm95|zm95]] (681d0c3).

## 2026-09-28, from 22:03: the laptop on battery and a phone hotspot (observation)

Not an incident; a trial. The human drove home with the laptop on battery and a personal
hotspot, to see whether Remote Control and bridle's work hold up. At 22:03 it was online,
with two workers running (`base-rules`, `pypack-merge`).

Result: no drop. From 22:04 to 22:31 (home, switching to Wi-Fi) the minute-by-minute check
(`.bridle/connectivity.log` in the workspace) reached api.anthropic.com every time, with no
gaps; Remote Control and this session stayed up. Five merges landed on the way. The
battery fell from 99% to 70% in 27 minutes (about 1.1% a minute) with two workers building
(load 40-131); see [[build-cost-on-the-laptop-b7cz|b7cz]].

## 2026-09-28, 19:23–21:27: Remote Control and the orchestrator session lost

- **What happened:**
  - The human lost the connection to all their Remote Control sessions at about 19:23.
  - The orchestrator's Claude Code session (2928d1d4) did nothing from 19:23:34 to 21:27:20,
    when the human resumed it. Its session-only cron jobs didn't fire in that time (the 19:25
    one-shot check and the :17/:47 heartbeats). The session was probably not running at all,
    since cron jobs fire whenever the REPL is idle.
  - The advisor's session kept running, but the human couldn't reach it over Remote Control.
- **What still worked:**
  - The machine was awake, on power and online. The human saw it on Tailscale. `pmset -g log`
    has no Sleep/Wake entries in the window, and uptime was 30 days.
    `caffeinate -si` (pid 67952, started 2026-09-26 18:39 UTC) held `PreventSystemSleep`
    and `PreventUserIdleSystemSleep` throughout.
  - The bridle daemon (pid 83314) kept running. After the five_hour reset at 19:20 it resumed
    pm-1 and python-pack-2 itself.
- **Impact:**
  - Two hours with no orchestration. manager-2, stopped by the earlier wind-down, stayed stopped
    until 21:27, so the urgent CI fix (g3ck, br-0838) didn't start.
  - The orchestrator had also turned its watcher off at 18:48, to stop repeated "all idle"
    wakes during the budget pause, and relied on the 19:25 cron to restart it. With the session
    gone, nothing noticed.
  - The orchestrator first told the human the laptop had slept. That was wrong: it had misread
    the "PrevSleep" state flags on power assertions as sleep events.
- **Why bridle stopped too:** a separate bridle bug, k7nr. At 19:20:36 the governor resumed
  only pm-1 and python-pack-2 (`max_workers = 2` counts managers too, oldest paused first),
  never manager-2. Both finished by 19:27, and python-pack-2's done report to manager-2 sat
  undelivered. Bridle should have carried on without the orchestrator.
- **Cause of the connection loss:** unknown. The session's own log stops at 19:23:34 without an
  error. The timing matches the Remote Control drop, so the likely cause is on the Remote
  Control side (the connection, or the `claude` process exiting when it dropped), not bridle.
- **Category:** `connectivity`, `external`, `daemon` (k7nr).
- **Follow-up:**
  - [[budget-resume-skips-the-manager-k7nr|k7nr]] (c70b971): managers always resume.
  - Nothing outside the orchestrator's session noticed it stopped:
    [[the-orchestrator-stays-running-fx7x|fx7x]] (the daemon supervises it).
  - Keep the watcher running through budget pauses; filter the idle wake instead of stopping it.

## 2026-09-28, 18:06: the budget override was overridden by Claude Code's warning

- **What happened:** with `bridle budget override burst` (thresholds 96-99%), the governor still
  wound down at 91% on `five_hour`, because Claude Code sent `allowed_warning`.
- **Impact:** agents wound down at 91% although the human had raised the thresholds to 96-99%.
- **Cause:** by design, `allowed_warning` forced a wind-down whatever the thresholds said; Claude
  Code sends it at about 90%.
- **Category:** `daemon`, `external`.
- **Follow-up:** [[allowed-warning-overrides-the-override-kv7d|kv7d]] (the human's thresholds
  win, 538ca98).

## 2026-09-28, 17:43 and 17:53: renew and resume left agents dead under a budget hold

- **What happened:** `bridle renew pm-1` was refused for the budget hold but had already stopped
  the old session, leaving pm-1 `stopped`. Ten minutes later, after a daemon restart, worker
  `python-pack` died on its first turn at every `bridle resume`.
- **Impact:** a stopped product manager and a stuck worker until fixed by hand (its work was safe
  at 482825b).
- **Cause:** the hold check ran after the stop; the stored Claude Code session id pointed at a
  session claude no longer had, so `--resume` failed at once.
- **Category:** `daemon`.
- **Follow-up:** [[renew-under-a-budget-hold-leaves-the-agent-stopped-r3nh|r3nh]];
  [[resumed-worker-dies-on-its-first-turn-p4ks|p4ks]] (a98b1b3).

## 2026-09-28, 17:49 to about 21:35: CI red, tests' repos started on `master`

- **What happened:** CI on `main` failed on every push after c533cb0 (about 21 tests,
  `invalid reference: main`). Reported by the human through the advisor (m-1075).
- **Impact:** about four hours of red `main`; the fix was delayed by the session loss above.
- **Cause:** br-29f9 defaulted the integration branch to `main`; tests used plain `git init`,
  which makes `master` on ubuntu. Local runs passed because the human's global git config sets
  `init.defaultBranch=main`.
- **Category:** `ci`, `config`.
- **Follow-up:** [[ci-red-tests-assume-main-branch-g3ck|g3ck]] (6625557; `just test` isolates
  global git config, br-d0c3).

## 2026-09-28, about 16:30: workers couldn't merge `main` under `merge.ff=only`

- **What happened:** worker `spike-path-rules` tried five merge commands before one went through.
- **Impact:** wasted turns on every worker update.
- **Cause:** the human's global `merge.ff = only`; worker instructions said plain `git merge main`.
- **Category:** `config`.
- **Follow-up:** [[merging-main-fails-under-merge-ff-only-m2fq|m2fq]] (br-544b, cd57ecc).

## 2026-09-28, about 16:00: build output filled the disk and slowed the tests

- **What happened:** the main clone's `target/` reached about 28-41 GB (186,000+ `.o` files).
  After `cargo clean`, `just check` went from about 400 s to 35 s.
- **Impact:** disk at risk ("the bridle work system will die if I run out of disk space") and
  10x slower tests.
- **Cause:** macOS debug builds keep every crate's `.o` files, and cargo never removes stale ones.
- **Category:** `host`.
- **Follow-up:** [[clean-stale-build-output-f75x|f75x]], [[smaller-debug-builds-nbkj|nbkj]],
  [[disk-usage-monitoring-m3wq|m3wq]]; builds also load the laptop
  ([[build-cost-on-the-laptop-b7cz|b7cz]]).

## 2026-09-28, about 15:15: the task queue was empty while work existed

- **What happened:** `bridle ready` printed "no ready tasks" (54 open, nothing planned) because
  pm-1 kept sending briefs and priorities by message despite being told to use `bridle task`.
- **Impact:** no recoverable or visible queue; the human couldn't see what was next.
- **Cause:** the role prompt still said to send prepared tasks by message; task records had no
  priority.
- **Category:** `role`, `coordination`.
- **Follow-up:** [[the-task-queue-in-bridle-task-not-messages-j479|j479]] (f401a7f).

## 2026-09-28: the human's inbox filled with things they couldn't act on (recurring)

- **What happened:** 160 unread messages (150 routine notes from managers and pm-1 over 21
  hours). On 2026-09-29 it again held workers' `done:` reports, an empty note and a stale
  question; and a question the orchestrator had answered for the human stayed open (m-1210).
- **Impact:** the human's attention; they can't mark others' messages read in bulk.
- **Cause:** role prompts said to report to the human; only the recipient can mark read.
- **Category:** `role`.
- **Follow-up:** [[stop-status-notes-to-the-human-inbox-kp3f|kp3f]],
  [[routine-notes-reach-the-humans-inbox-hx7t|hx7t]],
  [[answer-the-humans-questions-on-their-behalf-h5qd|h5qd]].

## 2026-09-28 and 2026-09-29: Haiku workers printed their report instead of sending it

- **What happened:** a worker finished but printed its `bridle send manager-2 "done ..."` as text,
  so its slot sat idle until the human noticed; on 2026-09-29 the orchestrator relayed four such
  reports (printed, or sent to the wrong address).
- **Impact:** idle slots; orchestrator toil.
- **Cause:** the handoff depended on the worker's last step, and Haiku sometimes skips the tool call.
- **Category:** `role`.
- **Follow-up:** the stop-check insists on a `done:` report (br-d99e); role-notes, 2026-09-28.

## 2026-09-28: a research worker merged an unverified catalogue

- **What happened:** the geem name-research worker had no web access and merged a catalogue it
  couldn't check; the orchestrator redid it with a web-enabled subagent.
- **Impact:** a wasted task and a redo.
- **Cause:** the worker role has no WebSearch/WebFetch.
- **Category:** `role`, `config`.
- **Follow-up:** a `researcher` role queued (role-notes); [[a-new-name-for-the-project-geem|geem]].

## 2026-09-28: a plain Claude Code session couldn't read bridle

- **What happened:** the human asked a separate session to look at completed work; every
  `bridle` call failed with "set `$BRIDLE_TOKEN`".
- **Impact:** any ad hoc helper session was useless.
- **Cause:** reads required a token, and the CLI never uses the human's token inside Claude Code.
- **Category:** `config`.
- **Follow-up:** [[read-only-access-without-a-token-9c63|9c63]].

## 2026-09-28, fourth session start: `bridle logs` showed hours-old output

- **What happened:** `bridle logs manager-2 | tail` showed a review of a worktree already
  removed, and the watcher started from seq 500 woke at once on day-old exits.
- **Impact:** the orchestrator misread an agent's state; spurious wakes.
- **Cause:** with no `--since`, logs and events returned the first 500 lines, not the last.
- **Category:** `daemon`.
- **Follow-up:** [[bridle-logs-shows-the-oldest-500-lines-x7gp|x7gp]].

## 2026-09-28 01:08: context sizes overcounted several times over

- **What happened:** `bridle agents` showed manager-2 at 942,368 tokens of context.
- **Impact:** the context governor would have acted on a wrong number.
- **Cause:** `result.usage` is summed over a turn's API calls (spike 01), not one call's context.
- **Category:** `daemon`.
- **Follow-up:** [[context-tokens-overcounts-multi-call-turns-kc4v|kc4v]] (3610a7e, uses
  `get_context_usage`).

## 2026-09-28, about 04:30-05:45: Linux-only CI failures

- **What happened:** CI on ubuntu failed on `ensure_orphan_branch` with no git identity, a
  shutdown-ordering race, and a zombie-leak flake in a containment test.
- **Impact:** red CI until each fix merged.
- **Cause:** the laptop (macOS, configured git) hid differences from the CI runner.
- **Category:** `ci`.
- **Follow-up:** a3a03e5, 67489eb, dce5b1c.

## 2026-09-27 to 2026-10-02: tests that fail only under load (recurring)

- **What happened:** timing-bound tests failed on a loaded machine or on CI, then passed on
  retry: two on 2026-09-27 (f1ky), governor staleness tests, a renew test, the statusline test
  (2026-09-28), `upgrade_test` (2026-09-30), `parses_fast` (2026-09-30 and 2026-10-02), a focus
  test on macOS (2026-10-01).
- **Impact:** re-runs, held merges, and managers judging whether red is really red.
- **Cause:** wall-clock bounds and races in test code under heavy load (laptop load up to 131).
- **Category:** `ci`.
- **Follow-up:** [[flaky-time-based-tests-on-a-loaded-machine-n6gy|n6gy]],
  [[two-flaky-test-failures-under-load-5heh|5heh]] (a29f8e4, 04fa476, 77f5dba, br-648a),
  br-eb1f, [[parses-fast-fails-under-load-a-wall-clock-bound-in-a-unit-te-k562|k562]] (1855ead,
  br-7826), 5a09b21.

## 2026-09-27: first self-hosted run, a test drove the live daemon

- **What happened:** `cli_e2e.rs` spawned child `bridle` processes that inherited the agent's
  `BRIDLE_URL`/`BRIDLE_TOKEN`, so the tests drove the live daemon and spawned two stray real
  agents.
- **Impact:** stray agents (and spend) on the real daemon.
- **Cause:** test children not stripped of `BRIDLE_*` environment.
- **Category:** `ci`, `daemon`.
- **Follow-up:** [[cli-e2e-leaked-bridle-env-into-child-processes-q7fx|q7fx]] (1980c84); test
  daemons also isolated from `~/.bridle/config.toml` (54a0f57).

## 2026-09-27: backticks in a message body were silently denied

- **What happened:** the manager's `bridle send`/`spawn` calls were denied by Claude Code's
  permission layer whenever the body contained a backtick, though safely quoted.
- **Impact:** a blocked message mid-run, with no obvious reason.
- **Cause:** the permission classifier flags backticks in Bash arguments whatever the quoting.
- **Category:** `external`.
- **Follow-up:** [[backtick-in-bridle-send-body-denied-by-permissions-tk3m|tk3m]] (`--text-file -`).

## 2026-09-27: stopping agents took minutes, and shutdown panicked

- **What happened:** during the first self-hosted run, stops took minutes, and the daemon panicked
  on graceful shutdown (also leaking test daemons).
- **Impact:** slow, unreliable restarts on day one.
- **Cause:** the containment sweep ran a `ps` scan per tracked pid; a `JoinError::Cancelled` from
  `spawn_blocking` wasn't handled.
- **Category:** `daemon`.
- **Follow-up:** 2a40265, c7e4098, e69e67b.

## 2026-09-27: a worker started a turn on its own

- **What happened:** a worker that had gone idle started a new turn about 40 s later, when a
  background Agent task it had launched finished.
- **Impact:** none seen; the daemon couldn't tell this idle from done.
- **Cause:** Claude Code's background-task completion starts a turn (external behaviour).
- **Category:** `external`.
- **Follow-up:** [[idle-with-a-live-background-task-w8bz|w8bz]] (open question).

## 2026-10-09 17:02: main red on Linux from a store test flake

- **What happened:** CI run 37962490045 on 233e0c96 failed on Linux:
  `store::tests::cancelled_blocking_task_returns_shutting_down_not_a_panic` got `Ok(1)` from an
  aborted `spawn_blocking`. The same code passed the two runs before. Found by the
  orchestrator's watch on the h7gt fix's CI run.
- **Impact:** main red again just as h7gt turned it green; merges held (br-hua2 waits).
- **Cause:** the test assumes a one-thread blocking pool, but `#[tokio::test(worker_threads = 1)]`
  doesn't cap `max_blocking_threads`, so the "queued" task can run before the abort.
- **Category:** `ci`.
- **Follow-up:** [[flaky-on-linux-ci-store-cancelled-blocking-task-returns-shut-vabu|vabu]]
  (br-vabu, critical).

## 2026-10-09 23:06: main red twice in an hour from two test flakes

- **What happened:** CI on b798c8bc (br-k6jd landing) failed on macOS only:
  `lifecycle_test::spawn_child_orphan_is_swept_on_stop` timed out waiting for
  `agent.orphans_killed` (62 s); it passed on re-run. Then d9f8505b (docs only) failed on Linux:
  `settle_wake_test::a_task_blocked_by_a_dependency_makes_no_note` got a note for the blocked task
  too. The next commit was green. Reported by manager-2 (the first) and the orchestrator's check.
- **Impact:** main red ~23:06-23:20Z; merges held.
- **Cause:** the settle test plans both tasks before adding the `blocks` edge, with a 1 s settle,
  so a slow runner lets the blocked task settle unblocked. The orphan timeout: unknown (its second
  time; see tr22).
- **Category:** `ci`.
- **Follow-up:** [[flaky-on-ci-settle-wake-test-blocked-task-note-races-its-edg-8ff8|8ff8]]
  (br-8ff8, critical); the orphan test gets a ticket if it fails again.
