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
proposals below are the advisor's, for review. Q1-Q4 are answered ("Decided: Q1-Q4"); P1-P8 are
still for review.

## What's wanted (summary)

1. **The daemons deliver mail between machines.** A sender on one machine sends, and the
   message ends up in the recipient's inbox on the recipient's own daemon.

> [!comment] human, 2026-10-04 10:57, on "sender on one machine sends"
> Test comment, when received, reply here

2. **No agent pulls mail from another machine's daemon.** Each agent watches only its own
   daemon.
3. **When the other daemon is offline, the message waits** and the sending daemon keeps retrying
   ("a polling mechanism"), then delivers when it's back. Nothing is lost.
4. Every message, between any two daemons, same machine included (decided, Q1).

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

## Decided: Q1-Q4 (the human, 2026-10-04)

Verbatim (via the advisor):

> Q1 - all messages need to arrive timely and guaranteed, be marked read reliably, and so should
> responses.
>
> Q2 - I agree to build ourselves.
>
> Q3 - task wakes are now messages, or should be, so irrelevant. That said I'm not sure if an
> agent on another machine or project can watch a task - that should be built though if not
> isn't - and task wakes will send a message.
>
> Q4 - do you mean actively with another message? No, but they should be able to check I think
> delivered and read. Delivered should mean it reached the correct daemon and the agent was awake
> and running and received the message. For now, I think we should mark delivered messages as
> read also - but I'm still considering that design.

What that means for the build:

- **Q1, scope: every message, not only between machines.** Same machine, other project, other
  machine: each arrives promptly and surely, is marked read reliably, and so does its reply. So
  the outbox carries mail between any two daemons, same machine included (it can retire cy2v's
  one-waiter-per-project rule). The proposals' "between machines first" is withdrawn.
- **Q2: build it ourselves.** fbfy's question is answered for messages.
- **Q3: task wakes are messages** (`task_update`), so they ride the same delivery. **Watching a
  task from another project or machine must work.** Checked 2026-10-04 at 3da7155: a visitor
  (`<name>@<machine>`) *can* `bridle task watch` (`set_watching` in `server.rs` has no visitor
  check), but its `task_update` messages land in the visitor's inbox on the task's daemon, which
  nobody watches; P2 (mail for a visitor is forwarded home) fixes that. An agent of another
  project (a worker, a manager) has no token on another project's daemon, so it can't watch there
  at all; that needs building (a peer daemon watching on its agent's behalf, through P5's peer
  token, is the advisor's guess).
- **Q4: no "read" receipt message**, but the sender can check a message's state:
  - **delivered** = it reached the right daemon **and** the recipient was awake and running and
    received it (its wake or inbox returned it), not just "stored on the daemon";
  - **read**: for now, a delivered message is also marked read (the human is "still considering
    that design"; related: [[read-on-delivery-can-lose-messages-marked-read-before-the-re-k8jn|k8jn]],
    where read-on-delivery can lose messages);
  - so a message has at least: queued (in the sender's outbox), arrived (stored on the
    recipient's daemon, recipient not yet woken), delivered (= read, for now). P6's `bridle
    message show <id>` reports which.

## Decided: one waiter per principal, on its home daemon (the human, 2026-10-04)

The advisor explained that delivery between daemons removes the orchestrator's one waiter per
project only if every reason to wake it is a message (today a daemon also wakes it for its own
project's red CI, a dead agent, a stalled task, a context warning), and asked whether 3haz should
make every wake reason a message. The human, verbatim:

> Great yes. Cutting down waiters would be great.

So:

- **Every principal has a home daemon** (the orchestrator's: this machine's bridle daemon, say);
  each other daemon forwards that principal's messages there.
- **Every wake reason becomes a message**, sent like any other and forwarded home: CI failures,
  agent deaths, stalls, context warnings, task wakes (Q3).
- **One waiter per principal**, on its home daemon. The role's "one waiter per project"
  (7b22c1e) goes, and [[one-watcher-for-every-project-bridle-agent-wake-all-projects-cy2v|cy2v]]
  (`bridle agent wake --all-projects`, the CLI fanning out) is superseded; resolve it when this
  lands.

Asked whether to hold cy2v's planned task (br-1ddd) meanwhile, the human, verbatim:

> Orch is special today. I'm not sure. . But other agents only wake on messages.

So the non-message wake reasons are the orchestrator's alone: every other agent wakes only on
messages, and 3haz's forwarding is all they need. br-1ddd is left as it is (not decided).

Then, verbatim:

> I'm fine with everything as a message if there are no caveats

### Caveats of every orchestrator wake as a message (advisor, for the human)

Checked against `orchestrator-supervision.md` section 5 (wake conditions, `wake.rs`). Context
warnings are already messages (`sessions.rs`). None looks like a blocker; each has a fix:

1. **Noise.** Wakes fire once per condition and are kept out of the human's counts; as messages
   they need the same: one message per crossing (not per event), a `system` kind that
   `unread_human_messages` and the human's to-dos skip.
2. **State, not news.** Usage over 93%, a budget hold, "idle" are states that can be over by the
   time a forwarded message is read. Each message says when it was true, and a later message
   clears it ("budget hold ended").
3. **One home daemon is one point of failure.** Today a dead track-web daemon doesn't stop
   bridle's waiter, and vice versa. With one waiter on the home daemon: if the home daemon is
   down the orchestrator hears nothing from any project (but its waiter fails, so it knows); and
   a project's daemon being down can't be reported by that daemon, so the home daemon's outbox
   reports a peer unreachable (P6) as a message.
4. **Read on delivery** ([[read-on-delivery-can-lose-messages-marked-read-before-the-re-k8jn|k8jn]]):
   wakes today move a cursor only when delivered, so a restart re-derives them. Messages marked
   read on delivery can be lost the same way k8jn describes; the k8jn fix must cover them.
5. **"Nobody is waiting" stays.** The waiter incident (no wake command running) is about the
   waiter, not a message; it moves to the home daemon, per principal.

### Decided (the human, 2026-10-04)

Shown the caveats, verbatim:

> Leave all-projects for now. It'll be a NOOP after this lands. We can warn and remove later. Add
> a follow up ticket to remove it after this lands and rolls out so we don't forget

Then, explicitly (verbatim):

> Approve every wake is a message. I think this simplifies a lot of things.

So: every orchestrator wake becomes a message, the caveats above accepted. cy2v's `--all-projects` (br-1ddd) is still built
meanwhile; its removal after rollout is
[[remove-bridle-agent-wake-all-projects-once-3haz-lands-and-ro-kuvh|kuvh]].

## The questions as asked (answered in "Decided: Q1-Q4" above)

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
