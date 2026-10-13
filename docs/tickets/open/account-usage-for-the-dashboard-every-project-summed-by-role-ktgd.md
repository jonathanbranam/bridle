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
