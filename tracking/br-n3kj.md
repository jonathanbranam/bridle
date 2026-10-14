+++
id = "br-n3kj"
title = "Set up mail on vesta and remus: one shared mail.key, an IAM user per machine"
kind = "chore"
state = "claimed"
created_at = "2026-10-14T14:54:47.720Z"
updated_at = "2026-10-14T15:58:56.508072Z"
created_by = "external:advisor/nuc-rename"
watchers = [
    "external:advisor/nuc-rename",
    "human",
]
priority_at = "2026-10-14T14:54:47.925436Z"
ticket = "n3kj"
+++

docs/tickets/open/set-up-mail-on-vesta-and-remus-one-shared-mail-key-an-iam-us-n3kj.md

## Thread

### note · external:advisor/nuc-rename · 2026-10-14T14:54:47.925Z
created for the human, priority normal

### note · external:advisor/nuc-rename · 2026-10-14T14:54:47.929Z
To-do for you (normal priority): Set up mail on vesta and remus: one shared mail.key, an IAM user per machine. Finish it with `bridle ticket done br-n3kj`.

### note · external:advisor/nuc-rename · 2026-10-14T15:01:39.432Z
advisor nuc-rename, 2026-10-14: vesta's mail.key copied and verified identical to dalek's (sha256 of the raw file matches; 32 bytes, mode 600). Fixed the hash command in the steps: od output differs between macOS and Linux, so the od|md5sum hashes didn't match though the keys did.

### note · external:advisor/nuc-rename · 2026-10-14T15:58:56.508Z
advisor nuc-rename, 2026-10-14: vesta's four bridle-mail units are active and enabled; no warnings. On first start each sent a digest, and meta-notes-ui sent question m-0012. Open point, raised with the human and not decided (the human: "something to consider here; I'm not sure yet the right solution though"): once the daemon accepts an email, the bridge deletes it from S3. If no aide is running, it goes to external:orchestrator, and if no orchestrator waiter watches that daemon, it sits unread with no notice to the sender. Today that is the case for notes on vesta (no aide). Options discussed: an aide per mailed project; ticket addresses (<project>+t-<id>@); a 'delivered, but nobody is listening' reply from the bridge.
