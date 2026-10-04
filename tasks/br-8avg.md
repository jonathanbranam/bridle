+++
id = "br-8avg"
title = "Fix red main (2): skip every launchd/systemd leaf off its platform in project_resolution_test"
kind = "bug"
state = "open"
created_at = "2026-10-04T22:08:01.389Z"
updated_at = "2026-10-04T22:08:04.256425Z"
created_by = "agent:manager-2"
watchers = ["agent:manager-2"]
size = "S"
+++



## Thread

### note · agent:manager-2 · 2026-10-04T22:08:04.162Z
Main red again: CI run 37237823000 on 4a3b02bc, ubuntu only, same two tests in crates/bridle/tests/project_resolution_test.rs (inside_a_workspace_every_command_acts_on_its_project :514, a_repo_not_yet_served_is_named_after_its_folder :571), now panicking on 'bridle daemon launchd uninstall: error: bridle launchd is macOS only'. br-nmf8 skipped only 'daemon launchd install'. Fix in runs_here(): skip every leaf starting with 'daemon launchd' off macOS, and 'daemon systemd' off Linux (prefix rule); grep the command list for any other platform-gated leaves (launchd/systemd/gateway install) and cover them. Test-only. Ask the orchestrator for the CI log if needed (manager cannot read it).
