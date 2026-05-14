#!/usr/bin/env node
/**
 * agent-modal — opens the chat modal via ship:chat:toggle, picks a
 * provider through <app-agent-picker>, and verifies the picker swaps
 * to the thread + composer once a session is spawned.
 *
 * Acceptance:
 *   - GET /shell/ → page loads with <app-agent-modal> mounted
 *   - dispatching `ship:chat:toggle` on window opens the modal
 *   - <app-agent-picker> renders an <option> per installed agent
 *   - clicking "Start chat" with a valid (agent, provider) → POST
 *     /api/v1/sessions/spawn → modal's `session-id` host attr populates
 *   - the picker hides; <app-agent-thread> + <app-agent-composer> show
 *
 * Doesn't send a turn (turn round-trips touch claude binary; covered
 * by channel-roundtrip.spec.mjs at the substrate layer). This spec
 * only validates the SFC composition.
 */
// Playwright lives under apps/shell (the package that depends on it).
// Importing through shell's node_modules keeps the spec independent of
// where the workspace installer places the package.
import { chromium } from '../../apps/shell/node_modules/playwright/index.mjs';
import { readFileSync } from 'node:fs';
import { homedir } from 'node:os';
import { join } from 'node:path';

const BASE = process.env.SHIPD_URL ?? 'http://localhost:9315';
const TOKEN = process.env.TOKEN ??
  readFileSync(join(homedir(), '.ship/daemon.token'), 'utf8').trim();

function fail(msg, extra = {}) {
  console.error(`FAIL: ${msg}`);
  for (const [k, v] of Object.entries(extra)) {
    console.error(`  ${k}:`, typeof v === 'string' ? v : JSON.stringify(v));
  }
  process.exit(1);
}

async function main() {
  const browser = await chromium.launch({ headless: true });
  const context = await browser.newContext({
    extraHTTPHeaders: { Authorization: `Bearer ${TOKEN}` },
    viewport: { width: 1280, height: 800 },
  });
  const page = await context.newPage();

  // Surface JS console errors to stderr so a broken SFC doesn't pass
  // silently. Filter known noise from third-party deps.
  const consoleErrors = [];
  page.on('console', (msg) => {
    if (msg.type() === 'error') consoleErrors.push(msg.text());
  });
  page.on('pageerror', (e) => consoleErrors.push(`pageerror: ${e.message}`));

  try {
    // 'networkidle' would never fire — SSE connections from the
    // mounted components keep the network busy. 'load' is enough; we
    // explicitly wait for `app-agent-modal` to attach below.
    const resp = await page.goto(`${BASE}/shell/`, { waitUntil: 'load' });
    if (!resp || resp.status() >= 400) fail('shell page failed to load', { status: resp?.status() });

    // Modal must be in the DOM (mounted at shell root).
    await page.waitForSelector('app-agent-modal', { state: 'attached', timeout: 5000 });

    // Give the SFC IIFE + Datastar's first sweep a beat so the modal's
    // window-level event listeners are registered before we fire.
    await page.waitForTimeout(500);

    // Open via the same DOM event the chrome's ⌘J shortcut dispatches.
    await page.evaluate(() => window.dispatchEvent(new CustomEvent('ship:chat:toggle')));

    // The modal reflects `open` to the host attribute via reflectAttr.
    await page.waitForFunction(() => {
      const m = document.querySelector('app-agent-modal');
      return m && m.hasAttribute('open');
    }, { timeout: 3000 });

    // Picker should be visible — no session yet, no defaults set.
    try {
      // Wait for the form row (data-show="$agents_resp && $providers_resp")
      // — that confirms data() resolved AND Datastar re-evaluated.
      await page.waitForFunction(() => {
        const row = document.querySelector('app-agent-picker .row');
        return row && getComputedStyle(row).display !== 'none';
      }, { timeout: 8000 });
    } catch {
      const dump = await page.evaluate(() => {
        const m = document.querySelector('app-agent-modal');
        const host = document.querySelector('app-agent-modal app-agent-picker');
        const inner = document.querySelector('app-agent-modal app-agent-picker .picker');
        const signalAttrs = {};
        if (m) for (const a of m.attributes) {
          if (a.name.startsWith('data-signals:')) signalAttrs[a.name] = a.value;
        }
        const pickerSignalAttrs = {};
        if (host) for (const a of host.attributes) {
          if (a.name.startsWith('data-signals:')) pickerSignalAttrs[a.name] = a.value;
        }
        return {
          modalSignals: signalAttrs,
          pickerHostSignals: pickerSignalAttrs,
          pickerHostInner: host?.children.length ?? 0,
          pickerHostInline: host?.style.display,
          pickerInnerExists: !!inner,
          pickerInnerStyle: inner ? getComputedStyle(inner).display : null,
          pickerInnerVisibility: inner ? getComputedStyle(inner).visibility : null,
          stateRows: Array.from(document.querySelectorAll('app-agent-picker .state, app-agent-picker .row')).map(el => ({
            cls: el.className,
            display: getComputedStyle(el).display,
            text: el.textContent?.slice(0, 40),
          })),
        };
      });
      fail('picker not visible after toggle', dump);
    }

    // At least one agent option populated from /api/v1/agents.
    const agentCount = await page.locator('app-agent-picker select').first().locator('option').count();
    if (agentCount < 1) fail('agent picker rendered no options', { agentCount });

    // Pick first option in each select (Playwright's selectOption picks
    // the value, not just clicks). Then commit.
    const firstAgent = await page.locator('app-agent-picker select').first().locator('option').first().getAttribute('value');
    const firstProvider = await page.locator('app-agent-picker select').nth(1).locator('option').first().getAttribute('value');
    if (!firstAgent || !firstProvider) fail('could not read default agent/provider', { firstAgent, firstProvider });

    await page.locator('app-agent-picker select').first().selectOption(firstAgent);
    await page.locator('app-agent-picker select').nth(1).selectOption(firstProvider);
    await page.locator('app-agent-picker .primary').click();

    // Spawn → session-id host attr populates.
    await page.waitForFunction(() => {
      const m = document.querySelector('app-agent-modal');
      const id = m?.getAttribute('session-id') ?? '';
      return id.length === 26;
    }, { timeout: 10000 });

    const sessionId = await page.locator('app-agent-modal').getAttribute('session-id');
    console.log(`spawned: ${sessionId}`);

    // Picker hides; thread + composer show.
    await page.waitForSelector('app-agent-modal app-agent-thread', { state: 'visible', timeout: 3000 });
    await page.waitForSelector('app-agent-modal app-agent-composer', { state: 'visible', timeout: 3000 });
    const pickerVisible = await page.locator('app-agent-modal app-agent-picker').isVisible();
    if (pickerVisible) fail('picker did not hide after spawn');

    // Close with Escape.
    await page.keyboard.press('Escape');
    await page.waitForFunction(() => {
      const m = document.querySelector('app-agent-modal');
      return !m.hasAttribute('open');
    }, { timeout: 3000 });

    // Clean up the spawned session.
    await fetch(`${BASE}/api/v1/sessions/${sessionId}`, {
      method: 'DELETE',
      headers: { Authorization: `Bearer ${TOKEN}` },
    });

    if (consoleErrors.length) {
      console.warn('WARN: console errors during run:');
      for (const e of consoleErrors) console.warn(`  ${e}`);
    }

    console.log(`PASS — modal opened, picker → spawn (${sessionId}) → thread+composer`);
  } catch (e) {
    fail(String(e?.message ?? e));
  } finally {
    await browser.close();
  }
}

main().catch((e) => {
  console.error('ERROR:', e?.message ?? e);
  process.exit(2);
});
