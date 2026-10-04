+++
id = "br-fc9a"
title = "bridle session launches with autoMode.environment in its --settings (ufrw, conservative variant)"
kind = "feature"
state = "planned"
created_at = "2026-10-04T13:39:51.820Z"
updated_at = "2026-10-04T13:40:38.978186Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
+++

original id: fc9a
Feature. Ticket: docs/tickets/open/bridle-session-launches-with-automode-environment-in-its-set-fc9a.md; design: docs/tickets/open/auto-mode-s-classifier-context-per-machine-for-bridle-projec-ufrw.md, 'Recommendation' and 'Smallest first slice'. The human, 2026-10-04 (br-bedz): 'bedz - yes conservative approved'.
Build the ufrw smallest first slice, conservative variant: (1) in crates/bridle-daemon/src/config.rs a pure function auto_mode_environment(machine, project, paths) -> Vec<String> (leading "$defaults", derived workspace line, machine lines, project lines) plus the two [auto_mode] environment config structs (machine file ~/.bridle/config.toml and project .bridle/config.toml; both must be accepted by RawConfig). Trust lines (repos, domains, buckets) come ONLY from the machine file; a project's [auto_mode] environment may only tighten (sensitive targets, prod hosts); define how that is enforced as ufrw says. Unit-tested. (2) crates/bridle/src/session.rs: add autoMode.environment to orchestrator_settings and advisor_settings output (beside with_layer_hooks), tests next to settings_are_valid_json. (3) Docs: docs/design/cli.md (config keys), the doc that describes session settings (workflow-layers.md or an agent-host doc), CHANGELOG.
Never write ~/.claude/settings.json. Deferred (ufrw, with reasons): 'bridle auto-mode print'; autoMode in the daemon's spawn --settings.
Conflicts: config.rs RawConfig is also edited by br-jmpf (accept-only [gateway]/[interactions]); session.rs by br-m7mp later. Land after br-jmpf; rebase on main first.
Acceptance: just check passes. Model: Sonnet. Migration: none (no existing project file changes; absent [auto_mode] adds only $defaults and the derived line). Out of scope: editing any existing project's config.
