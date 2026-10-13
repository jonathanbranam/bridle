+++
id = "br-mksa"
title = "External sessions: count a new session's first statusline report in full (ktgd note 3)"
kind = "feature"
state = "integrated"
created_at = "2026-10-13T23:05:26.992Z"
updated_at = "2026-10-13T23:33:47.275623Z"
created_by = "external:advisor/product-manager"
watchers = [
    "external:advisor/product-manager",
    "agent:mksa",
]
branch = "bridle/mksa"
commit = "3142585e5ea7cf48c9b529fbb82f95431ec4e565"
summary = "A never-seen external session whose first running totals are all small (cost < FRESH_SESSION_MAX_COST_USD = $1, each token total < FRESH_SESSION_MAX_TOKENS = 100000) now counts its first statusline report in full; larger first totals (resumed) stay a baseline. store.rs running_delta/record_interactive_usage; test added; usage-and-budget.md and CHANGELOG updated. No schema change."
ticket = "mksa"
+++

docs/tickets/open/external-sessions-count-a-new-session-s-first-statusline-rep-mksa.md

## Thread

### note · agent:pm-1 · 2026-10-13T23:06:32.371Z
Brief (pm-1). Model: Sonnet. Size: S. The ask is the spec; ktgd note 3 gives the rule. Start: the `interactive_usage` insert/delta code behind `bridle statusline` (br-hd28; grep interactive_usage in crates/bridle-daemon and docs/design/usage-and-budget.md). Rule: a session never seen before whose first running total is near zero counts that report in full; a total that clearly includes earlier spend (a resumed session: first report large, or the session id is known to be a resume) keeps the baseline rule. Pick a simple threshold, name it as a constant, and say in the docs what it is. Test: new session's first report counted, resumed one's not. Docs: usage-and-budget.md, CHANGELOG.md. No schema change expected. Out of scope: outside-project sessions (drvx).

### note · agent:mksa · 2026-10-13T23:32:59.908Z
built: fe0a92df (branch bridle/mksa)

### note · agent:mksa · 2026-10-13T23:32:59.975Z
done: new external session's first statusline report counts in full (cost<$1, tokens<100k; resumed stays baseline); just check exit 0, 1648 tests passed; fe0a92df (main merged, already up to date; CHANGELOG edit came after the check run but is docs-only)

### note · agent:manager-2 · 2026-10-13T23:33:07.119Z
integrated: 3142585e5ea7cf48c9b529fbb82f95431ec4e565 (branch bridle/mksa)

### note · agent:manager-2 · 2026-10-13T23:33:47.275Z
cleanup: removed agent mksa, branch bridle/mksa
