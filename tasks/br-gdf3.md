+++
id = "br-gdf3"
title = "Peer-token setup guidance: a token per receiving project per sending machine, minted on the receiver"
kind = "bug"
state = "planned"
created_at = "2026-10-06T23:09:15.299Z"
updated_at = "2026-10-09T14:25:16.609587Z"
created_by = "external:orchestrator"
watchers = [
    "external:orchestrator",
    "external:advisor/product-manager",
]
priority = "high"
priority_at = "2026-10-09T14:10:40.525104Z"
ticket = "gdf3"
+++

Ticket: docs/tickets/open/peer-token-setup-guidance-a-token-per-receiving-project-per-gdf3.md (read it: the human's complaint and the rule). Human priority 2026-10-09 (machine setup).

Rule to state everywhere: a peer token exists per RECEIVING project (daemon) per SENDING machine; mint it on the receiver with `bridle token create --peer <SENDING machine> --project <receiving project>` and paste it under `[peer]` on the sender. Projects on the same machine need one each. `--peer` takes the name of the machine that will SEND, not the receiver (the human got this backwards and hit `conflict: principal peer:dalek already exists`).

Do:
1. The error: where the daemon returns `principal {id:?} already exists` (crates/bridle-daemon/src/store.rs:1754) for a `peer:<machine>` principal, make the message say which machine name `--peer` expects, e.g. `peer:dalek already exists in project bridle: --peer takes the machine that SENDS to this project (the receiver is this machine); to replace its token, revoke it first (<the real revoke command>)`. Keep the plain message for other principal kinds. Check the exact revoke/rotate command in cli.md and use it.
2. `bridle token create --help` (clap text in crates/bridle/src/cli.rs): add a sentence on `--peer` with the direction rule.
3. Docs: docs/design/cli.md (token create entry), docs/design/mail.md (peer tokens section, grep `\[peer\]`), docs/context/adding-a-project.md and docs/context/nuc-host.md if they mention peer tokens: state the rule once, with a worked example (dalek sends to project notes on the NUC: on the NUC run `bridle --project notes token create --peer dalek --project notes`... use the real flag forms from --help; paste under [peer] in dalek's credentials.toml with the real key shape from the code).
4. A test for the new error text (store or API test next to the existing one for the duplicate principal). CHANGELOG.md entry on top.

Acceptance: just check passes. Model: Sonnet. Migration: none. Out of scope: n63z (one command mints every token), changing the token model.

## Thread

### note · external:advisor/product-manager · 2026-10-09T11:04:39.629Z
watching the task

### note · external:advisor/product-manager · 2026-10-09T14:10:40.525Z
priority: normal -> high

### note · external:advisor/product-manager · 2026-10-09T14:10:41.322Z
PdM (advisor product-manager): machine setup is the human's priority workstream (2026-10-09 ~10:15 AM ET: "I want that as a priority so that I can use it to set up the new Windows machine and add it to the network"; quoted in hua2/xrkh/kt25). Priority high; roadmap docs/notes/roadmap.md.
