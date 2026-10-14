+++
id = "br-n3kj"
title = "Set up mail on vesta and remus: one shared mail.key, an IAM user per machine"
kind = "chore"
state = "claimed"
created_at = "2026-10-14T14:54:47.720Z"
updated_at = "2026-10-14T18:27:42.043464Z"
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

### note · external:advisor/nuc-rename · 2026-10-14T18:23:57.262Z
advisor nuc-rename, 2026-10-14: verified complete: vesta's four bridges are active and the test mail arrived; remus has the shared mail.key (sha256 matches), the same [mail] block and bridle-mail-remus credentials (inbound/ lists). Step 4 (answer_for_human) is left alone. The human: "nah, idk about any of this yet; I don't use email much; let's just leave it alone for now". Discussed: external:mail closes on any reply, even 'give me more info'; an alternative is routing emailed replies through the aide with external:aide authority. Separate finding: dalek's bridle bridge (pid 12666, started Oct 7, an old binary from before the gdyy logging fix) looks stuck: no S3 connection, no CPU. An allowlisted, DMARC-passing mail to bridle@ from 2026-10-13 5:29 PM ET is still in the bucket. The fix is launchctl kickstart -k gui/$(id -u)/dev.bridle.mail.bridle.

### note · external:advisor/nuc-rename · 2026-10-14T18:27:42.043Z
advisor nuc-rename, 2026-10-14: correction to the earlier comment: dalek's bridge isn't stuck. It never owns bridle because it compares the raw hostname (dalek.local) with owner.toml's [machine] name (dalek). Filed br-pcde; the orchestrator is asked to fix it (m-12880). Keep this to-do open until br-pcde lands and the waiting bridle@ mail is delivered.
