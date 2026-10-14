# Add a machine to the network

*Written 2026-10-09 for the human (ticket hua2). One procedure, in order, from a bare OS to a
machine that other machines can message and be messaged by. It works for macOS, Linux and WSL2;
OS specifics are linked, not copied.*

> **Status (2026-10-09):** Every command below was checked against `bridle <cmd> --help` and the
> docs on 2026-10-09. None of it was run end to end on a new machine; steps or lines marked
> "(unverified on a machine)" are from the help text and code only. Planned, not built: `bridle
> token pair` (br-8c25, ticket sk7p), `bridle systemd uninstall` (br-xrkh), automated project
> transfer.

Do the steps in order. The example adds a machine named `newpc` (the Windows PC under WSL2) to a
network of `dalek` (the Mac) and `nuc`. Replace the names with yours. A machine name is its
`[machine] name`, and the same string is a key of `[machines]` on every box.

Before step 1, the machine needs its OS side done:

- **WSL2 on the Windows PC:** follow [[docs/context/windows-wsl2-host|the WSL2 guide]] up to and
  including `just install`, then come back here at step 1. (Tailscale, linger and SSH are in
  that guide.)
- **Linux (like the NUC):** [[docs/context/nuc-host|the NUC host]], "How it's reached": Tailscale,
  SSH, `gh`, Node, Claude Code. Install bridle with the release script below.
- **macOS:** install with the release script below (macOS note after it).

Install bridle from a release, with no clone and no Rust toolchain (needs `curl`, `tar` and
`shasum` or `sha256sum`):

```bash
curl -fsSL https://raw.githubusercontent.com/jonathanbranam/bridle/main/scripts/install-release.sh | bash
# or a named release: ... | bash -s v0.6.0
```

It downloads `bridle-<tag>-<target>.tar.gz` and `SHA256SUMS` from the GitHub release, refuses
to install on a checksum mismatch, puts `bridle` in `~/.local/bin` (say if that is not on your
PATH) and prints the next step. Then make the daemon keep itself current: in
`~/.bridle/config.toml` set

```toml
[daemon]
self_upgrade = "release"
```

(`docs/design/agent-host/daemon.md`, "Release upgrade"; it also keeps the base workflow at the
binary's tag.) A machine that builds from a clone uses `just install` instead and leaves
`self_upgrade` alone. Leave `release_repo` unset: it defaults to bridle's own repo, never the
project's origin.

On macOS, a downloaded release binary is ad-hoc signed by the build, and the signature is a new
identity for each release, so the application firewall asks "Allow incoming connections?" again
after each self-upgrade; click Allow (or run `just sign-setup` from a clone once and
`bridle sign binary` on the installed file, which gives it the stable local identity; the next
self-upgrade replaces the file and loses it). Nothing else is needed for it to run.

`bridle --version` and `tailscale status` must both work before you go on. The machines reach each
other by the host name in `[machines]` (a Tailscale MagicDNS name); `ping <host>` from each other
machine is the check.

## 1. GitHub and git

On the new machine:

1. **SSH key for GitHub.** `ssh-keygen -t ed25519 -C "newpc"`, add `~/.ssh/id_ed25519.pub` at
   github.com under Settings, SSH keys, and check with `ssh -T git@github.com`. Bridle pushes
   `bridle/state` and the integration branch over this key, so it must work with no passphrase
   prompt in a daemon (a key in `ssh-agent` is fine in a login shell, but a boot-time unit has no
   agent: use a key without a passphrase on a machine you trust, or load it in the unit's
   environment) (unverified on a machine).
2. **`gh auth login`.** Needed where a project sets `[ci] github = true` (the daemon asks `gh` for
   CI results to wake you when CI fails). Choose SSH as the protocol. Check with `gh auth status`.
3. **A distinct git identity per machine** (postmortem
   [[docs/tickets/open/postmortem-git-push-to-origin-rejected-because-two-machines-j7r4|j7r4]],
   recommendation 6), so a log shows which box made a commit:

   ```bash
   git config --global user.name  "Jonathan Branam (newpc)"
   git config --global user.email "<your email>"
   ```

   Any scheme works if it differs per machine. The host in the name is the point.

## 2. `~/.bridle/config.toml`

The machine config is the same on every box except `[machine] name`, plus the machine-local
sections (`[gateway]`, `[mail]`, `[budget]`, `workflow`). It is trusted, with no probing. The
reference is [[docs/design/cli|cli]], "Projects on other machines"; the budget is
[[docs/design/usage-and-budget|usage and budget]].

### On the new machine

```toml
# top level, above the first table: where the shared workflow lives on THIS machine.
# Overrides every project's own workflow path. Required if a project's
# .bridle/config.toml names another machine's path (the NUC has this).
workflow = "/home/<user>/src/bridle/workflow"

[machine]
name = "newpc"

[machines]
dalek = "dalek"          # the host name to reach each machine by (Tailscale MagicDNS)
nuc   = "nuc"
newpc = "newpc"          # list yourself too

[projects]
# the SAME lines as on every other machine; copy them whole.
bridle     = { machine = "dalek", port = 7401 }
meta-notes = { machine = "nuc",   port = 7402 }
# a project that will live here gets the next free port:
# my-project = { machine = "newpc", port = 7407 }

[budget]
max_workers = 2          # a PC that also games: keep it low; see usage-and-budget.md
```

Add `[gateway]` only on the machine that serves the human's web UI (normally dalek). If the new
machine should have it, copy the shape from dalek's file; generate the hash with `bridle gateway
hash-password` and set `bind` to the machine's own Tailscale IP (`tailscale ip -4`).

**Optional, if you want mail** for the projects on this machine: `[mail]` (copied from dalek's,
unchanged), an IAM user and AWS credentials of its own, and dalek's `mail.key`, all before any
bridge runs here. Steps: [[docs/context/mail-on-a-machine|set up mail on a machine]].

### On every other machine (dalek and the NUC)

Add the new machine to `[machines]`, and any project that will live on it to `[projects]`:

```toml
[machines]
newpc = "newpc"

[projects]
# only when a project lives on newpc:
# my-project = { machine = "newpc", port = 7407 }
```

Once this machine's `[machines]` and `[projects]` are right, `bridle projects sync` compares the
`[machines]` and `[projects]` lists with every `[machines]` entry over ssh, and `bridle projects sync --apply` copies
this machine's entries to the others (adds missing, fixes differing, never deletes), and notes any
machine whose `[machine] name` is not a key of its `[machines]`.

Without the `[machines]` line a project on `newpc` can't be reached. A listed project with no
`[machine] name` on the machine that reads it is an error. Check the file loads: `bridle doctor`
warns about unknown sections and keys (`--strict` fails on them).

### Remote PATH: bridle over ssh

Commands that run bridle on another machine over ssh (`bridle token pair`, `bridle project stop|start
--machine`, `bridle project preflight --to`) run it in a non-interactive shell. That shell reads the
system zshenv, not your login or interactive dotfiles, so bridle is found only if its directory is on
the remote's PATH there. Since br-sh8y those commands also look in `~/.local/bin` and `~/.cargo/bin`,
so a release or `just install` into either directory needs nothing more. Put the directory on the
system PATH anyway when bridle lives elsewhere, or when you type `ssh <host> bridle ...` by hand. On
each machine, once:

```bash
# macOS (dalek)
echo 'path=($HOME/.local/bin $HOME/.cargo/bin $path)' | sudo tee -a /etc/zshenv
# Ubuntu and WSL2 (Debian's zsh reads /etc/zsh/zshenv, not /etc/zshenv)
echo 'path=($HOME/.local/bin $HOME/.cargo/bin $path)' | sudo tee -a /etc/zsh/zshenv
```

Check from each other machine: `ssh <host> 'command -v bridle'` prints a path. If it prints nothing,
bridle is not found there, and the commands above say "bridle not found on <host>" instead of
"unreachable".

## 3. Tokens

All token commands are human-only: run them in a plain terminal, not inside Claude Code. A token
is minted on the machine whose daemon will **check** it (the receiver) and pasted on the machine
that **presents** it. Mixing up the direction is the common mistake (ticket
[[docs/tickets/open/peer-token-setup-guidance-a-token-per-receiving-project-per-gdf3|gdf3]]).
A daemon must be running to mint: `bridle serve` for that project first (step 5 for a project
that moves here; for a project already on dalek or the NUC it is already up).

`~/.bridle/credentials.toml` must be mode 0600 (`chmod 600`).

### 3a. Visitor tokens, both ways, by hand

A visitor token lets a principal on one machine use a project's daemon on another (read, send,
query). Once ssh works, `bridle token pair` (br-8c25, sk7p) does this for every role, project and machine in one command and is safe to re-run; run it first, and use the by-hand steps below only to see what it does or to fix one entry. It also mints the peer tokens of step 3b (`peer:<sending machine>` on the receiver, `[peer] <project>` on the sender); a project with `[mail] peers = false` is left out.

**newpc's human reaches a project on dalek.** On dalek, for the project `bridle`:

```bash
bridle token create human --project bridle --machine newpc    # prints once
```

On newpc, paste it into `~/.bridle/credentials.toml`:

```toml
[human.dalek]
bridle = "<the token>"
```

Do the same for each project you want newpc to drive, and for `orchestrator` or `advisor` in place
of `human` if newpc runs those sessions (`[orchestrator.dalek]`, `[advisor.dalek]`). The plain
human falls back to `[human.<machine>]`
([[docs/design/agent-host/principals|principals]], "The credentials file").

**The other way: dalek's human reaches a project on newpc.** On newpc, once a project is served
there:

```bash
bridle token create human --project my-project --machine dalek
```

On dalek:

```toml
[human.newpc]
my-project = "<the token>"
```

Repeat with `nuc` in place of `dalek` for each project the NUC should reach. (The `--machine`
flag, the printed-once behaviour and the `[<principal>.<machine>]` tables are from `bridle token
create --help` and principals.md.)

### 3b. Peer tokens, for mail between daemons

Every message to a principal on another daemon goes from the sender's own daemon to the receiving
one. The receiving daemon needs a **peer token** from each sending machine.

The rule (gdf3): **one peer token per receiving project, per sending machine, minted on the
receiver**, pasted under `[peer]` on the sender, keyed by the receiving project. Projects on the
same machine need one each too (a message from the NUC's `meta-notes` to the NUC's `notes` crosses
daemons).

`--peer` takes the **sending** machine's name, not the receiver's. Wrong, from newpc to a dalek
project: running `--peer dalek` on dalek fails with `conflict: principal peer:dalek already
exists`. Right, so that newpc can send to dalek's `bridle`:

```bash
# on dalek (the receiver):
bridle token create --peer newpc --project bridle           # printed once
```

```toml
# on newpc, credentials.toml (the sender): keyed by the RECEIVING project
[peer]
bridle = "<the token>"
```

And for messages the other way, to `my-project` on newpc from dalek:

```bash
# on newpc (the receiver):
bridle token create --peer dalek --project my-project
```

```toml
# on dalek, credentials.toml
[peer]
my-project = "<the token>"
```

br-gdf3 is fixing the CLI's help and error text to say this; the rule above stays the guide.

A project needs a peer token for every project it sends to, from every machine it sends from. A
machine that runs several projects shares one `[peer]` entry per destination project across its
daemons.

### 3c. Checklist: who mints what, where it goes

For a network of dalek, nuc and newpc, with project `P` on machine `M`:

| Token | Minted on | Command | Pasted on | Where |
|---|---|---|---|---|
| visitor, human of machine `X` into `P` | `M` | `bridle token create human --project P --machine X` | `X` | `[human.M]` `P = ".."` |
| visitor, orchestrator or advisor of `X` into `P` | `M` | `bridle token create orchestrator --project P --machine X` | `X` | `[orchestrator.M]` `P = ".."` |
| peer, mail from machine `X` to `P` | `M` | `bridle token create --peer X --project P` | `X` | `[peer]` `P = ".."` |
| the project's own principals | `M` | `bridle token create orchestrator --project P` | saved on `M` | `[orchestrator]` (saved for you) |
| the mail bridge's `mail` principal | `M` | `bridle token create mail --project P` | saved on `M` | `[mail]` (saved for you; needed by `mail install`) (unverified on a machine) |

Tokens do not move with a project; mint them again on the new owner.

## 4. Services

Bridle's commands here write the file and print the commands to load it; they never run
`systemctl` or `launchctl`. Run what they print.

**Workspace layout.** A project's clone sits in its own workspace folder, and its worktrees go
beside it. The clone's directory name is the project name:

```
~/work/<P>-workspace/<P>        # the clone (on dalek: /Volumes/Data/work/<P>-workspace/<P>)
~/work/<P>-workspace/wt/        # worktrees, made by the daemon
```

**Linux and WSL2 (systemd user units):**

```bash
sudo loginctl enable-linger "$USER"      # so user units start at boot, with no login
bridle systemd install --project P --projects-dir ~/work/P-workspace
# prints the systemctl --user commands; run them, normally:
systemctl --user daemon-reload
systemctl --user enable --now bridle-P.service
```

Always give `--project`; without it one unit is written for every project `[projects]` puts on
this machine. `--projects-dir` is where `<dir>/<project>` is the clone, and `<dir>` is the
workspace. On WSL2 systemd must be on (`/etc/wsl.conf`), which the WSL2 guide sets up.

**macOS (a LaunchAgent), from the clone:**

```bash
bridle launchd install --project P       # writes ~/Library/LaunchAgents/dev.bridle.P.plist
# run the launchctl bootstrap command it prints
```

**The gateway** (only the machine that serves the human's web UI; needs `[gateway]` configured):

```bash
bridle gateway install                   # launchd plist or systemd user unit; run what it prints
```

**The mail bridge** (optional, if you want mail; one per project on each machine that has its
mail set up by [[docs/context/mail-on-a-machine|set up mail on a machine]]; needs `[mail]` and a
`mail` token for the project, step 3c; a moved project gets it from `project start`):

```bash
bridle mail install --project P          # one per project it bridges
```

Both take `--force` to overwrite an existing file. `bridle mail uninstall` and
`bridle launchd uninstall` exist; `bridle systemd uninstall` does not yet (br-xrkh).

**Check with `bridle doctor`** in the project's clone. Green is every line a pass (or a
deliberate warning) and exit status 0. It reads only the project's clone and `~/.bridle/config.toml`,
so a machine with no bridle clone needs nothing more for it (not yet run on a machine installed
this way; if it prints a failure, that is a bug to file):

```bash
cd ~/work/P-workspace/P && bridle doctor        # exit 0; --strict also fails on warnings
bridle --project P status --json                # the daemon answers
systemctl --user status bridle-P.service        # Linux: active (running)
```

If the unit isn't there after a reboot, linger is off (`loginctl show-user "$USER" | grep Linger`
says `Linger=yes` when on). Do not start a daemon over SSH on macOS (ticket nrbf); see
[[docs/context/adding-a-project|adding a project]], "Gotchas today".

## 5. Moving a project onto the new machine

The procedure is "Moving a project to another machine" in
[[docs/context/nuc-host|the NUC host]]: push, stop the old daemon, clone, `bridle project
take-over`, create tokens, start sessions. Adding a brand-new project instead is
[[docs/context/adding-a-project|adding a project]]. Two traps the runbook's steps hide:

1. **Stop the old machine first.** `bridle stop-daemon` is not enough: a launchd plist or
   systemd unit restarts the daemon, and it fights the new owner. On the old machine:

   ```bash
   bridle project stop            # or: bridle --project P project stop --machine old
   ```

   It stops the agents, the daemon and its units (unloaded and disabled, files kept), pushes
   `bridle/state` and the integration branch, confirms origin has both, and pushes the `released`
   mark: that is the giving up of ownership. It also commits the project's scheduled messages.

2. **Take over, then start.** In the new clone:

   ```bash
   bridle project take-over
   bridle project start
   ```

   `take-over` claims the project, confirms origin has the claim, updates `[projects]` on every
   machine and exits; it starts no daemon. `start` installs the unit if it is missing, enables it
   (boot, restart on failure) and starts it, then re-creates the project's tokens on every machine
   (`token pair`'s code) and re-adds the scheduled messages from the old owner. `bridle.db` is not
   copied; events, usage and agent history stay on the old machine. Run `start` again if it
   reports a failure: it keeps what is left to do until it is done.

Only the owner's clone can push the project's integration branch (8z7j): `bridle serve` and
`bridle sync` install a `pre-push` hook that refuses it elsewhere, and `bridle project take-over` moves the
right with the project. Pushes of other branches are not touched.

Also update `[projects]` on **every** machine to the new `machine = "newpc"` line (edit it here,
then `bridle projects sync --apply`), and re-mint
the tokens for that project (steps 3a, 3b): a visitor and a peer token are tied to the daemon
that minted them. Inbox messages do not move.

Automated project transfer between machines is a later feature (workstream "machine setup",
`docs/notes/roadmap.md`). It is not available; everything above is manual.

## 6. The home project

Each machine that runs an orchestrator runs it in that machine's **home project**, `<host>-home`
(`dalek-home`, `vesta-home`, `remus-home`), not in a work project such as bridle. The home holds
the orchestrator, its schedules, waiters, handovers and incidents, and the machine-level services
that only exist where a project lives: the daemon and its self-upgrade, token pair, and mail
(`<host>-home@` is "mail this machine", read by the orchestrator). It has no workers or managers
configured, so the daemon stays light. It is a stop-gap: when the machine daemon (ticket kuw2) is
built, it takes this project's place.

Senders address the orchestrator with `--project <host>-home`. Until a machine has its home
project, its orchestrator is still in bridle, so use `--project bridle` there.

Set the home project up after steps 1 to 5, in this order. It is the order advisor wsl2 ran on
remus, 2026-10-14, and it is not yet a one-command setup. Every machine must be on bridle 0.7.2
or later first (the token pair fix).

1. **Repo:** a git repository with a private remote (GitHub). State push is on by default, so
   tickets about the machine's setup can live in it. Run `bridle init` in the clone.
2. **Enrol it on every machine:** add `<host>-home` to `[projects]` in `~/.bridle/config.toml`
   (the `machine` is this machine), then `bridle projects sync --apply` on any machine. Add it to
   `[mail] projects` on every machine by hand too: `projects sync` does not spread `[mail]` until
   ticket afzu.
3. **Start:** `bridle project start`. On a brand-new project (no `bridle/state` here or on
   origin) it needs no `take-over` first: the daemon it starts creates the branch and records this
   machine as owner (ticket kf7t). It installs the daemon as a service and enables it. Its
   token pair can fail on some visitor tokens (ticket g9qm); that is fine for now, see step 4.
4. **Tokens:** run `bridle token pair --machines dalek,vesta,remus` twice, so every machine has the
   tokens for `<host>-home`.
5. **Mail:** `bridle mail install --project <host>-home`, then enable its unit (`systemctl --user
   enable --now` on Linux; `start` can stop before its mail step).
7. **Check:** `bridle --project <host>-home status --json` answers, and a test mail to
   `<host>-home@` gets a reply.
8. **Restart the orchestrator there:** the human stops the orchestrator session in its old project
   and starts it in the home project with `bridle session orchestrator --project <host>-home`, and
   starts the aide there too.

**Visitor tokens follow the home, and they don't yet.** A visitor token on another daemon (for
example `orchestrator@remus` on dalek) records the daemon it was minted for as its home. After the
orchestrator moves to `<host>-home`, that record still names the old project, and a send to
`external:orchestrator@<host>` loops back to the sender: dalek's daemon forwards it to `bridle`,
which on dalek is dalek itself. Until ticket x8c8 lands, re-pair the visitor tokens (`bridle token
pair`, step 5) or update the home, and check with a test mail that it does not loop.

Order the machines: remus first (data-contracts stops being its home), then vesta, and dalek last,
because dalek's orchestrator is the one everything else addresses today.

## Is it working? Ten checks

Run them from a machine other than the new one where it says so.

1. `ping newpc` and `ssh newpc` work from dalek and the nuc.
2. On newpc: `bridle doctor` exits 0 in a project clone.
3. On newpc: `bridle --project P status --json` shows the daemon; `P` is a project on newpc.
4. From dalek: `bridle --project my-project status --json` answers (visitor token, step 3a).
5. From newpc: `bridle --project bridle status --json` answers (the other direction).
6. From dalek: `bridle send --project my-project advisor "ping from dalek"` prints `delivered`
   (peer token, step 3b). `queued` with an error means the token is missing or the wrong direction.
7. From newpc: `bridle send --project bridle advisor "ping from newpc"` prints `delivered`.
8. The message shows up: `bridle --project my-project inbox --json` on the receiver, as the
   principal it was sent to.
9. Reboot newpc (no login): `systemctl --user is-active bridle-P.service` says `active`
   (linger, step 4).
10. A document read through the gateway: open the gateway's `public_url` from your phone or
    laptop, log in, and open a project's document (a ticket) that lives on newpc (unverified on a
    machine).
