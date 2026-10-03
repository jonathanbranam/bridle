# Adding a project, on dalek or the NUC

*Written 2026-09-30 for the human. How the machines are set up: [[docs/context/nuc-host|nuc host]].
The machine config: [[docs/design/cli|cli]] ("Projects on other machines", k7mw). Tokens:
[[docs/design/agent-host/principals|principals]].*

> **Status (checked 2026-10-03):** Built and in use: every step below (init, serve, tokens,
> sessions, doctor) · Built, not wired in: the `packs` line `init` writes and the project's
> `.bridle/rules/` reach spawned agents only from 9561950 on, once the daemon runs a binary from
> that commit; `bridle sync` renders files, but nothing runs it for you and its skills are
> gitignored ([[the-workflow-doesn-t-reach-agents-resolved-rules-hooks-and-o-34bw|34bw]]) ·
> Planned: `bridle project add` (below).

Short answer: yes, you can add a project straight on the NUC. No single command does it all.
`bridle init` only writes two files. The rest is a few commands of yours, then an orchestrator
session that prepares the project. Your flow (make a workspace folder, clone the repo, ask the
orchestrator to prepare it) is right. You also have to do the `~/.bridle` config and the tokens,
because the orchestrator can't edit `~/.bridle` files under auto mode.

## Which machine

- **dalek** (Intel MacBook, 16 threads): anything with heavy builds (Rust, big TS monorepos), or
  anything you want next to you. Workspaces go at `/Volumes/Data/work/<project>-workspace/<project>`.
- **nuc** (Intel NUC, Linux, 4 threads): light projects (scripts, dotfiles, Python, Vim) that
  can run while the laptop sleeps. meta-notes runs there, in `/srv/shared/work/meta-notes-work`.
  A cold `cargo install` of bridle takes ~11 minutes there, so keep heavy Rust builds off it.

A project lives on one machine. Moving it later is a known procedure: see "Moving a project to
another machine" in [[docs/context/nuc-host|nuc host]].

## What `bridle init` does and doesn't do

Run inside a clone. It:

- writes `.bridle/config.toml`: the integration branch (defaults to the branch you're on),
  `packs = ["<stack>"]` with `--stack python|typescript` (0.4.0 binaries also accept `rust`, a pack
  that doesn't exist; dropped in f55537d), and a `check` command guessed
  from a justfile, `Cargo.toml`, `package.json` or `pyproject.toml`. It never overwrites.
- appends bridle's runtime files to `.gitignore`.
- vendors the base workflow into `.bridle/workflow/`, but only if nothing names a workflow
  already. On the NUC, the machine-level `workflow` path does, so nothing is vendored there.

It doesn't touch `~/.bridle`, create a branch, pick a port, run `sync`, start the daemon, make
tokens, commit, or write any project rules or tasks. That part is yours and the orchestrator's.

## Steps, either machine

Pick the next free port: bridle uses 7401 and meta-notes 7402, so the next is 7403. Below, the
project is `P`. **The clone's directory name is the project name**: bridle takes the name from
the directory.

1. **Both machines, `~/.bridle/config.toml`**: add the project under `[projects]`, the same line
   on each box.

   ```toml
   [projects]
   P = { machine = "nuc", port = 7403 }     # or machine = "dalek"
   ```

2. **The project's machine, clone it into its own workspace.** The workspace is the clone's
   parent, and the worktrees go there as well (`wt/`).

   ```bash
   mkdir -p /srv/shared/work/P-work && cd /srv/shared/work/P-work     # NUC
   # mkdir -p /Volumes/Data/work/P-workspace && cd "$_"               # dalek
   git clone git@github.com:jonathanbranam/P.git && cd P
   ```

   For an **existing** project, also run `git switch -c bridle-adopt` so the trial has its own
   branch (see below).

3. **Start the daemon** in the clone: `bridle serve --detach`. It listens on the port from
   `[projects]`, on 127.0.0.1 and the machine's Tailscale address. It runs without
   `.bridle/config.toml`, and picks the config up on the next restart.
4. **Make the orchestrator's token**, in a plain terminal and not inside Claude Code. It's
   human-only, and it's saved to `~/.bridle/credentials.toml` under `[orchestrator]`:
   `bridle token create orchestrator --project P`. Add `advisor` too if you'll use one.
5. **Start the orchestrator** on the same machine, in tmux (`ssh nuc` first for a NUC project):
   `bridle session orchestrator --project P`. Then ask it to prepare the project (last section).

### NUC specifics

- **No new build is needed.** The NUC already has a bridle checkout and binary for meta-notes.
  Every project on the box shares them. Only update with `cargo install --path crates/bridle`
  when you want a newer bridle; the binary is in `~/.cargo/bin`.
- **Workflow path.** A project's `.bridle/config.toml` `workflow = "/Volumes/Data/..."` path is
  a dalek path. The NUC's `~/.bridle/config.toml` needs a top-level
  `workflow = "<NUC bridle checkout>/workflow"` above its first table, which overrides it for
  every project there. meta-notes depends on it, so it should already be there.
- **Starting at boot (optional).** Run
  `bridle systemd install --project P --projects-dir /srv/shared/work/P-work`, then the
  `systemctl --user` and `loginctl enable-linger` commands it prints. It doesn't run them for
  you. Always give `--project`. Without it, one unit is written per NUC project, all with the
  same `--projects-dir` as their shared workspace, which only suits clones that sit side by side.
  Stop a `serve --detach` daemon before you enable the unit. Until you decide bridle is stable
  enough for that ([[docs/context/nuc-host|nuc host]], "How bridle is run for now"), running
  it by hand is fine.

### dalek specifics

- A dalek project's config can name dalek's checkout directly:
  `workflow = "/Volumes/Data/work/bridle/bridle/workflow"`. This is "path mode": it follows
  bridle's workflow as it changes, as meta-notes and track-web do. Leave it unset to keep a
  frozen vendored copy that you refresh yourself with `bridle workflow update`.
- `bridle launchd install --project P` writes a LaunchAgent if you want the daemon supervised,
  and prints the `launchctl` commands.

## Reaching it from the other machine

The project's own orchestrator runs on the project's machine. To reach the project from the
other box as well (for example, bridle's orchestrator on dalek sending to a NUC project), mint a
visitor token on the project's machine:

```bash
bridle token create orchestrator --project P --machine dalek     # on the NUC; printed once
```

Then paste it on dalek into `~/.bridle/credentials.toml`, mode 0600:

```toml
[orchestrator.nuc]
P = "..."
```

Now `bridle --project P ...` works from dalek. It routes to `http://nuc:7403` using the
`[projects]` line from step 1, and `[machine] name` must be set on both boxes. A visitor
(`external:orchestrator@dalek`) can send, read its inbox and run queries. It doesn't get the
orchestrator's wake or supervision, and it can't edit the queue. Swap `nuc` and `dalek` for a
dalek project.

## What to ask the orchestrator to do

Your flow is right, with the split below. **You** do everything under `~/.bridle` (config lines,
tokens, credentials), `serve`, and starting the session. The orchestrator can't edit those files
under auto mode, and `token create` needs your token. **The orchestrator** prepares the repo.
Tell it whether this is a new or an existing project:

- **New project** (nothing to protect): "work directly on main". The work lands on `main`.
- **Existing project**: the trial runs on `bridle-adopt`, with `integration = "bridle-adopt"`,
  and `main` is never touched until you approve (`workflow/base/rules/existing-projects.md`,
  ticket 63rv). To skip the trial, say "directly on main" explicitly, as for dotfiles-local
  (ticket 35mw).

Then something like: *"Prepare P for bridle: run `bridle init --stack <s>` if it isn't done,
fill in `.bridle/config.toml` (task prefix, check command, worker role, and for a small project
`[roles.manager] autostart = false` and `resume_on_restart = false`), add any project rules
under `.bridle/rules/`, run `bridle sync` and `bridle doctor`, try the check command, commit it
on <branch>, `bridle restart`, then file the first tasks from <write-up>."* The manager settings
come from ticket w2hj: with no manager running, the orchestrator starts one when there's work.
As of commit 82c42db (ticket gnar) it can also edit the queue on a project with no product
manager. That needs a daemon built from that commit or later.

## Gotchas today

- **On dalek, never start a daemon over SSH** (ticket nrbf). Claude Code's login is in the
  macOS login keychain, which an SSH session can't read, so every agent that daemon spawns is
  logged out and silently does nothing. Start it in a local terminal, or from SSH inside the
  laptop's own tmux server, with the launchd SSH agent so git can push (a window opened from SSH
  has none, and the daemon then hangs at a passphrase prompt before it starts):
  `tmux new-window -d -c <clone> 'SSH_AUTH_SOCK=/private/tmp/com.apple.launchd.<id>/Listeners bridle serve'`.
  Copy the `SSH_AUTH_SOCK` value from a running daemon's environment (`ps eww -p <pid>`;
  `bridle daemons` lists the pids). `launchctl getenv SSH_AUTH_SOCK` prints nothing on dalek.
- The first `bridle restart` after a project's port changes reports failure, although the daemon
  came back (ticket 6d5y). Check with `bridle --project P status`. A new project that starts on
  its `[projects]` port doesn't hit this.
- Marking a leftover clone tools-only (`bridle machine tools-only-install`) refuses over your
  git-template hooks (ticket ged2). Move them aside first.
- The first time the NUC connects to a daemon on dalek, macOS shows a firewall prompt. Allow
  it on dalek.
- To be woken when CI fails, set `[ci] github = true` in the project's config, and the repo
  needs a GitHub Actions workflow.
- Inbox messages stay on the machine where they were sent (hw6c).

**Possible ticket (not filed):** a `bridle project add P --machine M --repo URL` could replace
the manual part. It would clone into the workspace, run `init`, pick the next free port, print
the `[projects]` line for the other machine, start the daemon, mint and save the orchestrator
token (and print the visitor token), and then print the `session` command.
