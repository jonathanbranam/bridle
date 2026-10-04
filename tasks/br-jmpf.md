+++
id = "br-jmpf"
title = "A [gateway] section in ~/.bridle/config.toml stops every daemon from starting (unknown field)"
kind = "bug"
state = "planned"
created_at = "2026-10-04T13:11:47.608Z"
updated_at = "2026-10-04T13:48:53.548950Z"
created_by = "external:advisor"
watchers = ["external:advisor"]
+++

original id: jmpf
Critical bug, size S. Ticket: docs/tickets/open/a-gateway-section-in-bridle-config-toml-stops-every-daemon-f-jmpf.md. The gateway reads [gateway] and [interactions] from ~/.bridle/config.toml (crates/bridle-gateway/src/config.rs), but the daemon's RawConfig (crates/bridle-daemon/src/config.rs ~2189, deny_unknown_fields) has no field for either, so adding either section makes every daemon fail to parse its config and not start.
Fix: add accept-only 'gateway' and 'interactions' fields to RawConfig exactly the way [mail] is accepted (config.rs ~2246: 'mail: Option<toml::Value>' with allow(dead_code) and a comment saying who owns the section). Tests: a config with a [gateway] section parses; a config with an [interactions] section parses; both together parse. Also check no other section a bridle crate reads from this file is missing from RawConfig (bridle-mail's [mail] is covered) and fix any found in the same way. Docs: roles-and-config.md note that [gateway]/[interactions] belong to bridle gateway and the daemon only accepts them; CHANGELOG. Acceptance: just check passes. Model: Sonnet. Migration: none. Out of scope: the gateway's own parsing, the service install (br-c657). Do not edit any existing project's config.
Worker start waits for the orchestrator's go (the human is moving the daemons to launchd).

## Thread

### note · agent:gateway-config · 2026-10-04T13:48:53.548Z
Change done (gateway/interactions accept-only fields, test accepts_sections_owned_by_other_crates, roles-and-config.md, CHANGELOG), uncommitted. just check fails 3 of 3 runs on unrelated bridle serve::tests::login_warning_only_when_known_logged_out (serve.rs:285, br-faf5; passes alone, fails in the parallel suite). First run also failed sessions::tests::the_hard_limit_has_no_override_and_forces_a_restart (ENOENT, not repeated). Not touching them (out-of-scope). Asked manager how to proceed.
