---
id: 76aq
title: "bridle sign setup fails with Homebrew OpenSSL 3+: macOS can't import the p12 ('MAC verification failed ... wrong password?')"
kind: bug
opened: 2026-10-06
repos: [bridle]
changes: []
specs: []
needs: []
see: [p88z]
tasks: [br-76aq]
---

## The ask

The human, 2026-10-06 evening, at dalek: "I did this four times and it won't accept my password: % bridle sign setup ... error: security import failed (exit status: 1): security: SecKeychainItemImport: MAC verification failed during PKCS12 import (wrong password?)"

Cause (aide, verified in a scratch keychain): `crates/bridle-daemon/src/signing.rs` runs the first `openssl` on PATH. On dalek that is Homebrew's OpenSSL 4.0.2 (`/usr/local/bin/openssl`), whose `pkcs12 -export` defaults (PBKDF2/AES, SHA-256 MAC) macOS `security import` can't read; it reports a wrong password. The same steps with `/usr/bin/openssl` (LibreSSL) import fine. The human's password is never the problem: the p12 password is the same string on both sides.

Fix: call `/usr/bin/openssl` explicitly, or pass `-legacy` (OpenSSL 3+), so the p12 uses algorithms `security` accepts. Add a test or doc note. Workaround meanwhile: `PATH=/usr/bin:$PATH bridle sign setup`.
