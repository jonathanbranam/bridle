# Gates: where the human is and isn't

Each gate is configured, not hardcoded:

```toml
[gates.plan]
default  = "manager"                             # manager reviews plans
human_when = ["impact.protected", "new_capability", "kind == 'arch-revision'"]  # arch-revision is locked
skip_when  = ["kind == 'explore'"]            # explorations are not plan-gated

[gates.merge]
default  = "reviewer+tests"

[gates.accept]
default  = "human"                               # locked in base
batch    = true                                  # human reviews a queue, not each task live
when     = "after-merge"                         # or "before-merge" per project
```

Where each current human check goes:

| Current check | New home |
|---|---|
| Approve every plan | **manager**, unless the plan touches a spec requirement marked `protected`, creates a capability, or the manager chooses to escalate |
| Blocking question stops the lead agent | **async**: the question goes on the task, the task blocks, and the agent claims other work ([questions do not stop work](docs/design/coordination.md)) |
| Archive after "land the work" | **gone**. Specs fold on merge; acceptance is a state change |
| Finish one same-capability change before starting the next | **gone**. The impact registry decides ([impact registry](docs/design/impact-and-conflicts.md)) |
| Accept finished work | **kept**, as a batched queue: `bridle review` |
| *(new)* Revise the architecture | **human, always**, via an `arch-revision` task ([architecture](docs/design/architecture-tier.md)) |
| *(new)* Change a goal's firmness, priority or stance | **human**; agents may propose ([goals](docs/design/goals-tier.md)) |
| *(new)* Divergence introduced by an exploration | **not a human check**. The human hears about it when the exploration concludes ([explorations](docs/design/explorations.md)) |

`when = "before-merge"` is for projects where an unaccepted change on the
integration branch is itself a hazard; harness is the likely candidate. The
default is after-merge, with `reopened` for rejected work.
