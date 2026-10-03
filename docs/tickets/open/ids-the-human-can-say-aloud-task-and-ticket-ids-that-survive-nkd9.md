---
id: nkd9
title: "IDs the human can say aloud: task and ticket IDs that survive dictation"
kind: feature
opened: 2026-10-03
repos: [bridle]
changes: []
specs: []
needs: []
see: [k7tm]
tasks: []
---

## The ask


A feature that needs design first (no task). The human, verbatim (2026-10-03, via the advisor):

> I've got another update, I think maybe for both the context naming document and possibly for a
> separate ticket. Dictating task IDs is troublesome. Wispr Flow's maybe better at this, but
> Claude's transcription fails pretty regularly, and some IDs are just utterly ambiguous in
> English, like the difference between the task ID, the number 3, the letter D, and then the
> number 4 and the letter D. There are probably other ones like this, but when I say that out loud,
> it's 3D, 4D. I'm going to go ahead and send this over to Wispr Flow and see what happens, but it
> basically sounds identical to 30, 40.

## Today

- **Task IDs** are the project prefix plus 4 random hex characters (`br-e5f1`;
  `new_task_id`/`random_hex` in `crates/bridle-daemon/src/store.rs`). Hex is `0-9a-f`: the
  digits, plus b, c, d, e, which all rhyme with "ee".
- **Ticket IDs** are 4 characters from `abcdefghjkmnpqrstuvwxyz23456789` (`docs/README.md`), which
  drops 0/o and 1/i/l for reading but not for saying.
- Neither was chosen to be spoken. A dictionary (Wispr Flow) can't fix random IDs: there's no
  fixed word to add.

## Sound-alikes (examples)

- A digit then a letter vs a number: "3d" / "30", "4d" / "40", "8e" / "80"; "6t" / "sixty".
- Letters rhyming with "ee": b c d e g p t v z (and "3" vs "e" in some accents).
- m / n; f / s; a / 8 ("a" / "eight"); j / k / a.
- Letters that are words: c (see), u (you), y (why), r (are), i (eye), o (oh), b (be), t (tea);
  digits that are words: 2 (to, too), 4 (for), 8 (ate).

## Options for the design (advisor's, not decided)

1. **A speakable alphabet:** drop the sound-alike letters and keep only one of each group, and
   maybe no digits. Short, but a small alphabet needs longer IDs.
2. **Word IDs:** two or three short common words (`amber-otter`), like Docker container names.
   Easy to say and to add to a dictionary; longer to type and wider in tables (see
   [[docs/context/naming|naming]]: short names in lists).
3. **Speak the name, not the ID:** refer to tickets and tasks by the start of their slug or title
   (the `ticket-references` rule already leads with the file name's start), and have the CLI
   accept a unique prefix of the title or slug wherever it takes an ID.
4. **NATO-style reading:** agents read IDs back phonetically ("bravo-echo-five-foxtrot") when
   confirming. Fixes the agent side only, not the human saying it.

Option 3 needs no new ID scheme and suits the naming doc's "short in tables"; 1 or 2 if a
dictatable ID is wanted too.

## More from the human (2026-10-03, verbatim)

> One possible solution to this would be to avoid homonyms in English, but I think that's going to
> be a problem. Another option is that I learn the international alphabet for letters, and that's
> certainly something I could figure out. I have a feeling the agents would understand. I might
> need an app to drill me on that, but I feel like we only have 4 letters today, in that ticket or
> right now. Update the ticket and tell me how many combinations we have today.
>
> I'm interested in the number of collisions we have, and I think there are too many collisions
> that would be a problem. Wispr Flow handled that perfectly. I spoke a little bit slowly, so this
> time I'll speak a little bit faster. Please give me an update on the ticket: 3D, 4D. Thank you.
> That was just a test.

So: Wispr Flow transcribed "3D, 4D" correctly, spoken fast and slow; Claude's transcription is
the one that fails. Two more options:

5. **Avoid sound-alikes in the alphabet** (option 1): the human expects English has too many.
6. **The human learns the NATO alphabet** ("alpha, bravo, charlie, delta"), maybe with a drill
   app; agents understand it already. Works with today's IDs.

## The numbers (advisor, 2026-10-03)

Both kinds of ID are 4 characters today.

| | Alphabet | Combinations | In use |
|---|---|---|---|
| Task IDs (`br-` + hex) | 16 (`0-9a-f`) | 16^4 = 65,536 per project | 474 tasks (0.7%) |
| Ticket IDs | 31 | 31^4 = 923,521 | 241 tickets and spikes (0.03%) |

**Exact collisions never happen:** a new task ID that's taken is retried (`TASK_ID_ATTEMPTS`,
`store.rs`), and `ticket new` checks for collisions. The space is far from full.

**Sound-alike collisions do happen.** Counting b/c/d/e (hex) or b/c/d/e/g/p/t/v/z and m/n (tickets)
as one sound each, the alphabets behave like about 9 distinct sounds, so 4 characters give about
7,000 distinguishable IDs, not 65,536 or 923,521. Expected pairs of IDs that sound alike today:

- tasks: about **16** pairs among 474;
- tickets: about **4** pairs among 241.

That's before digit-letter run-ons like "3d" / "30", which add more. It grows with the square
of the count: at 1,000 tasks, about 70 pairs.

With NATO letters (option 6) every character is distinct when spoken, so these collisions go to
zero without changing any ID.
