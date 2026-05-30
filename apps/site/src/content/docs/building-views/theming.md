---
title: Theming
description: Ship themes are full aesthetic bundles — palette, surfaces, radii, motion, type — not recolors. Seven built in, switchable at runtime.
section: Building views
order: 2
kicker: Ship's view framework
tags: [sui, themes, design-system]
related: [what-is-sui]
---

Ship ships a **full design system**, not a color palette. A theme isn't a swap of one `--primary` — it's a coordinated change to palette, surface depth, corner geometry, motion curves, shadows, typography, and interaction strength. Switch themes and the app *feels* different within a second; your views ride along without edits.

## The seven built-in themes

| Theme | Feel | Good for |
|---|---|---|
| **SHIP Light** | Canonical — warm off-white, amber primary, soft shadows | Default, day-to-day |
| **SHIP Dark** | The same warmth after sundown | Night work |
| **Paper** | Editorial cream + terracotta, rounded, calm motion | Reading, writing, docs |
| **Halo** | Glassy pale-blue, spring motion, snappy | Fast UI work, premium feel |
| **Console** | 90s beige-box nostalgia — tan/amber, teal accent, sharp | Terminal-native, tactile |
| **High Contrast** | Near-black + amber, pure white text, blocky | Low-vision, presentation |
| **Glass** | Translucent surfaces, soft gradients, generous radii | Showcase, demos |

## What a theme actually changes

A theme can bend palette, **surface depth** (how nested panels separate), **radii**, **motion** (easing + durations), **shadows**, **typography roles**, and **interaction strength** (hover/active/selected tint). That's why it reads as a different feel, not a recolor.

## Switching and customizing

The **theme picker** in the top bar is the canonical control — a flat grid of live theme cards; click one and it applies immediately, persisting per browser. To go further you have two paths: override individual CSS custom properties for a tweak, or author a full custom theme. Custom apps you build with Ship inherit the active theme automatically.
