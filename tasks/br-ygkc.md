+++
id = "br-ygkc"
title = "release::tests::installs_and_keeps_the_previous_binary fails with checksum mismatch (br-88d4 test, seen in br-re57 landing check on main+docs)"
kind = "bug"
state = "planned"
created_at = "2026-10-10T14:35:40.098Z"
updated_at = "2026-10-10T16:19:02.787559Z"
created_by = "agent:manager-2"
watchers = [
    "agent:manager-2",
    "external:orchestrator",
    "external:advisor/product-manager",
]
size = "S"
summary = "release::tests::installs_and_keeps_the_previous_binary failed on a checksum mismatch because the test built the fixture tarball twice (once served, once hashed for SHA256SUMS) and tar -czf puts a timestamp in the gzip header, so two builds a second apart differ. Fix (crates/bridle-daemon/src/release.rs tests): setup() now hashes the one tarball it serves unless given other bytes (Option). 30 solo runs of the release:: tests: 30/30 pass. Full just check NOT green on this loaded machine (load 35-55): bridle::cli_e2e sigint_shuts_down_cleanly_with_a_store_call_in_flight failed twice in the full run (36-46 s) yet passes alone (61 s, slow). Unrelated to this change; looks load-sensitive."
parent = "br-88d4"
+++

Landing check for br-re57 (docs-only branch on top of main 1af6f764) failed: crates/bridle-daemon/src/release.rs:378 'checksum mismatch for bridle-v9.9.9-aarch64-apple-darwin.tar.gz: expected 09a7..., got 5067...', in 0.087 s, not load. Find why the test's tarball checksum differs (nondeterministic archive: mtime/ordering? a shared temp path between tests? tar version?) and make the test deterministic. Run the test 30x alone and in the full nextest run. Acceptance: just check passes. Model: Sonnet. Out of scope: other tests.

## Thread

### note · external:orchestrator · 2026-10-10T14:36:33.317Z
orchestrator: likely cause: the test builds the tarball twice (once in the test via tarball("new binary"), once inside setup()) with separate `tar -czf` runs. The gzip header carries a timestamp (bsdtar/libarchive writes the current time), so the two archives differ whenever the runs straddle a second boundary; the fixed file mtime doesn't cover it. Hence 0.09 s and not load. Fix: hash the same bytes setup() serves (build the tarball once and pass it in) rather than making tar deterministic. Critical (a flake on main, per the human 2026-10-03): next free slot, ahead of the queue.

### note · agent:wygkc · 2026-10-10T16:18:36.191Z
fix committed 41805eab (main merged): cause = test built the tarball twice, gzip header timestamp differs across seconds; setup() now hashes the one served tarball. release:: tests 30/30 pass solo. BUT just check is NOT green: bridle::cli_e2e sigint_shuts_down_cleanly_with_a_store_call_in_flight failed in 2 full runs (36-46 s, machine load 35-55), passes alone (61 s, slow). Unrelated to my change; fails fast so only ~70 of 1451 tests ran. Your call: land on the solo evidence, or have me re-run when load drops.

### note · external:orchestrator · 2026-10-10T16:19:02.787Z
From orchestrator: wygkc's fix (41805eab) is in, but just check failed twice on cli_e2e sigint_shuts_down_cleanly_with_a_store_call_in_flight under load 35-55 (that's br-bcw6, Tier 1). Load is down to 1.8/core now: have wygkc re-run just check once now, and land ygkc if green. If sigint fails again, tell me before anything else. Then br-751e (wrel751 merges main, checks once), then I cut v0.6.0.
