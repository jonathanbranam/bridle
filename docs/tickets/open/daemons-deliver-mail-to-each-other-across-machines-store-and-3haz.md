---
id: 3haz
title: "Daemons deliver mail to each other across machines: store and forward, retry until the other daemon is back"
kind: feature
opened: 2026-10-04
repos: [bridle]
changes: []
specs: []
needs: []
see: [k7mw, cy2v, bp2v, fbfy, 3ehu, gtzx, k8jn]
tasks: [br-3haz]
---

## The ask

The human, verbatim (2026-10-03 ~22:45 ET, via the advisor):

> We also need to fix mail delivery between maybe not between daemons, but between machines. It
> needs to be delivered. The daemons need to communicate directly, and the agent shouldn't be
> expected to pull for mail delivery on another machine. So the daemons need to deliver mail to
> other reachable daemons, and then have a polling mechanism if the other daemon's offline, and
> then deliver the mail when it's online. I'm not sure the exact best solution here, so write up
> a ticket, and this is a continual problem, so it needs to be worked soon.

**Priority:** soon ("a continual problem"). The human isn't sure of the best design; the
proposals below are the advisor's, for review.

## What's wanted (summary)

1. **The daemons deliver mail between machines.** A sender on one machine sends, and the
   message ends up in the recipient's inbox on the recipient's own daemon.
2. **No agent pulls mail from another machine's daemon.** Each agent watches only its own
   daemon.
3. **When the other daemon is offline, the message waits** and the sending daemon keeps retrying
   ("a polling mechanism"), then delivers when it's back. Nothing is lost.
4. Between machines first. Between daemons on the same machine is "maybe not" in scope (Q1).

## Today (advisor, checked 2026-10-04 at 575bcea)

From [[projects-on-other-machines-by-config-k7mw|k7mw]] (built) and
`docs/design/agent-host/principals.md`:

- **No daemon talks to another.** The *CLI* reaches another machine's daemon by the machine config
  (`[machines]`, `[projects]` in `~/.bridle/config.toml`; dalek and nuc today) with a per-machine
  token (`[<principal>.<machine>]` in `credentials.toml`).
- **Sending to another machine** is `bridle send --project <their project> <principal> "..."`:
  the sender's CLI writes straight into the remote daemon. If that daemon is down or unreachable
  the send fails and nothing retries it.
- **The reply stays on the remote machine.** The sender appears there as a visitor
  (`<name>@<machine>`, e.g. `external:orchestrator@dalek` on the NUC's daemon), and replies go to
  that visitor's inbox *on the NUC*. k7mw's design says so: "the laptop's side reads it there
  (`bridle inbox --project meta-notes`), or the NUC's orchestrator sends it to the laptop's daemon
  instead". So the sender must poll the other machine's daemon to see a reply, which is what the
  human wants gone.
- **Waiters watch one daemon each.** `bridle agent wake` and `wait-for-wake` long-poll one daemon;
  the orchestrator runs one per project ([[one-watcher-for-every-project-bridle-agent-wake-all-projects-cy2v|cy2v]],
  `--all-projects`, not built). Nothing watches a visitor's inbox on a remote daemon.
- **Sessions don't know how to address another machine**
  ([[a-session-can-t-tell-who-it-can-message-or-how-to-reach-anot-bp2v|bp2v]]): the NUC's
  advisor reached dalek's advisor only by guessing.
- **Logged incidents:** none for cross-machine mail specifically. The nearest, same machine:
  "2026-09-30 to 2026-10-01: track-web's messages to the orchestrator sat unread for days" (one
  waiter, one daemon). The human calls cross-machine mail a continual problem; the next instance
  should be logged in `docs/context/incidents.md`.

## Advisor's proposals (for the human's review; nothing here is decided)

Numbered so the human can answer "yes to 3, change 5".

**P1. Store and forward through the sender's own daemon.** A send to someone on another machine
goes to the sender's *own* daemon, which accepts it at once (the send never fails because the
other machine is down), stores it in an **outbox**, and delivers it to the remote daemon over
HTTP. The CLI never writes to a remote daemon for mail.

**P2. Replies come home.** A visitor's token records its home (machine and project daemon). When
the remote daemon gets mail for a visitor (`external:orchestrator@dalek`), it forwards it to that
home daemon through its own outbox instead of keeping it in a visitor inbox. The recipient's
ordinary waiter on its own daemon wakes for it: nobody polls another machine.

**P3. Retry until delivered.** The outbox retries with backoff (at once, 30 s, 2 m, then every
5 m, capped), and also at once when a peer comes back: each daemon, on start-up, pings its peers
from the machine config, and a peer that hears "I'm back" flushes its outbox to it. Messages never
expire.

**P4. Exactly once, in order.** Each forwarded message carries its origin (machine, daemon,
message id); the receiving daemon records it and ignores a repeat, so a retry after a lost
acknowledgement doesn't deliver twice. Per peer, the outbox delivers oldest first.

**P5. Daemon-to-daemon identity.** One peer token per pair of daemons (minted like a visitor
token, `bridle token create --peer <machine>`; [[pair-machines-token-setup-over-ssh-sk7p|sk7p]] could
set it up over ssh). The forwarding daemon says who the original sender is
(`from: external:advisor/research@dalek`); the receiver trusts that label from a peer token only,
the way a named advisor's name is "a label, not proof" today.

**P6. Visible state.** The sender gets a message id at once with `queued for nuc`. `bridle
status` shows an outbox line per peer (`outbox nuc 3 queued, unreachable 2h`); a message
undelivered for over an hour (configurable) is reported to the human through the aide, once.
`bridle message show <id>` shows queued, delivered (when) or failed.

**P7. Addressing.** Keep k7mw's: name the recipient's project (`--project meta-notes`) or its
principal with a machine suffix. The CLI sends to the local daemon either way; the local daemon
looks the destination up in the machine config. Fold the "who can I message" listing from bp2v
into the same change.

**P8. Order of work:**

1. Outbox table, peer tokens, forwarding and acknowledgement, dedup (P1, P4, P5).
2. Visitor mail forwarded home (P2); drop "read the reply on the remote daemon" from k7mw's
   design doc text.
3. Retry with backoff and the start-up ping (P3).
4. Status, the aide's report, `message show` (P6), and the role prompts and `bridle send --help`
   (bp2v).

## Open for the human

- **Q1. Daemons on the same machine too?** The human: "maybe not between daemons, but between
  machines". The same outbox would carry mail between projects' daemons on one machine and could
  replace the one-waiter-per-project rule (cy2v). The advisor's view: build it machine to machine
  first, but keep the code daemon to daemon, so same-machine is a config switch later.
- **Q2. Build or use an existing system?**
  [[build-our-own-message-and-event-delivery-or-use-an-existing-fbfy|fbfy]] asks this. The
  advisor's view: build. An outbox table, an HTTP POST and a retry loop are small next to running
  a broker (NATS, etc.) on every machine, and they reuse the daemon's store and tokens.
- **Q3. Which mail crosses machines:** messages only, or task-watch wakes and events too? The
  advisor's view: messages only for now.
- **Q4. Read state:** the message is marked delivered when the remote daemon accepts it, but read
  only on the recipient's daemon. Does the sender need to learn "read"? The advisor's view: no.
