---
id: anmx
title: "'bridle gateway hash-password' echoes the password as it's typed: read it hidden when stdin is a terminal"
kind: bug
opened: 2026-10-04
repos: [bridle]
changes: []
specs: []
needs: []
see: [essy]
tasks: [br-anmx]
---

## The ask


The human, verbatim (2026-10-04, via the advisor), setting up the gateway on dalek:

> the hash-password echod the password; didn't hit the chars; update it to print **** instead or
> use proper secret input

("didn't hit the chars" is likely "didn't hide the chars".)

## Today (advisor, checked 2026-10-04)

`crates/bridle/src/gateway.rs`, `hash_password`: reads one line with `std::io::stdin().read_line`,
so a terminal echoes it in clear (and it stays in the scrollback and tmux history). It reads stdin
rather than an argument on purpose, to keep it out of shell history and `ps`.

## Wanted

- When stdin is a terminal: prompt (`Password:` to stderr), read with echo off (a no-`unsafe`
  crate such as `rpassword`, or `****` per keystroke as the human suggests), then ask a second
  time to confirm and refuse on a mismatch.
- When stdin is not a terminal (a pipe, `printf ... |`): read the line as today, no prompt, so
  scripts keep working.
- Update `docs/design/cli.md` and the gateway setup text in `docs/design/human-web-ui.md`.
