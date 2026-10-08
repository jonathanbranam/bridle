# The Windows PC as a bridle host (WSL2 + Ubuntu)

*Written 2026-10-07 for the human, to follow step by step. The plan and the choice (WSL2, not dual
boot) are in [[docs/tickets/open/run-bridle-s-heavy-work-on-the-windows-pc-under-wsl2-v7ug|the WSL2 host ticket (v7ug)]]
and [[docs/tickets/open/the-windows-gaming-pc-as-a-bridle-host-without-wiping-window-ur9v|the research (ur9v)]].
The shape follows [[docs/context/nuc-host|the NUC host]].*

> **Status (2026-10-07):** Planned. Nothing in this guide has been run on the PC yet. Steps marked
> **unverified** are from general knowledge of WSL, Task Scheduler and bridle's install path; check
> them on the machine and fix this doc when you do.

## What we know and assume

- **Known (the human, 2026-10-03):** Windows Home (so no Hyper-V, and WSL2 is the route), BitLocker
  off, plenty of free disk, and a PC that sits idle about 90% of the year. Gaming on it means bridle
  stops; the big red button (step 4) is how that happens.
- **Unknown:** CPU and RAM. Fill these in before step 2:
  - CPU cores (Task Manager, Performance, CPU): `<fill in>`
  - RAM in GB (Task Manager, Performance, Memory): `<fill in>`
  - Free disk on C: (File Explorer, This PC): `<fill in>`
- **Assumed:** the PC is on wired Ethernet, and the Windows 11 build is recent enough for
  `networkingMode=mirrored` (Windows 11 22H2 or later). Check with `winver`. Unverified.
- **Assumed:** the name of the PC is still open. Naming it is a to-do for the human, see
  [[docs/context/naming|naming]]; this guide writes it as `<pc-name>`.

Bridle does not run on native Windows (see the research). The PC needs Linux, and WSL2 is that Linux.

## The steps

Do them in order. Each step ends with a check. If a check fails, stop and note what you saw before
going on.

### 1. Install WSL2 and Ubuntu

In an **administrator** PowerShell:

```powershell
wsl --install -d Ubuntu
```

Reboot when asked. Ubuntu opens once after the reboot and asks for a Linux user name and password.
Use the same name as on the NUC and dalek if you can, so `ssh` and paths look alike.

**Check:**

```powershell
wsl --status
wsl -l -v
```

`wsl -l -v` should show `Ubuntu` with `VERSION 2` and `RUNNING`. Then in the Ubuntu shell:

```bash
lsb_release -a     # Ubuntu LTS release; note it here: <fill in>
```

### 2. Configure WSL: `.wslconfig` and `/etc/wsl.conf`

Two files, two places.

**a. Windows side: `%UserProfile%\.wslconfig`** (for example `C:\Users\<you>\.wslconfig`), in
Notepad. This sets the VM's limits. Leave room for games: keep memory and processors well under the
PC's totals (the numbers below are placeholders until CPU and RAM are known).

```ini
[wsl2]
# Keep the VM running while idle. -1 = never time out (WSL 2.0 and later). Unverified on this build.
vmIdleTimeout=-1
# Share the PC's network: WSL gets the PC's address (Windows 11 22H2 and later). Unverified.
networkingMode=mirrored
# Fill in from step 2's notes. Example for a 32 GB PC: memory=16GB, processors=6.
memory=<fill in, e.g. 16GB>
processors=<fill in, e.g. 6>
```

**b. Linux side: `/etc/wsl.conf`** (inside Ubuntu, `sudo nano /etc/wsl.conf`):

```ini
[boot]
systemd=true
```

Then restart WSL so both files take effect. From PowerShell:

```powershell
wsl --shutdown
```

**Check:**

```bash
# in Ubuntu, after reopening it
ps -p 1 -o comm=                 # prints: systemd
systemctl is-system-running      # prints: running (or degraded; note which)
```

From Windows, `wsl --shutdown` followed by reopening Ubuntu should not change the Linux user or files.
Unverified: `networkingMode=mirrored` may need `wsl --shutdown` twice on some builds.

### 3. Tailscale: inside WSL, not on Windows

Install Tailscale **inside Ubuntu**, as the NUC does. Then the Linux host has its own Tailscale
name and address, dalek reaches it the same way it reaches the NUC, and `ufw` on Linux decides what
comes in. Why not Windows: the Windows client would be a second thing to keep updated and would
not show up as a Linux host to bridle's discovery.

In Ubuntu:

```bash
curl -fsSL https://tailscale.com/install.sh | sh     # unverified: check the current install page
sudo tailscale up                                    # opens a login URL; sign in as the human
```

Turn on MagicDNS in the Tailscale admin console (it is on for the NUC). Name the machine
`<pc-name>` (the to-do from the Assumptions section).

**Check:**

```bash
tailscale status                 # <pc-name> listed, online
tailscale ip -4                  # note the 100.x address: <fill in>
```

From dalek: `ping <pc-name>` and `ssh <pc-name>` both work (SSH keys only, as on the NUC).
Unverified: SSH server in Ubuntu (`sudo apt install openssh-server`, then `sudo systemctl enable --now ssh`)
and `ufw` rules matching the NUC's (deny inbound except SSH on `tailscale0`).

### 4. Start at boot, and the big green and red buttons

WSL starts when someone opens Ubuntu, and systemd only runs once WSL is up. So a Task Scheduler task
starts it at boot.

**a. The start-up task.** In Task Scheduler (Windows Home has it), Create Task, not Basic Task:

- General: name `bridle WSL start`; "Run whether user is logged on or not"; "Run with highest
  privileges". Unverified on Home: Windows may ask for the account password here.
- Triggers: New, "At startup".
- Actions: New, Program `C:\Windows\System32\wsl.exe`, arguments `-d Ubuntu --exec sleep infinity`.
  This keeps the distro up. Unverified: a long-running `--exec` may be needed because
  `vmIdleTimeout` alone is not always enough.
- Conditions: untick "Start only if on AC power" (it matters only on a laptop; unverified that
  this PC is a desktop).

Also in Ubuntu, so user services (the bridle daemons) start without a login:

```bash
sudo loginctl enable-linger "$USER"
```

**b. Green button** (start), on the desktop: right-click, New, Shortcut. Target:

```
C:\Windows\System32\schtasks.exe /run /tn "bridle WSL start"
```

Name it "bridle green: start".

**c. Red button** (stop bridle, free the memory), on the desktop: New, Shortcut. Target:

```
C:\Windows\System32\wsl.exe --shutdown
```

Name it "bridle red: stop". This stops every WSL distro at once, bridle included. Bridle is
resilient to a stop (the human, 2026-10-03), but unverified: check the daemon's state after a
red-then-green cycle with `bridle status --json` and note what needed fixing.

**Check:**

1. Press green. Within a minute, `wsl -l -v` shows Ubuntu `RUNNING`.
2. Restart the PC. Without logging in, `wsl -l -v` (from another machine via `ssh <pc-name>`
   and `powershell.exe -c "wsl -l -v"`) shows Ubuntu `RUNNING`. Unverified.
3. Press red. `wsl -l -v` shows Ubuntu `Stopped`.

### 5. Power and Windows Update

The PC must not sleep while bridle runs, and must not reboot in the middle of a run (the NUC's
04:00 reboot is the lesson, see [[docs/context/nuc-host|the NUC host]]).

**a. Power** (Settings, System, Power and battery):

- Sleep: "Never" on power (plugged in).
- Screen: as you like (the screen can turn off; the PC must not sleep).
- Hibernate: off (Control Panel, Power Options, or `powercfg /h off` in an admin shell; unverified
  on Home).

**b. Windows Update** (Settings, Windows Update):

- Active hours: set to the hours the PC is used (Settings, Windows Update, Update settings, Active
  hours). Windows will not restart in those hours. Choose the human's hours: `<fill in>`.
- Restart options: "Get the latest updates as soon as they're available" off, and "Restart this
  device as soon as possible when... " off, if the switch is shown. Unverified: the wording
  changes by build.

**c. Ubuntu updates** (inside WSL): stop unattended reboots, as on the NUC.

```bash
grep -n Automatic-Reboot /etc/apt/apt.conf.d/50unattended-upgrades
```

If it is not `"false"`, edit the file so `Unattended-Upgrade::Automatic-Reboot "false";`, then
reboot the WSL side with `wsl --shutdown`, as in step 2. Unverified: `unattended-upgrades` is
installed by default on Ubuntu; check with `dpkg -s unattended-upgrades`.

**Check:** `powercfg /a` (Windows) lists no sleep state you need, and the Update settings show the
active hours you set.

### 6. Install the toolchain and bridle

Inside Ubuntu:

```bash
sudo apt update
sudo apt install -y git tmux build-essential curl pkg-config libssl-dev
```

Rust, via the official installer (choose the default):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
. "$HOME/.cargo/env"
```

Task runner and test runner, as in the repo's `CLAUDE.md`:

```bash
cargo install just cargo-nextest --locked
```

Claude Code, via the native installer (the same as the NUC). Unverified: check the current install
command on Claude Code's own docs before running it; the one here is a placeholder:

```bash
curl -fsSL https://claude.ai/install.sh | bash     # unverified
claude --version
```

Then clone bridle and install it (the clone is the project's home on this host, as on the NUC):

```bash
git clone <bridle-repo-url> ~/src/bridle
cd ~/src/bridle
just install
bridle --version
```

`just install` runs `cargo install --path crates/bridle --locked` and, on a Mac, re-signs the binary
(the signing step is a no-op on Linux). Unverified: `just install` on Linux.

Then the daemon's units (this step is **to be verified under WSL's systemd**; the audit task
br-4yc8 will report whether `bridle systemd install` works here):

```bash
bridle systemd install           # writes the daemons' user units and prints the commands it does not run
```

**Check:**

```bash
systemctl --user list-unit-files | grep -i bridle    # to be verified under WSL's systemd (br-4yc8)
bridle status --json                                 # the daemon is reachable or shows not running
cargo nextest --version && just --version && tmux -V && claude --version
```

Also run `just check` in a clone of a bridle project, once, to confirm the toolchain builds the
project. Unverified: first builds on this host may take a while (build-time research is task npj2).

## How the human uses it

As the NUC: the daemons run on the PC; the orchestrator and advisor run from dalek or the PC, and
reach the PC's daemon over Tailscale (see [[docs/context/nuc-host|the NUC host]], "How it's reached").
Start and stop with the buttons from step 4, not by hand in a shell. Gaming means red, then green
when the game is done.

To move a project here, follow the NUC's steps in
[[docs/context/nuc-host|the NUC host]], "Moving a project to another machine", and
[[docs/context/adding-a-project|adding a project]]. Tokens do not move with the project.

## Not in this guide

- Naming the PC: a to-do for the human ([[docs/context/naming|naming]]).
- Bridle-side checks of WSL (systemd units, discovery across machines, budget hold on a game): the
  audit task br-4yc8.
- Dual boot: the fallback only if WSL proves unreliable as an always-on host (the research).
