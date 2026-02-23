---
title: Hello, World
date: 2026-02-23
subtitle: A first post to test the plumbing
---

This is the opening paragraph. The layout is set in ET Book, the margins are generous, and notes live to the right of the text where they can be read without interrupting the flow.

Here is a sentence with a sidenote.^[This appears numbered in the margin on wide screens. On narrow screens it collapses inline. The syntax is `^[text]`.] The prose continues normally after it.

Margin notes work the same way but carry no number.>[This is a margin note — use `>[text]` for asides that don't warrant a citation number.] Use them for commentary that would interrupt the sentence if parenthetical.

---

## A Second Section

Horizontal rules in the markdown become `<section>` breaks in the HTML, which is the structural unit Tufte CSS expects. Each section can open with a new heading or simply continue the argument.

A block quote looks like this:

> The commonplace book is a way of making the reading permanent, and of making it your own.

Code spans are `monospaced` and fit naturally into the body text without visual shouting.

```js
// fenced code blocks work too
const site = 'arcadia'
```
