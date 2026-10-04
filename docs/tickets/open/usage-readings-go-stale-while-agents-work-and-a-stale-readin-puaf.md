---
id: puaf
title: Usage readings go stale while agents work, and a stale reading at low usage holds the workforce
kind: bug
opened: 2026-10-04
repos: [bridle]
changes: []
specs: []
needs: []
see: [xypj]
tasks: [br-puaf]
---

## The ask

Raised by the human, 2026-10-04 ~11:20 ET, after the orchestrator's alert:

> The budget governor went into a hold at 14:59 UTC. It's not about spending: usage is at 20%
> for five hours and 37% for the week. The hold is because the usage reading is more than 10
> minutes old. [...] only new spawns wait.

The human, verbatim:

> This is a noisy alert that shouldn't be happening. [...] Put a hold on after 10 minutes. If we
> need this kind of thing, it should be set to a lot longer, or maybe on some sliding scale based
> on how much usage is left. If we're at 20% or 30% usage, I don't care if the meeting is probably
> an hour old. If we're at 80% usage, then the sampling makes sense.

> [...] we need a reliable way to get usage statistics that don't depend on the particular
> performance of an active agent. Suggest some solutions. I don't know. I guess we can call the
> Claude API, which seems like that should exist. I can get a token for that, or find some way for
> the daemon itself to initiate a session just to grab the usage information. Of course, that
> session gets tossed [...], so it would be like having everything stripped: no skills, no tools,
> nothing except running the status line, only permissions for that. It should have a really lean
> context set up and then shut down immediately, but I prefer a better solution. I just feel that
> is a much more sustainable solution than having that happen.

Two asks:

1. **Stop the noise now.** A stale reading at low usage shouldn't hold. Make staleness scale with
   the last known utilisation (for example: below 50% of `hold_at`, allow an hour or more; near
   `hold_at`, keep about 10 minutes), or at least raise `max_staleness` a lot.
2. **A reliable usage source** that doesn't depend on whichever agent happens to be running.

## What happens today (from the code, 2026-10-04)

Usage doesn't come from Remote Control or the status line (`bridle statusline` stopped posting in
s8kn). `Governor::poll_usage` (`crates/bridle-daemon/src/governor.rs`) sends the undocumented
`get_usage` control request **to any running agent** if one exists, and only otherwise to its own
promptless `claude -p` probe process. It polls every 5 min below `hold_at`, the timeout is 10 s
(`PROBE_TIMEOUT`), and a failure is logged at `debug`, so nothing shows in `daemon.log` at the
default level. With `max_staleness = "10m"`, two failed or skipped polls in a row are enough to hold.

Not verified: the holds came while two workers were running, so the polls likely went to a busy
worker. Either `get_usage` sent mid-turn misses the 10 s timeout, or the reply is lost. The holds
cleared on their own in about 15 minutes, once a poll got through. This is the first thing to confirm:
log the failure at `info`/`warn` with which target was asked.

## Options for a reliable source

- **A. Always use the daemon's own probe** (never a working agent). The design already has it: a
  promptless `claude -p`, stream-json, no tools. That's close to the human's "lean session that
  only fetches usage". Tighten it further: `--strict-mcp-config` with no servers, no skills, no
  settings sources, no tools, memory off. Keep one alive across polls, or spawn one per poll and
  exit. Cheap: `get_usage` makes no model call. Recommended first step: small, and it removes the
  dependence on agent behaviour.
- **B. A retry with fallback**: if the agent doesn't answer in time, ask the probe right away
  instead of waiting 5 min for the next poll. A smaller change than A, but it still treats
  working agents as the first choice.
- **C. Call the usage endpoint over HTTP directly**, with the account's OAuth token (what
  `get_usage` and `/usage` read underneath). Needs no `claude` process at all, but the endpoint is
  undocumented, the token's handling and refresh are Claude Code's, and it could break on
  any release. The contract suite would have to cover it. The human offered to get a token.
  A plain Anthropic API key does **not** show subscription windows, so the public Messages
  API isn't a source.
- **D. Sliding staleness (ask 1) on its own** makes any of these failures cheap at low
  usage, and is worth doing whatever source is chosen.

Suggested: D plus A, plus `warn`-level logging of probe failures. Keep C as a fallback to research
if A proves flaky.

## The human on option A, 2026-10-04

Measured on dalek: an idle `claude -p` probe holds about 150 MB (RSS); agents hold 240–370 MB.
One probe per daemon, three daemons, so about 0.5 GB always on. The human, verbatim:

> Let's pause that, then. That's way too much memory, especially, and that's always on, always
> going, and scales with every project. I do not like that solution. Let's find a better one.

So option A (always use a long-lived probe per daemon) is **rejected**. Wanted: a usage source
with no long-lived `claude` process, and no per-project cost. Candidates still to weigh
(research, not settled):

- **C, direct HTTP** to the endpoint `/usage` and `get_usage` read, with the account's OAuth
  token: no process at all. Questions: which endpoint, where the token lives (macOS keychain on
  dalek, a file on Linux), who refreshes it, and how breakage is caught (the contract suite).
- **One machine-wide source** shared by every daemon (see xypj): at most one reader per machine,
  not one per project, whatever the reader is.
- **Spawn per poll, then exit:** about 1–2 s of startup per poll and no memory between polls.
  Still a `claude` process, but not an always-on one.
- **D, sliding staleness**, still wanted on its own: it makes fewer polls safe at low usage.
