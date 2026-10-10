# The macOS "Verifying…" popup on dalek

Status: working notes, kept by the aide for the human to follow up on. Not a design. Tracked by
ticket [[the-macos-verifying-popup-still-flashes-during-bridle-builds-p29s|p29s]]; the earlier
investigation is [[gatekeeper-verify-dialog-steals-focus-qr8z|qr8z]] (resolved).

## What the human sees

The human, 2026-10-09 ~10 PM ET, verbatim:

> the constant popup that immediately disappears on my mac - what is that again? a permissions
> dialog? it's very frustrating. [...] It interrupts my typing and it cancels a selection in the
> OS X upper right system menu thing like for wifi bluetooth and sound

> it's impossible to see it; it flashes at some point a dozen times or more during a build or
> test of bridle

## What it is

Gatekeeper's code-evaluation progress panel ("Verifying…"), drawn by `CoreServicesUIAgent`. It
shows for a split second while `syspolicyd` scans an executable the **first time it runs**. A
bridle build or test run creates dozens of new executables (rustc build scripts, test binaries,
`target/debug/bridle` in each worker's worktree), so the panel flashes once per new binary. Each
flash takes keyboard focus, which drops keystrokes and closes any open menu-bar menu.

## What we know (qr8z, 2026-09-28)

- The panel shows only when the process that runs the new binary has a GUI "responsible" app
  (`allowUI is YES`). Processes started under iTerm (directly or through tmux started from
  iTerm) have iTerm as their responsible app; processes started by launchd have none.
- Tested by hand: adding iTerm to System Settings → Privacy & Security → Developer Tools did
  **not** stop the window; ad-hoc signing did not either; a binary started by `launchctl
  submit` was scanned but showed **no** window.
- The fix taken: `bridle launchd install` runs each daemon as a LaunchAgent (br-936d).

## What the aide checked (2026-10-09 ~10 PM ET)

- All of dalek's daemons run under launchd (`dev.bridle.bridle`, `.bridle-ui`, `.track-web`,
  `.gateway`, `.mail.bridle`).
- Every running `cargo`/`rustc`/`cargo-nextest` process traced back to `bridle serve` (pid 640)
  under `launchd`, through the worker's `claude` and its shell. Nothing build-related was a
  descendant of iTerm2; under iTerm2 were only its login shells, ssh and a `tmux a` client.
- `CoreServicesUIAgent` logged ~600 XPC requests in the hour to ~10 PM, during builds.
- So the parent-process tree no longer explains it. Open question: macOS's *responsible
  process* is not the same as the parent tree (it is set at spawn and can be inherited oddly),
  and the log hides the paths (`<private>`). The gateway being started from a Claude session
  (incident rztb, fixed 2026-10-09) is the kind of thing that could leave a process with iTerm as
  its responsible app.

## Fixes found online

1. **Developer Tools for the terminal.** `sudo spctl developer-mode enable-terminal` shows the
   Developer Tools panel in System Settings; turn on your terminal (iTerm) under "Allow
   applications to use developer tools", restart it, and possibly `cargo clean`. The fix most
   sources recommend (Zed, nextest). On dalek it didn't stop the window in the 09-28 test, but
   check it's still on.
2. **Turn Gatekeeper off** (`sudo spctl --master-disable`). Broad: gives up real protection, and
   it's unclear that it stops the XProtect scan behind the panel. Not recommended.
3. **No builds with a GUI responsible app.** What worked in our own test (launchd). The ticket is
   to find what still has one.

## Read more

- [Zed: how to disable the "Verifying…" popup on macOS](https://git.secluded.site/zed/commit/9ef454d7eb7fddf89332d63b077ff5514ff997e8):
  the Developer Tools steps for a terminal.
- [nextest: macOS Gatekeeper and XProtect](https://nexte.st/docs/installation/macos/): why test
  runs slow down and the Developer Tools fix, including `cargo clean` after.
- [nextest: antivirus and Gatekeeper](https://nexte.st/book/antivirus-gatekeeper): the same, older page.
- [Rust forum: `cargo run` slow on macOS](https://users.rust-lang.org/t/cargo-run-slow-on-macos-when-binary-already-built/117450):
  a fresh build "taints" the binary; iTerm2 had to be added by hand (Terminal was listed already).
- [Avoiding Gatekeeper in your terminal](https://notes.billmill.org/computer_usage/mac_os/Avoiding_gatekeeper_in_your_terminal.html):
  short note on the Developer Tools category.
- [Apple: enable and disable Gatekeeper](https://help.apple.com/xcode/mac/current/en.lproj/dev9b7736b0e.html):
  the official `spctl --master-disable` route (the broad option).
