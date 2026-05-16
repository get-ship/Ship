---
group: Smart Skills
title: Variables
description: Make one skill produce different output per user, per project, and per machine.
order: 2
---

Variables let one skill produce different output for different users and different projects. You declare what's variable in `assets/vars.json`; the runtime fills in the values when the skill is compiled.

## A small example

A `code-review` skill that adapts to your team's commit style:

```jsonc
// .ship/skills/code-review/assets/vars.json
{
  "commit_style": {
    "type": "enum",
    "values": ["conventional", "gitmoji", "angular"],
    "default": "conventional",
    "storage-hint": "project"
  }
}
```

Inside `SKILL.md`:

```markdown
Commits should follow {{ commit_style }} style.
```

When the runtime renders this skill, it substitutes the active value before handing the result to the agent.

## Where a variable lives

Each variable picks one home via the `storage-hint` field:

<sui-card padding="md">
<strong slot="header">Global</strong>
Stored on your machine. Travels with you across all projects. Use this for personal preferences (e.g. preferred tone of voice).
</sui-card>

<sui-card padding="md">
<strong slot="header">Local</strong>
Stored for one project on this machine. Personal, not shared. Use this for things you want different for one project but only for yourself.
</sui-card>

<sui-card padding="md">
<strong slot="header">Project</strong>
Committed to <code>.ship/</code>. Shared with everyone working in the project. Use this for team conventions (e.g. commit style, branch naming).
</sui-card>

The author of the skill picks the storage-hint that fits the variable. There's no merge or override across scopes — each variable lives in exactly one place.

## Where to set values

The Ship app has a Skills surface where each variable shows up with a typed input — a toggle for booleans, a dropdown for enums, a text field for strings. You set values there; the runtime stores them in whichever scope the variable was declared with.

Agents can also set variables at runtime when it makes sense (for example, capturing a preference the user just expressed in chat).

## Types

Variables can be `string`, `bool`, `enum`, `array`, or `object`. Pick the type that matches what you're configuring. The Ship app renders an appropriate input for each.
