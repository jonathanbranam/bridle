---
id: p29s
title: The macOS Verifying popup still flashes during bridle builds, though the daemons run under launchd
kind: bug
opened: 2026-10-10
filed_by: external:aide
repos: [bridle]
changes: []
specs: []
needs: []
see: [qr8z, rztb, p88z]
tasks: [br-p29s]
---

## The ask

The human, 2026-10-09 ~10 PM ET, verbatim (to the aide):

> the constant popup that immediately disappears on my mac - what is that again? a permissions dialog? it's very frustrating. Can you search online about ways to stop that from appearing completely. It interrupts my typing and it cancels a selection in the OS X upper right system menu thing like for wifi bluetooth and sound

> it's impossible to see it; it flashes at some point a dozen times or more during a build or test of bridle

> don't raise priority, but yes, create a new ticket to track that it is still occurring [...]
>
> I don't run any builds; I don't think the orchestrator runs any builds. I'm not sure how it happens. But also, the bridle gateway has sometimes been run from a claude session which is strange. the workers and managers run builds, but aren't they owned by the launch bridle?

## Facts so far

- It is Gatekeeper's "Verifying…" panel (`CoreServicesUIAgent`), shown while `syspolicyd` scans
  each new executable on first run; [[gatekeeper-verify-dialog-steals-focus-qr8z|qr8z]] found it
  shows only when the process has a GUI responsible app (iTerm), and fixed it by running the
  daemons under launchd (br-936d).
- 2026-10-09 ~10 PM ET the aide checked: all of dalek's daemons are LaunchAgents, and every running
  `cargo`/`rustc`/`cargo-nextest` traced to `bridle serve` (pid 640) under `launchd`; nothing
  build-related descended from iTerm2. `CoreServicesUIAgent` still logged ~600 requests in the hour.
- So the parent tree doesn't explain it. macOS's responsible process is set at spawn and is not
  the parent tree; the unified log hides paths (`<private>`).
- Notes, the options found online and links: `docs/notes/macos-verifying-popup.md`.

## The ask

Find which process still has a GUI responsible app when new binaries run (for example with
`launchctl procinfo <pid>` as root, or the private log enabled), and stop it, so the panel no
longer flashes during builds. Priority: normal (the human: "don't raise priority").
