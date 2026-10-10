---
id: v7ug
title: Run bridle's heavy work on the Windows PC under WSL2
kind: feature
opened: 2026-10-07
filed_by: external:orchestrator
repos: [bridle]
changes: []
specs: []
needs: []
see: [ur9v, b7cz, npj2]
tasks: [br-v7ug]
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
