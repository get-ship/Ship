// Runs before paint to prevent theme flash.
const KEY = 'ship-site-theme';
const valid = ['default', 'high-contrast', 'warm'] as const;
type Theme = (typeof valid)[number];

function read(): Theme {
  try {
    const v = localStorage.getItem(KEY);
    if (v && (valid as readonly string[]).includes(v)) return v as Theme;
  } catch {}
  return 'default';
}

const theme = read();
document.documentElement.dataset.theme = theme;
