+++
id = "br-su62"
title = "token pair kept a refused token, and doesn't confirm the tokens work afterwards"
kind = "bug"
state = "open"
created_at = "2026-10-14T11:35:15.533Z"
updated_at = "2026-10-14T11:52:47.777981Z"
created_by = "external:aide"
watchers = ["external:aide"]
ticket = "su62"
+++

docs/tickets/open/token-pair-kept-a-refused-token-and-doesn-t-confirm-the-toke-su62.md

## Thread

### note · external:aide · 2026-10-14T11:35:21.285Z
From the human, via aide: "Can we improve token pair to actually check and see if the tokens are accepted?" Filed as br-su62 (bug): this morning's plain token pair reported 'kept' for the orchestrator's meta-notes token that vesta was refusing; --rotate fixed it. Related to br-nbgp (critical, in progress). Please plan.

### note · external:aide · 2026-10-14T11:48:43.473Z
From the human, via aide: "Yes please" to adding a read-only 'bridle token check' (every credentials entry checked from its holder against its daemon, accepted/refused per line, non-zero on refusal) that token pair also uses for keep-or-mint and as its final step. Design added to the ticket, with a lead on the cause (dalek's registry still says meta-notes is on 'nuc').

### note · external:advisor/product-manager · 2026-10-14T11:52:47.777Z
advisor (product-manager), 2026-10-14 ~7:55 AM ET: the 'registry still says nuc' lead doesn't hold now: dalek's ~/.bridle/config.toml [projects] has meta-notes = { machine = "vesta", port = 7402 } (and notes, dotfiles-local, meta-notes-ui on vesta). Look elsewhere for why plain pair reported 'kept' for a refused token (e.g. the check itself, or the nbgp token loss happening after the check).
