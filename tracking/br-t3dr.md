+++
id = "br-t3dr"
title = "bridle-ui ships with each bridle release, fixed to that version, and self-upgrade moves both"
kind = "feature"
state = "planned"
created_at = "2026-10-10T17:17:50.352Z"
updated_at = "2026-10-11T22:40:44.057525Z"
created_by = "external:advisor/product-manager"
watchers = ["external:advisor/product-manager"]
ticket = "t3dr"
+++

docs/tickets/open/bridle-ui-ships-with-each-bridle-release-fixed-to-that-versi-t3dr.md

## Thread

### note · external:aide · 2026-10-11T18:07:00.414Z
From the human, via aide (2026-10-11 ~1:35 PM ET), on t3dr, verbatim: "A1 is good, but yes my only concern is slowing down and already slow build process; but it should be ok. it happens in CI though? that would be good." Context: the aide had listed the drawbacks of A and recommended A1 with Q2 bundled, Q3 yes with the ui.pin bump made part of cutting a release, Q4 yes. The human approved A1 (Q2). Q1 is moot: bridle-ui and bridle are both public repos (checked with gh), so no secret is needed. The aide answered the CI question: yes, the UI build runs only in release.yml on a v* tag, in parallel with the three Rust target builds; just check, worker builds and CI on main are unaffected. Q3 and Q4 were not answered explicitly: treat them as the aide recommendation, not yet confirmed by the human.

### note · external:aide · 2026-10-11T21:02:01.604Z
From the human, via aide (2026-10-11 ~4:45 PM ET), verbatim: "Q3 yes / Q4 yes". Q3: the UI reaches release machines with each bridle release, and bumping the pinned UI version is part of cutting a release. Q4: the dev machine keeps its own bridle-ui clone. Both were my recommendations and are now confirmed. A1 and Q1 were settled earlier.

### note · external:aide · 2026-10-11T21:02:10.162Z
Correction from aide: the human answered at ~5:02 PM ET, not ~4:45.

### note · agent:pm-1 · 2026-10-11T22:40:42.641Z
Build brief (pm-1). Build option A1 as in "Build shape", all four steps, one branch. Model: Sonnet.

Files likely: ui.pin (new, repo root), .github/workflows/release.yml, scripts/package-release.sh, crates/bridle-daemon/src/release.rs and upgrade.rs, the fv86 installer script, docs/design/human-web-ui.md, docs/design/agent-host/daemon.md (Upgrade), docs/README.md release section, CHANGELOG.

Q1: bridle-ui is public (aide checked), so no secret is needed. The ui job checks it out at the pin with the default token. If the checkout fails as private, stop and ticket ask the human; do not add a secret.

Rules: the UI build runs only in release.yml on a v* tag, never in just check or worker builds. The daemon writes ~/.bridle/ui/ only in self_upgrade = "release" mode, and only if the tarball has ui/ (older tags: leave the folder alone). Extract to ui.tmp then atomic rename. Release fails if the built api-version differs from the gateway API_VERSION. ui/bridle-version records tag + bridle-ui sha.

Acceptance: just check passes; unit tests for the daemon extraction (tarball with ui/, without ui/, non-release mode leaves the folder alone, atomic replace) using a fixture tarball; package-release.sh tested with a stub ui dir. The workflow itself cannot run locally: say in the done note what was not exercised, and the first real run is the next tag.

Migration: none for projects. Existing release machines gain the UI at their next upgrade past the first tag carrying it.

Out of scope: a bridle ui install command, embedding in the binary (D), bridle-ui repo changes, br-785a (dev-machine follow).
