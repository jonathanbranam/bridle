+++
id = "br-r9tk"
title = "token pair after a machine rename: revoke role@old and peer:old tokens, move old credential sections (eurm follow-up, for nuc->vesta)"
kind = "feature"
state = "planned"
created_at = "2026-10-12T01:35:58.573Z"
updated_at = "2026-10-12T01:36:24.319531Z"
created_by = "agent:pm-1"
watchers = [
    "agent:pm-1",
    "external:aide",
    "agent:weurm",
]
parent = "br-eurm"
+++



## Thread

### note · agent:pm-1 · 2026-10-12T01:36:03.408Z
Brief (pm-1). Model: Sonnet. Follow-up to br-eurm (landed c9a308a5), which did not include the PdM's renamed-machine cleanup; the human wants it for the nuc->vesta rename (r2nr).

Do: after a machine is renamed in [machines], a plain `bridle token pair` run also cleans up the old name: discover the old name from the daemons' token lists (visitor tokens role@old and peer:old that match no [machines] entry), revoke them, and move or remove the old credential sections in the holders' config so nothing keeps the old name. Idempotent; list what it will revoke and require --yes (or print and do it, matching how pair already reports changes; follow pair's existing style). Never revoke a token whose machine name is still in [machines].
Read first: br-eurm's thread and the PdM's renamed-machine comment on it, crates/bridle/src/token_pair.rs, docs/design/specs/token-pairing.md (add the rename rule there), principals.md.
Acceptance: just check passes; tests with the existing pair seams: renamed machine's old tokens revoked and sections moved, current machines untouched, second run no-op.
Migration: none. Out of scope: a rename command, the one-token-per-machine design, evy6.
