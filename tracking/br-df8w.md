+++
id = "br-df8w"
title = "Opt-in migrations 0001-0005: decide which existing projects still need them, retire the rest, surface pending ones as to-dos (from rdag item 2)"
kind = "fix"
state = "integrated"
created_at = "2026-10-18T23:47:18.317Z"
updated_at = "2026-10-20T03:26:22.712966Z"
created_by = "agent:pm-1"
watchers = [
    "agent:pm-1",
    "agent:wdf8w",
]
branch = "bridle/wdf8w"
commit = "3f446252fd265cda8770bcab600eed8d0844a6b3"
summary = "Retired opt-in migrations 0001 (role alias in config::canonical_role makes the rename cosmetic) and 0003 (unused bug->fix relabel): removed from MIGRATIONS with their fns, tests and ticket::bug_to_fix; migrations.toml needs no edit since pending comes from the shipped list. 0002, 0004, 0005 stay. Start-up now files ONE chore to-do per newly-pending opt-in migration set, messaged to external:aide (not human, not PM), recorded as 'announced' in migrations.toml so the next start files none (test: a_pending_opt_in_migration_is_announced_once_not_at_the_next_start). rdag's commit-leftovers to-do already targeted external:aide; both now share one loop in serve.rs. Docs: docs/design/migrations.md. Note: the stale Status line there (never automatic) predates this ticket."
ticket = "df8w"
+++

docs/tickets/open/opt-in-migrations-0001-0005-decide-which-existing-projects-s-df8w.md

## Thread

### note · external:aide · 2026-10-19T12:09:48.859Z
The human, 2026-10-19 (to aide), verbatim: "There is often no project manager. So. Idk. The only agent we have running on many projects is an aide. I think it goes to the aide." So: a pending opt-in migration files its to-do for the project's aide (external:aide), not --for-human and not the project manager. The ask says "a to-do for the human"; read it as "for the aide". br-rdag's "else file a to-do" (built at 4189c3eb) should get the same treatment if it files for the human.

### note · agent:pm-1 · 2026-10-19T12:09:57.001Z
pm-1: confirmed, the human's decision above governs: the to-do goes to the project's aide (external:aide), not --for-human and not the project manager. Also, as part of this ticket, check br-rdag's built "else file a to-do" path (4189c3eb) and switch it to the same target if it files for the human. Everything else in the brief stands.

### note · external:orchestrator · 2026-10-19T12:10:25.741Z
human via aide, 2026-10-19 (m-0293): "There is often no project manager. So. Idk. The only agent we have running on many projects is an aide. I think it goes to the aide." Brief change (orchestrator): file the opt-in-migration to-dos for external:aide on the project, not --for-human and not the manager.

### note · agent:wdf8w · 2026-10-20T03:26:14.600Z
built: eb295bb2e60e2032fb0e7043c3b4eafe1a39efdc (branch bridle/wdf8w)

### note · agent:wdf8w · 2026-10-20T03:26:15.103Z
done: retired 0001/0003, opt-in migrations announced once to external:aide; check exit 0, 1916 tests; eb295bb2e60e2032fb0e7043c3b4eafe1a39efdc

### note · agent:manager-2 · 2026-10-20T03:26:22.712Z
integrated: 3f446252fd265cda8770bcab600eed8d0844a6b3 (branch bridle/wdf8w)
