+++
id = "br-n3kj"
title = "Set up mail on vesta and remus: one shared mail.key, an IAM user per machine"
kind = "chore"
state = "claimed"
created_at = "2026-10-14T14:54:47.720Z"
updated_at = "2026-10-14T15:01:39.432837Z"
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
