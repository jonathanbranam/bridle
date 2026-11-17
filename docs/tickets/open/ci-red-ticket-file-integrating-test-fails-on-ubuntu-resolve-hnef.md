---
id: br-hnef
title: "CI red: ticket_file integrating test fails on ubuntu (resolve commit has no git identity)"
kind: fix
created: 2026-11-17
created_by: external:orchestrator
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

## The ask

Make `ticket_file::tests::integrating_moves_stamps_and_commits_once_and_is_idempotent` pass on ubuntu CI, so main goes green. Critical: main is red and landings are held.

## What happened

CI run https://github.com/jonathanbranam/bridle/actions/runs/44141420535 (main b2b07a61, first run after br-ggnp landed as 17a6255c): `check (ubuntu-latest)` failed, macOS passed.

    panicked at crates/bridle-daemon/src/ticket_file.rs:196:9:
    assertion failed: note.contains("resolved/thing-ab12.md") && !note.contains("not committed")

## Cause (orchestrator read of the code)

The test's `git()` helper passes `-c user.name=t -c user.email=t@t`, but `resolve_and_commit` -> `commit_ticket_files` runs git without an identity. The ubuntu runner has no global git identity, so that commit fails and the note says "not committed". Dev Macs (and the macOS runner) have one, so it passed locally.

## Fix

In the test, give the temp repo an identity (`git config user.name t` / `user.email t@t` in `repo_with_open_ticket`), so the code under test commits. Check for other tests in br-ggnp's change that call `commit_ticket_files` the same way. Don't change production code unless it is wrong for a real repo.

## Verify

`just check` green; CI green on ubuntu after landing.

Model: Haiku (one-line test fix).
