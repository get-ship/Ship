// calendar.controller.js — a self-contained MONTH-VIEW twin of sui-calendar.
// The real component is engine-backed (createCalendar: month/week/day view-models,
// agenda, range, drag — see _repo-src/sui-calendar/engine). Porting that whole
// engine isn't the design gap; the gap is the VISUAL month grid. So this is a
// minimal, dependency-free month engine: it builds the same 6×7 MonthCell grid
// the real view-model produces (iso · day · today · outside · events · overflow)
// and wires real prev/next/today nav — faithful to the month surface, no engine.
import { controller, signal, derived } from '@ship/sui-runtime';

const MONTHS = ['January', 'February', 'March', 'April', 'May', 'June', 'July', 'August', 'September', 'October', 'November', 'December'];
const DOW = ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat'];
const TODAY = '2026-06-04';

// Fixture events (daemon-shaped: id · date · title · tone). June 2026.
const EVENTS = [
  { id: 'e1',  date: '2026-06-02', title: 'Sprint planning', tone: 'info' },
  { id: 'e2',  date: '2026-06-04', title: 'Standup',         tone: 'info' },
  { id: 'e3',  date: '2026-06-04', title: 'Design review',   tone: 'accent' },
  { id: 'e4',  date: '2026-06-04', title: '1:1 with Sam',    tone: 'success' },
  { id: 'e5',  date: '2026-06-04', title: 'Deploy window',   tone: 'warning' },
  { id: 'e6',  date: '2026-06-09', title: 'Eng sync',        tone: 'info' },
  { id: 'e7',  date: '2026-06-12', title: 'Release 2.4',     tone: 'success' },
  { id: 'e8',  date: '2026-06-12', title: 'Retro',           tone: 'default' },
  { id: 'e9',  date: '2026-06-18', title: 'Board demo',      tone: 'accent' },
  { id: 'e10', date: '2026-06-24', title: 'Oncall handoff',  tone: 'destructive' },
  { id: 'e11', date: '2026-06-29', title: 'Roadmap review',  tone: 'info' },
];

const pad = (n) => String(n).padStart(2, '0');
const isoOf = (y, m, d) => `${y}-${pad(m + 1)}-${pad(d)}`;

export default controller({
  scope: () => {
    const focus = signal({ y: 2026, m: 5 }); // June (0-indexed)

    const title = derived(() => `${MONTHS[focus().m]} ${focus().y}`);
    const weekdays = derived(() => DOW.map((l, i) => ({ key: 'dow-' + i, label: l })));

    const weekRows = derived(() => {
      const { y, m } = focus();
      const startDow = new Date(y, m, 1).getDay();
      const start = new Date(y, m, 1 - startDow);
      const rows = [];
      for (let r = 0; r < 6; r++) {
        const cells = [];
        for (let c = 0; c < 7; c++) {
          const d = new Date(start);
          d.setDate(start.getDate() + r * 7 + c);
          const cy = d.getFullYear(), cm = d.getMonth(), cd = d.getDate();
          const iso = isoOf(cy, cm, cd);
          const outside = cm !== m;
          const today = iso === TODAY;
          const all = EVENTS.filter((e) => e.date === iso);
          const shown = all.slice(0, 3).map((e) => ({ id: e.id, title: e.title, cls: 'evt evt-' + (e.tone || 'default') }));
          const overflow = Math.max(0, all.length - 3);
          cells.push({
            key: iso, iso, day: cd, outside, today,
            cls: 'cell' + (today ? ' today' : '') + (outside ? ' outside' : ''),
            events: shown, overflow, hasOverflow: overflow > 0,
          });
        }
        rows.push({ key: 'row-' + r + '-' + cells[0].iso, cells });
      }
      return rows;
    });

    return { focus, title, weekdays, weekRows };
  },

  mount(state, host) {
    const onClick = (e) => {
      const nav = e.target.closest && e.target.closest('[data-nav]');
      if (!nav) return;
      const act = nav.dataset.nav;
      const { y, m } = state.focus;
      if (act === 'prev')  state.focus = m === 0  ? { y: y - 1, m: 11 } : { y, m: m - 1 };
      else if (act === 'next')  state.focus = m === 11 ? { y: y + 1, m: 0 }  : { y, m: m + 1 };
      else if (act === 'today') state.focus = { y: 2026, m: 5 };
    };
    const sr = host.shadowRoot;
    sr && sr.addEventListener('click', onClick);
    return () => { sr && sr.removeEventListener('click', onClick); };
  },
});
