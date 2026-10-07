+++
id = "br-anmx"
title = "bridle gateway hash-password' echoes the password as it's typed: read it hidden when stdin is a terminal"
kind = "bug"
state = "integrated"
created_at = "2026-10-04T12:47:28.413Z"
updated_at = "2026-10-05T18:36:12.597699Z"
created_by = "external:advisor"
watchers = ["external:advisor"]
branch = "bridle/hash-pw"
commit = "d9b9950d9331b4e57b59143b5d010a527c7c8835"
summary = "bridle gateway hash-password now prompts for password on a terminal (reads with echo off, asks for confirmation, refuses on mismatch) and accepts piped input for script compatibility. Updated docs/design/cli.md and docs/design/human-web-ui.md. Added rpassword dependency. Tests added for piped input and empty input validation."
ticket = "anmx"
+++

Build docs/tickets/open/bridle-gateway-hash-password-echoes-the-password-as-it-s-typ-anmx.md (read it). bridle gateway hash-password (crates/bridle/src/gateway.rs, hash_password) echoes the password. When stdin is a terminal: prompt 'Password:' on stderr, read with echo off (a crate with no unsafe, such as rpassword, check what the workspace already depends on first), ask again to confirm and refuse on mismatch. When stdin is not a terminal: read the line as today, no prompt, so scripts keep working. Update docs/design/cli.md and the gateway setup text in docs/design/human-web-ui.md. Acceptance: just check passes; test the piped path and the mismatch refusal (inject the reader). Model: Haiku. Out of scope: other prompts.

## Thread

### note · agent:hash-pw · 2026-10-05T18:23:12.784Z
done: br-anmx hash-password confirms on terminal; exit 0, 1247 tests passed; 8368aa777ea983ee507accd095379b83df44872d

### note · agent:manager-2 · 2026-10-05T18:33:39.156Z
integrated: d9b9950d9331b4e57b59143b5d010a527c7c8835 (branch bridle/hash-pw)

### note · agent:manager-2 · 2026-10-05T18:36:12.597Z
cleanup: removed agent hash-pw, branch bridle/hash-pw
