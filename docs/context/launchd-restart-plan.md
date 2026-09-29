# Moving the daemons from tmux to launchd

Why: ticket qr8z. Processes under iTerm/tmux have iTerm as their GUI responsible app, so every
first exec of a new executable flashes Gatekeeper's Verifying window and steals keystrokes. A
launchd-started process has none. Nothing here was run by the worker; these are the steps for you.

## What a daemon restart does to agents

- Stopping the daemon (`bridle stop-daemon`, SIGTERM) stops every running agent, tagged
  `daemon_shutdown`; that can take up to `stop_grace` (30 s) + 5 s.
- On the next start, roles with `resume_on_restart` (manager, orchestrator) resume with
  `--resume`, keeping their conversation. Workers do **not** resume (open question 2fkk): their
  worktrees and branches stay, but you re-spawn them.
- Messages not yet acked go back to `pending`. The budget governor's human hold survives only
  if it's persisted; check `bridle budget` after the restart and re-run `bridle budget hold` if
  needed.
- Do it per project, at a quiet moment: wait for workers to reach a task boundary, or
  `bridle budget hold` first so nothing new starts, and let running turns finish.

## Steps, per project (bridle, meta-notes, track-web)

1. In the project's clone, install (writes only the plist; `--project` if the directory name
   isn't the project name):
   `bridle launchd install --project <name>`. Run it with the `bridle` binary you want the
   daemon to use (the path is baked in); rebuilding that binary later needs a restart anyway.
2. `bridle budget hold` (optional), then check `bridle agents` for running workers.
3. `bridle --project <name> stop-daemon`, and wait until `bridle daemons` no longer lists it.
   Then end the old `bridle serve` in its tmux pane if it didn't exit.
4. Load it: `launchctl bootstrap gui/$(id -u) ~/Library/LaunchAgents/dev.bridle.<name>.plist`
   (the command `install` printed).
5. `bridle daemons` shows the project; `bridle status` looks right; `bridle budget release` if
   you held. Re-spawn any workers you stopped. Log: `<workspace>/.bridle/daemon.log`.

Do one project first (track-web or meta-notes, not bridle itself, which runs your other
agents) and verify before the rest.

## Verify the window is gone

While a build runs in a worktree of the moved project:

```
log stream --predicate 'process == "CoreServicesUIAgent"' --info
```

No new code-evaluation lines during builds means it worked; projects still under tmux still
produce them. Also `ps -o pid,ppid,command -p <daemon pid>` shows parent 1 (launchd).

## Roll back

1. `bridle --project <name> stop-daemon`; `launchctl bootout gui/$(id -u)/dev.bridle.<name>`.
2. `bridle launchd uninstall --project <name>` (removes the plist).
3. Start `bridle serve` in tmux as before.

## Notes

- The daemon isn't restarted by launchd after a deliberate stop or `stop-daemon` (exit 0),
  only after a crash.
- The plist copies `PATH` from the shell that ran `install`; if you change toolchains, run
  `install --force` and reload.
- Uninstall does not unload; it prints the `bootout` command.
