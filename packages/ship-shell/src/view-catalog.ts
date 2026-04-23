//! Display metadata for the rail + launcher. Maps view name → icon + label.
//! Phase 2a hardcodes bundled view metadata here. Phase 2b moves this into
//! each view's manifest so new views (including user-authored) declare
//! themselves.

export interface ViewDisplay {
  /** Short display label. */
  label: string;
  /** ship-icon name. */
  icon: string;
  /** Whether this view is pinned to the rail by default. */
  default_pinned: boolean;
  /** Sort order within the pinned rail; lower = higher up. */
  order: number;
}

/** Catalog for the bundled first-party views. */
export const BUILTIN_DISPLAY: Record<string, ViewDisplay> = {
  dashboard:   { label: "Dashboard",   icon: "sparkles",     default_pinned: true,  order: 10 },
  pipelines:   { label: "Pipelines",   icon: "play",         default_pinned: true,  order: 20 },
  agents:      { label: "Agents",      icon: "users",        default_pinned: true,  order: 30 },
  skills:      { label: "Skills",      icon: "zap",          default_pinned: true,  order: 40 },
  git:         { label: "Git",         icon: "git",          default_pinned: true,  order: 50 },
  collections: { label: "Collections", icon: "database",     default_pinned: true,  order: 60 },
  canvas:      { label: "Canvas",      icon: "image",        default_pinned: false, order: 70 },
  events:      { label: "Events",      icon: "sparkles",     default_pinned: false, order: 80 },
  specs:       { label: "Specs",       icon: "file",         default_pinned: false, order: 85 },
  overview:    { label: "Overview",    icon: "git",          default_pinned: false, order: 90 },
  chat:        { label: "Chat",        icon: "message-square-plus", default_pinned: false, order: 100 },
  settings:    { label: "Settings",    icon: "settings",     default_pinned: true,  order: 1000 },
};

/** Fallback display for views without catalog entries (user-authored). */
export function displayFor(name: string): ViewDisplay {
  return (
    BUILTIN_DISPLAY[name] ?? {
      label: name.replace(/-/g, " ").replace(/\b\w/g, (c) => c.toUpperCase()),
      icon: "app-window",
      default_pinned: false,
      order: 500,
    }
  );
}

export function defaultPinned(): string[] {
  return Object.entries(BUILTIN_DISPLAY)
    .filter(([, d]) => d.default_pinned)
    .sort(([, a], [, b]) => a.order - b.order)
    .map(([name]) => name);
}
