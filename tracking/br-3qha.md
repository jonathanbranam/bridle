+++
id = "br-3qha"
title = "What should each ticket watcher be woken for? Notices are 69% of the messages that wake the Opus sessions"
kind = "question"
state = "pending"
created_at = "2026-10-20T00:02:27.786Z"
updated_at = "2026-10-20T00:04:27.148153Z"
created_by = "external:advisor/usage"
watchers = ["external:advisor/usage"]
ticket = "3qha"
+++

docs/tickets/open/what-should-each-ticket-watcher-be-woken-for-notices-are-69-3qha.md

## Thread

### note · external:advisor/usage · 2026-10-20T00:04:27.148Z
From the human, via advisor (usage), 2026-10-19 ~8:10 PM ET:

> Right. Some state changes aren't important. But eg knowing when a ticket lands is usually important. But not for every switch.

What the 1,047 notices this week were, by change (bridle's daemon, to interactive sessions):

- **comment by ...: 507 (48%).** The largest group. Many are workers' "done:" reports and pm-1's briefs.
- **Routine steps: about 370 (35%).** open -> planned 116, planned -> claimed 103, claimed -> built 92, pending -> open 41, planned -> built 15.
- **Landing, `-> integrated`: about 127 (12%).** built -> integrated 106, plus planned/claimed/pending -> integrated. The human wants these.
- **Endings and reversals: about 30 (3%).** Dropped 14, reopened 14.

So waking a session only for landing, dropped and reopened, plus questions and answers addressed to it, would cut about 85% of the watch notices. The routine steps and most comments could go to a digest that the next real wake or `bridle inbox` shows. Which comments still wake (a "sent back", a hold's reason, a question) is part of the design.
