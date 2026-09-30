---
id: cvaq
title: "Focus hours: quiet and locked periods that keep the human on their real work"
opened: 2026-09-30
repos: [bridle]
changes: []
specs: [docs/design/cli.md, docs/design/agent-host/roles-and-config.md, workflow/base/roles/advisor.md, workflow/base/roles/orchestrator.md]
needs: []
see: [higher-budget-thresholds-on-a-schedule-n9qh, email-and-texting-for-bridle-and-track-web-rs7p, accounting-bridle-work-vs-real-projects-kpgy]
---

## The ask

The human, verbatim (2026-09-30, to the advisor):

> another ask to research: when I'm at work I need a self-governing ability so that you can
> help me stay focused on my work and not spend all day on bridle or personal projects (which
> is more fun). suggest a design for something like: quiet hours - work can continue, orch does
> things, but more quietly, advisors can run but during quiet hours they prompt me to stop
> fiddling and go work (or go to sleep, etc). I might enforce a full lock out if it becomes a
> bigger problem, so consdier that in the design as well.
>
> there should be an override mechanism, but it should be hard to trip, not a casual (ignore
> that) from me, but something I have to go edit a file; this would apply to remote control as
> well. I can always send an email if I want to note something important without having an
> interaactive session.

After the advisor's proposal (below), verbatim:

> yes on override by manual file creation. only concern is if I'm remote that will be hard but
> it's ok for now.
>
> for lock mode, I think orch must be running or the system will not do work effectively. but I
> agree we could lock it and shut down all advisors and refuse to start them and orch refuses
> all chat.
>
> 1. work hours 8-6 ET weekdays
> 2. nudge on first prompt, then every 5 minutes or more often is fine; 30min is too long
> 3. sure, let's go with 10min delay for now make it a config option.
>
> file this - it isn't overly urgent but something that can be built in the next few days.

A correction, verbatim, the same day:

> sorry 8am to 5pm on weekdays should be quiet; and I'll likely disable quiet time through
> config during travel; restarting the daemone is pretty easy now for config updates, yeah?
> Otherwise, that will be annoying.

So config edits must take effect without fuss: the gate hook is a short-lived CLI that can read
`~/.bridle/config.toml` on every call, so it needs no restart; the daemon's side (holding the
human's notes, shutting down advisors) should pick up a changed `[[focus]]` without a restart
too, or at most with `bridle restart` (see
[[reload-config-without-a-restart-9t54|reload config without a restart]]).

## Design, as agreed

**Schedule.** `[[focus]]` periods in `~/.bridle/config.toml`, the same shape as
`[[budget.schedule]]` (n9qh: `days`, `start`, `end`, host-local time, midnight crossing), each
with a `mode`:

```toml
[[focus]]
name  = "work"
days  = ["mon", "tue", "wed", "thu", "fri"]
start = "08:00"
end   = "17:00"
mode  = "quiet"          # or "locked"
```

Sleep hours are the same shape (the human's sleep is about 11:00 PM to 7:00 AM Eastern, per
n9qh; not set here).

**Quiet.**

- The workforce is untouched: managers, workers and the orchestrator carry on.
- Less noise to the human: routine notes and questions are recorded but not pushed; the
  orchestrator doesn't page the human except for something that stops all work. One catch-up
  summary when the period ends (the mail bridge's digest could carry it).
- Interactive sessions (advisor, orchestrator, Remote Control) still work, but nudge: a
  `UserPromptSubmit` hook, `bridle focus gate`, adds context such as "Quiet hours (work) until
  5:00 PM ET" on the **first prompt, then again when 5 minutes or more have passed** since the
  last nudge. The role text says: lead with a one-line nudge to go back to work (or to bed), keep
  the answer minimal.
- Optional: quote the current plan from the human's Time Block through meta-notes'
  `checkin status --json` ("your plan at 10:00: write spec").

**Locked.**

- The orchestrator **keeps running**; the system doesn't work well without it. It does its
  work but refuses all chat: the gate hook blocks every prompt with "Locked until 5:00 PM. Email
  bridle@dev.branam.us if it matters."
- All advisors are shut down when the period starts, and `bridle session advisor` refuses to
  start one.
- Email still reaches the orchestrator (rs7p): the human's one channel in.

**Where the gate runs.** In the `--settings` that `bridle session` already passes (beside the
`SessionStart` hook, `crates/bridle/src/session.rs:25`), and in each bridle project's
`.claude/settings.json`, so a plain `claude` in the repo is gated too. Remote Control prompts are
prompts in the same local session, so the same hook should cover them; verify this.

**Scope.** Only bridle-managed projects on the machine, never the human's day job. A project can
opt out in its own config. Each machine has its own config (the work laptop included).

**Override: a file the human writes by hand.**

- No CLI command (unlike `bridle budget override`). The human creates
  `~/.bridle/focus-override.toml` with `until` (capped, e.g. 2 hours) and a written `reason`.
- It takes effect only after a delay: `[focus] override_delay` (default 10 minutes).
- Agents can't write it for the human: deny rules on `Edit`/`Write` of `~/.bridle/focus*`
  and the `[[focus]]` config, and role text that says never to create or edit it, even when
  asked.
- Each override is an event, shown in `bridle status` and the catch-up summary.
- Known gap, accepted for now: hard to use when the human is remote.

## Order

Quiet and the gate hook first; locked on top (small once the gate exists).
