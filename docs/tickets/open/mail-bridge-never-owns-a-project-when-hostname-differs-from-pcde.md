---
id: br-pcde
title: "Mail bridge never owns a project when hostname differs from [machine] name: dalek's bridle mail is stuck"
kind: bug
created: 2026-10-14
created_by: external:advisor/nuc-rename
theme: multi-machine
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-ga6s, br-n3kj, br-gdyy]
---

## The ask

The human, 2026-10-14 (to advisor nuc-rename): "yes, file that as a bug and ask for a fix; ineresting - some mail has gotten through I'm not sure why"

## What happens

dalek's mail bridge for bridle has taken no inbound mail since 2026-10-12. Mail to `bridle@dev.branam.us` stays in the bucket. For example, an allowlisted, DMARC-passing mail from 2026-10-13 5:29 PM ET (subject "Feature request for meta-notes") was still there on 2026-10-14 after a `launchctl kickstart -k` of `dev.bridle.mail.bridle`. The last mail delivered to bridle was m-10355, 2026-10-11 08:40.

## Why (advisor nuc-rename, from the code)

- Since br-ga6s (`e73e455e`, 2026-10-12, "owner.toml machine identity"), the daemon writes the owner as `[machine] name`, falling back to `hostname` (`crates/bridle-daemon/src/lib.rs:152`). bridle's `owner.toml` says `host = "dalek"`.
- `bridle mail run` still builds `FileLocal { host: local_hostname() }` (`crates/bridle/src/commands/orchestrator.rs:360,376`), which is the raw `hostname` output. On dalek that is `dalek.local`.
- `FileLocal::owns_project` (`crates/bridle-mail/src/local.rs`) compares the two. They differ, so the bridge thinks another machine owns bridle and leaves each mail in the bucket (`bridge.rs:389`, `Outcome::Waiting`). After `not_delivered_after_secs` the sender is told "not delivered yet: no machine is running bridle".
- Only inbound is gated, so outbound (question mails, the daily digest, "got it" relays) still works. That is why some mail seemed to get through. The gate also blocks emailed replies to question mails.
- vesta isn't affected: its `[machine] name` and `hostname` are both `vesta`.

## Fix

The bridge should take the machine name from the same place the daemon uses for `owner.toml` (`[machine] name`, then `hostname`). Share the daemon's function rather than copying it, so the two can't drift again. Check for other readers of `owner.toml` that use the raw `hostname` too: the pre-push hook and `project start`/`take-over`. A test: `[machine] name` set and different from `hostname` still owns the project.

Once it is released and dalek's bridge has restarted, the waiting mail is delivered on the next poll. Nothing is lost.
