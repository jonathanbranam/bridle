+++
id = "br-kkdr"
title = "[at next reboot] Check dalek comes back on its own: daemons, SSH key, gateway"
kind = "chore"
state = "claimed"
created_at = "2026-10-04T13:39:39.271Z"
updated_at = "2026-10-04T13:39:39.275660Z"
created_by = "external:advisor"
watchers = [
    "external:advisor",
    "human",
]
priority = "low"
+++

After the next reboot, before starting anything by hand:
- bridle daemon list shows bridle (:7401), bridle-ui (:7405), track-web (:7406), started by launchd.
- ssh -T git@github.com works without asking for the passphrase (Keychain).
- Allow any firewall prompts for bridle (until br-p88z, they recur after rebuilds).
- The gateway: start it by hand until br-c657 is installed.

## Thread

### note · external:advisor · 2026-10-04T13:39:39.273Z
created for the human, priority low

### note · external:advisor · 2026-10-04T13:39:39.275Z
To-do for you (low priority): [at next reboot] Check dalek comes back on its own: daemons, SSH key, gateway. Finish it with `bridle task done br-kkdr`.
