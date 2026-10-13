---
id: br-dtf6
title: Per-project focus and budget switches, and one policy for the whole cluster (notes assistant always on)
kind: explore
created: 2026-10-13
created_by: external:advisor/product-manager
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
theme: operations
---

## The ask

Not scheduled: write-up only. The human, 2026-10-13 ~5:10 PM ET: "I'm happy to get this written up, but let's not start working on it until we have a clear, simple design, since it looks like I have a solution for what I need and it's not that big of a deal right now."

## The human's words

2026-10-13 ~6:40 AM ET (to advisor product-manager), verbatim, trimmed: "there is currently one type of project here that is pretty unique from the others, and that's my notes project. The notes project is my personal assistant ... It's currently an advisor running there, but I'm going to create a brand-new role for it, which will be a local role. That'll be interesting to see how we fix messaging. I haven't done that partially because of concerns about listing existing roles and messages, listing who can. Nobody else knows that role by name anyway. ... I want my personal assistant from the notes project to be always available, so I don't want any focus hours for it. ... probably every project on this box has the same focus rules and budget constraints, except this one project. I don't want to overcomplicate this, but I also don't really want to be forced to copy and keep up to date. ... Can a local project override the global focus and budget configs? ... Frankly, some of this config I would share between machines as well. This is another question on the machine config versus a global cluster config. I'm going to start referring to this as a cluster unless you can think of a better word. ... My ideal setup is that the entire cluster follows the same rules there, but I'm going to have certain projects I want to change both of those for. ... The practical consequence today is that on [Vesta, the NUC], I do not have any focus constraints because I don't want to interrupt the work on the notes project."

2026-10-13 ~5:10 PM ET: "there are a lot of complexities here. I just feel like it's kind of a big design thing that I'm not too interested in taking on. If there's a simple solution, how do you override the budget and focus hours? Are you going to have to relist a new... You have to recreate those sections and then blank them out, or is it much easier to add more constraints than it is to remove an existing constraint? I'm concerned about the complexity of that design. I want something really simple here. This is a very basic kind of thing that I don't want to overcomplicate."

## What exists (2026-10-13)

- Focus: `[[focus]]` in each machine's `~/.bridle/config.toml`. A project opts out with one line, `focus_hours = false` in its `.bridle/config.toml` (the gate is silent there). That already covers the notes project: Vesta can turn focus back on and opt notes out. Gap: a `locked` period kills every advisor pane on the machine regardless of the opt-out (`crates/bridle-daemon/src/focus.rs` `stop_advisors_if_locked`).
- Budget: `[budget]` in the machine config; a project may only lower thresholds, never raise or opt out. It governs bridle-hosted agents; an advisor's usage still counts toward the shared account limit.
- Cluster: none. Each machine has its own config; nothing is shared.

## Design direction (simple)

A project never re-lists or blanks out the machine's sections. It gets on/off switches only:

1. `focus_hours = false` (exists); make locked mode honour it too.
2. A matching one-line switch for the budget governor (e.g. `budget = false`), if needed at all.
3. Cluster: one shared policy (focus + budget) for every machine; projects keep only their switches. How it is shared (a synced file, the state branch, a peer pull) is the open question.
4. A local role for the notes assistant, and how others message a role they don't know by name.

Write the design only when the human asks; keep it to switches, no list merging.

## Budget pacing (the human, 2026-10-13 ~5:20 PM ET)

Verbatim: "The behavior I really want is something like pacing, right? Generally speaking, I don't want to use all my budget up in the first 3 days of the week. Maybe I do in some cases, but I kind of want to pace the week. The current limits are okay, but I could potentially run into a problem where I go up to 85% on the first day of the week. That would be disappointing and surprising. On the other hand, when we're getting close to the end, what I really wanted to do is drive really hard and try to use up all of the budget before it runs out. If it's Wednesday at 1 am and I still have 5% budget left, I actually just want it to go wild and try to use every bit of that budget until 3 am because I'm asleep anyway. I don't know how to do that. I don't know how to design that, really. ... budget and focus are a little bit different, but these are kind of similar things where we have some configuration that needs to be shared and coordinated. Budget in particular is a shared resource, so if every project tries to burst the budget right before it expires, we'll probably hit the limit, which would be okay."

What exists: the governor's `seven_day` thresholds are flat percentages (e.g. wind down at 90) whatever the day of the week; nothing paces and nothing sprints. `resets_at` for the 7-day window is known (statusline, rate_limit_event).

The human's design, 2026-10-13 ~5:50 PM ET, verbatim (trimmed): "1/7 is that we can use 14% of our budget daily and be fine, but that kind of strict pacing is a little bit more than I want. I want the freedom to go past that. ... If there's a line that shows the budget window, it goes up day by day: the first day's budget would be 14%, then 28%. We don't want to slavishly hit that because I work in bursts. ... If you exceed 10% more than the line, there should just be a notification, not a hold. That's probably an enhancement that would be a good benefit right now. Maybe a message goes to the orchestrator and the human: 'Hey, you're more than 10% over. Just letting you know.' Then there would be holds and wind-downs and such, but I wouldn't set a forced shut-off. I might if we're blowing past the budget on day one. I probably would want to have a hard cap on that. ... I would do day-based, not really hour-based. ... It's like PTO. We don't want to accrue PTO hourly. ... As soon as the clock rolls over, it's Wednesday at 3 am Eastern for me. ... As soon as that rolls over, you have 24 hours to spend 14% plus those settings. It'd always be 14%, whatever that is: 1/7 of the week, plus 10% gives you a warning, plus 15% gives you a wind-down, plus 30% gives you a stop, something like that. I think that does make sense."

So (the human's numbers):
- The line: a step per day from the 7-day reset (`resets_at`; the human's is Wednesday 3 AM ET). Day d of the week (1..7) has line = d/7 of the budget (14%, 28%, ... 100%), granted in full when the day starts, not accrued hourly.
- Above the line: +10 points: a notification only, to the orchestrator and the human ("more than 10% over the line"); +15: wind-down; +30: stop (a hard cap). Margins are configurable.
- The end-of-week sprint (above) still applies: in the last hours before the reset, use what is left.
- The +10 warning alone is "a good benefit right now": it can ship first, on its own.
It is account-wide, so it belongs in the shared (cluster) policy, not per project.

## Simplest version (the human, 2026-10-13 ~6 PM ET)

The human worked the numbers: with a 14%-a-day line, a +20 hold on day 5 is 72 + 20 = 92%, and on day 6 the line is already near 100, so margins on top of the line get hard to reason about. "What we have now works pretty well, but it's annoying to update. I just updated the budget to get some more work done, and that's obviously a bad thing. ... Let's try to keep this simple. What's the very simplest thing that would work? We have not exceeded our pace so far, so I'm not too concerned about building that yet."

Two small pieces, nothing else:

1. **Pace warning only** (no holds): when the day starts (counted from the 7-day reset), the line is day x 1/7 of the week (the human said "day number - 1 x 14, and then add 10"; advisor's reading: day 1 = 14%), plus a configured margin (10). When `seven_day` goes over it, one message to the orchestrator and the human. Thresholds stay as they are.
2. **End-of-week sprint with the current schedule**: today it can't be configured. `[[budget.schedule]]` periods replace only the `five_hour` thresholds ("seven_day (and every other window) is never affected", usage-and-budget.md), and a period's `max_workers` applies only through `bridle budget override`. The small change: let a schedule period also set the `seven_day` thresholds (and `max_workers` while it matches by the clock). Then the human writes one period, e.g. `name = "sprint", days = ["wed"], start = "00:00", end = "03:00"` with everything at 100, and nobody edits the config by hand. Needs a guard so the period follows the real reset day (`resets_at`), or a warning when they disagree.

Not now: the human isn't scheduling any of this yet.
