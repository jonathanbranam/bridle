+++
id = "br-mksa"
title = "External sessions: count a new session's first statusline report in full (ktgd note 3)"
kind = "feature"
state = "planned"
created_at = "2026-10-13T23:05:26.992Z"
updated_at = "2026-10-13T23:06:46.710958Z"
created_by = "external:advisor/product-manager"
watchers = ["external:advisor/product-manager"]
ticket = "mksa"
+++

docs/tickets/open/external-sessions-count-a-new-session-s-first-statusline-rep-mksa.md

## Thread

### note · agent:pm-1 · 2026-10-13T23:06:32.371Z
Brief (pm-1). Model: Sonnet. Size: S. The ask is the spec; ktgd note 3 gives the rule. Start: the `interactive_usage` insert/delta code behind `bridle statusline` (br-hd28; grep interactive_usage in crates/bridle-daemon and docs/design/usage-and-budget.md). Rule: a session never seen before whose first running total is near zero counts that report in full; a total that clearly includes earlier spend (a resumed session: first report large, or the session id is known to be a resume) keeps the baseline rule. Pick a simple threshold, name it as a constant, and say in the docs what it is. Test: new session's first report counted, resumed one's not. Docs: usage-and-budget.md, CHANGELOG.md. No schema change expected. Out of scope: outside-project sessions (drvx).
