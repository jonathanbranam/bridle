---
id: br-5cru
title: Commands that run bridle on another machine over ssh fail unless the user adds bridle to the non-interactive PATH
kind: bug
created: 2026-10-14
created_by: external:advisor/wsl2
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-r2nr, br-v7ug, br-eurm]
theme: multi-machine
---

## The ask

The human, 2026-10-13 (to advisor wsl2), verbatim: "let's create a ticket for this requirement;
I'd prefer in the future users don't need to do this; it's fine for now though."

"This" is adding bridle's install directory to the PATH that a non-interactive ssh command sees,
on every machine, so that `ssh <host> bridle ...` works.

## What happened

During the NUC rename (ticket r2nr), on vesta:

```
% bridle token pair --machines dalek,vesta --rotate
machine dalek: unreachable, skipped: zsh:1: command not found: bridle
```

Found by advisor wsl2 (2026-10-13):

- `ssh <host> <cmd>` runs a non-interactive, non-login zsh, which reads only the zshenv files.
  The human's thoughtbot dotfiles set PATH in `.zshrc`, and their `.zshenv` warns whenever
  `~/.zshenv.local` changes PATH. So the remote PATH is the system default, without
  `~/.cargo/bin` (dalek and vesta, where `just install` puts bridle) or `~/.local/bin` (remus, where
  `scripts/install-release.sh` puts it).
- Checked on all three machines: `ssh <host> 'command -v bridle'` finds nothing on dalek, vesta or
  remus. vesta's pair run worked for vesta only because pair runs bridle locally on its own machine.
- The machine is reported "unreachable", which is wrong: ssh worked, and the command wasn't found.
- Code that runs `bridle` on another machine over plain ssh: `crates/bridle/src/token_pair.rs`
  (`bridle <args>`, around line 92-111) and `crates/bridle/src/preflight_remote.rs` (`bridle
  --version`, line 212). `bridle projects sync` (`project_sync.rs`) uses only `cat`/`mv` and is not
  affected.
- `docs/design/cli.md` says pair needs "bridle on the remote PATH", but neither
  `docs/context/add-a-machine.md` nor `docs/context/windows-wsl2-host.md` says how to arrange it.

## Interim (the human's machines, 2026-10-13)

One line in the system zshenv on each machine:

```
# macOS (dalek)
echo 'path=($HOME/.local/bin $HOME/.cargo/bin $path)' | sudo tee -a /etc/zshenv
# Ubuntu, WSL2 included (vesta, remus): Debian's zsh reads /etc/zsh/zshenv, not /etc/zshenv
echo 'path=($HOME/.local/bin $HOME/.cargo/bin $path)' | sudo tee -a /etc/zsh/zshenv
```

Check: `ssh <host> 'command -v bridle'` prints a path.

## What's wanted

A machine set up by add-a-machine works with pair and preflight, with no PATH edit by the user.
Options for whoever plans it (not decided):

- Remote commands try `bridle` on PATH, then the known install directories (`~/.local/bin/bridle`,
  `~/.cargo/bin/bridle`), and report which one they used.
- Record the bridle path per machine (in `[machines]`, or found once and cached).
- Run the remote command in a login shell (`$SHELL -lc`): less predictable, since it depends on the
  user's dotfiles.

Whatever is chosen: "command not found" must be reported as "bridle not found on <host>", not
"unreachable", and add-a-machine should say what's needed until then.
