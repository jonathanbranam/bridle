+++
id = "br-f4d1"
title = "One ~/.bridle/credentials.toml for external principals, picked by BRIDLE_AS and project (t6kq)"
kind = "feature"
state = "planned"
created_at = "2026-09-29T02:46:37.164Z"
updated_at = "2026-09-29T02:46:39.186240Z"
+++

Goal: replace the six ~/.bridle-*.token files with one file, and make the CLI pick the token itself. Ticket: docs/questions/open/one-credentials-file-per-machine-t6kq.md (read it: the human's ask and the design). Read docs/design/agent-host/principals.md for how the CLI picks a token today. Do: (1) ~/.bridle/credentials.toml, mode 0600, one table per principal and one key per project (project names as in the registry); (2) BRIDLE_AS=<principal> makes every bridle command use that principal's entry for the project it talks to (--project, or the cwd's daemon); an explicit --token or BRIDLE_TOKEN still wins, and the existing discovery order otherwise stays; a missing entry is a clear error naming the file, principal and project; (3) bridle token create <name> --project <p> writes the entry into the file (create the file 0600 and the directory if missing, keep other entries, preserve nothing else), and token revoke removes it; (4) update scripts/claude-orchestrator and scripts/claude-advisor to set BRIDLE_AS; (5) update principals.md, cli.md. Refuse to read the file if its mode is looser than 0600 (warn and say chmod). Acceptance: just check passes; tests for the pick order (--token, BRIDLE_TOKEN, BRIDLE_AS entry), missing entry error, create and revoke round trip, file mode. Model: Sonnet. Out of scope: migrating the six existing files (the orchestrator does that by hand after this lands), dropping tokens or changing the principal model, encryption or keychain. Touches cli.rs token args and the client credential code; not the same area as the task-store tasks in the queue, but check for conflicts in cli.rs.
