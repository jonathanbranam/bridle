---
id: sk7p
title: "`bridle token pair`: set up role and peer tokens between machines over SSH, like ssh-copy-id"
opened: 2026-10-01
repos: [bridle]
changes: []
specs: []
needs: []
see: [3ehu, k7mw, 9mxw, n63z, gdf3]
kind: feature
closed: 2026-10-09T23:00:17Z
---

## The ask

The human, 2026-10-01, verbatim: "I know agents shouldn't handle tokens, but can we also build
commands into bridle so that I can automate token setup between machines? Something like how SSH
copy keys command works? If I can successfully SSH to a machine, then it's safe to upload my public
key there. Similarly, if I'm authenticated on dalek, and I authenticate on NUC, I should be able to
run a bridle command to create tokens on both sides for humans and agents."

## Today

Cross-machine access (k7mw) works through `[<principal>.<machine>]` tables in
`~/.bridle/credentials.toml`. The human fills them by hand: run `bridle token create <name>
--print` on the daemon's machine, then paste the token into the other machine's file. Their own
CLI has no cross-machine token at all (3ehu). This has to be done for every principal (human,
orchestrator, advisor), every project and both directions.

## Proposal

(2026-10-09: widened by the Design section below, which wins where they differ.)

`bridle token pair <machine>` (human only), run on one machine where the human is authenticated:

- **Trust comes from SSH**, as with `ssh-copy-id`. If `ssh <machine>` works, the human is
  authenticated on both sides, and the human's token on each daemon's own machine (the
  workspace token file) can mint there.
- **For each project whose daemon runs on the remote machine** (from the remote registry or k7mw
  config): over SSH, run the remote `bridle token create` for each wanted principal (`human`,
  `orchestrator`, `advisor`; `--for` to choose). Stream the tokens back on stdout and write them
  into the local `credentials.toml` under `[<principal>.<machine>]` (0600).
- **Then the same in the other direction**: mint locally for the projects on this machine, and
  write them into the remote `credentials.toml` by piping them to a remote `bridle` helper over
  SSH stdin.
- Tokens travel only over the SSH channel's stdin and stdout: never in argv (visible in `ps`),
  never in logs or printed output.
- Idempotent: an entry that already works (checked with a `GET /v1/status`) is kept. `--rotate`
  revokes it and mints a new one. Names stay unique per k7mw (`<name>@<machine>` for remote
  principals).
- **Agents never run it.** It's human-only like `token create`. The agents' Claude Code
  permissions deny `bridle token *` (check the session/worker deny lists).
- `--dry-run` lists what would be minted and written.

## Decided (2026-10-01)

The human, verbatim: "Yes I should be addressable and traceable from machine to machine. I need
replies sent to human@NUC if that's where I'm connected. I assume if we drop the @ then it will be
same-machine."

- The human on another machine is a principal of its own, `human@<machine>` (e.g. `human@nuc`),
  following k7mw's `<name>@<machine>` naming. It acts with the human's authority, but it is recorded,
  traceable and revocable separately from the local `human`.
- A bare `human` always means the human on the daemon's own machine (workspace token).
- Replies go back to the principal that sent: a message from `human@nuc` is answered to
  `human@nuc`, and shows in the human's inbox on the NUC, not only on the daemon's machine.
- `human@<machine>` may run every human-only command `human` can (`token create`, `shutdown`,
  ...): SSH already proved it's the human (the human, 2026-10-01: "Yes agree").
- 3ehu part 1 (br-8b98) mints and reads exactly this principal: `[human.<machine>]` holds the
  `human@<machine>` token.

## Answered (2026-10-01)

- Whether a remote `bridle` is on SSH's non-interactive PATH (`~/.cargo/bin`). The human,
  verbatim: "Yes. We should assume proper bridle setup on both machines." So `bridle` is assumed
  on the remote PATH; no `--remote-bridle` flag is needed.

## Design (approved by the human 2026-10-09 ~2:20 PM ET)

Written by advisor (product-manager) from the human's answer below. It widens the Proposal
above and replaces it where they differ. **This section is the design; br-8c25's brief points
here and restates nothing.** n63z (peer tokens for every project) is folded in here.

### The human's words

The human, 2026-10-09 ~1:55 PM ET, verbatim (to advisor product-manager):

> 1. one command, by default it does both, support --(no-)peer and another option for the other
> types of tokens. Update n63z with your proposal on the name of all CLI options before
> scheduling.
>
> 1a. this command needs to be updated for all roles that need it. We should have a spec that
> enforces this: "All roles that need a token pairing are paired when the command runs." that
> list should include aide and product-manager today.
>
> Spell out the behavior exactly when we add a new role. I think when we add a new role, the
> command has to be run again. So the command finds all the valid roles that need tokens and
> processes them all.
>
> The command takes parameters, default is --all-roles (or similar). Or the user can specify
> --roles aide,orchestrator,human then it only applies those roles. Don't have the opposite (do
> not add --exclude-roles). This is enough control.
>
> so, when a new role is added, the human runs this once and it just works.
>
> Apply the same "all" default to everything: machines, projects, and roles (is there anything
> else)? Also, as said, give a name to each of the token types. One is "peer", what is the other
> type called? Then both default to true, but user can pick. A, perhaps better - follow the same
> plan --all-tokens ? then --tokens owned,peer . <- I'm not sure what to call the "my" token here.

### Two token types: `role` and `peer`

- **`role`**: a token an external role uses to reach a project's daemon. It lives in the
  credentials file of the machine the role runs on: `[<role>] <project>` for a daemon on the same
  machine, `[<role>.<machine>]` for a daemon on another (k7mw). Named `role` because `--roles`
  selects them. (Not "owned" or "my": the orchestrator's and aide's tokens are not the human's.
  Not "principal": a peer token is a principal too, `peer:<machine>`.)
- **`peer`**: a token a daemon uses to forward mail to another project's daemon (3haz). It is
  minted on the receiving daemon as `peer:<sending machine>` and written into the sending
  machine's file as `[peer] <receiving project>` (the direction rule, gdf3). Daemons on the same
  machine need them too: all cross-project mail goes through `/v1/forward`.

### The command

```
bridle token pair [--machines <m>,...] [--projects <p>,...] [--roles <r>,...]
                  [--tokens role,peer] [--rotate] [--dry-run]
```

Every selector defaults to **all**; naming some narrows to exactly those. There is no
`--exclude-*` and no `--no-peer`: `--tokens role` is "no peer". Leaving a selector out is the
only way to say "all" (no `--all-roles` flag), so there is one way to say each thing.

| Option | Default (left out) | Values |
|---|---|---|
| `--machines` | this machine plus every `[machines]` entry in `~/.bridle/config.toml` | machine names (`[machine] name`; `local` if unnamed) |
| `--projects` | every project whose daemon runs on a selected machine (the local registry; a remote machine's registry over ssh; `[projects]`) | project names |
| `--roles` | every role in the token-role list (below) | role names; applies to `role` tokens only (an error with `--tokens peer`) |
| `--tokens` | `role,peer` | `role`, `peer` |
| `--rotate` | off | revoke and re-mint the selected tokens instead of keeping working ones |
| `--dry-run` | off | print the plan; mint and write nothing |

"Anything else?": direction is not an option. Every run covers both directions between every
selected pair of machines (a token is no use one way only).

Behaviour:

- **Who**: the human only (`human`, or `human@<machine>` from br-8b98). Agents never run it: the
  build adds `Bash(bridle token *)` to the agents' deny lists (there is none today).
- **Trust** comes from ssh (the Proposal): the machine it runs on reaches each other selected
  machine with plain `ssh <host>` and `bridle` on the remote PATH (Answered, above).
- **Mesh**: for each selected role, every selected machine's credentials file gets a token for
  every selected project on every selected machine (same machine as `[<role>] <project>`, others as
  `[<role>.<machine>]`). For peer tokens, every selected project that sends gets one per
  receiving selected project. The running machine brokers: a token minted on B for C's file
  passes through its memory and the ssh channels' stdin and stdout only, never argv, a file
  other than the target credentials file, logs or output.
- **Idempotent**: an entry that works is kept (a role token is checked with `GET /v1/status`; a
  peer token present and not revoked is kept). Only missing or broken entries are minted. A second
  run changes nothing.
- **Partial failure**: an unreachable machine or daemon is reported and skipped; the rest goes
  on; the exit code is non-zero if anything failed. Re-running fills the gaps.
- **Output**: a summary per machine and token type (minted, kept, skipped, failed with the
  reason), never a token value. Peer lines say so ("peer: minted on the receiver for <sender>").

### The token-role list, and adding a role

The spec requirement (the human's words): **"All roles that need a token pairing are paired when
the command runs."**

- **One list in code** names the external roles that hold a token in `credentials.toml` (today
  nothing lists them; they are implicit in the session launchers, `session.rs`). The session
  launchers and `token pair` both read it. A test fails if a launcher sets `BRIDLE_AS` to a role
  that is not in the list, so a new role can't be added without it.
- **Today the list is**: `human` (remote only, as `human@<machine>`; the local human uses the
  workspace token file), `orchestrator`, `advisor`, `aide`, `mail`.
- **product-manager**: today it is a named advisor (`advisor/product-manager`) and shares the
  `advisor` token, so pairing `advisor` pairs it; the list says so in a comment. When
  product-manager becomes a role with its own identity, it joins the list like any new role.
- **Adding a role, exactly**: (1) the change that adds the role's launcher adds the role to the
  list (the test above enforces it). (2) The human upgrades bridle on each machine. (3) The human
  runs `bridle token pair` once, with no options. It finds the new role in the list, mints its
  token for every project on every machine, and keeps everything else. Nothing else is needed.
- **The same for a new machine** (add it to `[machines]`, run) **and a new project**: creating a
  project runs `bridle token pair --projects <new>` when the human creates it (n63z), so it can
  send and receive mail with every other project at once.

The build adds the spec `design/specs/token-pairing.md` (format: docs/design/specs.md) with
these requirements and a scenario each: all listed roles paired; defaults are all and a
selector narrows to exactly what's named; idempotent; the opt-out below; no token in argv, logs
or output. The new-role scenario: given a role added to the list, when `bridle token pair`
runs, then every selected machine has that role's token for every selected project, and no
existing entry changed.

### Opt-out per project (n63z): approved (comment c1)

`[mail] peers = false` in a project's `.bridle/config.toml` leaves it out of peer tokens in
both directions: it can't send mail to other projects or receive mail from them. Its role
tokens are still minted. The default is `true` (everyone talks to everyone). The advisor's
recommendation (2026-10-09): an on/off setting, not a list of allowed projects (YAGNI).

> [!comment] c1 human, 2026-10-09 14:20 EDT, on "The advisor's recommendation (2026-10-09): an on/off setting, not a list of allowed projects (YAGNI)." [read 2026-10-09 14:42 EDT]
> Agree.
>
> **doc-sk7p, 2026-10-09 14:42 EDT:** Noted, the on/off setting stands. No change to the document needed.
>
> **resolved by human via doc-sk7p, 2026-10-09 14:42 EDT**

### Ticket or task: where design lives

This ticket. The docs index (docs/README.md, "Tickets hold design decisions (the why and the
what), tasks the work and its status") puts design in the ticket: it is in git, reviewed and
diffed, and outlives the task. The task's brief says how to build it and points here.

## Done when

On dalek, `bridle token pair nuc` leaves the human, the orchestrator and the advisor able to reach
every project's daemon on both machines, with no token copied by hand. A second run changes nothing.

2026-10-09 (with the Design): on dalek, `bridle token pair` with no options leaves every
listed role able to reach every project's daemon on every machine, and every project that has
not opted out able to send mail to every other; a second run changes nothing; after a role is
added to the list, one more run pairs it.
