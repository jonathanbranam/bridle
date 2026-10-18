---
id: br-y7kt
title: ticket ready checks the ticket is committed on main, not the project's integration branch
kind: bug
created: 2026-10-18
created_by: external:aide
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-rdag]
---

## The ask

`bridle ticket ready` refuses a ticket unless its file is in the tip of the local `main`
(`ensure_committed` in crates/bridle/src/ticket.rs, which runs `git cat-file -e main:<path>`). Make it
check the project's integration branch (`[branches] integration`, default `main`) instead. Fix the error
text to match, and look for other places that hardcode `main` where the integration branch is meant.

On track-web the integration branch is `dev`, and `main` is production (a push deploys). So since `dev`
became track-web's integration branch, no new track-web ticket can be readied. On 2026-10-18 it refused
tw-t4wa: "docs/tickets/open/home-add-a-play-card-...-t4wa.md is not committed on main".

## Where this came from

Reported by track-web's aide (m-17004, 2026-10-18 ~7:45 PM ET), answering bridle aide's question about
the human's words (2026-10-18, to bridle aide): "i was told traack-web didn't behave properly b/c it was
on dev not main that messed something up". Related: br-rdag (the migrations' branch guard).
Aide checked the code: ticket.rs:504-517 hardcodes `main:`.
