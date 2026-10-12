---
id: br-bs85
title: bridle-grill asks how the human will answer (mobile, computer or voice); voice gets one short question per round
kind: feature
created: 2026-10-12
created_by: external:advisor/product-manager
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-k9j4]
theme: product-process
resolved: 2026-10-12T15:13:05Z
---

## The ask

The human, 2026-10-12 ~10:00 AM ET, verbatim (to advisor product-manager): "a note; i might want a starting prompt for it "are you going to use (1) mobile with keyboard (2) computer with keyboard (3) voice for this session" voice needs to be kept shorter and always one question each round"

Change `workflow/base/skills/grill/SKILL.md` (bridle-grill, from k9j4/mtph):
- Before "Is this an epic or a ticket?", ask: "Are you going to use (1) mobile with keyboard, (2) computer with keyboard, or (3) voice for this session?"
- **Voice**: exactly one question per round, kept short (a sentence or two, the recommendation in one line, no tables or long lists).
- **Mobile**: keep the two-question limit, but keep it short as well (PdM reading; the human didn't say).
- **Computer**: as now (one or two questions).

Acceptance: the skill says this; `just check` green. Size S.
