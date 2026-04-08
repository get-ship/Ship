---
name: browse
stable-id: browse
description: Headless browser for page verification, interaction testing, and screenshots. Uses Playwright CLI — no extra binary required.
tags: [qa, testing, browser, screenshots, headless]
authors: [ship]
compatibility: Requires Playwright (@playwright/test) with browsers installed. Install: npm install -D @playwright/test && npx playwright install chromium
artifacts: [image, markdown]
allowed-tools: Bash(playwright*) Bash(npx playwright*) Bash(pnpm exec playwright*)
---

# Browse

Headless browser using the Playwright CLI. Navigate, interact, screenshot, and assert page state.

## Setup

```bash
# Discover playwright
if command -v playwright &>/dev/null; then
  PW="playwright"
elif npx --no-install playwright --version &>/dev/null 2>&1; then
  PW="npx playwright"
else
  _root="$(git rev-parse --show-toplevel)"
  _found=""
  for _c in "$_root"/node_modules/.bin/playwright "$_root"/apps/*/node_modules/.bin/playwright "$_root"/packages/*/node_modules/.bin/playwright; do
    [ -x "$_c" ] && { _found="$_c"; break; }
  done
  if [ -n "$_found" ]; then PW="$_found"
  else echo "Playwright not found. Install: npm install -D @playwright/test && npx playwright install chromium" >&2; exit 1
  fi
fi
$PW --version
```

Use `$PW` for all commands below.

## Core Patterns

### Verify a page loads
```bash
$PW open --browser=chromium <url>
$PW snapshot
$PW console
```

### Test a user flow
```bash
$PW goto <url>
$PW snapshot
$PW fill e3 "user@test.com"
$PW fill e4 "password"
$PW click e5
$PW snapshot
$PW is visible ".dashboard"
```

### Screenshot
```bash
$PW screenshot --filename="{{ screenshot_dir }}/page.png"
```

### Responsive layouts
```bash
$PW viewport 375x812
$PW screenshot --filename="{{ screenshot_dir }}/mobile.png"
$PW viewport 1280x800
$PW screenshot --filename="{{ screenshot_dir }}/desktop.png"
```

### Auth state
```bash
$PW state-save "{{ screenshot_dir }}/auth.json"
$PW state-load "{{ screenshot_dir }}/auth.json"
```

## Commands

| Command | Description |
|---------|-------------|
| `open --browser=<b> <url>` | Launch browser and navigate |
| `goto <url>` | Navigate in open browser |
| `back` / `forward` / `reload` | History |
| `snapshot` | Accessibility tree with element refs |
| `text` | Page text content |
| `html [sel]` | innerHTML |
| `links` | All links |
| `click <sel>` | Click element |
| `fill <sel> <val>` | Fill input |
| `type <text>` | Type into focused element |
| `select <sel> <val>` | Select dropdown option |
| `hover <sel>` | Hover element |
| `press <key>` | Keypress |
| `scroll [sel]` | Scroll into view |
| `upload <sel> <file>` | File upload |
| `wait <sel>` | Wait for element |
| `screenshot [--filename=p]` | Screenshot |
| `is visible <sel>` | Assert visible |
| `is hidden <sel>` | Assert hidden |
| `is enabled <sel>` | Assert enabled |
| `is disabled <sel>` | Assert disabled |
| `is checked <sel>` | Assert checked |
| `console [--errors]` | Console log |
| `network [--clear]` | Network requests |
| `cookies` | All cookies |
| `viewport <WxH>` | Set viewport size |
| `js <expr>` | Run JavaScript |
| `close` | Close browser |

## Browsers

`--browser=chromium` (default), `--browser=firefox`, `--browser=webkit`, `--browser=msedge`

## Artifacts

Screenshots go to `{{ screenshot_dir }}/`. Viewable in Ship Studio's Session artifacts tab.
