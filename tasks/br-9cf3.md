+++
id = "br-9cf3"
title = "Ad-hoc sign binaries at link time on x86_64 macOS (cs7x)"
kind = "bug"
state = "integrated"
created_at = "2026-09-29T01:59:17.176Z"
updated_at = "2026-09-29T02:29:52.666092Z"
+++

Goal: stop syspolicyd crashing on unsigned Intel-Mac binaries, which hung every bridle command for about 40 minutes twice on 2026-09-28. Ticket: docs/questions/open/sign-binaries-on-intel-macs-cs7x.md (read it: crash evidence and proposal). Do: add to the workspace .cargo/config.toml (create it if missing, merge if present) a section [target.x86_64-apple-darwin] with rustflags = [-C, link-arg=-Wl,-adhoc_codesign]. Check that it doesn't clash with any existing rustflags or RUSTFLAGS use (a target.rustflags entry is replaced by the RUSTFLAGS env var, so look at the justfile, CI workflows and the target/ warming code for RUSTFLAGS); it must not affect arm64 or Linux. On this machine, if it is x86_64, build and run codesign -dv on the binary to confirm it is signed; otherwise say in the handoff that this was not verified locally. Add a short note to docs (where install or building is documented, and the ticket's Status: not proven to prevent the crash, watch for new syspolicyd-*.ips reports). Acceptance: just check passes. Model: Haiku. Out of scope: real Developer ID signing, notarization, the post-install codesign workaround.

## Thread

### note · agent:pm-1 · 2026-09-29T02:29:52.666Z
integrated: cad2b61
