---
id: t4rf
title: "bridle-ui: the login form follows web standards so password managers fill and save it"
kind: bug
opened: 2026-10-04
repos: [bridle-ui]
changes: []
specs: []
needs: []
see: [essy]
tasks: []
---

## The ask


The human, verbatim (2026-10-04, to the bridle-ui aide; dictated, "one password" is 1Password):

> Are the username and password fields on the login tagged properly as username and password
> fields? I know we're kind of doing a prototype here, but we don't need to be lazy. Follow web
> standards wherever we're building something. I don't think they're tagged properly because one
> password checks on something, I don't know exactly what, in a web form to see where it should
> insert a username and password. So every time we have a username and password, it should have
> those tags. TrackWeb has it set up properly, it works perfectly there.

## Context

bridle-ui `src/Login.tsx` at 515ad47: each input is wrapped in its `<label>` and has
`autoComplete="username"` / `autoComplete="current-password"`, and the password input has
`type="password"`. Neither input has an `id` or a `name`, the username input has no `type`, and
neither is `required`. The `<form>` has only `onSubmit`.

track-web, which works with 1Password (`packages/auth/src/LoginPage.tsx`): `<label htmlFor="email">`
with `<input id="email" type="email" autoComplete="email" required>`, and `<label
htmlFor="password">` with `<input id="password" type="password" autoComplete="current-password"
required>`.

The human's rule applies beyond this form: "every time we have a username and password, it should
have those tags", and "Follow web standards wherever we're building something".
