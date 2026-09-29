# Incidents

A log of lost connections and sessions that stopped unexpectedly, so patterns show up over
time (the human, 2026-09-28: "Record this as an incident of losing connection so that we can
keep track of these"). Newest first. Times are UTC. Add an entry whenever a session, Remote
Control connection, daemon or agent is lost for no known reason. Related:
[[laptop-sleep-and-network-loss-prvy|prvy]] (the laptop sleeping or losing its network).

Each entry covers what was lost, when, what still worked, the evidence, the impact and the
cause (or "unknown").

## 2026-09-28, from 22:03: the laptop on battery and a phone hotspot (observation)

Not an incident; a trial. The human is driving home with the laptop on battery and a personal
hotspot, to see whether Remote Control and bridle's work hold up. At 22:03 it was online,
with two workers running (`base-rules`, `pypack-merge`). Record any drop here, with times.

Result: no drop. From 22:04 to 22:31 (home, switching to Wi-Fi) the minute-by-minute check
(`.bridle/connectivity.log` in the workspace) reached api.anthropic.com every time, with no
gaps; Remote Control and this session stayed up. Five merges landed on the way. The
battery fell from 99% to 70% in 27 minutes (about 1.1% a minute) with two workers building
(load 40-131); see b7cz.

## 2026-09-28, 19:23–21:27: Remote Control and the orchestrator session lost

- **What was lost:**
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
- **Cause of the connection loss:** unknown. The session's own log stops at 19:23:34 without an error. The timing
  matches the Remote Control drop, so the likely cause is on the Remote Control side (the
  connection, or the `claude` process exiting when it dropped), not bridle.
- **Follow-ups:**
  - Nothing outside the orchestrator's session notices when it stops. Options: the advisor or
    bridle checks the orchestrator is alive, or the watcher runs outside the session.
  - Keep the watcher running through budget pauses; filter the idle wake instead of stopping it.

## 2026-09-29 01:05-01:40 UTC: macOS hung every new program on exec

- **What happened:** right after `cargo install --path crates/bridle` (01:05 UTC), every run of
  the new `bridle` hung at exec: the CLI for all agents, the orchestrator, the advisor and the
  human's `bridle tui`. The daemons (already running) were fine and answered HTTP.
- **Cause:** macOS, not bridle. `syspolicyd` wasn't running, so the first-run assessment of any
  new executable waited forever; a freshly compiled C program hung the same way, while
  programs run before kept working. Two earlier installs that evening had run fine. Why
  `syspolicyd` stopped is unknown.
- **Fix:** none needed; `syspolicyd` came back by itself (launchd relaunched it around 01:40)
  and the hung calls then completed. `sudo launchctl kickstart -k
  system/com.apple.security.syspolicy` fails with "Operation not permitted" (SIP), so if it
  happens again and doesn't recover, the fix is a reboot.
- **Diagnosis tips:** `ps` itself hangs when it reads a process stuck in exec; use `pgrep -l`.
  Reach the daemons over HTTP (`curl` to the URL in `~/.bridle/daemons/<project>.json`) to send
  messages while the CLI is down.
