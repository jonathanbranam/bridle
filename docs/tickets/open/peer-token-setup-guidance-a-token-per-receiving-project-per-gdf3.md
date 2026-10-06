---
id: gdf3
title: "Peer-token setup guidance: a token per receiving project per sending machine, minted on the receiver"
kind: bug
opened: 2026-10-06
repos: [bridle]
changes: []
specs: []
needs: []
see: [n63z, 3haz]
tasks: []
---

## The ask

Reported by orchestrator@nuc (meta-notes) for the human, 2026-10-06. The human: "nobody told me that I needed peer tokens across machines too ... I just asked how to set up peer tokens, and nobody mentioned that."

The setup guidance (docs, the CLI's error and help text, what agents tell the human) should say:

- A peer token exists per receiving project (daemon) per sending machine.
- Make it on the receiver with `bridle token create --peer <sending machine> --project <project>` and paste it under `[peer]` on the sender.
- Projects on the same machine need one each too.

It is easy to get the direction backwards. The human first ran `--peer dalek --project bridle` (the receiver's name) and got `conflict: principal peer:dalek already exists`. That error should say which machine name `--peer` expects.

n63z (one command mints every token) would remove most of this. This ticket is the guidance until then.
