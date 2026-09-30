---
id: t6kq
title: One credentials file for external principals, picked by project automatically
opened: 2026-09-29
resolved: 2026-09-29
repos: [bridle]
changes: [76bee0a]
specs: []
needs: []
see: [9c63, a7h3]
closed: 2026-09-30T05:12:44Z
---

## The ask

The human, verbatim (2026-09-29), after making advisor tokens for meta-notes and track-web:

> this is way too many files; we need a better way to manage and provide tokens to these two
> interactive agents; also, it isn't really secure so, IDK about the security theater of the
> tokens; at any rate, maybe a token file for each agent that has tokens for different
> projects? Either that or a .env style file with all the tokens with different ENV names;
> I'm not sure, but I already have too many files like this and also they're littering my
> home folder

Today: one file per principal per project in `~` (`~/.bridle-orchestrator.token`,
`~/.bridle-orchestrator-meta-notes.token`, `~/.bridle-orchestrator-track-web.token`,
`~/.bridle-advisor.token`, `~/.bridle-advisor-meta-notes.token`,
`~/.bridle-advisor-track-web.token`), and every command has to pick the right one by hand
(`BRIDLE_TOKEN=$(cat ...) bridle ... --project x`).

## Proposal (the orchestrator's recommendation)

- **One file, inside `~/.bridle/`**: `~/.bridle/credentials.toml` (0600), one table per
  principal, one key per project:

  ```toml
  [orchestrator]
  bridle = "..."
  meta-notes = "..."
  track-web = "..."

  [advisor]
  bridle = "..."
  ```

- **The CLI picks the token itself.** A session sets `BRIDLE_AS=orchestrator` once
  (`scripts/claude-orchestrator` and `scripts/claude-advisor` do it); every `bridle` command
  then uses the entry for the project it's talking to (`--project`, or the cwd's daemon).
  `BRIDLE_TOKEN` still wins when set.
- **`bridle token create <name> --project <p>` writes the entry** into the file (the human
  runs one command, no `jq`, no redirection); `token revoke` removes it.
- **Migrate** the six existing files into it once and delete them.
- On the security point: the daemons listen on localhost and every principal is the same
  Unix user, so the tokens mostly name who's speaking rather than keep anyone out. Keeping
  them (for attribution and the human-only actions) but making them invisible is the KISS
  answer; dropping them would be a bigger design change (principals.md).

## Resolution

Resolved by 76bee0a: one `~/.bridle/credentials.toml`, picked by `BRIDLE_AS`.
