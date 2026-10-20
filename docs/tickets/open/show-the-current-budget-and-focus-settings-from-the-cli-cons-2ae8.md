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
  design: {requires: [proposal], review: human}
  specs: {requires: [proposal], review: human}
  tasks: {requires: [design, specs], review: human}
written: [proposal, design, specs]
signed: [proposal, design]
claimed_by: human
---

Stages: [design](http://dalek.tailbc91f5.ts.net:7878/p/bridle/docs?path=docs/tickets/open/show-the-current-budget-and-focus-settings-from-the-cli-cons-2ae8/design.md), [specs](http://dalek.tailbc91f5.ts.net:7878/p/bridle/docs?path=docs/tickets/open/show-the-current-budget-and-focus-settings-from-the-cli-cons-2ae8/specs.md)

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

`bridle budget` with no verb already prints the governor's status, and `--schedule` prints every period. `bridle focus` has nothing: only the two hooks. So the human cannot check their focus hours from the CLI, and the two commands read differently. The human asked for them to match (2026-10-08), and agents cannot read `~/.bridle/config.toml`.

### What changes

- Add a read for focus with the same shape budget already has: bare `bridle focus` prints the configured `[[focus]]` periods, the period in effect now and when it ends, and any override. The human said either spelling is fine if both match; budget's bare form is already built, so use it (no `show` verb, no rename).
- Give focus the `--schedule` flag budget has, with the same layout (every period, its span and settings), and `--json` on both, one layout.
- Make budget's output match: configured periods, period in effect now and its end, active override or hold. Fill any gap the current status leaves.
- Make the change verbs match in naming if they differ. Focus has none today (no override); budget keeps `hold`, `release`, `override`, `override-clear`, `max-workers`. Adding a focus override is not part of this.
- Capabilities touched: `docs/design/cli.md`, `docs/design/usage-and-budget.md`, `docs/design/agent-host/roles-and-config.md`. No spec file exists for either command; the specs stage decides whether to add one.

### What does not change

- The hooks `focus gate` and `focus reply`, and their silence rules.
- Budget's verbs and the governor's behaviour (hold, wind down, stop).
- The config format of `[[focus]]` and `[[budget.schedule]]`.
- No **BREAKING** change: bare `bridle budget` and `--schedule` keep working.

### Risk and impact

Low. Mostly read-only output in the CLI crate and the docs. Focus is read from the human's config by the CLI or daemon; the design must decide which, and agents must get the answer without reading the file. Risk is a changed budget layout breaking a script that parses `--json`; keep existing JSON fields.

### Out of scope

- A focus override or hold: not yet, because nobody asked and focus has no state to override.
- A combined `bridle status` section for focus: not yet, because the two commands are the ask; add it when wanted.
- A `show` verb on both: not needed, bare form already exists on budget (KISS, one name per action).

### Open questions

None.

---
