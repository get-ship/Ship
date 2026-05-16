---
title: "Composing UI primitives"
description: "Using the built-in <sui-*> components in your app."
group: "Tutorial"
order: 8
slug: "08-sui-primitives"
prev: "07-event-modifiers"
next: null
chapter: "components"
---

Ship ships a primitives library — `<sui-button>`, `<sui-card>`,
`<sui-input>`, `<sui-tabs>`, `<sui-status-dot>`, `<sui-alert>`, and
about a hundred more. Every one is a SUI custom element with a stable
prop / event / slot contract, and they're already registered globally
on this page (the docs editor you're reading uses `<sui-code-editor>`
the same way).

You don't write `import { Button } from '@ship/ui'` and you don't
declare what tags your template uses. You just type them:

```sui
<sui-card>
  <sui-chip tone="primary" slot="header">draft</sui-chip>
  <p>{body()}</p>
  <sui-button on:click={save} tone="primary" slot="footer">Save</sui-button>
</sui-card>
```

Three things that are good to know:

- **Props are reactive.** Just like native attributes — `<sui-button
  disabled={busy()}>` re-evaluates on every change.
- **Events look native.** `<sui-button on:click={…}>`, no special
  prefix. Component-specific events are also `on:`-prefixed:
  `on:sui-change`, `on:sui-tab-change`.
- **Slots project.** Most components define named slots
  (`slot="header"`, `slot="footer"`, `slot="leading"`); the default
  slot takes the rest. The component owns the chrome — you own the
  content.

## Try it

The dashboard composes `<sui-card>`, `<sui-stat>`, `<sui-status-dot>`,
and `<sui-button>` into a small ops widget. Try:

1. **Pulse on alarm.** When `errorRate() > 5`, set the status dot's
   `tone="destructive"` and `pulse` attribute so it animates.
2. **Add a tone toggle.** Two `<sui-button>`s that flip the card's
   `tone` between `"default"` and `"warning"` — observe how the
   card chrome re-skins to match.
3. **Slot a sui-alert.** When `errorRate() > 5`, render a `<sui-alert
   tone="destructive">` inside the card with a "rate limit exceeded"
   message.
