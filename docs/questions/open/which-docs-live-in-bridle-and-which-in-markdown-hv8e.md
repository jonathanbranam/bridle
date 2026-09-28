---
id: hv8e
title: Which docs live in bridle's records and which in markdown files
opened: 2026-09-28
repos: [bridle, data-contracts]
changes: []
specs: []
needs: []
see: []
---

## The ask

The human, verbatim (2026-09-28), answering whether bridle should take over link checking
for data-contracts (survey question 7 in `docs/context/onboarding-data-contracts.md`):

> link-checking - can keep for now; I'm still struggling to determine which docs live in
> bridle and which in markdown and how that should all work

## Notes

- Today bridle's own tickets are markdown files in `docs/questions/` and `docs/spikes/`,
  each with a matching bridle task whose body points at the file (`docs/README.md`). New
  work in bridle is queued as tasks on the state branch (j479).
- data-contracts has its own ticket tree (`docs/tickets/`, frozen at onboarding, to be
  deleted once bridle is adopted: its ticket 3fm6), a plan of record, and specs under
  `openspec/specs/`.
- Link checking (`check-tickets.py`, in both repos) depends on the answer.
- Related: [[ticket-state-without-moving-files-p2ys|ticket state without moving files]].
