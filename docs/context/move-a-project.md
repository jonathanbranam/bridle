# Move a project to another machine (the runbook)

Status: built and in use by hand (kx5u K1-K9). Run in order by the human or the orchestrator. The
one-command move (`bridle project move`, K7) is not built. OLD is the machine that serves the project
now, NEW the one that will. Both are keys of `[machines]` in `~/.bridle/config.toml`.

Reference for each command: [[docs/design/cli|the CLI]], `bridle project <command> --help`. The
new machine itself (git, logins, services) is [[docs/context/add-a-machine|add a machine]]; do that first.

Run steps 1-2 from OLD, step 3 on OLD, steps 4-5 on NEW. `--project P` selects the project's daemon
from anywhere; `--machine M` on `stop` and `start` runs them over ssh.

## 1. Preflight (read-only; changes nothing)

```bash
bridle project preflight --to NEW [--path ~/work/P-workspace/P]
```

`--path` is where the clone is (or will be cloned) on NEW; without it preflight uses OLD's path under
`$HOME`, else `~/work/<project>-workspace/<project>`.

Each check prints `pass` or `FAIL` with what to run; exit is non-zero on any FAIL. It covers:

- no absolute or `~` `workflow` or `system_prompt` in `.bridle/config.toml` (fix by hand, commit);
- the project is stable (see step 2: it will normally FAIL here until the drain is done, which is fine);
- NEW: ssh, clone present (it asks before cloning), bridle version, `claude` and `gh` logins, git push
  access, the project's toolchain, tmux, timezone, `[machine] name` equals the key `NEW`, AWS
  credentials when `[mail]` is set, and `projects sync` shows no difference.

Fix every FAIL that is not "stable project". **Back out:** nothing to undo.

## 2. Drain (the project stops taking new work)

```bash
bridle project drain            # or: bridle --project P project drain
```

It holds new claims and spawns for this project only (running agents finish their turns, messages
still flow), tells every running agent and registered session, then waits with no timeout until
the project is stable, messaging `orchestrator` at the start and on each change. It names a list that has
not moved for 30 minutes to `orchestrator` and `aide`. It returns when nothing is left.

Stable means: no claimed or built ticket, no running agent, no `bridle/*` or `integrate/*` branch ahead of
the integration branch, a clean main checkout (untracked files count; `--leave PATH` lets a path through and
lists it as LEFT), no undelivered outbox mail. Land or drop the tickets it names, push or delete
the branches, and let the agents finish. Run `project preflight --to NEW` again; all passes means go.

Caveats: the hold lives in the daemon's memory, so a daemon restart drops it (a running `drain` re-asserts it;
do not restart the daemon while draining without the drain running). Known drain bugs are br-ef3u and br-8swr.

**Back out:** `bridle project drain --undo` lifts the hold. Nothing else changed.

## 3. Stop on OLD (this is the giving up)

```bash
bridle project stop --list      # what it would leave behind; stops nothing
bridle project stop             # or: bridle --project P project stop --machine OLD
```

It re-checks stability (refuses, naming each failure, with nothing stopped), shows what is left behind
(unread messages, active schedules, agent records, events, `bridle.db`) and asks (`--yes` skips; with no
terminal it refuses without `--yes`). Then it stops the agents and the daemon, pushes `bridle/state` then
the integration branch (confirming origin has each), pushes a `released` mark in `owner.toml` as
the last commit, and unloads and disables the daemon and mail units (the files stay). It commits the
scheduled messages (`schedules.json`) and the outbox counter (`outbox-seq`) for NEW to pick up. Last it
retires OLD: the clone goes into `[machine] tools_only` (so `bridle serve` refuses there), the registry
entry and the `.bridle/upgrade-*` trees go, `bridle.db` stays.

Claude sessions of running agents do not resume on NEW. Inbox messages, events and usage stay in OLD's `bridle.db`.

If it fails it says why, exits non-zero, writes no mark and leaves the units as they were: fix and rerun.
Check afterwards that `git ls-remote origin bridle/state` shows the mark's commit.

**Back out (before it has succeeded):** the units are as they were; `bridle project drain --undo`.
**Back out (after success):** the project is ownerless and OLD is tools-only. Run step 4 on OLD instead of NEW
(`take-over` there removes the tools-only entry and the registry entry returns), then `project start`.
The old `bridle.db` is backed up (see step 4) and the outbox counter is seeded so message ids do not repeat.

## 4. Take over, then start, on NEW

```bash
bridle project take-over
bridle project start            # or, from elsewhere: bridle --project P project start --machine NEW
```

`take-over` refuses a project with no `released` mark (`--force` only for a dead OLD: whatever it never pushed is
lost) and refuses if origin is unreachable or `bridle/state` or the integration branch has diverged; it
rebases nothing, so fix by hand and rerun. It commits and pushes `owner.toml` (host is `[machine] name`),
moves the `[projects]` entry to NEW on every machine (`projects sync --apply`), moves any old
`<workspace>/.bridle/bridle.db` to `.bridle/db-backups/<date>-<sha>/` (a return trip) and seeds the outbox counter. It
starts no daemon and is safe to run twice.

`start` installs the launchd or systemd unit if missing, enables it (boot, restart on failure) and starts it.
After a take-over it then re-creates the project's tokens on every machine (`token pair --projects P`),
re-adds the scheduled messages (a past one-shot is reported, not added) and, if `[mail] projects` names P,
copies OLD's `[mail]` section and `mail.key` and installs and starts the mail bridge. Pending work is
kept in `<workspace>/.bridle/moved-in.json`; if `start` reports a failure, fix it and run it again. Do not
start a daemon over ssh on macOS (ticket nrbf): use a terminal on NEW or `--machine` as above.

**Back out (before `start`):** run `bridle project stop` on NEW (it needs no daemon), then step 4 on OLD.
**Back out (after `start`):** go back to step 3 on NEW (drain first if work started), then step 4 on OLD.
Never `take-over --force` on OLD while NEW's daemon runs: two writers.

## 5. Check on NEW

1. `bridle --project P status --json` answers and shows the daemon on NEW.
2. `bridle doctor` exits 0 in the clone.
3. Tokens both ways: from another machine `bridle --project P status --json` answers (visitor token); and
   `bridle send --project P advisor "ping"` prints `delivered`, not `queued` (peer token). Send one the other way too. `project start` has already re-paired every machine's peer token toward P (not only P's own outgoing ones) and checked them, so a refused one shows in its output; if a machine was unreachable, run `bridle token pair --tokens peer` once.
4. Scheduled messages: `bridle schedule list` shows what OLD had.
5. Mail (only if P uses `[mail]`): `bridle mail status` or the unit is running; send a mail and see the reply.
   If `start` said replies to mail from OLD will fail verification, `mail.key` differs: copy OLD's.
6. Gateway: the gateway (bridle-ui) lists P at NEW. The gateway follows `[projects]` with no code of its own;
   K6 built and tested the mail half only, so look at it yourself.
7. `git ls-remote origin` shows `bridle/state` at `owner.toml` host = NEW, and a push of `main` from NEW works
   (only the owner's clone may push it).

**Back out:** as step 4.

## 6. Restart the interactive sessions on NEW

Sessions do not move. On NEW, from each session's last handover note:

```bash
bridle session orchestrator --project P
bridle session advisor --project P [alias]
```

Handover notes are restored from the state branch. Remove the old sessions on OLD if they are still open.

## Gaps found writing this (code versus design)

- `project preflight --to NEW` always fails "stable project" until the drain finishes; it is not
  split into a "can NEW take it" half.
- The drain hold is in memory only (stated in the design); a restart in the middle loses it until `drain` runs again.
- Back-out after step 3 has no command: it is `take-over` on OLD, which works only because `stop` wrote the
  mark; the K5 retirement is undone by that `take-over`.
- No automated check that the gateway follows the move (step 5.6 is by hand).
- Inbox messages, events and usage do not move; the old database is only backed up on NEW's return trip.
