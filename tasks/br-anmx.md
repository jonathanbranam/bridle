+++
id = "br-anmx"
title = "bridle gateway hash-password' echoes the password as it's typed: read it hidden when stdin is a terminal"
kind = "bug"
state = "planned"
created_at = "2026-10-04T12:47:28.413Z"
updated_at = "2026-10-04T22:08:45.055992Z"
created_by = "external:advisor"
watchers = ["external:advisor"]
+++

original id: anmx
Build docs/tickets/open/bridle-gateway-hash-password-echoes-the-password-as-it-s-typ-anmx.md (read it). bridle gateway hash-password (crates/bridle/src/gateway.rs, hash_password) echoes the password. When stdin is a terminal: prompt 'Password:' on stderr, read with echo off (a crate with no unsafe, such as rpassword, check what the workspace already depends on first), ask again to confirm and refuse on mismatch. When stdin is not a terminal: read the line as today, no prompt, so scripts keep working. Update docs/design/cli.md and the gateway setup text in docs/design/human-web-ui.md. Acceptance: just check passes; test the piped path and the mismatch refusal (inject the reader). Model: Haiku. Out of scope: other prompts.
