---
id: br-3ch7
title: "K1: refuse a project move while committed config has absolute paths"
kind: feature
created: 2026-10-12
created_by: external:advisor/product-manager
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-kt25]
parent: br-kx5u
theme: multi-machine
resolved: 2026-10-12T14:06:27Z
---

## The ask

Child K1 of the kx5u design (br-kt25, '## Design', section 4), written by the designer; the human, 2026-10-12: "order - whatever order you think is best" (PdM: K1 and K2 first, independent of the open grilling questions).

**K1. Refuse a move while committed config has absolute paths (S).** Addresses R1/5.6. Ask: a
check, `bridle project preflight <P> [--to M]` (read-only, started here), whose first item scans
P's committed `.bridle/config.toml` (and `.bridle/roles`, rules) for absolute or `~` values in
`workflow` and `system_prompt` and refuses with file and line; `bridle doctor` runs the same scan.
Acceptance: a fixture config with `workflow = "/Volumes/..."` is refused naming the key; a
relative or unset one passes; doctor warns. Migration: no automatic rewrite (where the workflow
should come from is a decision: release mode or a checkout, see jw75); bridle-ui, track-web and
data-contracts must be fixed by hand before they move, and the refusal says so.
