+++
id = "br-jkkp"
title = "Tests for 36a3: role tokens rejected at start raise the incident; a random-token 401 is not counted (then nbgp too)"
kind = "fix"
state = "built"
created_at = "2026-10-16T10:03:24.458Z"
updated_at = "2026-10-16T11:37:45.985012Z"
created_by = "agent:pm-1"
watchers = [
    "agent:pm-1",
    "agent:jkkp",
]
branch = "bridle/jkkp"
commit = "ed8c66f34a50ed641b4b233cc4abaed2d5379fc6"
summary = "Tests for br-36a3/br-nbgp. Two small testability refactors, no behaviour change: token_watch::raise and warn_if_role_tokens_rejected take `&impl Incidents` instead of a RealIncidents (the caller builds it), and the 401 filter's lookup is a pure `held_role` in server.rs. New tests: role tokens rejected at start raise one incident naming the roles and `bridle token pair` (and none when nothing held); held_role counts only held non-human role tokens, so a random-token or human-token 401 is not counted. nbgp ask 1 (restart keeps principals) and ask 2 were already tested (lib.rs role_tokens_the_store_lost_...; token_watch burst/window tests). Not tested: the full HTTP 401 middleware path end to end. just check exit 0, 1758 tests. No CHANGELOG (not user-visible)."
ticket = "jkkp"
+++

docs/tickets/open/tests-for-36a3-role-tokens-rejected-at-start-raise-the-incid-jkkp.md

## Thread

### note · agent:jkkp · 2026-10-16T11:37:45.863Z
built: ed8c66f34a50ed641b4b233cc4abaed2d5379fc6 (branch bridle/jkkp)

### note · agent:jkkp · 2026-10-16T11:37:45.985Z
done: tests for 36a3 start-time incident and 401 filter (small testability refactor, no behaviour change); just check exit 0, 1758 tests; ed8c66f34a50ed641b4b233cc4abaed2d5379fc6
