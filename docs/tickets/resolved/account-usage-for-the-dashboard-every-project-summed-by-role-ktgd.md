---
id: br-ktgd
title: "Account usage for the dashboard: every project summed by role, name and instance, per week, day and five-hour window, with context per turn"
kind: feature
created: 2026-10-13
created_by: external:advisor/product-manager
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-qhsa, br-xxw9, ui-h79b, br-xfsp]
theme: performance
parent: br-rdkn
resolved: 2026-10-17T02:08:19Z
---

The bridle half of the account usage dashboard (UI half: the bridle-ui ticket linked below). Epic br-rdkn.

## The human's words

2026-10-13 ~6:30 PM ET (to advisor product-manager), verbatim, trimmed: "I think this is important enough to go ahead and schedule and get built with our window that we have left. I want to know all the statistics. ... I want to see a weekly breakdown, a daily breakdown. I also want to see a per-5-hour-block breakdown. The kinds of breakdowns that we should have are per role. We should be tracking all of this per instance, right? ... every agent should have a unique ID somewhere. I don't really need to see it, but it should be in the system. ... For bridle, we run bridle sessions, so every time we start a bridle session, give that an ID, and then that should be tracked for everything. ... We should also include the role and name, right? I want to know, at the end of the week, did my product management advisor use up all the tokens? I want to roll up by name across sessions, but I also want to be able to see the individual session details. This data is very small. ... 5 minutes, 10 minutes. I think both are fine. ... And we should track two things for every agent: 1. Context: How is their context growing? Every tick that that agent has an action of some kind, record its context or get it every 10 minutes. Don't wake up an agent to get its context. We just store whatever we have. 2. We want this cost number. Cost also grows, but it's not going to grow the same as context ... The only other overlay on top of that would be keeping track of when the 7-day usage resets and when these 5-hour resets happen. ... I have no idea if we've hit any 5-hour limits in the last week. I don't think we have, but it should be in a dashboard."

On a share of the weekly budget (earlier, same evening): "I spent 30% of that in interactive sessions, and 20% went to the manager. I just want that breakdown. That is what I'm really after."

On five-hour windows: "I think I've read that it's not every 5 hours, but only 5 hours where you have usage." No assumption needed: `rate_limit_history` records each window's `resets_at`, so the real windows come from the data.

## What is recorded today (2026-10-13)

- Hosted agents: `turns` has one row per turn with `agent_id` (unique per instance, kept after removal), name, role, model, tokens and cost. Context is only the latest value on `agents.context_tokens`: no history.
- External sessions (`bridle session`, the human's own, via `bridle statusline`): `interactive_usage` has one row per change with `session_id` (unique per session), identity (role/name), cost, tokens and context.
- Limits: `rate_limit_history` has five_hour and seven_day utilization and `resets_at` on every change, for 90 days.
- Each project's daemon keeps only its own; nothing sums the account across projects.

## The ask

1. Record context per turn for hosted agents (a `context_tokens` column on `turns`, from what the turn already reports; never wake an agent to get it). External sessions already have it.
2. One account-wide view through the gateway, summed over every project it reaches (unreachable ones reported as unreachable, never zero; this takes over br-qhsa's merged route):
   - periods: the week from the seven_day reset (the human's: Wednesday 3 AM ET), each day of it, and each real five-hour window (from `resets_at`);
   - per period: cost, tokens and context growth by role, by name (rolled up across sessions and instances) and by instance or session, hosted and external together;
   - the share of the week's seven_day percentage each group used, estimated as its share of the week's cost (label it an estimate: Anthropic doesn't publish the limit's weights), plus the measured ratio (seven_day points per dollar) so the estimate can be checked week to week;
   - the five_hour and seven_day series with each window's peak and whether it hit the limit.
3. `bridle usage` can show the same (e.g. `--week`, `--by name`), so it works without the UI.
4. Docs: usage-and-budget.md, storage.md, api.md; CHANGELOG.

Acceptance: a test with two projects, a hosted agent and an external session across two five-hour windows gives the right per-window, per-day and per-week sums by role, name and instance, and the shares add up to the week's seven_day percentage.

Out of scope: cost per ticket (separate ticket), other machines' projects if the gateway can't reach them.

## Decisions (stepping through with the human, 2026-10-13 ~7:20 PM ET)

1. **Every machine records from the upgrade on; combining comes later.** The human: "The first version should track usage across machines for everything. That needs to work immediately, and then we can build the solution later to combine those logs in a reasonable way. As soon as the upgrade is installed, all the data should be tracked." So: every daemon on every machine (dalek, Remus, Vesta) records the per-turn and per-session data as soon as it runs the new binary, and each record carries what a later merge needs (machine and project, alongside instance/session id, role, name). This ticket's sums cover what the gateway reaches; merging across machines is a follow-up ticket.
2. **Count every Claude Code session bridle can see, including those outside any bridle project.** The human: "Sure, count everything you can, but we understand that sessions not started from the command line won't be counted. Sessions outside of Claude Code won't be counted. That's an acceptable risk." Today `bridle statusline` (the global status line on dalek) reports only to the daemon of the project the session works in, and drops the rest. Send those to a machine-level "outside any project" bucket (e.g. a daemon chosen per machine) so they are counted. The desktop app, claude.ai and other tools are not counted (accepted).
3. (Advisor note for the planner, not the human's words.) An external session's first statusline report is its baseline and counts 0 (usage-and-budget.md, hd28), so each session's first turn or more is never counted. For a session bridle has never seen whose running total starts near zero, count the first report in full; keep the baseline rule only where the total clearly includes spend from before (a resumed session). The human's "count everything you can" covers this.
4. **Groups.** The human: "for now, that grouping is correct. There's one Orchestrator per machine, so that should be handled there. There's one Aide per project, so that should be tracked to the project. Advisors need name tracking. I don't use names for anything else right now." And: "This is a time when we should have had specs, and specs would help here." So: every record keeps role, name, instance/session id, project and machine; the view groups by role (the human's own sessions as `unbridled`, orchestrator per machine, aide per project, advisors by name, pm, managers, workers, others), and any group can be expanded to its names and its single instances or sessions. No per-role special cases.
5. **Startup cost** (the human): "we should have the ability to tell, if possible, how much context our startup eats up and the dollar amount there. Wherever that's possible, we can do it after the fact if we have the data, but that would be very interesting to have." So the data must allow it: per-turn context and cost from the first turn of every hosted instance (step 1), and the first report of every external session counted (note 3). A report or query for it can come later; recording it is this ticket.
6. **Sessions for hosted agents too.** Today `turns` keys on `agent_id` only (store.rs), so a long-lived manager or pm-1 is one undivided line across restarts and context renewals. Record the Claude `session_id` on every turn, so every agent has three levels: role, name (all its sessions), and session (from start or restart to handover or renewal). The human: "yes, absolutely, that's what we want. We want to know what we're spending in restarts."
7. **Why: compare costs before and after changing the renewal limits.** The human: "we're going to ... collect this baseline for a few weeks or days or whatever, and then we have to change our limits, right? We can lower those context limits where handoff is enforced and see what happens ... Usage goes up because we're restarting fresh and we had a bunch of cached data and cached tokens that are now uncached ... The startup tokens are always going to be uncached, I assume. I don't know if there's caching between sessions or not. ... I want to be able to track the cost, then make a change, like forcing restarts at 150 or 100, and see if that ends up lowering our costs or not." So each session also records how it started and ended (spawn, restart, renewal at N tokens, handover, stop) and the renewal limit in effect, so a week at one limit can be compared with a week at another. Per-turn cache_read vs cache_write on a session's first turns answers whether startup is cached across sessions.

## Split (the human, 2026-10-13 ~7:45 PM ET: "split up into nice little chunks")

Held as the design record; built as: pac3 (turns: session id, context), 2kam (session start/end, renewal limit), mksa (count a new session's first report), drvx (sessions outside any project), pemk (periods: week, days, five-hour windows), zn6s (breakdown by role, name, session; shares), v6cg (gateway sum; takes over qhsa), v58q (CLI), a65d (across machines, future). UI: ui-h79b. Resolve this when all but a65d have landed.
