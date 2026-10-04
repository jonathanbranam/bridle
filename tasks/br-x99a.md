+++
id = "br-x99a"
title = "Tue 10-06, at dalek: set up bridle's local signing certificate (bridle sign setup) once br-p88z lands, and check the firewall survives a rebuild"
kind = "chore"
state = "claimed"
created_at = "2026-10-04T23:55:55.619Z"
updated_at = "2026-10-04T23:55:55.622469Z"
created_by = "external:aide"
watchers = [
    "external:aide",
    "human",
]
+++

At dalek, not over SSH (the trust step may need a keychain dialog). Wait until br-p88z has landed on
main and bridle is reinstalled (`bridle task show br-p88z` says integrated).

You, 2026-10-04 (via the aide): "Yeah, send me those as to-dos for me to handle when it's done and
ready. I agree, I'll probably just wait until I get back to do it. That'll be Tuesday."

1. One-time setup of the local signing certificate: `bridle sign setup` (or `just sign-setup` in the
   bridle clone). Answer any keychain prompt.
2. Re-sign and reinstall so the running binary uses it (install and self-upgrade sign automatically
   once the certificate exists).
3. Check it holds: after the next rebuild or self-upgrade, the gateway still answers from another
   machine, and the firewall still allows bridle without re-registering it (no more socketfilterfw
   --remove/--add/--unblockapp).
4. If anything fails, tell the aide. The worker couldn't test the keychain or firewall steps; the
   verification commands are in br-p88z's summary.

Finish with `bridle task done <this id>`.

## Thread

### note · external:aide · 2026-10-04T23:55:55.621Z
created for the human, priority normal

### note · external:aide · 2026-10-04T23:55:55.622Z
To-do for you (normal priority): Tue 10-06, at dalek: set up bridle's local signing certificate (bridle sign setup) once br-p88z lands, and check the firewall survives a rebuild. Finish it with `bridle task done br-x99a`.
