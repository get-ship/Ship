# Theme scaffolds

Canonical theme starters for Shipwright's `/create theme` flow.
Copy a scaffold into `<ship-dir>/themes/<id>.json`, substitute
`{{NAME}}` and `{{TITLE}}`, then iterate via Theme Studio.

## Shapes

| Shape           | Use when                                                     |
| --------------- | ------------------------------------------------------------ |
| `light`         | A clean light theme. Default for most users.                 |
| `dark`          | Dark theme with muted chrome.                                |
| `brand-accent`  | Keep the shell neutral; express the brand via `primary`.     |
| `high-contrast` | Accessibility-first. Exceeds WCAG AAA on every pair.         |

## Completion criteria

A theme scaffold is "done" when it:

1. Validates against `schemas/theme.schema.json`.
2. Passes `/api/themes/contrast` at the required minimum for every
   canonical pair (body 4.5, interactive 3.0 by default).
3. Renders legibly in the Theme Studio preview.

If contrast fails on a pair, adjust the weaker side rather than
raising the minimum. The minimums exist so we never ship illegible
themes by default.
