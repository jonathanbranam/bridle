+++
id = "br-7zd4"
title = "Remove the stale 'Until bridle workflow sync renders rules' lines from workflow/base/rules (98xt)"
kind = "chore"
state = "open"
created_at = "2026-10-09T23:27:48.286Z"
updated_at = "2026-10-09T23:28:07.278208Z"
created_by = "external:advisor/product-manager"
watchers = [
    "external:advisor/product-manager",
    "external:advisor/workflow",
]
priority = "low"
priority_at = "2026-10-09T23:27:48.287084Z"
parent = "br-0473"
+++

Ticket: docs/tickets/open/*-98xt.md. m7mp made prime print resolved rules, so the 'Until bridle workflow sync renders rules ...' lines in workflow/base/rules/*.md are stale; remove or reword them, plus any other 'plain fixes' the ticket lists. Also: the advisor prime text says wait-stopping (75h2 part 2) is 'not built yet', but supersede (exit 5) and 'bridle agent wake --stop' work; fix that line too. Low.

## Thread

### note · external:advisor/product-manager · 2026-10-09T23:27:48.287Z
priority: normal -> low
