+++
id = "br-3ch7"
title = "K1: refuse a project move while committed config has absolute paths"
kind = "feature"
state = "integrated"
created_at = "2026-10-12T13:38:40.841Z"
updated_at = "2026-10-12T14:05:56.335846Z"
created_by = "external:advisor/product-manager"
watchers = [
    "external:advisor/product-manager",
    "agent:w3ch7",
]
branch = "bridle/w3ch7"
commit = "0a296efd3ead6ffb6a3aa8268e960f5af2b2f7a3"
summary = "Added 'bridle project preflight [--repo]': a list of named checks (CHECKS in crates/bridle/src/preflight.rs, K2 appends), first 'absolute paths' scanning .bridle/config.toml for absolute/~ workflow and system_prompt, refusing with file:line key and 'fix by hand'. doctor warns via the same absolute_path_findings ('portable paths'). Working-tree scan, not git show HEAD. No --to flag (yagni, K2 adds). Docs: cli.md, CHANGELOG; classified in project_resolution_test."
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

### note · agent:w3ch7 · 2026-10-12T14:05:50.969Z
built: 23d71388 (branch bridle/w3ch7)

### note · agent:w3ch7 · 2026-10-12T14:05:51.002Z
done: bridle project preflight (named-check list, first check refuses absolute/~ workflow and system_prompt with file:line key), doctor warns; just check exit 0, 1598 tests passed, main merged; 23d71388. Note: no --to flag yet (K2); scans working tree not HEAD.

### note · agent:manager-2 · 2026-10-12T14:05:56.335Z
integrated: 0a296efd3ead6ffb6a3aa8268e960f5af2b2f7a3 (branch bridle/w3ch7)
