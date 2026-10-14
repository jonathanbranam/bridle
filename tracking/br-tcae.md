+++
id = "br-tcae"
title = "Mail bridge runs a stale binary after upgrades, and its digest fails silently on new ticket kinds/states: no digest 10-08 to 10-13"
kind = "bug"
state = "pending"
created_at = "2026-10-14T21:05:25.085Z"
updated_at = "2026-10-14T22:11:48.035876Z"
created_by = "external:aide"
watchers = ["external:aide"]
ticket = "tcae"
+++

docs/tickets/open/mail-bridge-runs-a-stale-binary-after-upgrades-and-its-diges-tcae.md

## Thread

### note · external:aide · 2026-10-14T21:05:32.408Z
From aide, for the human's question ("I haven't seen a bridle daily digest email since I think 10.7. Can you check on that?"): filed br-tcae. The mail bridge ran the 10-07 binary for a week (nothing restarts mail units on upgrade) and its digest died silently on new ticket kinds/states. Digest resumed today after the pcde restart. Please plan; the stale-binary half also affects vesta's bridges.

### note · external:aide · 2026-10-14T22:11:48.035Z
From the human, via aide, verbatim: "yes, please fix the mail daemon it should self-upgrade and restart with the rest of bridle. That should work on the machine that builds bridle and those that install it from release". Recorded on br-tcae as the decision. Please plan it.
