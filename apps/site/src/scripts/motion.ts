/**
 * Site-wide motion behaviors. Opt-out via prefers-reduced-motion: reduce.
 *
 * Each behavior is small and additive:
 *   - reveal-on-scroll  (IntersectionObserver → adds .is-revealed)
 *   - card spotlight    (mouse-tracked CSS custom prop on .spotlight cards)
 *   - sticky-nav state  (scroll past hero → adds .is-scrolled on header)
 *   - counter           (data-counter element animates 0 → value on enter)
 *   - bar viz           (.fx-view-bars span gets --h applied on enter)
 *
 * No external deps; uses native browser APIs. Total: < 4kb gzipped.
 */

const reduceMotion = window.matchMedia('(prefers-reduced-motion: reduce)').matches;

/* ── 1. Reveal-on-scroll ─────────────────────────────────────────── */
// Anything with [data-reveal] starts at opacity:0 translate-y, transitions
// to opacity:1 once it crosses the viewport. Children with [data-reveal-child]
// stagger off their parent's reveal time.

const revealObserver = new IntersectionObserver(
  (entries) => {
    for (const entry of entries) {
      if (!entry.isIntersecting) continue;
      const el = entry.target as HTMLElement;
      el.classList.add('is-revealed');
      const children = el.querySelectorAll<HTMLElement>('[data-reveal-child]');
      children.forEach((child, i) => {
        child.style.setProperty('--reveal-delay', `${i * 70}ms`);
      });
      revealObserver.unobserve(el);
    }
  },
  { rootMargin: '0px 0px -10% 0px', threshold: 0.05 },
);

if (!reduceMotion) {
  for (const el of document.querySelectorAll('[data-reveal]')) {
    revealObserver.observe(el);
  }
}

/* ── 2. Card spotlight (cursor-tracked gradient) ─────────────────── */
// Add class="spotlight" to any card. Pointer position is exposed as
// --mx / --my (0..1) so CSS can paint a soft radial highlight from
// the cursor location.

if (!reduceMotion) {
  for (const card of document.querySelectorAll<HTMLElement>('.spotlight')) {
    card.addEventListener('pointermove', (e) => {
      const rect = card.getBoundingClientRect();
      const x = (e.clientX - rect.left) / rect.width;
      const y = (e.clientY - rect.top) / rect.height;
      card.style.setProperty('--mx', `${(x * 100).toFixed(1)}%`);
      card.style.setProperty('--my', `${(y * 100).toFixed(1)}%`);
      card.classList.add('is-spotlit');
    });
    card.addEventListener('pointerleave', () => {
      card.classList.remove('is-spotlit');
    });
  }
}

/* ── 3. Sticky-nav state ─────────────────────────────────────────── */
// Past 80px of scroll the nav gets a stronger backdrop + border.
// Single boolean, debounced to scroll frames.

const nav = document.querySelector<HTMLElement>('header.site-nav');
if (nav) {
  let pending = false;
  const update = () => {
    pending = false;
    nav.classList.toggle('is-scrolled', window.scrollY > 80);
  };
  window.addEventListener(
    'scroll',
    () => {
      if (pending) return;
      pending = true;
      requestAnimationFrame(update);
    },
    { passive: true },
  );
  update();
}

/* ── 4. Counter animations ───────────────────────────────────────── */
// [data-counter] elements with [data-counter-to="128"] count up when they
// enter the viewport. Optional [data-counter-suffix="%"] appends a string.

const counterObserver = new IntersectionObserver(
  (entries) => {
    for (const entry of entries) {
      if (!entry.isIntersecting) continue;
      const el = entry.target as HTMLElement;
      const to = Number(el.dataset.counterTo ?? '0');
      const suffix = el.dataset.counterSuffix ?? '';
      const duration = 900;
      const start = performance.now();
      const animate = (now: number) => {
        const t = Math.min(1, (now - start) / duration);
        const eased = 1 - Math.pow(1 - t, 3); // easeOutCubic
        el.textContent = Math.round(to * eased) + suffix;
        if (t < 1) requestAnimationFrame(animate);
      };
      requestAnimationFrame(animate);
      counterObserver.unobserve(el);
    }
  },
  { threshold: 0.6 },
);

if (!reduceMotion) {
  for (const el of document.querySelectorAll('[data-counter]')) {
    counterObserver.observe(el);
  }
} else {
  // Reduced motion: jump to final value immediately.
  for (const el of document.querySelectorAll<HTMLElement>('[data-counter]')) {
    const to = el.dataset.counterTo ?? '0';
    const suffix = el.dataset.counterSuffix ?? '';
    el.textContent = `${to}${suffix}`;
  }
}

/* ── 5. Bar viz reveal ───────────────────────────────────────────── */
// .fx-view-bars span elements have inline --h; we start them at 0 and let
// CSS transition them to --h once their container enters view.

const barsObserver = new IntersectionObserver(
  (entries) => {
    for (const entry of entries) {
      if (!entry.isIntersecting) continue;
      const wrap = entry.target as HTMLElement;
      wrap.classList.add('is-revealed');
      barsObserver.unobserve(wrap);
    }
  },
  { threshold: 0.4 },
);

if (!reduceMotion) {
  for (const el of document.querySelectorAll('.fx-view-bars')) {
    barsObserver.observe(el);
  }
} else {
  for (const el of document.querySelectorAll('.fx-view-bars')) {
    el.classList.add('is-revealed');
  }
}
