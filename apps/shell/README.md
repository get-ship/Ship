# apps/shell — Ship hypermedia shell (POC)

A tiny Vite app that demonstrates a shell loading `ship-elements` and
`datastar` once, then mounting HTML fragments from the daemon. See
`POC-findings.md` for what worked, what didn't, and the recommendation.

## Run

```bash
# install fixture views (one-time)
for v in poc-home poc-pipelines poc-chat poc-form poc-drawer; do
  mkdir -p ~/.ship/views/$v
  cp apps/shell/fixtures/views/$v/index.html ~/.ship/views/$v/index.html
done

cd apps/shell
VITE_DAEMON_TOKEN=$(cat ~/.ship/daemon.token) \
VITE_DAEMON_URL=http://127.0.0.1:51742 \
pnpm dev
```

Open <http://127.0.0.1:3100/>.

## Re-capture evidence

With dev server running:

```bash
cd apps/shell
node scripts/capture-evidence.mjs
```

Writes PNGs + `report.json` to `findings-evidence/`.

## Layout

- `index.html` — boots `ship-elements.js` + `datastar.js` from the daemon
  at `/api/assets/*` (proxied same-origin via Vite in dev)
- `src/mount.ts` — `mountDaemonFragment` primitive (+ `openDaemonSSE`)
- `src/main.ts` — minimal router, drawer toggle, form interception
- `fixtures/views/` — sample daemon-served fragments used by the POC

Bundles are built on demand by `pnpm build:bundles` (root); they are no
longer vendored into `public/`.

## Scope

This app is a **proof of concept**, not a replacement for `apps/web`. If
findings are accepted, a follow-up promotes this into
`ship daemon --with-ui` and the standalone shell route.
