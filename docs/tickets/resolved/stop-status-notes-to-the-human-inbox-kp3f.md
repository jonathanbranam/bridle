---
id: kp3f
title: Stop status notes piling up in the human's inbox
opened: 2026-09-28
resolved: 2026-09-29
repos: [bridle]
changes: [3bd16dc]
specs: []
needs: []
see: [8ups, fgu6]
closed: 2026-09-30T05:12:44Z
---

## What happened

Filed by the advisor. On 2026-09-28 the human's inbox held 160 unread messages: 150
notes and 10 questions. Notes by sender: manager-1 63, manager-2 60, pm-1 26,
ci-no-failfast 1. They covered about 21 hours (2026-09-27 17:42Z to 2026-09-28 15:01Z).
Nearly all were routine updates ("merged X", "spawned Y", "renewed", "noted"). Of the
10 questions, only three still needed a decision (j2vq, a7h3, sqt6); the rest had
already been settled elsewhere.

The notes come from the role prompts:

- `.bridle/roles/manager.md`: "**Report to the human** with `bridle send human
  "<summary>"`: what was done, on which branch, and anything that needs their
  decision."
- `.bridle/roles/product-manager.md`: "**Report** to the human briefly (`bridle send
  human "<summary>"`) when the queue or priorities change."

Nothing marks them read. Only the recipient can mark a message read (the advisor's
token gets `403 not the recipient of this message`), and there is no delete.

The human, verbatim (2026-09-28):

> I see a lot of notes in an inbox for me in the tui, most look like they are not
> important just updates; I don't want to read those; I don't think we need minor notes
> like this

> I don't want to see a pile up of status updates or messages that I can't act on; If
> the agents need to communicate with each other, that is fine but don' fill my inbox
> with messages. Major updates and version releases could go there, but really shoudl be
> written to a changelog in the repo that I can also review;

## The advisor's recommendation

- The human's inbox is for things the human has to act on: questions, blockers,
  decisions. No routine status notes.
- Agents keep talking to each other as they do now. Progress goes where it can be read
  when wanted: the task's thread (`bridle task note`), `docs/context/orchestrator-state.md`,
  and git.
- Major updates and releases go in a changelog in the repo (there is none yet, e.g.
  `CHANGELOG.md`). At most one short note to the human pointing at it, for a release.
- Update `manager.md` and `product-manager.md` (and any rule in `workflow/base/rules/`
  that says the same) to match.

Worst case if we don't: the inbox stays unreadable, and the questions that do need the
human get lost among the notes (as with the 10 here).

## Notes

- The TUI inbox problems that made this visible: [[tui-inbox-doesnt-scroll-8ups|8ups]],
  [[tui-inbox-open-a-message-in-full-fgu6|fgu6]].
- The inbox could also filter by kind (show questions only by default), but fixing what
  gets sent is the simpler first step.

## Resolution

Resolved by 3bd16dc: the human's inbox restricted to actionable items, and CHANGELOG.md added for release notes.
