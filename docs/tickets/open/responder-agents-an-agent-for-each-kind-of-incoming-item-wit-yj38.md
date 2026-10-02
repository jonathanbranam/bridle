---
id: yj38
title: "Responder agents: an agent for each kind of incoming item, with guidelines fit to who sent it"
kind: question
opened: 2026-10-02
repos: [bridle]
changes: []
specs: []
needs: []
see: [x8jt, pdmd, 93xm, 2bzw]
tasks: [br-021e]
---

## The ask


Research and open questions, not a build. Raised while discussing
[[review-a-document-with-an-agent-highlight-comment-and-the-ag-x8jt|x8jt]] (which agent answers
the human's comments on a document). The human, verbatim (2026-10-02, via the advisor):

> But this, that agent handling something like this is going to be another repeated things that
> comes up. Not an interactive agent, but a specific kind of agent to respond to something. Some
> other examples. Let's include some of this here, but then also let's create another ticket as a
> kind of research or open questions thing. But that other kinds of agents that would be similar
> to this would be agents that respond to GitHub issues that are filed or other bug reports that
> come in from external sources. And another one would be like the orchestrator does a lot of
> that today, and that seems pretty inappropriate, like a bad sharing of responsibilities for the
> orchestrator. The orchestrator handles it today because he's always up and ready and reading
> messages. So I think in the future we want to have another agent who's available to do those
> types of things. Yeah, what else? I think that... Oh, yeah, I, as we go forwards, I want to open
> up some of my games work to allow some of my family members and trusted friends to be able to
> interact with the agents that are responsible for them. So an example is both my son and
> daughter help me and are, are the driving force besides making some of these game projects. And
> I want them to have the ability to... file some sort of ticket or feedback or comment on the
> behavior of the game and I want to have that triaged by an agent you know we need to have and
> that needs to have different guidelines than coming from me I'm a human the owner of the project
> and an engineer and these people aren't you know my son is an adult and and certainly very smart
> but he's not an engineer but my daughter is young and will have wild ideas and has no idea
> what's possible. That's part of the fun here, but she should be able to give a lot of her input
> and ideas. But we need an agent who's able to understand, like, you're speaking to a child, and
> this child, you know, isn't going to understand technical language and may ask for things that
> are just wild and impossible. So that's kind of a different... type of understanding and
> interaction. Similarly to like externally filed bug reports, you know, let's say people use
> Bridal and file bug reports. We don't just trust everything that comes in. We don't implement
> every feature that somebody asks for. We need kind of a reasoning hand behind that to review
> them. I would trust an agent to be able to discern, you know, what kinds of things would need my
> specific input and decision making and what kind of things would need, could, you know, are
> legitimate and reasonable feature requests and bugs that could just be directly implemented.

## The pattern

A **responder**: not an interactive session and not the orchestrator, but an agent of a
specific kind that bridle starts when something comes in, to answer it. Examples the human gave:

1. **Comments on a document** the human is reviewing (x8jt): an agent for that document.
2. **GitHub issues and bug reports from outside**, e.g. people using bridle.
3. **What the orchestrator handles today only because it's always up and reading messages**:
   a poor split of responsibilities (fits
   [[a-leaner-orchestrator-pdmd|pdmd]]).
4. **Feedback from family and trusted friends on the game projects**, e.g. the human's son (an
   adult, not an engineer) and daughter (young, wild ideas, no idea what's possible: her input
   is wanted and part of the fun).

Each needs guidelines fit to the sender: the owner and engineer, a non-engineer adult, a child
(no technical language; wild or impossible asks handled kindly), or an untrusted outsider (not
everything is trusted, not every feature is built). The agent should judge what needs the
human's decision and what is a reasonable bug or feature to schedule directly.

## Open questions

- What starts a responder (a daemon rule on a kind of incoming item?), how long it lives, and
  whether one handles one item, one document or one stream.
- How the sender's role and guidelines are set: per principal, per channel, per project.
- How outsiders reach it (GitHub, email via rs7p, a web form in the UI) and what they're
  allowed to see; provenance (2bzw).
- Triage outcomes: answer, file a ticket, schedule a task, or ask the human; and where the
  record lives.
- Related: [[other-projects-submit-bridle-tickets-directly-for-triage-93xm|93xm]] (submitted
  tickets triaged, not blindly accepted).
