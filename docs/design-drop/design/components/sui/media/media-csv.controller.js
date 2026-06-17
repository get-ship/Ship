// media-csv.controller.js — live CSV → table parser for the media-csv renderer.
// NOTE: media renderers were NOT in the imported _repo-src, so this is a design
// PROPOSAL faithful to the documented 3-renderer split (image · csv · av), not a
// source port. The parse is self-contained (no daemon): split rows, flag numeric
// cells for right-alignment. Pairs with media-csv.after.sui via setupUrl.
import { controller, derived } from '@ship/sui-runtime';

const isNum = (v) => v !== '' && v !== '—' && /^-?[\d.,]+%?$/.test(v);

function parseCSV(text) {
  const lines = text.replace(/\r/g, '').trim().split('\n').filter((l) => l.length);
  const split = (l) => l.split(',').map((s) => s.trim());
  const headers = lines.length ? split(lines[0]).map((h, i) => ({ key: 'h' + i, label: h })) : [];
  const rows = lines.slice(1).map((l, ri) => ({
    key: 'r' + ri,
    cells: split(l).map((v, ci) => ({ key: ri + '-' + ci, v, cls: isNum(v) ? 'num' : '' })),
  }));
  return { headers, rows };
}

export default controller({
  scope: ({ data }) => {
    const parsed = derived(() => parseCSV(data() || ''));
    const headers = derived(() => parsed().headers);
    const rows = derived(() => parsed().rows);
    const count = derived(() => parsed().rows.length);
    return { headers, rows, count };
  },
  mount() { return () => {}; },
});
