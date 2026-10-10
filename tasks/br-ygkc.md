+++
id = "br-ygkc"
title = "release::tests::installs_and_keeps_the_previous_binary fails with checksum mismatch (br-88d4 test, seen in br-re57 landing check on main+docs)"
kind = "bug"
state = "planned"
created_at = "2026-10-10T14:35:40.098Z"
updated_at = "2026-10-10T14:36:33.317255Z"
created_by = "agent:manager-2"
watchers = [
    "agent:manager-2",
    "external:orchestrator",
    "external:advisor/product-manager",
]
size = "S"
parent = "br-88d4"
+++

Landing check for br-re57 (docs-only branch on top of main 1af6f764) failed: crates/bridle-daemon/src/release.rs:378 'checksum mismatch for bridle-v9.9.9-aarch64-apple-darwin.tar.gz: expected 09a7..., got 5067...', in 0.087 s, not load. Find why the test's tarball checksum differs (nondeterministic archive: mtime/ordering? a shared temp path between tests? tar version?) and make the test deterministic. Run the test 30x alone and in the full nextest run. Acceptance: just check passes. Model: Sonnet. Out of scope: other tests.

## Thread

### note · external:orchestrator · 2026-10-10T14:36:33.317Z
orchestrator: likely cause: the test builds the tarball twice (once in the test via tarball("new binary"), once inside setup()) with separate `tar -czf` runs. The gzip header carries a timestamp (bsdtar/libarchive writes the current time), so the two archives differ whenever the runs straddle a second boundary; the fixed file mtime doesn't cover it. Hence 0.09 s and not load. Fix: hash the same bytes setup() serves (build the tarball once and pass it in) rather than making tar deterministic. Critical (a flake on main, per the human 2026-10-03): next free slot, ahead of the queue.
