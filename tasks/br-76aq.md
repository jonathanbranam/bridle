+++
id = "br-76aq"
title = "bridle sign setup fails with Homebrew OpenSSL 3+: macOS can't import the p12 ('MAC verification failed ... wrong password?')"
kind = "bug"
state = "planned"
created_at = "2026-10-06T23:42:35.043Z"
updated_at = "2026-10-06T23:43:05.961153Z"
created_by = "external:aide"
watchers = ["external:aide"]
+++

original id: 76aq
Ticket (the human's words, cause, workaround): docs/tickets/open/bridle-sign-setup-fails-with-homebrew-openssl-3-macos-can-t-76aq.md . Related: p88z (sign setup).
Cause: crates/bridle-daemon/src/signing.rs runs the first `openssl` on PATH (two calls, ~lines 126 and 139: `openssl req` and `openssl pkcs12`). On a Mac with Homebrew OpenSSL 3+/4 that binary's `pkcs12 -export` defaults (PBKDF2/AES, SHA-256 MAC) are unreadable by macOS `security import`, which reports "MAC verification failed ... wrong password?". /usr/bin/openssl (LibreSSL) works.
EXACT FIX (decided; `-legacy` is rejected because LibreSSL does not accept it):
- Add one small function in signing.rs, e.g. `fn openssl_path() -> &'static str`, returning "/usr/bin/openssl" when that file exists, else "openssl" (PATH lookup, for non-macOS and CI). Use it at both Command::new("openssl") sites. The error context strings stay ("openssl req", "openssl pkcs12").
- Keep the workaround note out of the docs; instead add one line to the `bridle sign setup` section of docs/design (grep for "sign setup" in docs/design and CLAUDE.md's signing paragraph only if it names openssl) saying it uses the system /usr/bin/openssl because Homebrew's OpenSSL 3+ writes a p12 that macOS `security` cannot import.
- Test: a unit test for `openssl_path` that does not need a keychain (e.g. factor the choice into `fn pick_openssl(exists: bool) -> &'static str` and test both branches). No test may touch the real keychain. CHANGELOG entry (read it with a limit).
Acceptance: just check passes. (The human verifies on dalek: `bridle sign setup` succeeds with Homebrew OpenSSL first on PATH; do not run sign setup yourself.)
Migration: none. Model: Haiku. Out of scope: any other signing change.

## Thread

### note · external:orchestrator · 2026-10-06T23:42:50.552Z
From orchestrator: br-76aq is ready (the human, via aide: 'Please schedule'). It's a small bug fix: signing.rs should use /usr/bin/openssl or -legacy. The human has a PATH workaround, so it's normal priority, after br-x56y. Spawns are on hold until the daemon restarts (incident br-z7y5).
