+++
id = "br-4mbd"
title = "Reserve xxxx: never mint it as a ticket id, and the link-check hook ignores br-xxxx"
kind = "fix"
state = "planned"
created_at = "2026-10-17T19:47:15.922Z"
updated_at = "2026-10-18T09:10:56.668716Z"
created_by = "external:advisor/tj7q"
watchers = [
    "external:advisor/tj7q",
    "agent:w4mbd",
]
summary = "xxxx is reserved via RESERVED_IDS/is_reserved_id in bridle-api types; id minting (store.rs random_id_chars, ticket.rs mint_id/reusable_id) redraws it, ticket check errors on a file with that id, link-check ignores br-xxxx and bare xxxx. Tests in ticket.rs and link_check.rs; no minting-redraw test (random draw can't be forced). CHANGELOG added."
ticket = "4mbd"
+++

docs/tickets/open/reserve-xxxx-never-mint-it-as-a-ticket-id-and-the-link-check-4mbd.md

## Thread

### note · external:advisor/tj7q · 2026-10-17T19:50:17.858Z
The human, 2026-10-17 ~4:10 PM ET, to advisor (tj7q): "Yes ready 4mbd." Readied by human via advisor (tj7q).

### note · agent:w4mbd · 2026-10-18T08:57:27.155Z
Implemented (RESERVED_IDS in bridle-api, minting, ticket check, link_check, tests, CHANGELOG); just check running, will commit and mark built when green.

### note · agent:w4mbd · 2026-10-18T09:10:56.453Z
Work committed on bridle/w4mbd, tip 91e0b2b9 (check green on 3a049832: exit 0, 1866 passed; merge of main after was docs-only). 'bridle ticket built' fails: 'forbidden: worker agents cannot use agent lifecycle endpoints'; the ticket shows planned, not claimed. Manager please mark built/land.

### note · agent:w4mbd · 2026-10-18T09:10:56.668Z
done: xxxx reserved; branch bridle/w4mbd tip 91e0b2b9, check green on 3a049832 (exit 0, 1866 tests). 'ticket built' is refused for me (forbidden: worker agents cannot use agent lifecycle endpoints), so please mark it built/land.
