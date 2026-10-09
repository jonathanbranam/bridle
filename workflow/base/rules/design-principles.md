---
id: design-principles
severity: must
roles: [designer]
---
The principles a design is measured against. Every option the designer writes is
checked against each of these, and the proposal says where an option falls short.

- **KISS and YAGNI** (`kiss`, `yagni`): the simplest thing that meets the need in front of
  us. An option that adds a layer, flag or extension point for a need nobody has yet
  says so as its cost.
- **Modularity**: each part does one thing and can be changed or replaced without
  touching its neighbours. Name what an option couples that wasn't coupled before.
- **One name per action, no near-duplicate commands**: before adding a command, flag or
  endpoint, find the existing one that does the same or nearly the same thing and extend
  or rename that instead (ticket fne2). Two spellings of one action is a defect.
- **The user's side first**: design what the human or agent types, reads and sees
  (the commands, their output, the docs) before how it is built inside. An option is
  judged by how it reads to its user.

## The human's own principles

Add yours below; the designer measures every option against them too. (Empty so far.)
