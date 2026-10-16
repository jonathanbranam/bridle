# How to add a project

*Written 2026-10-15 by advisor product-manager, for the human, as bridle 0.7.4 stands. A
checklist. The background and the why are in [[docs/context/adding-a-project|adding a project]]
and, for a machine's home project, [[docs/context/add-a-machine|add a machine]] section 6.*

> **Status:** Built and in use: every step below. Planned, not built: one command for all of it
> (ticket dffz, held). Below, the project is `P` and the machine that runs it is `M`.

## Before you start (once per machine)

- Every machine is on bridle 0.7.2 or later (`bridle --version`).
- On a Mac, `just sign-setup` has been run once, so the firewall's Allow survives upgrades
  (ticket 7bs2 says it doesn't always yet).

## Steps

1. **Repo.** Create the GitHub repo `P` (private). On `M`, clone it into its own workspace; the
   clone's directory name is the project name, and the workspace (its parent) also holds the
   worktrees.

   ```bash
   mkdir -p /Volumes/Data/work/P-workspace && cd /Volumes/Data/work/P-workspace   # dalek
   # mkdir -p /srv/shared/work/P-work && cd /srv/shared/work/P-work             # Linux
   git clone git@github.com:jonathanbranam/P.git && cd P
   ```

   An **existing** project you care about: `git switch -c bridle-adopt` too, so the trial has its
   own branch and `main` is never touched until you approve (rule `existing-projects`).

2. **Config.** In the clone: `bridle init`. It writes `.bridle/config.toml` and `.gitignore`
   lines and never overwrites. Commit them.

3. **Register it on every machine.** On `M`: `bridle project add P`. It adds
   `P = { machine = "M", port = N }` to `[projects]`, picks a port unused on any machine, syncs
   `[projects]` to every machine and adds `P` to `[mail] projects` everywhere. Register only: no
   daemon yet. (`--machine` and `--port` override.)

4. **Start the daemon.** In the clone on `M`: `bridle project start`. On a brand-new project it
   creates `bridle/state` and records `M` as owner (no `take-over` needed), installs the
   launchd/systemd unit, enables it (boot and crash restart) and starts it. It also makes the
   project's missing tokens on every machine (orchestrator, aide, advisor, mail, peer). Safe to run
   again.

5. **Tokens, only if step 4 named a failure.** `bridle token pair --machines dalek,vesta,remus`
   in a plain terminal (human only). A second run changes nothing that works.

6. **Mail.** `bridle mail install --project P`. It writes the unit and prints the load commands;
   run them (`launchctl ...` on a Mac, `systemctl --user enable --now ...` on Linux).

7. **Check.**
   - `bridle --project P status --json` answers, here and from another machine.
   - `bridle send --project P advisor "ping"` from another machine prints `delivered`.
   - A mail to `P@` gets a reply.
   - `bridle doctor` in the clone exits 0.

8. **Sessions.** Only what this project needs:
   - **A work project:** ask your machine's orchestrator to prepare it (new: "work directly on
     main"; existing: the `bridle-adopt` trial). It runs managers and workers there.
   - **A home project** (`<host>-home`): it has no managers or workers. Stop the orchestrator and
     aide in their old project, then start them here, in tmux shell windows:
     `bridle session orchestrator --project <host>-home` and `bridle session aide --project
     <host>-home`.

## Home projects: what's different

- Name `<host>-home` (`dalek-home`, `vesta-home`, `remus-home`); repo private; state push on (the
  default). It's the machine's own project (ticket 8uxe; the idea of keeping it as the machine's
  record is on ticket kuw2).
- Order: vesta before dalek. Everything addresses dalek's orchestrator in bridle today, so move it
  last.
- After the orchestrator moves, every other project on the machine needs the visiting
  `orchestrator@<host>` token so its wakes forward to the home and one waiter is enough (ticket
  7r2h). Step 4 or 5 makes it; check with step 7 from the orchestrator's side.

## When something fails

- `project start` says another machine owns `P`: it isn't new; see section 5, "Moving a project onto the new machine", in
  [[docs/context/add-a-machine|add a machine]] (`project stop` on the old machine, `take-over` here).
- A send is `queued` with "invalid token": run step 5.
- The gateway or daemon can't be reached from another machine after an upgrade on a Mac: click
  Allow in the firewall dialog (ticket 7bs2).
