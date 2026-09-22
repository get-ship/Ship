---
title: What is Sail
description: Sail is the framework Ship views are authored in — declarative templates, optional reactive controllers, no build step.
section: Building views
order: 1
kicker: Ship's view framework
tags: [sail, views]
related: [theming]
---

**Sail** is the framework every Ship view and built-in component is authored in. A view is a `.sail` **template** — declarative markup — with an optional sibling `controller.ts` that owns reactive state, lifecycle, and async I/O.

The split is the whole idea: **templates declare, controllers do.** The template describes what the surface looks like and binds to state; the controller holds the signals and the logic. Most views need only a template.

## How it runs

The compiler turns a template + controller into a standard ES module ahead of time — there's no parser shipping to the browser. The runtime serves the compiled module plus a server-rendered snapshot, then hydrates it on the client. You don't run a build: in edit mode, save a `.sail` file and it hot-reloads.

## You compose, you don't reinvent

Views are built from a library of Sail primitives — cards, stats, tables, charts, lists, form controls — so a real surface (a dashboard, a queue, a form) is composition, not from-scratch UI. An agent can author a view this way too; it's the same framework either way.

## Learn it hands-on

The fastest way in is the **[interactive tutorial](/docs/ship-ui/tutorial/)** — edit real code, watch it render, no install. Twenty-five steps from a first signal to a server-pushed view.
