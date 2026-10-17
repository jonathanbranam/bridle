---
id: link-ids-for-the-human
severity: should
---
When you name a ticket, document or spec to the human, add its link from `bridle link <target>`.
Always use `bridle link`; never build the URL by hand.

- **Ticket**: `bridle link br-yfjc` (or bare `yfjc`) prints `<public_url>/p/<project>/tickets/<id>`.
- **Document**: `bridle link docs/design/cli.md` (repo-relative path) prints
  `<public_url>/p/<project>/docs?path=<path>`.
- **Spec**: `bridle link r-1a2b` (requirement or scenario ID) prints
  `<public_url>/p/<project>/specs?path=<file>#<id>`.
- **Old URLs** (`/task?id=`, `/ticket?project=&id=`) still redirect in the UI; don't emit them.

- **Silent when unset**: with no `[gateway] public_url` configured, `bridle link` prints nothing
  and exits 0; then name the ticket as usual (rule `ticket-references`).
- **Managers**: only in messages that reach the human.

Backed by the `bridle hook link-check` Stop hook in the orchestrator, advisor and aide sessions: it
reminds once per reply, never blocks twice (ticket 9dcz). It does not see `bridle send` text.

Why: the human's words, 2026-10-04 (ticket yfjc): "I have links every time an agent that's
talking to me refers to a ticket or a ticket".
