---
id: br-2ae8
title: Show the current budget and focus settings from the CLI, consistently
kind: feature
created: 2026-10-18
created_by: external:advisor/tj7q
theme: agents-and-cli
version: 2
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-4ge4, br-ewk8, br-cvaq]
stages:
  proposal: {review: human}
  design: {requires: [proposal]}
  specs: {requires: [proposal], review: human}
  tasks: {requires: [design, specs]}
---

## The ask

Replaces br-4ge4 (version 1) as the real ticket for br-ewk8, the human's hands-on try of version 2 (Stages MVP step 6, epic br-tj7q). The human, 2026-10-18, to advisor tj7q: "yes, agree on 4ge4".

Neither focus nor budget has a command that shows its current settings. `bridle focus` has only the hooks (`gate`, `reply`), and `bridle status` says nothing about focus. `bridle budget` has only verbs that change things (`hold`, `release`, `override`, `override-clear`, `max-workers`). Today the only way to check is to read `~/.bridle/config.toml` by hand, and agents aren't allowed to read that file.

The human, 2026-10-08 (work hours), after editing their focus settings: "can you confirm the times? Is there a CLI that shows the current settings?" Told there is none, for either: "fine; make the CLI consistent for budget and focus."

The ask: give `bridle focus` and `bridle budget` one shared shape for reading their settings: the same verb (e.g. `show`), `--json`, and the same layout. Each should show:
- the configured periods or schedule
- the period in effect now, and when it ends
- any active override or hold

If the commands that change settings differ between the two in naming, make those consistent as well. The design picks the verb and the layout.

Naming, the human, 2026-10-08:

> I mean the naming here; budget show focus show or skip 'show' for both.

So the two read the same way: either `bridle budget show` and `bridle focus show`, or a bare `bridle budget` and `bridle focus` that print the settings. Either is fine; they just have to match.

---

## Proposal

### Why

### What changes

### What does not change

### Risk and impact

### Out of scope

### Open questions

---
