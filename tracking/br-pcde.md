+++
id = "br-pcde"
title = "Mail bridge never owns a project when hostname differs from [machine] name: dalek's bridle mail is stuck"
kind = "bug"
state = "planned"
created_at = "2026-10-14T18:27:25.943Z"
updated_at = "2026-10-14T18:56:31.171246Z"
created_by = "external:advisor/nuc-rename"
watchers = ["external:advisor/nuc-rename"]
ticket = "pcde"
+++

docs/tickets/open/mail-bridge-never-owns-a-project-when-hostname-differs-from-pcde.md

## Thread

### note · agent:pm-1 · 2026-10-14T18:56:30.825Z
Brief (pm-1). Model: Sonnet. Size: S. CRITICAL (dalek's inbound bridle mail dead since Oct 12): top tier, next free slot. The ticket's "Why" and "Fix" are the spec. Make `bridle mail run` (crates/bridle/src/commands/orchestrator.rs ~360, 376) take the machine name from the daemon's own function (crates/bridle-daemon/src/lib.rs ~152: [machine] name, then hostname) so FileLocal::owns_project (crates/bridle-mail/src/local.rs) compares like with like; share the function, do not copy it (bridle-mail must not gain a daemon dependency: put the shared fn where both can reach it, e.g. bridle-api or the config crate, and say where). Check other readers of owner.toml that use raw hostname (pre-push hook, project start, take-over) and fix any that differ. Test: [machine] name set and different from hostname still owns the project. Docs: docs/design/mail.md if it names the host source, CHANGELOG.md. No migration; after release and a bridge restart the waiting mail is delivered. Acceptance: just check. Out of scope: other mail behaviour.
