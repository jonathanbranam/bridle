+++
id = "br-xes7"
title = "After br-jmpf lands: uncomment [gateway] in ~/.bridle/config.toml and restart the gateway"
kind = "chore"
state = "integrated"
created_at = "2026-10-04T13:39:39.220Z"
updated_at = "2026-10-07T02:37:19.809780Z"
created_by = "external:advisor"
watchers = [
    "external:advisor",
    "human",
]
+++

br-jmpf (a [gateway] section stops every daemon from starting) must be integrated AND the daemons upgraded onto it first: check bridle daemon list / the upgrade events. Then: uncomment the four [gateway] lines, run bridle daemon doctor, restart bridle gateway (Ctrl-C in its tmux window, run again). When br-c657 lands, install the gateway as a service instead.

## Thread

### note · external:advisor · 2026-10-04T13:39:39.223Z
created for the human, priority normal

### note · external:advisor · 2026-10-04T13:39:39.225Z
To-do for you (normal priority): After br-jmpf lands: uncomment [gateway] in ~/.bridle/config.toml and restart the gateway. Finish it with `bridle task done br-xes7`.

### note · human · 2026-10-07T02:37:19.809Z
done
