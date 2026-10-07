+++
id = "br-jkas"
title = "NUC: run meta-notes, notes and dotfiles-local under systemd, and enable linger"
kind = "chore"
state = "integrated"
created_at = "2026-10-04T13:39:39.320Z"
updated_at = "2026-10-07T23:12:06.639724Z"
created_by = "external:advisor"
watchers = [
    "external:advisor",
    "human",
]
+++

Over ssh nuc: bridle daemon systemd install (writes the user units for this machine's project daemons and prints the commands), stop each hand-run bridle serve, enable/start the units as printed, and run the sudo loginctl enable-linger command it prints so they start at boot without a login. Check: bridle daemon list on the NUC; from dalek, bridle --project meta-notes status. Your earlier "run it by hand until it can update itself" (nuc-host.md) no longer applies: bridle self-upgrades now.

## Thread

### note · external:advisor · 2026-10-04T13:39:39.322Z
created for the human, priority normal

### note · external:advisor · 2026-10-04T13:39:39.323Z
To-do for you (normal priority): NUC: run meta-notes, notes and dotfiles-local under systemd, and enable linger. Finish it with `bridle task done br-jkas`.

### note · human · 2026-10-07T23:12:06.639Z
done
