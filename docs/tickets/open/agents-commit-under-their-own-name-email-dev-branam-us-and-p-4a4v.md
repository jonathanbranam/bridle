---
id: br-4a4v
title: Agents commit under their own name with bridle@branam.us, and push with a per-machine PAT, not the human's identity and SSH key
kind: feature
created: 2026-10-17
created_by: external:advisor/product-manager
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-xccp, br-8z7j]
theme: security
---

## The ask

The human, 2026-10-17 ~5:05 PM ET, to advisor (product-manager), verbatim (dictated):

> Also, I created PAT tokens for everyone, and I think that Remus, at least, is using the PAT token now.
>
> That's a TBD: I want to stop having agents commit with my git identity. I guess that's just for push: push with my credentials, my SSH key. Just send me a reminder to do it, or whatever needs to be done.
>
> I don't really care what my git identity is when I do my own commits, but ideally, it would be different. I don't care. It's fine. If we can assign emails too that are different instead of them all being my email, I don't know. I don't want anyone emailing bridle, though. All right, I don't really care. Everyone should commit as dev@branam.us, though.

TBD: the human has not decided the shape; this ticket waits for them. Two parts:

1. **Agents' commit identity.** Today agents' commits carry the clone's git identity, which is the
   human's (`Jonathan Branam (<machine>) <dev@branam.us>`). Wanted: agents commit under a name that
   is not the human's, email still `dev@branam.us` (the human: "Everyone should commit as
   dev@branam.us"; no bridle mailbox). Likely shape (for the human to confirm): bridle sets
   `GIT_AUTHOR_NAME`/`GIT_COMMITTER_NAME` for every agent it spawns, e.g.
   `bridle <agent-name> (<machine>)`, email unchanged; the human's own commits keep their identity.
2. **Push credentials.** Agents push with the human's SSH key today. The human made a PAT per
   machine and thinks remus already pushes with it. Wanted: every machine pushes with its own PAT
   (HTTPS remote + credential helper), not the human's SSH key. This is set-up work for the human
   per machine (see br-xccp "Push credentials, separately" and br-8z7j option C).

Reminder for the human (part 2), per machine (dalek, vesta, remus, NUC):
`git remote -v` in each clone shows `git@github.com:` (SSH key) or `https://github.com/` (PAT).
Switch with `git remote set-url origin https://github.com/jonathanbranam/<repo>.git`; then the
first push asks for the PAT once (osxkeychain/credential manager keeps it).

## Decision (the human, 2026-10-17 ~5:20 PM ET): the email

The human, verbatim (to advisor product-manager): "4a4v - different emails is fine too or a single
email for all bridle work. But it should be a google email like bridle@branam.us not at
dev.branam.us"

So part 1 is decided (this replaces "email still dev@branam.us" above): agents commit with a single
email, `bridle@branam.us` (a Google mailbox or alias on branam.us), and a name that says which agent
and machine, e.g. `bridle <agent-name> (<machine>)`. Simplest build: bridle sets
`GIT_AUTHOR_NAME`, `GIT_AUTHOR_EMAIL`, `GIT_COMMITTER_NAME`, `GIT_COMMITTER_EMAIL` in the
environment of every agent it spawns (workers, managers, document reviewers; also the state-branch
commits now made as `bridle <bridle@localhost>`). The human's own commits keep their identity.
Tests; docs (cli.md or agent-host), CHANGELOG.

The human, by hand: create `bridle@branam.us` in Google (mailbox or alias), and add it to the
GitHub account's emails if agent commits should link to the account (otherwise they show unlinked,
which is fine).

Part 2 (per-machine PAT push) stays the human's set-up work; reminder sent via aide.
