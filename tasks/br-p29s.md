+++
id = "br-p29s"
title = "The macOS Verifying popup still flashes during bridle builds, though the daemons run under launchd"
kind = "bug"
state = "pending"
created_at = "2026-10-10T02:03:16.671Z"
updated_at = "2026-10-10T02:03:42.295566Z"
created_by = "external:aide"
watchers = [
    "external:aide",
    "external:advisor/product-manager",
]
ticket = "p29s"
+++

docs/tickets/open/the-macos-verifying-popup-still-flashes-during-bridle-builds-p29s.md

## Thread

### note · external:advisor/product-manager · 2026-10-10T02:03:27.065Z
watching the task

### note · external:advisor/product-manager · 2026-10-10T02:03:27.163Z
advisor (product-manager): placed: theme reliability, no epic, normal (the human: "don't raise priority"). Cause unknown, so it needs investigation before a build; queue it after the machine-setup work (br-88d4, br-fpde, br-751e, br-hdbj).

### note · external:aide · 2026-10-10T02:03:42.295Z
aide: option from the human (2026-10-09 ~10:15 PM ET), verbatim: "can we launch tmux from launchd at boot like we set up on the nuc, then I attach to that session later, then iterm2 wouldn't be the parent of tmux?" and "DO NOT INTERRUPT planned work for this; it is only a minor irritation and not worth spending time on. Once the WSL2 box is running, bridle compiles will move there anyway." Shape: a LaunchAgent running the tmux server in the foreground (tmux -D, 3.2+), RunAtLoad/KeepAlive; the human attaches from iTerm with tmux a. Caveat: today's builds already trace to launchd, not tmux, so it helps only if the culprit is something started inside tmux (the interactive sessions).
