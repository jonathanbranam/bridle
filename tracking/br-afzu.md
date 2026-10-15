+++
id = "br-afzu"
title = "New projects join [mail] projects on every machine by default (opt-out)"
kind = "feature"
state = "integrated"
created_at = "2026-10-14T15:38:37.264Z"
updated_at = "2026-10-15T01:21:02.327296Z"
created_by = "external:advisor/wsl2"
watchers = [
    "external:advisor/wsl2",
    "agent:afzu",
]
branch = "bridle/afzu"
commit = "1a73fd3757675ec615913a55c462f6451594fc73"
summary = "projects sync now spreads this machine's [mail] section key by key (project_sync.rs, Entry::Mail), with [mail] projects derived as bridle first then every [projects] name; --apply fixes this machine's own file too. [machine] mail = false (config.rs machine_mail_on, never synced) makes project start skip the bridge with one line, mail install refuse, and preflight --to skip the AWS check (remote grep for mail = false). Mail on but no AWS credentials: project start skips with one warning; doctor warns once (AWS or mail.key). Caveat: project start checks AWS only, not mail.key (a fresh machine legitimately has none until first use). Docs: mail.md, cli.md, mail-on-a-machine.md, adding-a-project.md, CHANGELOG. Items 1-5 all done in this ticket."
ticket = "afzu"
+++

docs/tickets/open/new-projects-join-mail-projects-on-every-machine-by-default-afzu.md

## Thread

### note · agent:pm-1 · 2026-10-14T15:45:44.958Z
Brief (pm-1). Model: Sonnet. Size: M. The ticket body is the decided design ("What's wanted" plus the `[machine] mail = false` table); build that, no redesign. Stop point: land (1) and (2) first; if (3)-(5) get long, tell the manager and they become a second ticket.

1. `[mail] projects` is derived as every name in `[projects]`, `bridle` first, the same on every machine; no per-project opt-out.
2. `bridle projects sync` compares and with `--apply` fixes the `[mail]` section like `[machines]` and `[projects]` (find it: git grep -n "projects sync" crates/bridle/src).
3. `[machine] mail = false` (default true), never copied by sync: `project start` skips the bridge with one line, `mail install` refuses naming the key, `project preflight --to` skips the AWS check.
4. Mail on but machine not set up (no AWS credentials / mail.key): `project start` skips the bridge with one quiet warning instead of failing; `bridle doctor` warns once ("set up mail ... or set [machine] mail = false"). No inbox messages, no per-poll repeats.
5. Docs: docs/design/mail.md, cli.md, docs/context/mail-on-a-machine.md and adding-a-project.md; CHANGELOG.md.

Files: crates/bridle (projects sync, project_start, mail install, preflight_remote, doctor.rs), crates/bridle-daemon/src/config.rs for the `[machine]` key. Share cli.md and token_pair/project_start with br-sh8y and br-su62: run after them. Migration: none (sync --apply updates existing machines' `[mail]`; touches only ~/.bridle/config.toml, never a project's files). Tests: sync diff/apply on [mail]; project start with mail=false and with missing credentials. Acceptance: just check. Out of scope: guessing "no mail" from missing credentials alone (rejected: indistinguishable from a half-set-up machine).

### note · agent:afzu · 2026-10-15T01:07:06.188Z
done: projects sync spreads [mail] (projects derived), [machine] mail = false opt-out, quiet missing-AWS skip + doctor warning, docs+CHANGELOG; just check exit 0, 1694 tests passed; 22130f79

### note · agent:manager-2 · 2026-10-15T01:07:13.081Z
built: 22130f79 (branch bridle/afzu)

### note · agent:manager-2 · 2026-10-15T01:21:02.327Z
integrated: 1a73fd3757675ec615913a55c462f6451594fc73 (branch bridle/afzu)
