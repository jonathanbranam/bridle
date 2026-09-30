---
id: wtyn
title: Bridle in a consuming project's CI (the spec adapter needs the binary)
opened: 2026-09-30
repos: [bridle]
changes: []
specs: []
needs: []
see: [yghs]
---

## The ask


A project on bridle's Python pack copies `workflow/packs/python/adapters/bridle_specs.py`, which
runs `bridle spec export --format json` at pytest collection. Without the binary, collection fails
(`cannot run 'bridle' (set BRIDLE_BIN)`). meta-notes' new GitHub Actions CI failed on exactly that
(run 36781233500); for now it ignores `test_specs_bdd.py`. Bridle's own CI never sees it: it builds
bridle first, and the adapter's own tests skip without the binary.

The human (2026-09-30, via the NUC's orchestrator, m-2847): "this will be a big issue in the
future ... do we really want to install bridle during CI? I suppose it makes sense, but adds
complexity."

## Options

1. **Install bridle in CI.** The spec scenarios are the project's behaviour tests, so CI runs
   them for real. Three ways to get the binary, cheapest first:
   - **Cached `cargo install`**, pinned: `cargo install --git https://github.com/jonathanbranam/bridle
     --tag vX.Y.Z --locked bridle` under an `actions/cache` of `~/.cargo/bin` keyed on the tag.
     ~10 minutes on a cache miss (first run, a new pin, or 7 days unused), seconds after. No
     bridle-side work except a snippet in the pack's README. It needs a tag that has
     `spec export`; the newest, v0.3.0 (2026-09-28), predates it, so cut v0.4.0 (ours to cut
     on verified `main`).
   - **Release binaries**: a release job attaches Linux x86_64 and macOS arm64 builds to each tag
     (the repo is public, so a download needs no token), and the pack documents a
     two-line `curl | tar` step. Seconds every run; the cost is a release workflow to keep.
   - **A setup action** (`uses: jonathanbranam/setup-bridle@v1`) wrapping the download. Only
     worth it with several consumers.
2. **Commit the export.** The adapter reads a committed `specs.json` when bridle is absent, and
   the local check fails if it's stale. CI needs nothing, but a generated file is committed (the
   adapter deliberately commits nothing today) and can drift from the specs.
3. **Skip without bridle.** The adapter skips the spec tests when the binary is missing and they
   run only in the local check. The simplest, but CI then silently stops testing behaviour. The
   missing-tools rule (m-2741: don't substitute or skip a check tool) argues against it.

## Recommendation

Option 1, in two steps (YAGNI: one consumer today):

- **Now:** cut v0.4.0, and add the cached `cargo install` snippet to the Python pack's README.
  meta-notes adopts it and drops the `test_specs_bdd.py` ignore. Make the adapter's error name
  the README section.
- **Later**, when a second consumer's CI needs bridle or cache misses start to hurt: release
  binaries. The setup action only after that.

Not 2 or 3: both give up running the specs in CI, which is what the adapter is for.

## Decided (2026-09-30)

The human: "option 1 is good - but let's plan the work to publish releases; I want to add more
projects in the next week and this will hit us again and again." So release binaries now; skip the
cached `cargo install` step.

1. A GitHub Actions release job: on a `v*` tag, build `bridle` for Linux x86_64 and macOS (arm64
   and x86_64; the laptop is Intel) and attach tarballs and checksums to the GitHub release.
2. The Python pack documents a pinned download step for consumers' CI (`curl` the tarball for the
   runner's platform, put it on `PATH`), and the adapter's "cannot run bridle" error names that
   section. A setup action only if the snippet gets copied into enough projects to hurt.
3. Then cut v0.4.0 on verified `main` so the first release carries binaries; meta-notes adopts
   the step and stops ignoring `test_specs_bdd.py`.
