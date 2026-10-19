---
id: br-qwmd
title: "Spec-writing guidance: scenarios read as intent, not mechanics; file formats live in their own spec"
kind: docs
created: 2026-10-19
created_by: external:advisor/tj7q
theme: agents-and-cli
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-2ae8, br-ewk8, br-tj7q]
hold: design
---

## The ask

From the human's review of br-2ae8's specs stage (hands-on try br-ewk8), 2026-10-19, to advisor tj7q. The human is still thinking this through; this ticket holds their words until they shape it. Today's guidance is rule `stage-specs` (`workflow/base/rules/stage-specs.md`) and `docs/design/specs.md`.

The human, verbatim:

> I want to probably iterate on the specs a little bit. I think they're okay, but I've been thinking a lot about what an executable spec should look like and how it should read, especially as compared to what I've seen in some Gherkin specs. I'm still kind of thinking about this, but there are some things I like about these specs and some things that I'm a little concerned with. Some of this, I think, needs to be formalized for the rules and guidance in writing specs.

> One thing that I do like is that they're very readable to a human. I think that's really, really positive and something that we want to retain in the future. What I mean is that one of the first specs says there's a focus time and there's a clock inside of it. That's very readable when it's conveying the intent and not necessarily the mechanical part of it.

> I find when engineers, and sometimes the agents too, write a Gherkin scenario, they tend to write it extremely mechanically, like a unit test. I think the thinking is that if I write it extremely generically, step by step, it'll be reusable for a bunch of different tests and a bunch of different scenarios. That's completely true, but the problem is it masks the intent of what the test is actually saying. [...]

> You often see this with tables. Engineers will always add a bunch of tables to their Gherkin scenarios because if you make a table, you can put anything in there. It's just like a function call, right? I'm calling this function with this table to create all this test data. Then every step in the scenario is tied to this explicit test data or these explicit settings, without expressing why.

> [...] They'll list them, like Monday through Friday, 8 am to 5 pm, and then Saturday and Sunday, whatever. Then they'll say, "And the clock is set to 2:30 pm then." It'll say, "Okay, then the output should be this." Sure, it's perfectly correct, but it's speaking very mechanically and less on the intent of the test.

> I think I want to encourage the specs to be written to assume sort of an understanding, or to be readable by a human. We have these agents that can write a bunch of code for us, so having a lot of steps might be okay. I'm still trying to make this work in my head about what the steps that implement this actually look like.

> For example, if we want to make this more mechanical, I'd split. I'm thinking about suggesting splitting the first one to say, "When there's a focus period defined in this time range and the clock is a certain time," or you could say, "When there's a focus period defined in this time range and the clock is inside that focus period." That requires a little bit of coordination in the steps, but there's a context variable. You can stick stuff in there. That's how I usually do it when I'm writing these.

> I'm also considering maybe the steps should be written with a lot of ability to just eat up a lot of spare text. Don't specify all the text very clearly. Leave a lot of dot-open-text places, but where the important thing is captured and extracted. You could say, instead of saying the clock is at 5 pm on Tuesday or 3:30 on Tuesday, "the clock is within the focus period at 3:30 on Tuesday." You're able to describe what you mean while also specifying the exact time.

> Leave it a little bit less up to chance, but you're also specifying the same thing in two places. It's a little bit weird. I don't know. Again, I'm still thinking about this.

> In general, I mostly like how the specs are in, although they could be even less prescriptive in terms of the format of the file. I'm not sure. On the one hand, it is good to have a specification for the file. On the other hand, the majority of the specs shouldn't have anything to do with the format of the file. We should have a specific spec that defines the format of the file. I think the other specs should just say there's a focus period between Monday and Friday, between 8 and 8:05. They shouldn't have to specify any of the details of exactly what the file looks like, so that if that specification changes, only the specs that control the shape of the file would be the ones that update.

Themes so far (advisor's reading, for the human to correct):
- Scenarios state intent first and read well to a human; keep that.
- No mechanical, unit-test-style steps or data tables that hide why.
- Steps may carry free text around the extracted values ("the clock is within the focus period at 3:30 on Tuesday"), with shared context between steps.
- Behaviour specs don't spell out file formats (or, likely, exact output text); one spec owns each format, so a format change touches only that spec.

The ask: once the human has shaped it, turn this into guidance for spec writers (rule `stage-specs`, `docs/design/specs.md`, maybe the template) and, where it matters, for the step implementations. hold: design until the human says go.
