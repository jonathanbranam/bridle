---
id: n94h
title: TUI inbox: q closes an open message; opening a message no longer marks it read
opened: 2026-09-29
repos: [bridle]
changes: []
specs: []
needs: []
see: [tui-panels-and-seeing-the-work-y496]
---

## The ask

The human, verbatim (2026-09-29, via the advisor), about `bridle tui`'s inbox view:

> small request: add q as option to dismiss a message;
>
> Also - wow, hitting enter marks a message as read? erg... I didn't reply or handle the message.

## Today (at be6a72d)

- `crates/bridle-tui/src/app.rs`: in an opened message, `Esc`/`Enter` close it (line 238); `q`
  only quits the whole TUI from the list (line 245). Opening a message sets `pending_mark_read`
  (line 286), so it leaves the unread list though the human hasn't handled it.
- `bridle inbox show <id>` does the same by default (`--no-mark-read` to opt out;
  `docs/design/cli.md`).

## Shape

1. `q` closes an opened message (as `Esc`/`Enter` do); `q` on the list still quits.
2. Opening a message doesn't mark it read. It's marked read when the human handles it: replying
   (`r`, as now), or an explicit key (e.g. `d`, done) on the list or in the message. Show the key
   in the title bar.
3. `bridle inbox show` matches: no mark-read by default (`--mark-read` to opt in), so reading a
   message never hides it. Update `docs/design/cli.md` and the TUI section.
