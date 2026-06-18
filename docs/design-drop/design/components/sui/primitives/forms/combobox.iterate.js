/* combobox.iterate.js — three UI/UX DIRECTIONS for sui-combobox, as static
   render functions (no runtime; fidelity is exact). Each takes a state and
   returns HTML. Consumed by combobox.html on the design canvas, and the chosen
   one is specced in handoff.md for the coding agents. window.CBX. */
(function () {
  "use strict";
  const svg = (p, w) => '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="' + (w || 1.7) + '" stroke-linecap="round" stroke-linejoin="round">' + p + '</svg>';
  const I = {
    search: '<circle cx="11" cy="11" r="8"/><path d="M21 21l-4.3-4.3"/>',
    chev: '<path d="M6 9l6 6 6-6"/>', check: '<path d="M20 6L9 17l-5-5"/>',
    dot: '', enter: '<path d="M9 10l-5 5 5 5"/><path d="M20 4v7a4 4 0 0 1-4 4H4"/>',
  };
  // service options: [value, label, kind, status]
  const OPTS = [
    ['agent-router', 'Agent Router', 'service', 'ok'],
    ['mesh-gateway', 'Mesh Gateway', 'service', 'ok'],
    ['authority', 'Authority', 'service', 'degraded'],
    ['pipeline-exec', 'Pipeline Exec', 'worker', 'ok'],
    ['substrate-db', 'Substrate DB', 'datastore', 'ok'],
  ];
  const SEL = 'mesh-gateway';
  const dotColor = s => s === 'ok' ? 'var(--success)' : s === 'degraded' ? 'var(--warning)' : 'var(--destructive)';
  const filt = (q) => OPTS.filter(o => o[1].toLowerCase().includes(q.toLowerCase()));
  const hl = (label, q) => q ? label.replace(new RegExp('(' + q + ')', 'i'), '<mark>$1</mark>') : label;

  const wrap = (inner) => '<div class="cbx-frame">' + inner + '</div>';
  const foot = (txt) => '<div class="cbx-cap">' + txt + '</div>';

  /* ── DIRECTION A · Baseline (what exists today) ───────────────── */
  function dirA(state) {
    const open = state === 'open' || state === 'filtered';
    const q = state === 'filtered' ? 'me' : '';
    const empty = state === 'empty';
    const disabled = state === 'disabled';
    const field =
      '<div class="field-input cbx-a-field' + (disabled ? ' is-disabled' : '') + '"' + (open ? ' data-open' : '') + ' role="combobox" aria-expanded="' + open + '">' +
        svg(I.search) + '<span' + (disabled || (!open && !q) ? '' : '') + '>' + (empty ? 'zzz' : q ? 'me' : 'Mesh Gateway') + '</span>' +
        '<span class="chev">' + svg(I.chev) + '</span></div>';
    let list = '';
    if (open && !disabled) {
      if (empty) list = '<ul class="pop cbx-a-pop"><div class="pop-empty">No services match \u201czzz\u201d</div></ul>';
      else list = '<ul class="pop cbx-a-pop" role="listbox">' + filt(q).map((o, i) =>
        '<li class="pop-row" role="option"' + (i === 0 ? ' data-active' : '') + (o[0] === SEL ? ' aria-selected="true"' : '') + '>' +
          hl(o[1], q) + (o[0] === SEL ? '<span class="chk">' + svg(I.check) + '</span>' : '') + '</li>').join('') + '</ul>';
    }
    return wrap('<div class="cbx-lab">A · Baseline</div>' + field + list +
      foot('Current build. Search-icon field, flat list, trailing check. The a11y model to preserve.'));
  }

  /* ── DIRECTION B · Quiet + meta (denser, secondary line) ─────── */
  function dirB(state) {
    const open = state === 'open' || state === 'filtered';
    const q = state === 'filtered' ? 'me' : '';
    const empty = state === 'empty';
    const disabled = state === 'disabled';
    const field =
      '<div class="field-input cbx-b-field' + (disabled ? ' is-disabled' : '') + '"' + (open ? ' data-open' : '') + ' role="combobox" aria-expanded="' + open + '">' +
        '<span class="cbx-b-val">' + (empty ? 'zzz' : q ? 'me' : 'Mesh Gateway') + '</span>' +
        '<span class="chev">' + svg(I.chev) + '</span></div>';
    let list = '';
    if (open && !disabled) {
      if (empty) list = '<div class="pop cbx-b-pop"><div class="pop-empty">No services match \u201czzz\u201d</div></div>';
      else list = '<div class="pop cbx-b-pop" role="listbox"><div class="cbx-b-grp">Services</div>' + filt(q).map((o, i) =>
        '<div class="cbx-b-row pop-row" role="option"' + (i === 0 ? ' data-active' : '') + (o[0] === SEL ? ' aria-selected="true"' : '') + '>' +
          '<span class="cbx-b-bar"></span><span class="cbx-b-main"><b>' + hl(o[1], q) + '</b><span class="cbx-b-id">' + o[0] + '</span></span>' +
          (o[0] === SEL ? '<span class="chk">' + svg(I.check) + '</span>' : '') + '</div>').join('') + '</div>';
    }
    return wrap('<div class="cbx-lab">B · Quiet + meta</div>' + field + list +
      foot('No leading icon; each row carries the service id as a muted mono line. Selected row gets a left accent bar. Denser, power-tool feel.'));
  }

  /* ── DIRECTION C · Command (palette-style) ───────────────────── */
  function dirC(state) {
    const open = state === 'open' || state === 'filtered';
    const q = state === 'filtered' ? 'me' : '';
    const empty = state === 'empty';
    const disabled = state === 'disabled';
    const field =
      '<div class="field-input cbx-c-field' + (disabled ? ' is-disabled' : '') + '"' + (open ? ' data-open' : '') + ' role="combobox" aria-expanded="' + open + '">' +
        svg(I.search) + '<span class="' + (open || q || empty ? '' : 'cbx-c-val') + '">' + (empty ? 'zzz' : q ? 'me' : open ? 'Search services\u2026' : 'Mesh Gateway') + '</span></div>';
    let list = '';
    if (open && !disabled) {
      const body = empty
        ? '<div class="pop-empty">No services match \u201czzz\u201d</div>'
        : '<div class="cbx-c-sec">Services</div>' + filt(q).map((o, i) =>
            '<div class="cbx-c-row pop-row" role="option"' + (i === 0 ? ' data-active' : '') + (o[0] === SEL ? ' aria-selected="true"' : '') + '>' +
              '<span class="cbx-c-dot" style="background:' + dotColor(o[3]) + '"></span><span class="cbx-c-lab">' + hl(o[1], q) + '</span>' +
              (o[0] === SEL ? '<span class="cbx-c-kbd">\u21b5</span>' : '') + '</div>').join('');
      list = '<div class="pop cbx-c-pop" role="listbox">' + body +
        '<div class="cbx-c-foot"><span><span class="cbx-c-k">\u2191</span><span class="cbx-c-k">\u2193</span> navigate</span><span><span class="cbx-c-k">\u21b5</span> select</span><span><span class="cbx-c-k">esc</span> close</span></div></div>';
    }
    return wrap('<div class="cbx-lab">C · Command</div>' + field + list +
      foot('Search-forward, larger targets (44px), per-service status dot, keyboard hints in a footer. Spotlight feel; best when the set is large or unfamiliar.'));
  }

  window.CBX = { dirA, dirB, dirC };
})();
