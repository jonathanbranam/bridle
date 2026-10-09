---
id: n63z
title: "One command mints every peer token: all projects, all machines, at project creation"
kind: feature
opened: 2026-10-06
repos: [meta-notes]
changes: []
specs: []
needs: []
see: [sk7p, 3haz]
tasks: []
---

## The ask


## The ask

Found on the NUC, 2026-10-06 about 02:20 UTC. After the human installed
29296901 (3haz slice 1), every `bridle send --project <other>` failed with
"no peer token for '<project>': its human runs `bridle token create --peer
nuc` there and pastes the token under [peer]". That's one token per receiving
project, per sending machine, each minted and pasted by hand, for daemons
that already share `~/.bridle` on the same machine. dalek doesn't need them
yet only because it hasn't upgraded past 3haz slice 1.

The human, 2026-10-05 (to the meta-notes orchestrator):

> This whole thing smacks of security theater, but it's what we have for
> now. ... At some point in the future, the machine-to-machine stuff should
> all be tokenized anyway, and it does let us build it the right way.
>
> I think, just as a user, the setup is insane. I realize that maybe agents
> can't do this, and that's perfectly reasonable. If I have access to
> everything as a human, the files to write to it, and the ability to create
> the tokens on both machines and SSH across everything, then there should be
> a script to just do this for me. That should all happen automatically when
> I create a new project too.
>
> ... mint every token that you need for all the projects I have access to,
> and then just boom, boom, boom, boom, boom. All the machines that are
> connected get a token. Every project gets a token.
>
> It's possible that I would have a project, for some reason, that I don't
> want to be able to communicate with the other projects. ... it's like
> multi-tenancy from the start, kind of. It just needs to be a script.

## Wanted

- **One human-run command** mints and installs every peer token: for every
  project on every connected machine, a token for each other machine
  (and same-machine daemons), written into each machine's
  `~/.bridle/credentials.toml` over ssh. Re-running is safe (fills only
  what's missing).
- **Project creation runs it**, so a new project can send and receive mail
  with every other project at once.
- **Same machine needs nothing by hand**: daemons that share `~/.bridle`
  set up their peer tokens themselves (or the command covers them).
- **Opt-out per project**: a project can be left out of mail with the
  others (a config setting); the default is everyone talks to everyone.
- Agents needn't be able to run it; it's the human's command.

Builds on [[pair-machines-token-setup-over-ssh-sk7p|sk7p]] (machine pairing
over ssh) and [[daemons-deliver-mail-to-each-other-across-machines-store-and-3haz|3haz]]
(P5, peer tokens). Wanted before dalek upgrades past 3haz slice 1, or its
cross-project mail breaks the same way.

## Decided (2026-10-09): folded into `bridle token pair` (sk7p)

The human, 2026-10-09 ~1:55 PM ET, verbatim: "one command, by default it does both, support
--(no-)peer and another option for the other types of tokens. Update n63z with your proposal on
the name of all CLI options before scheduling." (Full answer quoted in sk7p.)

**The design is in [[pair-machines-token-setup-over-ssh-sk7p|sk7p]], section "Design", and that
section is authoritative**; this ticket keeps the ask. Built by br-8c25. Option names
(approved by the human 2026-10-09 ~2:20 PM ET, with the opt-out):

```
bridle token pair [--machines <m>,...] [--projects <p>,...] [--roles <r>,...]
                  [--tokens role,peer] [--rotate] [--dry-run]
```

Each selector left out means all; naming some narrows to exactly those; no `--exclude-*`.
Token types: `role` (a role reaching a daemon) and `peer` (this ticket: daemon to daemon mail).
Opt-out proposed as `[mail] peers = false` in the project's `.bridle/config.toml` (default
true). Project creation runs `bridle token pair --projects <new>`.
