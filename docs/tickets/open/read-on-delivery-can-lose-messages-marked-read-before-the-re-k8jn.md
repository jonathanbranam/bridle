---
id: k8jn
title: "Read on delivery can lose messages: marked read before the reply arrives, across machines and with the all-projects wait"
kind: bug
opened: 2026-10-03
repos: [bridle]
changes: []
specs: []
needs: []
see: [rmpq, cy2v, fbfy, r8kv]
tasks: []
---

## The ask


The human, verbatim (2026-10-03, via the advisor), after an advisor on the NUC messaged dalek's
advisor on the NUC's notes daemon and it went unseen:

> I'm confused on the design here. Aren't messages delivered by bridle from one machine to
> another?

> But also separate from wakes do you read messages on start from other machines?

> I'm concerned how this affects the decisions about sending messages and wakes and marking
> messages as read when delivered. How does that work between machines?

## Today (advisor, at 21bd096)

- Each project's daemon keeps its own messages and read state; daemons never forward messages or
  wakes. Another machine's session sends as a visitor (`<name>@<machine>`) straight to the
  daemon it targets, and the message waits there.
- Since rmpq (br-ee78, 21bd096), `GET /v1/wake` for a non-human caller takes the unread messages
  and marks them read (`Store::take_unread_messages`, `principal_wake.rs`) **before** the reply
  is sent. If the reply never reaches the caller (connection drop, laptop sleep, network blip,
  daemon restart, the CLI cancelled), the messages are read and nobody saw them. Small on one
  machine; real across machines.
- The planned `bridle agent wake --all-projects` (cy2v, br-1ddd) long-polls every daemon and
  exits on the first answer. A second daemon answering at nearly the same moment has already
  marked its messages read when the CLI drops its reply. cy2v's "wakes on other daemons stay
  queued, so exiting on the first one loses nothing" predates rmpq and no longer holds.
- A daemon nobody watches keeps messages unread indefinitely, unnoticed, and the same session
  has a different principal on each daemon (`external:advisor` on bridle's,
  `external:advisor@dalek` on the NUC's), so "my inbox" is several unconnected inboxes. That
  part is seats ([[seats-named-interactive-roles-splitting-the-advisor-retiring-r8kv|r8kv]]) and
  [[build-our-own-message-and-event-delivery-or-use-an-existing-fbfy|fbfy]].

## Proposal (advisor's)

- **Mark read on acknowledgement, not on sending.** The wake returns the messages; the CLI
  acknowledges the ids it actually printed (a second call), and only then are they read. A lost
  reply means redelivery next time, never a loss. Same for `bridle inbox` listing.
- Settle this before cy2v is built: its fan-out must acknowledge, or print every reply it got.
