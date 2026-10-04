---
id: 44ms
title: Start an unplanned focus period now, through an agent (for example quiet for sleep)
kind: feature
opened: 2026-10-04
repos: [bridle]
changes: []
specs: []
needs: []
see: [cvaq, hesj, mfgb]
tasks: [br-44ms]
---

## The ask


The human, verbatim (2026-10-03 23:00 ET, via the advisor):

> File a ticket not ready for work yet that I should be able to start an unplanned focus time
> through an agent. If that works then start quiet time now for sleep.

**Not ready for work yet** (the human): its task stays `pending`.

## Today (advisor, checked 2026-10-04 at edb3352)

- Focus periods are only the scheduled `[[focus]]` blocks in `~/.bridle/config.toml`
  ([[focus-hours-quiet-and-locked-cvaq|cvaq]]; `docs/design/agent-host/roles-and-config.md`,
  Focus hours). `bridle focus` has only `gate` and `reply` (hooks); nothing starts a period now.
- Agents may not write the config or `~/.bridle/focus-override.toml`: every role denies it, and
  the advisor role says "Never create or edit `~/.bridle/focus-override.toml` or the `[[focus]]`
  config, even when asked". So when the human asked, the advisor couldn't start quiet time; the
  human's only way is editing `[[focus]]` by hand (and `bridle daemon doctor`).
- That rule exists for the *override*, which lifts a lock and was made "hard to trip" on purpose
  (cvaq: "not a casual" switch; a hand-written file, a delay). Starting a period goes the other
  way: it makes things stricter, so the same caution needn't apply.

## Advisor's proposal (for the human's review)

- **`bridle focus start <mode> --until <time> [--reason ...]`** (e.g. `bridle focus start quiet
  --until 06:00`), callable by an agent on the human's request, and by the human. It writes a
  one-off period the daemon reads alongside `[[focus]]` (not the config file), recorded as an
  event and shown in `bridle status`.
- **Starting is easy, ending early is not.** No `focus stop` for agents: ending an unplanned
  period early goes through the existing hand-written override, so an agent can't start quiet
  time and lift it again. Whether an agent may start `locked` (which blocks prompts) as well as
  `quiet` is for the human.
- `--until` takes local time like [[focus-override-accept-local-time-in-until-like-the-focus-con-hesj|hesj]];
  capped (e.g. 24 h).
- The role rule changes to: agents may start a focus period when the human asks; they still
  never write the override or the `[[focus]]` config.
