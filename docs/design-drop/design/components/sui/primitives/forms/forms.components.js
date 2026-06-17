/* forms.components.js — pack OVERVIEW for the forms primitives.
   Same ComponentViewer surface as the cross-pack review, scoped to the 4 forms
   components. Renders are reused from ../../renders.js (window.SUIR). The
   per-component verdict + finding persist; this is the "where is this pack" view. */
(function () {
  "use strict";
  const R = window.SUIR;
  const svg = R.svg, I = R.I;
  const api = (p, e, s) => ({ props: p || [], events: e || [], slots: s || [] });

  ComponentViewer.mount({
    eyebrow: 'components/sui · forms',
    title: 'Forms — pack overview',
    domainAccent: 'var(--app-accent-collections)',
    storagePrefix: 'suifw',           /* shares verdicts with the cross-pack review */
    intro: 'The <b>forms</b> pack — zero runtime dependency, so fidelity here is exact. This is the pack\u2019s <b>overview</b>: every component, its states, across all six ship themes. To <b>iterate</b> on a component\u2019s UI/UX, open its canvas (links below); to ship a decision, see <code>handoff.md</code>.',
    links: [
      { label: '\u2190 All packs', href: '../../overview.html', ghost: true, icon: svg(I.grid, 1.8) },
      { label: 'Iterate: combobox \u2192', href: 'combobox.html' },
      { label: 'handoff.md', href: 'handoff.md', ghost: true },
    ],
    components: [
      { id: 'combobox', tag: 'sui-combobox', badges: ['new', 'forms'], state: 'open', states: ['closed', 'open', 'filtered', 'empty'], render: R.combobox,
        purpose: 'Type-to-filter <b>single select</b>. The keyboard + ARIA reference for the pack. <b>Direction B chosen</b> \u2014 3 directions explored; see handoff for the locked spec.',
        review: { sev: 'pass', label: 'Decided · B', text: '<b>Direction B</b> (quiet + meta): muted mono <code>service-id</code> per row, primary left bar on selected. Keeps the keyboard/ARIA model verbatim. Spec locked in <code>handoff.md</code>; ready for the coding agent.' },
        api: api([['value', 'string', 'Selected option value.'], ['options', 'json|<option>', 'Inline JSON or slotted options.'], ['disabled', 'boolean', '']], [['sui-change', '{value,label}', 'Commit on enter/click.']]) },
      { id: 'autocomplete', tag: 'sui-autocomplete', badges: ['new', 'forms'], state: 'suggesting', states: ['suggesting', 'loading'], render: R.autocomplete,
        purpose: 'Free-text field with <b>async suggestions</b>. Typed value is valid on its own \u2014 suggestions augment, they don\u2019t constrain.',
        review: { sev: 'change', label: 'Consistency', text: 'Pending + empty patterns must match whatever combobox lands on. Queued behind the combobox decision \u2014 it sets the pack\u2019s overlay language.' },
        api: api([['value', 'string', 'Current text.'], ['min-chars', 'number', 'Threshold before querying.'], ['debounce', 'ms', '']], [['sui-input', '{text}', ''], ['sui-select', '{value}', '']]) },
      { id: 'date-picker', tag: 'sui-date-picker', badges: ['new', 'forms'], state: 'open', states: ['closed', 'open', 'range'], render: R.datepicker,
        purpose: 'Text field + calendar popover sharing the <code>sui-calendar</code> grid engine. <code>range</code> flips to two-ended selection.',
        review: { sev: 'polish', label: 'Polish', text: 'Add a parse-format hint so typed entry is discoverable. Inherits the combobox field treatment once chosen \u2014 keep the field chrome identical across the pack.' },
        api: api([['value', 'ISODate', 'Selected date.'], ['range', 'boolean', 'Two-ended select.'], ['min / max', 'ISODate', '']], [['sui-change', '{value} | {start,end}', '']]) },
      { id: 'color-picker', tag: 'sui-color-picker', badges: ['new', 'forms'], state: 'swatches', states: ['swatches', 'custom'], render: R.colorpicker,
        purpose: 'Swatch grid bound to the <b>app-accent token ramp</b>, with an OKLCH custom mode. Drives per-app theming.',
        review: { sev: 'pass', label: 'Token \u2713', text: 'Stores OKLCH not hex \u2014 keep it. Only forms component already settled; share its popover elevation + radius with the rest once the field treatment is locked.' },
        api: api([['value', 'oklch|token', 'Resolved color or token ref.'], ['palette', 'token[]', 'Curated swatch ramp.'], ['allow-custom', 'boolean', '']], [['sui-change', '{value}', '']]) },
    ],
  });
})();
