+++
id = "br-nmf8"
title = "Fix red main: ubuntu CI fails on 2d9adce2 (br-qpr7 spec tests)"
kind = "bug"
state = "open"
created_at = "2026-10-04T21:05:38.666Z"
updated_at = "2026-10-04T21:37:46.224985Z"
created_by = "agent:manager-2"
watchers = ["agent:manager-2"]
size = "S"
summary = "project_resolution_test.rs: runs_here() now skips 'daemon launchd install' off macOS (mirroring the existing systemd-on-Linux skip), so the two naming tests no longer fail on Linux CI with 'bridle launchd is macOS only'. Test-only change; just check green locally (1183 passed). Linux not run locally; fix follows the CI log."
+++



## Thread

### note · agent:manager-2 · 2026-10-04T21:06:12.670Z
Diagnosis from the orchestrator: tests inside_a_workspace_every_command_acts_on_its_project (line 514) and a_repo_not_yet_served_is_named_after_its_folder (line 571) in crates/bridle/tests/project_resolution_test.rs fail on Linux with 'bridle daemon launchd install: error: bridle launchd is macOS only'. Fix: on non-macOS skip the launchd commands in the command list (cfg!(target_os = "macos")) or accept that error as a pass there; keep the spec coverage link intact.

### note · agent:fix-ubuntu-ci · 2026-10-04T21:33:23.587Z
done: project_resolution_test skips 'daemon launchd install' off macOS (like systemd off Linux); just check green, 1183 passed; fb5a53ba

### note · agent:fix-ubuntu-ci · 2026-10-04T21:37:46.224Z
done: merged main, just check green (1183 passed); 14917de6
