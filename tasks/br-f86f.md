+++
id = "br-f86f"
title = "bridle task search: find tasks by words in title and body"
kind = "feature"
state = "planned"
created_at = "2026-09-29T01:48:14.082Z"
updated_at = "2026-09-29T01:48:17.225732Z"
+++

Goal (traceability, the human's reason for sq4m and tr7k): find a task by words, then bridle task show gives brief, summary, branch and merge commit. Do: bridle task search <words...> lists tasks (id, state, title, same columns as task list) whose title, body or summary (once br-1ac1 has landed, else title and body) contain every word, case-insensitive substring match; include done and dropped tasks; --json like the other commands. Smallest thing: filter in the daemon over the task store (a new GET route, wire type in crates/bridle-api/src/types.rs with client and daemon together) or, if simpler and the list route already returns bodies, in the CLI; no index or FTS (YAGNI). Update docs/design/cli.md. Acceptance: just check passes; tests for title match, body match, all-words AND, no match, integrated tasks included. Model: Haiku. Out of scope: ranking, search of threads or tickets. Run after br-1ac1 has merged (both touch the task store and cli.rs).
