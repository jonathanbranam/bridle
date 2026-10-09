+++
id = "br-jw9e"
title = "bridle token pair, part 2: peer tokens, the [mail] peers opt-out, and pairing on project creation (sk7p, n63z)"
kind = "feature"
state = "planned"
created_at = "2026-10-09T18:34:18.344Z"
updated_at = "2026-10-09T22:43:56.220569Z"
created_by = "agent:pm-1"
watchers = [
    "agent:pm-1",
    "external:advisor/product-manager",
]
summary = """
Peer tokens in `bridle token pair` (token_pair.rs): per sending machine with a peers-enabled project and each receiving peers-enabled project, the receiver mints peer:<sender machine> (new hidden helpers pair-peer-held/-active/-mint, in cli.rs and commands/misc.rs) and the sender stores it as [peer] <receiving project> via pair-store. Kept when held and live, --rotate/--dry-run work, a project that is the machine's only sender skips itself, same-machine projects included. Opt-out: [mail] peers = false (Config::mail_peers in bridle-daemon config.rs; pair-projects now prints a third field). Spec scenarios, cli.md, roles-and-config.md, add-a-machine.md and CHANGELOG updated. Tests: Sim unit tests plus a fake-ssh binary test; no token in argv/output.
CAVEAT / decision needed: `bridle init` only scaffolds and starts nothing, so a new project has no daemon yet. init therefore prints `bridle token pair --projects <name>` as a next step (after serve) and runs it itself only when a daemon for the name is already registered; failure never fails init. Running it automatically at first `bridle serve` was not built (daemon would shell out to a human-only command); say if you want that."""
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

### note · agent:jw9e · 2026-10-09T22:28:45.911Z
done: peer tokens, [mail] peers opt-out, init pairing step (see summary caveat on init); just check exit 0, 1421 tests; def81032. Credential handling: human reviews at landing.

### note · external:orchestrator · 2026-10-09T22:29:58.751Z
Orchestrator, answering manager-2's m-8552: (2) Accept as built: init prints 'bridle token pair --projects <name>' as a next step and runs it only when a daemon for the name is registered; failure never fails init; no pairing from 'bridle serve' (YAGNI, and the daemon must not shell out to a human-only command). (1) Yes, it touches credential handling, so it lands only with the human's go, as br-8c25 did. Hold bridle/jw9e unmerged (def81032). The human is driving tonight; I've put it to them through aide as not urgent. Free the worker slot for the next task meanwhile; don't merge main into the branch again unless asked.

### note · external:orchestrator · 2026-10-09T22:29:58.809Z
From orchestrator: br-jw9e: comment added (accept the init decision; hold the branch for the human's go, not urgent tonight)

### note · external:aide · 2026-10-09T22:37:24.028Z
From the human, via aide (2026-10-09 ~6:40 PM ET), verbatim: "land jw9e". Approved to land (branch bridle/jw9e, def81032).

### note · external:orchestrator · 2026-10-09T22:37:39.188Z
From orchestrator: br-jw9e: the human approved landing (via aide, ~6:40 PM ET: "land jw9e"; quoted on the thread). Merge bridle/jw9e and push now, by the usual merge steps.

### note · agent:manager-2 · 2026-10-09T22:37:46.719Z
Approved to land. Merge main into bridle/jw9e (main will also contain br-k6jd shortly; wait a few minutes if git log main shows it not yet), just check if code changed, then message me the tip.

### note · agent:jw9e · 2026-10-09T22:42:13.384Z
main merged into bridle/jw9e; tip 6f222a9f. br-k6jd was not in main after ~4 min, so not included; the merge only added a roadmap note (no code), last full check green on def81032. Tell me to re-merge once k6jd lands.

### note · agent:manager-2 · 2026-10-09T22:43:56.220Z
br-k6jd is on main now. Merge main into bridle/jw9e again, just check green on the result, message me the tip; I land at once.
