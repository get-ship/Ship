/**
 * Theme loader — loads and switches Ship themes at runtime.
 *
 * Themes are CSS files served by the daemon at /api/runtime/assets/theme.css.
 * The active theme name is stored in localStorage and the daemon setting.
 * Theme changes dispatch 'ship-theme-changed' so island components
 * (CodeMirror, xterm, TipTap) can refresh their internal palettes.
 */

const STORAGE_KEY = "ship-theme-name";
const THEME_LINK_ID = "ship-theme-css";

let currentTheme = localStorage.getItem(STORAGE_KEY) || "ship";

/** Get the active theme name. */
export function getThemeName(): string {
  return currentTheme;
}

/**
 * Initialize theme loading. Call once at startup.
 * Injects <link> for theme.css and applies the saved theme.
 */
export function initThemeLoader() {
  // Inject theme CSS link (loaded after view-tokens.css in cascade)
  let link = document.getElementById(THEME_LINK_ID) as HTMLLinkElement | null;
  if (!link) {
    link = document.createElement("link");
    link.id = THEME_LINK_ID;
    link.rel = "stylesheet";
    document.head.appendChild(link);
  }

  // Load the active theme
  if (currentTheme && currentTheme !== "ship") {
    link.href = `/api/runtime/assets/theme.css?t=${Date.now()}`;
  }
}

/**
 * Switch to a different theme. Updates localStorage, reloads CSS,
 * persists to daemon, and dispatches change event.
 */
export async function setTheme(name: string) {
  currentTheme = name;
  localStorage.setItem(STORAGE_KEY, name);

  // Persist to daemon settings
  try {
    await fetch("/api/settings", {
      method: "PUT",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ key: "theme.name", value: name }),
    });
  } catch {
    /* settings save is best-effort */
  }

  // Reload theme CSS
  const link = document.getElementById(THEME_LINK_ID) as HTMLLinkElement;
  if (link) {
    if (name === "ship") {
      // Default theme — remove override CSS
      link.href = "";
    } else {
      link.href = `/api/runtime/assets/theme.css?t=${Date.now()}`;
    }
  }

  // Notify islands (CodeMirror, xterm, TipTap) to refresh
  document.dispatchEvent(
    new CustomEvent("ship-theme-changed", {
      detail: { theme: name },
      bubbles: true,
    }),
  );
}

/**
 * Fetch available themes from the daemon.
 */
export async function listThemes(): Promise<
  Array<{
    name: string;
    source: string;
    has_light: boolean;
    has_dark: boolean;
  }>
> {
  try {
    const res = await fetch("/api/runtime/themes");
    if (!res.ok) return [];
    return await res.json();
  } catch {
    return [];
  }
}
