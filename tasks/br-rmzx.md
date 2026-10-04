+++
id = "br-rmzx"
title = "Fix red main (3): never run daemon systemd/launchd install in project_resolution_test"
kind = "bug"
state = "planned"
created_at = "2026-10-04T23:08:43.505Z"
updated_at = "2026-10-04T23:08:44.096760Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
size = "S"
+++

Fix red main (3): CI run 37242072891 on fe5730a2 (br-8avg), ubuntu-latest only; macOS green.

Failing (crates/bridle/tests/project_resolution_test.rs), both with the same error:
- a_repo_not_yet_served_is_named_after_its_folder, panicked at :571:9
- inside_a_workspace_every_command_acts_on_its_project, panicked at :514:9
  "bridle daemon systemd install: error: this machine has no name: set `[machine] name = "..."` in ~/.bridle/config.toml"

Cause (orchestrator): br-8avg's runs_here() now RUNS `daemon systemd *` on Linux (and `daemon launchd *` on macOS) as a Names command. On Linux, systemd install calls owned_here() (crates/bridle/src/systemd.rs:72), which needs [machine] name and the project listed in [projects] on this machine; the test world has neither. Past that, install would write a unit and likely call systemctl --user on the CI runner.

Fix (recommended): a test must never install a real system service. Classify `daemon systemd` and `daemon launchd` as Skip in CLASSES (reason: "installs a system service; the unit/plist naming is covered by systemd.rs / launchd.rs unit tests"), and drop runs_here() if nothing else needs it. Check that launchd install on macOS doesn't call launchctl in this test today either (if it does, that's the same hazard on every dev machine; Skip removes it). If you'd rather keep the naming check, give the test world [machine] name and a [projects] entry AND prove no systemctl/launchctl runs (e.g. a dry-run path); but Skip is the KISS fix.

Verify: just check green on macOS; you have no Linux, so reason through both OS branches explicitly in the task comment (which leaves run where). Cost: the fourth red run, so be sure.
Model: sonnet. Files: crates/bridle/tests/project_resolution_test.rs only.
