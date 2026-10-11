---
id: link-ids-for-the-human
severity: should
roles: [orchestrator, advisor, aide, manager]
---
When you name a ticket to the human, add its link from `bridle link <id>` (a bare ID
like `yfjc`, or a qualified one like `br-yfjc`). Never build the URL by hand.

- **Silent when unset**: with no `[gateway] public_url` configured, `bridle link` prints nothing
  and exits 0; then name the ticket as usual (rule `ticket-references`).
- **Managers**: only in messages that reach the human.

Why: the human's words, 2026-10-04 (ticket yfjc): "I have links every time an agent that's
talking to me refers to a ticket or a ticket".
