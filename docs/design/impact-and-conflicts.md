# Impact and conflicts

## The impact registry

Every planned task declares its impact:

```
bridle impact set tw-7fa2 \
  --modify s-b310 --add-under r-7fa2 --remove s-11c0 \
  --files 'client-watch/**' 'packages/ratings/**'
```

`bridle impact check` compares every in-flight task and reports:

| Overlap | Level | Action |
|---|---|---|
| Same scenario modified/removed by two tasks | **conflict** | open a `conflict` thread between the claimants |
| Same requirement, different scenarios | warn | notify both; usually fine |
| Same capability, different requirements | info | nothing |
| Same files | warn | early warning (research 09 tier 1) |
| Real textual conflict from `git merge-tree` | **conflict** | same as the first row (research 09 tier 2) |

**Declared impact is checked against actual impact.** When a branch changes,
bridle diffs the spec ids and file paths it touched and flags anything the task
didn't declare. An agent can't avoid a conflict by under-declaring.

## The conflict protocol

1. Bridle opens a `conflict` thread between the two claimants (or the driver,
   for unclaimed tasks) and injects it into both sessions.
2. They decide between them, and one records the outcome:
   - `bridle conflict resolve C12 --compatible "…"`: not a real conflict; the
     reason is recorded.
   - `bridle conflict resolve C12 --order tw-7fa2,tw-a9d0`: adds a `blocks`
     edge. The blocked task's claimant writes a handoff note, releases or
     parks the claim, and takes other ready work.
   - `bridle conflict resolve C12 --merge-into tw-7fa2`: one task absorbs the
     other's scenario change.
3. If they disagree, or the conflict is a product question rather than an
   ordering question, it escalates to the driver, and to the human only if it's
   a product question.
4. **After a merge**, bridle sends `system: spec changed under you` to every
   in-flight task whose impact overlaps what just merged, so those agents rebase
   and re-read before building on stale text.

This replaces the same-spec pile-up rule. Serialisation happens only when two
tasks actually collide, and the agents involved decide the order.
