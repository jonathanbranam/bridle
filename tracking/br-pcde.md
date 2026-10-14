+++
id = "br-pcde"
title = "Mail bridge never owns a project when hostname differs from [machine] name: dalek's bridle mail is stuck"
kind = "bug"
state = "integrated"
created_at = "2026-10-14T18:27:25.943Z"
updated_at = "2026-10-14T19:58:38.591266Z"
created_by = "external:advisor/nuc-rename"
watchers = [
    "external:advisor/nuc-rename",
    "agent:pcde",
]
branch = "bridle/pcde"
commit = "5fe80fdd7758ed955eceed08b321d529b261951a"
summary = "Mail bridge ownership now uses bridle_api::machines::owner_names (the daemon's own function: [machine] name, then hostname) via FileLocal.hosts, replacing the raw hostname (removed local_hostname). Test: a name list containing either name owns the project. Other owner.toml readers (take_over, project_stop, daemon) already use owner_names. docs/design/mail.md and CHANGELOG updated."
ticket = "pcde"
+++

docs/tickets/open/mail-bridge-never-owns-a-project-when-hostname-differs-from-pcde.md

## Thread

### note · agent:pm-1 · 2026-10-14T18:56:30.825Z
Brief (pm-1). Model: Sonnet. Size: S. CRITICAL (dalek's inbound bridle mail dead since Oct 12): top tier, next free slot. The ticket's "Why" and "Fix" are the spec. Make `bridle mail run` (crates/bridle/src/commands/orchestrator.rs ~360, 376) take the machine name from the daemon's own function (crates/bridle-daemon/src/lib.rs ~152: [machine] name, then hostname) so FileLocal::owns_project (crates/bridle-mail/src/local.rs) compares like with like; share the function, do not copy it (bridle-mail must not gain a daemon dependency: put the shared fn where both can reach it, e.g. bridle-api or the config crate, and say where). Check other readers of owner.toml that use raw hostname (pre-push hook, project start, take-over) and fix any that differ. Test: [machine] name set and different from hostname still owns the project. Docs: docs/design/mail.md if it names the host source, CHANGELOG.md. No migration; after release and a bridge restart the waiting mail is delivered. Acceptance: just check. Out of scope: other mail behaviour.

### note · agent:pcde · 2026-10-14T19:47:32.592Z
done: mail bridge uses owner_names; just check exit 0, 1680 tests passed (after merging main); e602fba512727b9ddff9390abe7314375ad37066

### note · agent:manager-2 · 2026-10-14T19:47:38.198Z
built: e602fba512727b9ddff9390abe7314375ad37066 (branch bridle/pcde)

### note · agent:manager-2 · 2026-10-14T19:58:38.591Z
integrated: 5fe80fdd7758ed955eceed08b321d529b261951a (branch bridle/pcde)
