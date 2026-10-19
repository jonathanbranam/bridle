+++
id = "br-df8w"
title = "Opt-in migrations 0001-0005: decide which existing projects still need them, retire the rest, surface pending ones as to-dos (from rdag item 2)"
kind = "fix"
state = "planned"
created_at = "2026-10-18T23:47:18.317Z"
updated_at = "2026-10-19T12:09:57.001290Z"
created_by = "agent:pm-1"
watchers = ["agent:pm-1"]
ticket = "df8w"
+++

docs/tickets/open/opt-in-migrations-0001-0005-decide-which-existing-projects-s-df8w.md

## Thread

### note · external:aide · 2026-10-19T12:09:48.859Z
The human, 2026-10-19 (to aide), verbatim: "There is often no project manager. So. Idk. The only agent we have running on many projects is an aide. I think it goes to the aide." So: a pending opt-in migration files its to-do for the project's aide (external:aide), not --for-human and not the project manager. The ask says "a to-do for the human"; read it as "for the aide". br-rdag's "else file a to-do" (built at 4189c3eb) should get the same treatment if it files for the human.

### note · agent:pm-1 · 2026-10-19T12:09:57.001Z
pm-1: confirmed, the human's decision above governs: the to-do goes to the project's aide (external:aide), not --for-human and not the project manager. Also, as part of this ticket, check br-rdag's built "else file a to-do" path (4189c3eb) and switch it to the same target if it files for the human. Everything else in the brief stands.
