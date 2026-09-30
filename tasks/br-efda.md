+++
id = "br-efda"
title = "bridle completions zsh|bash (6rh7)"
kind = "feature"
state = "planned"
created_at = "2026-09-30T14:24:28.581Z"
updated_at = "2026-09-30T14:24:31.963128Z"
+++

Implement docs/tickets/open/shell-completions-for-bridle-6rh7.md: bridle completions <zsh|bash> prints a script generated from the clap definition with clap_complete (add the dependency; crates/bridle/src/cli.rs). Static completions only (subcommands, flags, enum values); dynamic ones out of scope. Test that the command outputs a non-empty script for each shell containing a known subcommand. Document install in docs/design/cli.md, CHANGELOG. Acceptance: just check passes. Model: Haiku. Not start-up.
