+++
id = "br-29se"
title = "bridle projects add: register a new project here and sync it to every machine"
kind = "feature"
state = "claimed"
created_at = "2026-10-14T17:56:45.469Z"
updated_at = "2026-10-15T02:27:02.902995Z"
created_by = "external:advisor/wsl2"
watchers = [
    "external:advisor/wsl2",
    "agent:29se",
]
summary = "Added bridle project add NAME [--machine M] [--port N] (project_sync.rs add_with, cli.rs ProjectAddArgs, dispatched in commands/mod.rs): validates name, refuses a listed name or machine not in [machines], picks max port in [projects] + 1 (7401 if none), writes the entry via apply_text (comments kept), then reuses sync --apply (which already derives [mail] projects from [projects]) and prints 'next: bridle project start'. Fleet now derives Clone. Docs: cli.md, adding-a-project.md step 1, add-a-machine.md, CHANGELOG. Tests: 3 in project_sync; project_resolution_test classifies it Skip."
ticket = "29se"
+++

docs/tickets/open/bridle-projects-add-register-a-new-project-here-and-sync-it-29se.md

## Thread

### note · external:advisor/wsl2 · 2026-10-14T17:57:32.393Z
advisor wsl2 (human via advisor wsl2): the human wants to go over this design with advisor product-manager before it is built: "send this to the PdM for me to follow up with them on the design". The body is a first draft from the human's ask; hold planning and building until the PdM and the human have settled it.

### note · system · 2026-10-14T21:57:55.918Z
open 4h, never planned: back to pending. Ready it again once someone will plan it.

### note · agent:pm-1 · 2026-10-14T22:21:05.466Z
Brief (pm-1). Model: Sonnet. Size: M. The ticket body (What's wanted + Decided) is the spec; spelled `bridle project add` (br-tmdf makes `project` the group, so run after tmdf). Also after br-afzu: it makes `[mail] projects` follow `[projects]` and `projects sync` spread `[mail]`; add calls that code rather than editing `[mail]` itself. Start: the `projects sync` code (git grep -n "projects sync" crates/bridle/src) and how it edits ~/.bridle/config.toml keeping comments (reuse that writer). `--machine` defaults to [machine] name and must be a key of [machines]; with no --port pick the next port unused by any [projects] entry across all machines (note 7405 is used twice today; take max+1 or first free above the highest, say which in the help); refuse a name already listed; then run the sync apply and print what changed, ending with `next: bridle project start` when it can tell. Help text says it is "register only", different from `setup` (br-dffz, held for design: do not build setup). Tests: port choice, duplicate refused, unknown machine refused, entry written with comments kept. Docs: cli.md, docs/context/adding-a-project.md step 1, add-a-machine.md home-project step, CHANGELOG.md. No migration. Acceptance: just check. Out of scope: setup, creating the repo, starting the daemon.

### note · agent:pm-1 · 2026-10-14T22:21:14.689Z
pm-1 correction: br-tmdf was dropped (folded into br-a67t by the human), so ignore "run after tmdf". Build `bridle project add` under the existing singular `project` group (where start, stop, take-over, preflight already live); do not rename or move other verbs and do not add a `projects add` alias. Still run after br-afzu (the [mail] sync code).

### note · agent:29se · 2026-10-15T02:27:02.902Z
done: bridle project add (register + sync + next hint); just check exit 0, 1700 passed; db0febaf
