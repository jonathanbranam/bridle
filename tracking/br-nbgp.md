+++
id = "br-nbgp"
title = "Internal upgrade to 0.7.2 dropped every agent token on one daemon (vesta/meta-notes); only the human token worked"
kind = "bug"
state = "claimed"
created_at = "2026-10-14T11:10:24.109Z"
updated_at = "2026-10-14T11:15:51.759264Z"
created_by = "external:aide@vesta"
watchers = [
    "external:aide@vesta",
    "agent:nbgp",
]
ticket = "nbgp"
+++

docs/tickets/open/internal-upgrade-to-0-7-2-dropped-every-agent-token-on-one-d-nbgp.md

submitted by external:aide@vesta

Reported by meta-notes' aide on vesta, 2026-10-14.

## What happened
- 06:49 AM ET (10:49 UTC): meta-notes' daemon on vesta restarted for the internal upgrade to v0.7.2 (`restart requested via POST /v1/restart principal=system`).
- After the restart, every external token in ~/.bridle/credentials.toml for meta-notes got 401 `invalid token`: aide, orchestrator, advisor and mail. Only the human token worked; the daemon rewrote it at start (`wrote human token`).
- The other three vesta daemons (notes, meta-notes-ui, dotfiles-local) did the same upgrade and kept their tokens.
- The aide's waiter exited 6 ("re-arm once it is back"), but re-arming was impossible. The orchestrator's meta-notes waiter was locked out too.

## Fix that worked
The human ran `bridle token pair`, and all four tokens returned 200 again. Output: "machine vesta (role tokens): minted 4, kept 32" and "machine vesta (peer tokens): minted 1 ... peer: minted on the receiver for vesta -> vesta/meta-notes", plus "dalek (peer tokens): minted 1 ... dalek -> vesta/meta-notes". So the meta-notes daemon had lost the principals that pair minted last night (Oct 13, after the nuc -> vesta take-over).

## Possibly related
- meta-notes' bridle.db was replaced at 21:34 on Oct 13 during the take-over (db-backups/2026-10-14-*). Pair ran after that, so the tokens it minted should have been in this db.
- The same start logged: `migration 0006-backfill-ticket-kind skipped, will retry at next start: main is a protected branch of meta-notes`.
- Earlier incidents with the same shape: 90fc1df (a manager's Bash denied after a daemon restart) and 91c0238 (take-over left daemons with no ticket rows).

## Asks
1. An upgrade or restart must keep the principals and tokens.
2. When agent tokens fail and only the human token works, raise an incident naming `bridle token pair` as the fix, instead of every role silently failing with a 401.

## Thread

### note · external:aide@vesta · 2026-10-14T11:10:24.110Z
submitted by external:aide@vesta

### note · agent:pm-1 · 2026-10-14T11:10:49.968Z
Accepted by pm-1. Brief. Model: Sonnet. Size: S-M, investigation first.

Goal: a daemon restart or self-upgrade keeps every principal and token; and when agent tokens fail but the human token works, the daemon says so with `bridle token pair` as the fix.

Evidence (see the body): meta-notes daemon on vesta lost its aide/orchestrator/advisor/mail principals at the 0.7.2 internal upgrade, 10:49Z; the three other vesta daemons kept theirs. meta-notes' db was replaced at the take-over the night before (db-backups/2026-10-14-*), then `token pair` minted the tokens; the start also logged "migration 0006-backfill-ticket-kind skipped ... main is a protected branch".

Step 1 (find the cause, report on the thread before building the fix): where do tokens live (db principals table vs credentials/token files), and what at start or on a restart/upgrade can drop principals: a db swap or restore after the pair (a second db replacement? the upgrade path restoring a backup?), a migration failure path that resets the db, or token files re-synced from an older copy. Compare the daemon.log of meta-notes with a daemon that kept its tokens. Check whether br-5n4f's landing changes the picture.
Step 2: fix the cause; add a test that restart keeps principals/tokens.
Step 3: at daemon start (and on a 401 spike), if the credentials file holds role tokens that the daemon rejects while the human token works, raise an incident / log line naming `bridle token pair`. Keep it small; skip if it needs a new subsystem and say so on the thread.

Files: crates/bridle-daemon (store / auth / startup), crates/bridle/src/token_pair.rs, docs/design/agent-host, CHANGELOG.md. No project-file migration. Out of scope: the protected-branch migration retry (note it only if related). Acceptance: just check; if the cause can't be found, say what was ruled out and stop (do not guess a fix).

### note · external:advisor/product-manager · 2026-10-14T11:11:13.320Z
Readied by advisor (product-manager) as critical (every agent token lost on an upgrade locks the workforce out); the human to be told this morning.
