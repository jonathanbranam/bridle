---
id: br-v7ug
title: Run bridle's heavy work on the Windows PC under WSL2
kind: feature
created: 2026-10-07
created_by: external:orchestrator
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-ur9v, br-b7cz, br-npj2]
---

## The ask

The human, 2026-10-07 ~7:20 PM ET, via aide (m-6585), answering whether to plan moving heavy
bridle work off dalek (the Intel Mac) to the Windows PC under WSL2:

> yes plan the work to run on the windows pc - it'll be a while maybe the weekend until I can get
> to that though.

The research and the choice (WSL2 with Ubuntu, not dual boot) are in
[[the-windows-gaming-pc-as-a-bridle-host-without-wiping-window-ur9v|ur9v]], with the human's
answers (Windows Home, BitLocker off, plenty of disk, gaming means bridle stops, a big red / big
green button). Why: Rust builds on dalek take ~10 minutes and macOS scanning adds load
([[build-cost-on-the-laptop-b7cz|b7cz]], research npj2 in flight). The NUC is too small for
bridle's own builds (`docs/context/nuc-host.md`).

## What to plan

Split it into tasks bridle's workforce can do now and to-dos for the human:

- **The human's hands-on steps** (maybe the weekend): each a `--for-human` to-do, never a blocker
  on the queue. Install WSL2 + Ubuntu, `.wslconfig`/`wsl.conf` (systemd, idle timeout, mirrored
  networking), Tailscale, the Task Scheduler start-up task, power and Windows Update settings, and
  note the CPU and RAM (still unknown). A setup guide they can follow step by step is the deliverable
  bridle writes; it should reuse the NUC guide's shape.
- **Bridle work, if any:** what has to change for a WSL2 host (likely little: it's Linux; check
  `bridle systemd install` under WSL's systemd, discovery across machines, and what "heavy work"
  means: which projects or which agents move, and how dalek's sessions reach that daemon).
  Decide whether bridle's own repo moves or only builds do (e.g. remote builds); recommend.
- **The button:** two desktop shortcuts (`wsl --shutdown`, and the start-up task) as the first form.
- **A name** for the PC (`docs/context/naming.md`).

Out of scope until the human has the PC set up: anything that needs the PC to exist.

## Audit

Task br-4yc8, 2026-10-07. From the code and docs only; the PC does not exist yet, so anything only
the real machine can confirm is marked **verify on the PC**. WSL2 with systemd reports
`std::env::consts::OS == "linux"`, so every `cfg!(target_os)` check treats it as plain Linux.

### 1. What assumes macOS or a non-WSL Linux

| Item | Verdict | Detail (backing) |
|---|---|---|
| `bridle systemd install` | works as is; verify on the PC | `check_os` accepts "linux"; it writes user units and prints `systemctl --user` and `sudo loginctl enable-linger` (`crates/bridle/src/systemd.rs:56,195`). Needs `systemd=true` in `/etc/wsl.conf`. Verify: user manager starts at WSL boot with linger on, and `After=network-online.target` in a *user* unit (line 141) is a no-op for the user manager (same on the NUC). |
| Keeping WSL alive | needs a doc, no code | Linger only helps while the WSL VM runs. It stops after the idle timeout when nothing holds it open. The Task Scheduler start-up task must run a held-open command (e.g. `wsl -d Ubuntu -- sleep infinity`) and `.wslconfig` needs `vmIdleTimeout` raised. Setup guide + the red/green button, not bridle. Verify on the PC. |
| Listening / Tailscale address | works if Tailscale runs *inside* WSL; one real gap | `serve` listens on loopback + `tailscale ip -4` (`crates/bridle-daemon/src/lib.rs:343,480`). That shells out to the `tailscale` CLI, so install Tailscale in the Ubuntu distro (needs systemd; the WSL2 kernel has `/dev/net/tun`), not only on Windows. Windows-side Tailscale plus mirrored networking would not give the CLI an address. **Gap:** the address is read once at start; if the daemon starts before Tailscale is up (boot, `wsl --shutdown` restart) it silently binds loopback only (`lib.rs:500`, an info log) and stays that way. See follow-up 1. |
| Cross-machine discovery | works as is | `[machine]`, `[machines]`, `[projects]` and `[principal.<machine>]` / `[human.<machine>]` tokens are pure config + HTTP (`docs/design/cli.md:484-491`, `docs/design/agent-host/principals.md:87`). Machine names are free text; the PC just needs a name. dalek reaches `http://<tailscale-name>:<port>`. br-8b98 (integrated) is what lets the plain human CLI on dalek use `[human.<pc>]`. Verify on the PC: MagicDNS name resolves from dalek. |
| Self-upgrade build | works as is; slow | `cargo install` into a persistent `upgrade-target` (`crates/bridle-daemon/src/upgrade.rs:250`), sign step runs only when configured and `signing::sign` is a no-op off macOS (`signing.rs:52`). Needs Rust toolchain, `build-essential`, `pkg-config`, git on the PC (the bundled sqlite and `aws-lc-sys` need a C compiler and cmake; verify the exact apt list). Speed depends on the PC's CPU/RAM (unknown). Build in the ext4 home, never `/mnt/c`. |
| `bridle sign`, `bridle daemon launchd`, `bridle gateway install` launchd half | not applicable | macOS only (`sign.rs`, `launchd.rs:15`, `signing.rs:52,89`). Nothing to do. |
| Warm `target/` for new worktrees (b7cz) | needs a change | `warm_target` is a macOS-only APFS `cp -cR` clone and a no-op elsewhere (`crates/bridle-daemon/src/worktree.rs:152`). On WSL2 every worker worktree builds cold, which is the very cost this move is meant to cut. WSL2's ext4 has no reflinks, so a plain `cp -a` (disk + copy time) or a shared `CARGO_TARGET_DIR`/sccache is the choice; measure on the PC. See follow-up 2. |
| Process containment | works as is | `ps` snapshots + POSIX `killpg` (`containment.rs:23,227`); the test already notes Linux zombie behaviour (`:379`). procps ships in Ubuntu. Spike 2mj9 (Linux cgroups) is not needed. |
| Load / capacity guard | works as is | `load.rs:27` reads `/proc/loadavg` first. Caveat: inside WSL2 it sees only the VM, not Windows' own load (a game running); gaming means bridle stops anyway (the button). The VM's RAM/core cap comes from `.wslconfig`; set `[machine] load_per_core` to match. |
| tmux sessions and panes | works as is | `tmux` shelled out in `pane.rs`, `session.rs:257,670`, `focus.rs:163`; all best effort. `apt install tmux`. |
| `/Volumes` or `/Users` paths | none found | `grep` of `crates/**/*.rs` finds no hard-coded `/Volumes`. Paths come from `$HOME`/`$BRIDLE_HOME`. Keep the workspace under `/home/<user>`, not `/mnt/c` (slow, no unix perms, git/file-mode trouble). |
| Claude Code login | works; message is macOS-flavoured | On Linux `claude` keeps its login in `~/.claude`, so the keychain-over-SSH problem does not apply. `doctor`'s fix text (`doctor.rs:423`) only mentions macOS. Verify on the PC: `claude auth login` works headless (device/URL flow). A separate login per machine shares the one account budget; the guard is per daemon (xypj). |
| `bridle machine tools-only-install` on dalek | works as is | For dalek's leftover bridle clone after the move (`docs/context/nuc-host.md`, "Tools-only clones"). |

### 2. Which work moves: recommendation

Options: (a) builds only (remote builds), (b) whole projects, (c) bridle's own repo too.

- (a) is **rejected**: bridle has no remote-build mechanism. A worker's `cargo` runs wherever its
  worktree is, and the self-upgrade builds where the daemon runs. Remote builds would be a new
  feature (rsync/ssh to a build box, sync `target/` back) that the project-move procedure makes
  unnecessary.
- (b)+(c) are the same mechanism: **recommend moving whole projects, bridle first** (bridle is the
  heavy one: ~10 min cold `just check`, release build 5 to 11 min, npj2). Use the existing "Moving
  a project to another machine" steps in `docs/context/nuc-host.md` (push, `stop-daemon`,
  clone + `serve --take-over`, new tokens, sessions). It needs no new code; the daemon, its
  agents' worktrees and their builds then all run on the PC.
  Do the smaller projects (track-web, bridle-ui) later if the PC copes; they are not the build pain.
- What stays on dalek: the human's own dalek sessions and the dashboard, driving the PC's daemon
  with `[machine] name = "dalek"`, `[machines] <pc> = "<tailscale-name>"`,
  `[projects] bridle = { machine = "<pc>", port = N }`, and `[human.<pc>]` / `[orchestrator.<pc>]`
  in `credentials.toml` (br-8b98 covers the plain human; `bridle project resolve` per cli.md:31).
  The orchestrator and advisor are Claude sessions that read the repo, so they either run on the PC
  (ssh + tmux; `bridle session orchestrator --project bridle` there) or on dalek against a
  stale tools-only clone, which is wrong for work on the moved code. Recommend they run on the PC,
  and dalek only attaches (ssh/tmux, or Remote Control, spike bagg). Verify on the PC.
- Cost of the move: cross-machine message sync is not built (hw6c), so the inbox does not move;
  drain it first. Gaming stops the daemon (the button), so a project on the PC is unavailable
  while the PC is used; only move projects that tolerate that.
- npj2 (findings part 1) changes the size, not the choice: dropping `bridle-mail`'s AWS stack from
  a default build and a lighter dev profile cut cold CPU ~40 to 50%, and help every host. Do them
  anyway; the PC just raises the ceiling. The macOS scan costs in npj2 (`syspolicyd`, Spotlight)
  vanish on WSL, which is a real part of the gain; the PC's CPU/RAM are still unknown, so
  **verify on the PC**: time `cargo build --release` and `just check` there before moving the
  second project.

### 3. Follow-up bridle tasks (for the PM to file)

1. Re-check Tailscale after start: if `serve` is configured with a port but found no Tailscale
   address, retry for a bounded time (or rebind when `tailscale ip -4` appears) so a boot-time race
   does not leave the daemon loopback-only. `crates/bridle-daemon/src/lib.rs` (+ docs/design daemon.md). Size S.
2. Warm worktree `target/` on Linux: a `cp -a`/`--reflink=auto` variant of `warm_target` behind the
   same best-effort guard, or a documented sccache/shared target choice; measure on the PC first.
   `crates/bridle-daemon/src/worktree.rs`, `warm_build.rs`. Size S, after the PC exists.
3. `bridle doctor`: Linux/WSL checks and message: `claude auth` fix text per OS (`doctor.rs:423`),
   warn when the workspace is under `/mnt/`, warn if `systemd` is not PID 1 or linger is off.
   `crates/bridle/src/doctor.rs`. Size S.
4. Docs: a WSL2 section (or new `docs/context/wsl2-host.md`) with the systemd/wsl.conf notes above;
   the setup-guide task covers the human steps. Size S, docs only.
5. npj2 follow-ups (feature-gate or replace the AWS SDKs in `bridle-mail`; dev profile deps
   `opt-level=0`) are filed under npj2, not here.

## Setup log

From the human's session with advisor wsl2, 2026-10-10 (times US Eastern).

- WSL was not installed. The human ran `wsl --install -d Ubuntu` (default LTS; fine, per the
  human) and rebooted. Linux user name and password still to choose.
- Naming the PC (Windows name and the WSL hostname) is deferred until before Tailscale (step 3);
  the human has no name yet.
- v0.6.0 has a Linux x86_64 release asset (orchestrator, m-9495), so step 6 can download bridle
  instead of building it. Rust is still needed for the builds this PC is for.
- The human, on the next step:

  > I want my next step to be authenticating and updating to 1Password. That will make all of
  > this a lot easier.

  So 1Password (the Windows app, with its CLI and SSH agent wired into WSL) comes before the
  guide's step 2. The guide (`docs/context/windows-wsl2-host.md`) does not have this step yet.
- 1Password is installed and signed in on Windows. The human skipped its WSL wiring (SSH agent,
  CLI integration) for now: "mostly i want 1pasword from windows; skip this for now".
- The PC: Intel Core i5-13400F (10 cores, 16 logical processors), 16 GB DDR4 (2 of 2 slots used;
  the human can upgrade to 32 GB), 1.8 TB NVMe SSD with 1.5 TB free. Windows user `micro`, PC name
  `EzekielsPC` (to be renamed). Linux user `jbranam`.
- `wsl --install -d Ubuntu` only installed WSL on the first run; the distro downloaded on a second
  run after the reboot. The guide should say so.
- `.wslconfig` written and checked: `memory=12GB`, `processors=14`, `swap=8GB`,
  `vmIdleTimeout=-1`, `networkingMode=mirrored`. After `wsl --shutdown`: systemd is PID 1 with no
  `/etc/wsl.conf` edit (the Ubuntu image turns it on), 11 GiB and 14 CPUs seen. With 32 GB,
  `memory=24GB`.
- Notepad is not installed on this PC; writing the file from PowerShell (`Set-Content`) worked.
- Paused before Tailscale while the human picks the PC's name. Next:
  `curl -fsSL https://tailscale.com/install.sh | sh && sudo tailscale up --ssh` in Ubuntu
  (Tailscale SSH, so no openssh-server or keys needed to reach it from dalek).
- Orchestrator (m-9537): br-fv86 (install from a release) landed, but its machine-file
  `self_upgrade = "release"` only takes effect once br-36dp ships. Advisor's call: set the PC up
  from v0.6.0 anyway, then re-check self-upgrade (and restart the daemon) after br-36dp is in a
  release.
- Orchestrator (m-9884): v0.6.1 is released and carries br-36dp and br-fv86. Install the PC from
  v0.6.1 (`scripts/install-release.sh v0.6.1`), not v0.6.0, so `self_upgrade = "release"` in the
  PC's `~/.bridle/config.toml` takes effect. Bridle is not installed on the PC yet.
- Names decided (the human, 2026-10-11, "Romulus and Remus it is"): Windows `romulus`, WSL2
  `remus` (different names for host and guest, by the human's choice; see
  `docs/context/name-ideas.md`). Tailscale runs inside WSL2, so `remus` is the name bridle and
  dalek use. Next: set both names (`/etc/wsl.conf` `[network] hostname = remus`, then
  `Rename-Computer romulus` and one Windows restart), then Tailscale.
- Rename done and rebooted (2026-10-11): `[network] hostname = remus` appended to `/etc/wsl.conf`,
  `Rename-Computer -NewName romulus`, `Restart-Computer`.
- Editor for the PC (the human: "add to doc"): Sublime Text 4 (`winget install
  SublimeHQ.SublimeText.4`) for opening files quickly, WSL files included via
  `\\wsl$\Ubuntu\home\jbranam\...`; Obsidian for markdown; optionally VS Code with its WSL
  extension (`code .` from Ubuntu). The guide should say so beside the Notepad note. Not installed
  yet ("will do later").
- Tailscale in Ubuntu: `curl -fsSL https://tailscale.com/install.sh | sh && sudo tailscale up
  --ssh --hostname=remus`; remus is 100.78.246.109. `ssh jbranam@remus` from dalek works after a
  one-time Tailscale SSH check (browser approval); it saw 14 CPUs and 11 GB.
- The human wants Tailscale on Windows (`romulus`) too, later. Untested risk: with mirrored
  networking, Windows and WSL share one network stack, so two Tailscale clients share it; no source
  found either way. After installing it, check from dalek that both remus and romulus answer;
  if remus breaks, disconnect Windows' Tailscale or leave mirrored mode. Next: guide step 4.
- Step 4a done, from an admin PowerShell instead of the Task Scheduler window: task `bridle WSL
  start` (wsl.exe `-d Ubuntu --exec sleep infinity`, at startup, runs as Windows user `micro`
  with password, highest privileges) with `-ExecutionTimeLimit (New-TimeSpan)`. The GUI's default
  72-hour limit would kill the task, and WSL with it, after three days; the guide should say so.
  Checked over SSH: Ready, ExecutionTimeLimit PT0S. `loginctl enable-linger jbranam`: Linger=yes.
  Not yet checked: that WSL really comes up after a reboot with no one logged in.
- Green and red buttons (4b/4c): skipped for now ("we can do the buttons later").
- Step 5a: `powercfg /change standby-timeout-ac 0`, `hibernate-timeout-ac 0`, `/h off`; `powercfg
  /a` shows hibernation off. Next: Windows Update active hours (the only step left that needs the
  Windows screen), then the rest over `ssh jbranam@remus` from dalek.
- Windows Update active hours: deferred by the human ("later"). Step 5c needs nothing: Ubuntu
  26.04.1 LTS, unattended-upgrades installed, `Automatic-Reboot` left commented out (default
  false). SSH from dalek is Tailscale SSH (browser check about every 12 hours; `"action":
  "accept"` in the tailnet's ssh rule would drop it). Next: step 6 (toolchain, Claude Code, bridle
  v0.6.1) over SSH from dalek.
- Step 6a done (checked over SSH): rustc 1.99.0, just 1.58.0, cargo-nextest 0.9.148, tmux 3.6,
  gcc 15.2. Step 6b (Claude Code) given. Work folder: `~/work` (Ubuntu's ext4 disk, a vhdx on C:,
  1 TB cap, 952 GB free); keep projects off `/mnt/c`.
- Missing from the guide: dotfiles. The human needs thoughtbot dotfiles + dotfiles-local on every
  host (both public, so clone over HTTPS): `apt install rcm zsh`, clone both, `git submodule
  update --init`, a new `host-remus/rcrc` (dalek's with `HOSTNAME="remus"`), `RCRC=... rcup -B
  remus`, `chsh` to zsh. Then dotfiles-local's auto-update as a systemd user timer (its README's
  Linux section; linger is on). GitHub push auth: `gh auth login` + `gh auth setup-git`.
- Dotfiles on remus: thoughtbot and dotfiles-local cloned to `~/work/dotfiles/thoughtbot-df` and
  `~/work/dotfiles/local-df` (the human's choice), `host-remus/rcrc` with full-path
  `DOTFILES_DIRS` and `EXCLUDES` (rcm resolves a relative `dotfiles:` prefix from `$PWD`), zsh,
  the `dotfiles-update` timer (with the clone path as its argument). The timer skips until
  `host-remus/` is committed. GitHub: a fine-grained PAT per machine (Contents rw, Actions r,
  Workflows rw as needed) through `gh auth login --with-token` + `gh auth setup-git`; done.
- Claude Code 2.1.296 installed. Bridle installed from the release script: got v0.6.2 (released
  minutes before), in `~/.local/bin`, with `[daemon] self_upgrade = "release"`. Also needs
  `release_repo = "jonathanbranam/bridle"` under `[daemon]` (aide, br-r2nr thread: without it
  the release repo and workflow fetch fall back to each project's origin). Next: add-a-machine
  steps 1 (git identity "Jonathan Branam (remus)") and 2 (config), then tokens.
- SSH out of remus (2026-10-12): dalek and vesta run plain OpenSSH with keys only (vesta has
  Tailscale SSH off), so remus got its own key (`ssh-keygen -t ed25519 -C jbranam@remus`, no
  passphrase so daemons can use it), added to `~/.ssh/authorized_keys` on dalek and vesta. Works.
  Tailscale SSH on vesta was not used: its browser check would break dalek's unattended SSH.
  Interim, while ticket r2nr is part done: remus's `[machines]` says `nuc = "vesta"`.
- 2026-10-13 (late, ET): remus upgraded to bridle 0.7.1 from the release script. The human's way
  with installers (write future steps like this): `curl -O` into `~/installs/`, read it, then run
  it from there. remus `~/.bridle/config.toml`: `[machine] name = "remus"`; the interim
  `nuc = "vesta"` is gone.
- `ssh <host> bridle` works on all three machines: the system zshenv (macOS `/etc/zshenv`, Ubuntu
  `/etc/zsh/zshenv`) has `path=($HOME/.local/bin $HOME/.cargo/bin $path)`. Ticket 5cru (bridle
  itself shouldn't need this) has a design waiting for the human's review.
- vesta -> dalek SSH was failing: vesta's key was missing from dalek's `authorized_keys`, probably
  overwritten when remus's key was added on Oct 12. The human fixed it.
- `bridle projects sync` from dalek, vesta and remus: all "same" (dalek, vesta, remus in
  `[machines]`; the projects are on dalek and vesta).
- `bridle token pair --machines dalek,vesta,remus` (plain run): role tokens all kept (dalek 36,
  remus 40, vesta 36); peer tokens kept 0, failed 16 (`conflict: peer:<machine> already exists`).
  Ticket xqwy: the receiver now replaces on conflict (7fc14218, on main, in 0.7.2). "Kept 0" was
  a wrong check (it matched the token name where the name is the bare machine; the held entries
  were probably fine), fixed by ticket 378y (a163726f, in 0.7.2). Next: with 0.7.2 on remus, rerun pair and expect failed 0 and kept peer entries on a second run.
- Next: move a project to remus (`docs/context/move-a-project.md`, `bridle project preflight --to
  remus` first). remus has no `bridle systemd install` and no daemon running yet.
