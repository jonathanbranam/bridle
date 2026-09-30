+++
id = "br-ce9f"
title = "Release workflow: publish bridle binaries on v* tags (wtyn 1)"
kind = "question"
state = "planned"
created_at = "2026-09-30T21:49:22.985Z"
updated_at = "2026-09-30T21:52:26.836405Z"
+++

Part 1 of docs/tickets/open/bridle-in-a-consuming-project-s-ci-the-spec-adapter-needs-th-wtyn.md ('Decided', read it). Add .github/workflows/release.yml: on push of a v* tag, build the release bridle binary (cargo build --release -p bridle; check the crate and binary name) for Linux x86_64 (use ubuntu-22.04 so the glibc floor is modest), macOS arm64 (macos-latest) and macOS x86_64 (an Intel runner such as macos-13, or cross-compile with the x86_64-apple-darwin target if Intel runners are gone; the repo's .cargo/config.toml adhoc codesign flag applies to x86_64-apple-darwin), package each as bridle-<tag>-<target>.tar.gz containing the bridle binary, produce a SHA256SUMS file, and attach all to the GitHub release for the tag (create it if missing; gh release create/upload or softprops/action-gh-release, with permissions: contents: write). Also add a workflow_dispatch trigger that builds and uploads the tarballs as workflow artifacts without publishing, so the workflow can be exercised without a tag. A worker can't run GitHub Actions and must not push tags: validate the YAML (actionlint if available, else a YAML parse) and the packaging steps locally (a small script run on this machine's target is fine); state in your summary exactly how the orchestrator verifies (run the workflow_dispatch from main after landing, check artifacts; the first real tag is v0.4.0, cut by the orchestrator). Docs: a short 'Releases' section in docs/README.md or the repo README (what a release contains, the asset naming, the checksum file), CHANGELOG. Acceptance: just check passes. Model: Sonnet. CI config and docs only, not daemon start-up. Out of scope: the Python pack README (separate task, after this), a setup action, publishing to package managers, cutting any tag.
