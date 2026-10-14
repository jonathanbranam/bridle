+++
id = "br-tmdf"
title = "One CLI group: bridle project <verb>; bridle projects only lists"
kind = "feature"
state = "dropped"
created_at = "2026-10-14T22:20:02.633Z"
updated_at = "2026-10-14T22:21:03.052155Z"
created_by = "external:advisor/product-manager"
watchers = ["external:advisor/product-manager"]
ticket = "tmdf"
+++

docs/tickets/open/one-cli-group-bridle-project-verb-bridle-projects-only-lists-tmdf.md

## Thread

### note · agent:pm-1 · 2026-10-14T22:21:01.547Z
Brief (pm-1). Model: Sonnet. Size: S-M, mostly mechanical. The ticket body is the spec. Start: the clap tree in crates/bridle/src/cli.rs (`project` and `projects` groups) and `git grep -n "bridle projects " docs workflow CLAUDE.md crates` for the doc sweep. Make `project` the group with all verbs (add comes from br-29se, which runs after this; do not build add here), `projects` list-only with no subcommands, and the old `projects <verb>` forms kept as hidden aliases that print a one-line warning naming the new form (stderr, exit status unchanged) for one release. Update every doc, role file, runbook, and message the CLI prints that says `bridle projects <verb>`, including workflow/base roles and rules (they reach projects via `bridle workflow update`; no project-file migration). Tests: `projects sync` warns and works; `projects` lists; `project sync` works. Docs: cli.md, CHANGELOG.md. Touches many docs: run after br-afzu, br-8usa, br-pmdh, br-qwd4 (they edit the same docs and projects sync code) to avoid conflicts. Acceptance: just check. Out of scope: removing the old spelling (next release).

### note · external:advisor/product-manager · 2026-10-14T22:21:03.052Z
dropped: Folded into br-a67t (the CLI grouping), by the human, 6:25 PM ET: 'We have a ticket to fix up all of the commands, that is something we can fix up then'. br-29se still builds now, as bridle project add.
