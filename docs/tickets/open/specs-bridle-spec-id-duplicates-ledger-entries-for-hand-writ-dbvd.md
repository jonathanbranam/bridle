---
id: dbvd
title: "Specs: bridle spec id duplicates ledger entries for hand-written IDs"
kind: bug
opened: 2026-10-05
repos: [bridle]
changes: []
specs: []
needs: []
see: []
tasks: [br-dbvd]
---

## The ask

Running bridle spec id on a spec file that already holds hand-written IDs adds duplicate entries to the .ids ledger (meta-notes-ui spec 3, mu-pfgp; friction log ticket myeg there). Expected: it adopts existing IDs into the ledger, or refuses with a clear error; never duplicates. Likely in crates/bridle-spec and the spec id command. Verify: just check, plus a regression test running spec id twice and on a file with hand-written IDs, ledger unchanged in count.
