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
  SSH, `gh`, Node, Claude Code. Build and install bridle with `just install` in a clone.
- **macOS:** build with `just install`, and run `just sign-setup` once (see `CLAUDE.md`, "Build
  configuration").

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

Add `[gateway]` and `[mail]` only on the machine that runs the human's web UI and email bridge.
Those are normally dalek's, not every box's. If the new machine should have them, copy the shapes
from dalek's file; generate the hash with `bridle gateway hash-password` and set `bind` to the
machine's own Tailscale IP (`tailscale ip -4`). `[mail]` needs `bucket`, `region`, `allow`,
`projects`, `to` and AWS credentials from the standard chain (design:
[[docs/design/mail|mail]]).

### On every other machine (dalek and the NUC)

Add the new machine to `[machines]`, and any project that will live on it to `[projects]`:

```toml
[machines]
newpc = "newpc"

[projects]
# only when a project lives on newpc:
# my-project = { machine = "newpc", port = 7407 }
```

Without the `[machines]` line a project on `newpc` can't be reached. A listed project with no
`[machine] name` on the machine that reads it is an error. Check the file loads: `bridle doctor`
warns about unknown sections and keys (`--strict` fails on them).

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
query). Once ssh works, `bridle token pair` (br-8c25, sk7p) does this for every role, project and machine in one command and is safe to re-run; run it first, and use the by-hand steps below only to see what it does or to fix one entry. Peer tokens (3b) are still by hand until br-jw9e.

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

**The mail bridge** (only the machine that runs the email bridge; needs `[mail]` and a `mail`
token for the project, step 3c):

```bash
bridle mail install --project P          # one per project it bridges
```

Both take `--force` to overwrite an existing file. `bridle mail uninstall` and
`bridle launchd uninstall` exist; `bridle systemd uninstall` does not yet (br-xrkh).

**Check with `bridle doctor`** in the project's clone. Green is every line a pass (or a
deliberate warning) and exit status 0:

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
[[docs/context/nuc-host|the NUC host]]: push, stop the old daemon, clone, `bridle serve
--take-over`, create tokens, start sessions. Adding a brand-new project instead is
[[docs/context/adding-a-project|adding a project]]. Two traps the runbook's steps hide:

1. **Remove the old machine's supervisor first.** If the old machine has a launchd plist or
   systemd unit for the project, `bridle stop-daemon` is not enough: the supervisor restarts the
   daemon, and it fights the new owner. On macOS, `bridle launchd uninstall --project P`, then the
   `launchctl bootout` command it prints. On Linux, `bridle systemd uninstall` is not built yet
   (br-xrkh); until then:

   ```bash
   systemctl --user disable --now bridle-P.service
   rm ~/.config/systemd/user/bridle-P.service && systemctl --user daemon-reload
   ```

2. **Take over by hand before you enable the new unit.** A systemd unit or LaunchAgent cannot
   take over: it runs plain `bridle serve`, which refuses while `bridle/state` on origin names the
   old host. In the new clone:

   ```bash
   bridle serve --take-over --detach
   ```

   Once it is up and the claim is pushed, you may stop it with `bridle stop-daemon` and enable
   the new unit (step 4), which then starts as the owner.

Also update `[projects]` on **every** machine to the new `machine = "newpc"` line, and re-mint
the tokens for that project (steps 3a, 3b): a visitor and a peer token are tied to the daemon
that minted them. Inbox messages do not move.

Automated project transfer between machines is a later feature (workstream "machine setup",
`docs/notes/roadmap.md`). It is not available; everything above is manual.

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
