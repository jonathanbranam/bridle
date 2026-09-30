+++
id = "br-ce9f"
title = "Release workflow: publish bridle binaries on v* tags (wtyn 1)"
kind = "question"
state = "integrated"
created_at = "2026-09-30T21:49:22.985Z"
updated_at = "2026-09-30T22:22:40.903693Z"
branch = "bridle/release-wf"
commit = "e3bdda5fca5f30a41e81d6d1ad347f2999ecb4a7"
summary = "Added .github/workflows/release.yml: on a v* tag it builds `cargo build --release --locked -p bridle` for x86_64-unknown-linux-gnu (ubuntu-22.04), aarch64-apple-darwin and x86_64-apple-darwin (cross-compiled on macos-latest, since Intel runners are retiring; .cargo/config.toml adhoc codesign applies), packages bridle-<tag>-<target>.tar.gz (just `bridle`) via scripts/package-release.sh, writes SHA256SUMS, and creates/updates the GitHub release with gh (contents: write only in the publish job). workflow_dispatch builds and uploads artifacts (tag named dev-<sha7>) without publishing. Docs: Releases section in docs/README.md, CHANGELOG. Validated locally: YAML parses, package script produces a tarball with `bridle`; actionlint not installed, and the workflow itself was not run. Orchestrator verifies: after landing, run the workflow via workflow_dispatch from main (gh workflow run release.yml), check the three bridle-<target> artifacts each hold bridle-dev-<sha>-<target>.tar.gz that runs `bridle --version` (macOS x86_64 needs Intel or Rosetta); then cut v0.4.0 and check the release has 3 tarballs + SHA256SUMS that verify with sha256sum -c."
+++

Part 1 of docs/tickets/open/bridle-in-a-consuming-project-s-ci-the-spec-adapter-needs-th-wtyn.md ('Decided', read it). Add .github/workflows/release.yml: on push of a v* tag, build the release bridle binary (cargo build --release -p bridle; check the crate and binary name) for Linux x86_64 (use ubuntu-22.04 so the glibc floor is modest), macOS arm64 (macos-latest) and macOS x86_64 (an Intel runner such as macos-13, or cross-compile with the x86_64-apple-darwin target if Intel runners are gone; the repo's .cargo/config.toml adhoc codesign flag applies to x86_64-apple-darwin), package each as bridle-<tag>-<target>.tar.gz containing the bridle binary, produce a SHA256SUMS file, and attach all to the GitHub release for the tag (create it if missing; gh release create/upload or softprops/action-gh-release, with permissions: contents: write). Also add a workflow_dispatch trigger that builds and uploads the tarballs as workflow artifacts without publishing, so the workflow can be exercised without a tag. A worker can't run GitHub Actions and must not push tags: validate the YAML (actionlint if available, else a YAML parse) and the packaging steps locally (a small script run on this machine's target is fine); state in your summary exactly how the orchestrator verifies (run the workflow_dispatch from main after landing, check artifacts; the first real tag is v0.4.0, cut by the orchestrator). Docs: a short 'Releases' section in docs/README.md or the repo README (what a release contains, the asset naming, the checksum file), CHANGELOG. Acceptance: just check passes. Model: Sonnet. CI config and docs only, not daemon start-up. Out of scope: the Python pack README (separate task, after this), a setup action, publishing to package managers, cutting any tag.

## Thread

### note · agent:release-wf · 2026-09-30T22:22:36.629Z
done: release.yml (v* tag -> 3 tarballs + SHA256SUMS on the release; workflow_dispatch -> artifacts only), scripts/package-release.sh, docs/README Releases, CHANGELOG; 77962bf (main merged). just check was green (930 tests) before the merge; after it, 932/933: bridle-spec parses_fast (a 1s wall-clock assertion, failed at 1.1-1.3s with host load ~94) failed twice in full runs and passes alone (0.19s). Unrelated to this change. actionlint absent, workflow not run; verify steps are in the task summary.

### note · agent:manager-2 · 2026-09-30T22:22:40.903Z
integrated: e3bdda5fca5f30a41e81d6d1ad347f2999ecb4a7 (branch bridle/release-wf)
