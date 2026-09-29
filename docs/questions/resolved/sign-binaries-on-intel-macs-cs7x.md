---
id: cs7x
title: Ad-hoc sign bridle's binaries on Intel Macs (syspolicyd crashes on unsigned ones)
opened: 2026-09-29
resolved: 2026-09-29
repos: [bridle]
changes: [0f4b23a]
specs: []
needs: []
see: []
---

## What happened

Twice on 2026-09-28 (21:05 and 21:50 local), right after `cargo install --path crates/bridle`,
macOS's `syspolicyd` crashed and every newly built program then hung on exec, bridle's CLI and
`bridle tui` included, for everyone (the human, the orchestrator, all agents), until launchd
brought `syspolicyd` back (about 35 minutes the first time). See `docs/context/incidents.md`.

The crash reports (`/Library/Logs/DiagnosticReports/syspolicyd-*.ips`, five that evening:
20:28, 20:50, 21:03, 21:31, 21:50) all show a SIGSEGV (null + 8) in
`Security::Universal::architecture()` under `MachORep::signingData()` /
`SecStaticCode::staticValidate`, i.e. while validating the code signature of an unsigned
Mach-O. The human's laptop is x86_64: unlike arm64, the Intel linker doesn't ad-hoc sign
binaries, so bridle's binaries (and test binaries) have no signature at all.

## Proposal

- Ad-hoc sign every binary at link time on x86_64 macOS, in the workspace's
  `.cargo/config.toml`:

  ```toml
  [target.x86_64-apple-darwin]
  rustflags = ["-C", "link-arg=-Wl,-adhoc_codesign"]
  ```

  That covers `cargo install`, test binaries and workers' builds. Check it doesn't upset CI's
  macOS runner (arm64 signs already) or the warm `target/` sharing (b7cz).
- Until then, after each install: `codesign -s - -f ~/.cargo/bin/bridle` (the orchestrator
  does this; done by hand once on 2026-09-29).
- Not proven that signing prevents the crash; confirm by watching for new `syspolicyd-*.ips`
  after a few installs.

## Resolution

Resolved by 0f4b23a: `.cargo/config.toml` ad-hoc code-signs x86_64-apple-darwin binaries at link time.
