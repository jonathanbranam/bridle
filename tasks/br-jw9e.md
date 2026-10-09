+++
id = "br-jw9e"
title = "bridle token pair, part 2: peer tokens, the [mail] peers opt-out, and pairing on project creation (sk7p, n63z)"
kind = "feature"
state = "planned"
created_at = "2026-10-09T18:34:18.344Z"
updated_at = "2026-10-09T18:34:43.328222Z"
created_by = "agent:pm-1"
watchers = [
    "agent:pm-1",
    "external:advisor/product-manager",
]
parent = "br-8c25"
+++

Ticket: docs/tickets/open/pair-machines-token-setup-over-ssh-sk7p.md, 'Design' section (approved 2026-10-09; authoritative) and ticket n63z, gdf3 (direction rule). Blocked by part 1 (br-8c25: the command, options, mesh, ssh plumbing, token-role list, spec file). Model: Sonnet.

Build:
1. The `peer` token type in `bridle token pair`: minted on the RECEIVING daemon as `peer:<sending machine>` and written into the sending machine's credentials.toml as `[peer] <receiving project>` (the direction rule); same-machine daemons need them too (all cross-project mail goes through /v1/forward). Every selected project that sends gets one per receiving selected project. Idempotent (present and not revoked is kept), --rotate, --dry-run, summary lines like "peer: minted on the receiver for <sender>", never a token value. Replace part 1's "not built yet" stub.
2. Opt-out: `[mail] peers = false` in a project's .bridle/config.toml leaves it out of peer tokens in both directions; role tokens still minted. Default true. Add the config field in crates/bridle-daemon/src/config.rs and document it where config is documented.
3. Project creation runs `bridle token pair --projects <new>` (n63z); find the project-create path (grep for project creation in crates/bridle and crates/bridle-daemon). If it fails (no ssh, unreachable machine) creation still succeeds and says to re-run the command.
4. Add to design/specs/token-pairing.md the scenarios for peer pairing, the opt-out, and project creation. Update docs/design/cli.md, CHANGELOG.md, docs/context/add-a-machine.md if present.
Tests with the same fake-ssh approach as part 1. Acceptance: just check passes. Migration: none; existing projects default to peers = true and get peer tokens on the next `bridle token pair` run (say so in CHANGELOG). Touches credential handling: the human reviews at landing.
Out of scope: anything in part 1, a per-project allowed-peers list (rejected: YAGNI, the human agreed on/off).

## Thread

### note · external:orchestrator · 2026-10-09T18:34:41.556Z
orchestrator: readied. Part 2 of the sk7p design the human approved (human via advisor/product-manager, 2026-10-09 ~2:20 PM ET: 'approved the design in ticket sk7p, section Design ... including the opt-out [mail] peers = false (comment c1). n63z (peer tokens) is folded' in).

### note · external:orchestrator · 2026-10-09T18:34:41.586Z
From orchestrator: br-jw9e readied; place it after br-8c25.
