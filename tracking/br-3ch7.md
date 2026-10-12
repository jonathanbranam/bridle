+++
id = "br-3ch7"
title = "K1: refuse a project move while committed config has absolute paths"
kind = "feature"
state = "claimed"
created_at = "2026-10-12T13:38:40.841Z"
updated_at = "2026-10-12T13:44:06.050833Z"
created_by = "external:advisor/product-manager"
watchers = [
    "external:advisor/product-manager",
    "agent:w3ch7",
]
ticket = "3ch7"
+++

docs/tickets/open/k1-refuse-a-project-move-while-committed-config-has-absolute-3ch7.md

## Thread

### note · agent:pm-1 · 2026-10-12T13:38:58.893Z
Brief (pm-1). Model: Sonnet. Size: S. Read the ask above and br-kt25's '## Design' (sections 4 and 5.6) for context.

Goal: start `bridle project preflight <P> [--to M]` as a read-only command, with one check so far: scan P's committed `.bridle/config.toml` (and `.bridle/roles`, rules) for absolute or `~` values in `workflow` and `system_prompt`; refuse with file, line and key, and say the value must be fixed by hand before the move (no automatic rewrite; where the workflow comes from is a decision, see jw75). `bridle doctor` runs the same scan and warns.

Files likely: crates/bridle/src (clap CLI: project subcommands; doctor), crates/bridle-daemon config code, docs/design/cli.md, CHANGELOG.md. Build the scan as a function the doctor and preflight both call. Design the preflight command so K2 (br-bb2n, blocked by this) can add checks to it: a list of named checks, each pass/fail with a message. Nothing beyond that (yagni).

Acceptance: fixture config with `workflow = "/Volumes/..."` is refused naming the key and line; a relative or unset one passes; doctor warns; `just check` passes.

Out of scope: the move command itself, ssh checks (K2), rewriting config.
