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
