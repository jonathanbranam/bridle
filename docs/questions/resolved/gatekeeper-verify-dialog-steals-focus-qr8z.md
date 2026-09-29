---
id: qr8z
title: Gatekeeper's "Verifying…" window flashes and steals keystrokes while bridle builds
opened: 2026-09-28
resolved: 2026-09-29
repos: [bridle]
changes: [74603a5]
specs: []
needs: []
see: [sign-binaries-on-intel-macs-cs7x]
---

## What the human said

2026-09-28, to the advisor:

> there is some dialog flashing repeatedly on my mac. It looks like some kind of confirmation or
> permissions dialog - it isn't very big, centered in the screen. I can't see anything when it
> happens, but it steals a keystroke or two when it occurs. It isn't a huge bother but it is
> pretty annoying and seems related to bridle running. I can't see what it says at all and it
> immediately closes and focus returns to iterm.

> I haven't restarted the bridle servers though, and the run foregrounded. I run everything in
> tmux also, btw, so restarting iterm didn't interrupt anything

## What the advisor found (2026-09-28, unified log)

- The window is `CoreServicesUIAgent`'s code-evaluation progress panel (Gatekeeper's
  "Verifying…"), shown for ~50 ms while `syspolicyd` runs an XProtect scan on the **first exec
  of each new executable file**. 78 of them in the hour to 23:00 local, tracking build activity.
- The new executables are what workers make: `rustc` build scripts, test binaries and
  `target/debug/bridle` in each `wt/*` worktree (bridle), and the esbuild binaries each
  `npm install` puts in a track-web worktree's `node_modules`. Per-worktree `target/` and
  `node_modules` mean fresh inodes, so every worktree rescans.
- Every process bridle runs has iTerm as its TCC "responsible" app: the daemons run under tmux,
  whose server was started from iTerm. Restarting iTerm doesn't change that.
- Tested by hand with a trivial C binary, after the human added iTerm to Privacy & Security →
  Developer Tools (the Developer Tool check then returned allowed):

  | Launched by | Signature | Scanned | Window |
  |---|---|---|---|
  | shell under iTerm | none | yes | yes |
  | shell under iTerm | ad-hoc | yes | yes |
  | `launchctl submit` | none | yes | **no** |

  So neither Developer Tools nor ad-hoc signing (cs7x) stops the window; a GUI responsible app
  does (`allowUI is YES`), and a launchd-started process has none.

## Options (not triaged)

- Start the daemons from launchd (a LaunchAgent, or `launchctl submit`) instead of a terminal,
  so agents and their builds have no GUI responsible app. Overlaps the launchd option in
  spike mnzh, [[detached-daemon-and-its-terminal-mnzh|detached daemon and its terminal]].
- Fewer fresh executables: share or warm `target/` and `node_modules` across worktrees
  (b7cz covers `target/`).
- Live with it.

## Resolution

Resolved by 74603a5 (br-936d): `bridle launchd install|uninstall` writes a per-project LaunchAgent, so the daemon and its agents' builds have no GUI responsible app and Gatekeeper's window doesn't show (the first option above). The answer lives in docs/design/cli.md (`launchd install|uninstall`); moving a running daemon is docs/context/launchd-restart-plan.md.
