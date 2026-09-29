+++
id = "br-eb7a"
title = "Doc audit: cli.md matches every CLI command and flag"
kind = "chore"
state = "planned"
created_at = "2026-09-29T09:35:22.662Z"
updated_at = "2026-09-29T09:35:24.852056Z"
+++

Goal: docs/design/cli.md is the CLI's reference; tonight added many commands (spec import/coverage/export flags, goals, arch, explore, impact, conflict, trace, port, probe, land, arch-guard, renew, wait, task plan/queue etc.). Run `bridle --help` and every subcommand's --help (recursively) and compare with cli.md: add missing commands/flags, fix wrong descriptions, move built items out of 'Planned' into 'Built', delete Planned entries that no longer apply. Keep the existing doc's style and density; don't paste whole --help output. Also check README.md's and docs/README.md's command mentions. Acceptance: just check passes (doc-link check); list in the thread what you changed. Model: Sonnet. Out of scope: code changes (report any --help text that is wrong).
