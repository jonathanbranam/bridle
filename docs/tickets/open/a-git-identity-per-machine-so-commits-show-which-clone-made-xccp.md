---
id: xccp
title: A git identity per machine, so commits show which clone made them
kind: chore
opened: 2026-10-09
filed_by: external:advisor/product-manager
repos: [bridle]
changes: []
specs: []
needs: []
see: [j7r4, 8z7j]
tasks: [br-xccp]
---

## The ask

From postmortem j7r4 (incident br-2y3m), recommendation 6: every clone commits as the same git
identity, so a postmortem can't tell machines apart without reading ticket text. The human,
2026-10-09 14:05 EDT, verbatim (comment c5 on j7r4): "Agree in principle, let's make a ticket for
this, but it is non-urgent and can be done later; also it involves human work to make tokens and
handle that."

The ask: each machine (and so each clone) commits with an identity that names it, e.g. a committer
name or email carrying the machine name. Non-urgent; it needs the human's work (keys or tokens per
machine), which may be shared with the 8z7j credential option.

## What needs to be done (aide, 2026-10-10)

The human, 2026-10-10 ~1:45 PM ET, verbatim (to the aide):

> For the XCCP ticket that every machine gets its own identity, just look at the ticket and write
> up what needs to be done there. My question is: do I need to make a new PAT token or a new SSH
> for each machine, or is there an easier way to do it? I'm fine with making PAT tokens for each
> machine at this point. I feel like that is perfectly reasonable, given the complexity of what
> we're building, to just go ahead and switch over to PAT tokens, but I don't want that to be a
> surprising thing. I want to understand and set it up before we ship any change like that, so
> that I'm prepared for it and we don't lose any momentum.

**No new token or key is needed for this ticket.** A commit's author and committer are plain local
git settings (`user.name`, `user.email`); git and GitHub don't check them against the credential
that pushes. Tokens and SSH keys only authenticate the push. Today dalek commits as
`Jonathan Branam <jonathan.branam@gmail.com>` and pushes over SSH with its own key
(`~/.ssh/id_ed25519`); `gh` is logged in with an OAuth token for API calls.

Options for the identity:

- **A. Machine in the name, same email (recommended).** `Jonathan Branam (dalek)`,
  `Jonathan Branam (nuc)`, `Jonathan Branam (<windows pc>)`. GitHub still links every commit to
  the human's account by the email, so avatars and contribution counts are unchanged. Nothing for
  the human to set up.
- B. Machine in the email by plus-addressing, `jonathan.branam+dalek@gmail.com`. Visible in
  `git log --format=%ae`, but GitHub links those commits to the account only after each address is
  added and verified in GitHub's email settings (human work per machine).
- C. A `Machine: dalek` trailer on every commit bridle makes. Doesn't touch identity at all; needs
  every committer (workers, managers, the human by hand) to add it, so it's easy to miss.

What to build (A): `bridle serve` and `bridle sync` set `user.name` in each managed clone's local
config to `<global user.name> (<machine name>)`, the machine name being the one `owner.toml` and
`push-check` use, idempotently and only if the clone has no local `user.name` of the human's own.
Every worktree of a clone shares that config, so workers' and managers' commits carry it too.
Document it in add-a-machine.md. Done when `git log --format='%an'` on main shows which machine
made each new commit.

### Push credentials, separately (8z7j option C; not this ticket)

Per-machine push rights are a different question. 8z7j built option A (a pre-push hook on every
non-owner clone; br-hdbj) and left credentials for later "only if a bypass actually happens".
If that day comes:

- Each machine already has, or should have, its own SSH key added to the GitHub account; the
  account's SSH keys page shows when each was last used. Making one key per machine is the usual
  set-up and costs one `ssh-keygen` and one paste per machine.
- Fine-grained PATs per machine (HTTPS remote) can limit a machine to some repos and to read-only,
  which SSH account keys can't; that is what would make "only the owner can push" enforced by
  GitHub. Cost: a token per machine, expiry and renewal, and switching remotes to HTTPS.
- Deploy keys (one per repo per machine) also restrict to one repo, but multiply with projects.

Nothing here should switch without the human's say-so; any change to push credentials gets its
own ticket and a set-up note for the human before it ships.
