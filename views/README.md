<!--
  Canonical tone semantics for the view layer.

  Tones carry meaning, not just color. The same tone must mean the same thing
  on every surface. Pick a tone by what it COMMUNICATES, never by what color
  you want. The palette is closed at eight tones — do not introduce new ones.

  | Tone        | Semantic                                                        |
  |-------------|-----------------------------------------------------------------|
  | default     | Neutral, unscoped (no signal)                                   |
  | primary     | Positive emphasis, current selection, primary action, "brand"  |
  | info        | Informational notice, in-progress state, "you might want to know" |
  | success     | Completion, "this worked", validation passed, active/live state |
  | warning     | Validation issue / advisory, "this needs your attention", recoverable |
  | destructive | Error, irreversible action, "this won't work"                   |
  | accent      | Decorative emphasis (color flair), NOT semantic                 |
  | muted       | Deemphasized, "this is here but quiet"                          |

  Rules of thumb:
  - Current selection / active branch / selected card / primary CTA button → `primary`.
    Reserve `accent` for non-actionable decorative labels (e.g. an "auto-merge"
    config chip), never for buttons that perform an action.
  - In-progress / editing / "validating…" / informational labels → `info`.
  - "active"/"running"/"live" status indicators use `success` + pulse (app-wide
    convention: a successfully-activated, live thing reads green).
  - Advisory form messages → `warning`; blocking form errors → `destructive`.
  - There is no `danger` tone — it falls back to `default`. Use `destructive`.

  Source of this map: .artifacts/job-spec.md (tone-semantics-sweep), derived from
  the Track 2 design audit. Reconcile any new tone usage against this table.
-->

# Ship views

Runtime view surfaces. Each view is a directory under `views/<name>/` holding a
`.sui` template, an optional `.controller.ts`, a `style.css`, and a
`manifest.jsonc`. Templates declare; controllers hold state in their `scope:`
block. See the `ship-views` skill for authoring guidance.

Tone usage across these views follows the canonical map in the comment above.
