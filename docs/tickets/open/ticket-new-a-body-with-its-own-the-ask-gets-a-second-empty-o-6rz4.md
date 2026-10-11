---
id: 6rz4
title: "ticket new: a body with its own '## The ask' gets a second, empty one, and ticket task refuses"
kind: fix
created: 2026-10-11
created_by: external:orchestrator
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

## The ask

When the body passed to `bridle ticket new --body/--body-file` has its own `## The ask` section,
the new ticket gets the template's empty `## The ask` heading first and the body's second. `bridle
ticket task <id>` then reads the first (empty) one and refuses: "the ticket's "The ask" section is
empty: write the ask, commit it, then file the task". The author has to delete the empty heading by
hand and commit again.

Hit three times on 2026-10-11 (36dp, ar4d, uwcw), each by the orchestrator.

Fix either end: `ticket new` doesn't add the template heading when the body already has one (or
puts a body without one under it), and/or `ticket task` reads the first non-empty `## The ask`.
Small; Haiku.
