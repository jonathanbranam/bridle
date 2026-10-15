---
id: br-mru4
title: "[at next reboot] Try bridle up: the orchestrator comes back by itself after a reboot (f8f9 slice 1)"
kind: chore
created: 2026-10-15
created_by: external:orchestrator
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

## The ask

br-f8f9 slice 1 landed (dedf27ae): `bridle up` starts this machine's orchestrator in a tmux pane tagged `@bridle=orchestrator` at boot, once. The reboot drill is yours (the brief said so). On the machine to try it on (remus first, per the brief; any machine works):

1. Update bridle there (the next release, or `just install` on dalek).
2. `bridle up --install`, then run the load command it prints (launchd `launchctl bootstrap ...` or `systemctl --user enable bridle-up.service`); on Linux also `sudo loginctl enable-linger $USER` if it says so.
3. If you use tmux-resurrect: `bridle up --print-tmux-hooks` and add the two lines to your tmux.conf, so the pane tag survives a restore.
4. Reboot. Expect one orchestrator session running in tmux without you starting it; `~/.bridle/up.log` says what it did. Running `bridle up` again starts nothing.

Kill switch: `touch ~/.bridle/no-autostart` stops it at boot.
Done when: it came back after one reboot; `bridle ticket done <id>`. If it didn't, tell aide what up.log says.
