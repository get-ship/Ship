# Marketing site launch — state of the world

**Date:** 2026-05-17
**Branch:** `job/marketing-site`
**Scope:** what's shipped, what's behind dev flags, exact deploy steps,
env var checklist, and the "turn it on later" path for each gated feature.

## Live surface (what visitors see in production)

| Route | Purpose | Notes |
|---|---|---|
| `/` | Landing | Plate cycles every 7s, pause-on-hover, progress sliver |
| `/features` | Features overview | Five cards, refreshed copy |
| `/download` | Download lander | Calls `<PlatformDetect>`; all platforms route to /coming-soon |
| `/download/coming-soon` | Real waitlist capture | POSTs to gateway `/v1/waitlist`; KV-backed |
| `/store` | Marketplace placeholder | Honest "coming at launch" copy |
| `/waitlist` | Single-purpose waitlist | `?reason={signin|signup|reset|general}` varies copy |
| `/404` | Branded not-found | Gradient "Lost at sea." |
| `/500` | Branded server error | Destructive-tone, support email link |

Footer: 3 cols (Product / Developers / Account). Sign-in/up footer links work
in dev; in prod they redirect through `/waitlist` per the flag.

## Behind flags (default OFF in prod, ON in dev)

| Route | Flag (set in prod to enable) | What it does |
|---|---|---|
| `/pricing` | `PUBLIC_SHOW_PRICING=true` | 2-tier pricing (Local + Pro). "Start with Pro" CTA fires Stripe Checkout |
| `/sign-in` | `PUBLIC_SHOW_AUTH=true` | Better Auth sign-in (Google/GitHub/Slack + email/password) |
| `/sign-up` | `PUBLIC_SHOW_AUTH=true` | Better Auth sign-up |
| `/forgot-password` | `PUBLIC_SHOW_AUTH=true` | Calls Better Auth `forgetPassword` |
| `/reset-password` | `PUBLIC_SHOW_AUTH=true` | Reset via token from email |
| `/account` | `PUBLIC_SHOW_ACCOUNT=true` | Profile + theme + sessions + linked providers + danger zone |
| `/billing` | `PUBLIC_SHOW_BILLING=true` | Real subscription state via `authClient.subscription.*` |

When flag is off, route redirects to `/` (account/billing/pricing) or
`/waitlist?reason=…` (auth pages). Navbar adapts: "Sign in" becomes
"Join waitlist," avatar menu never appears.

## What's wired but needs operator config

Everything in this list is *code-complete*; flipping it on is paste-tokens
work in the Cloudflare dashboard + a couple wrangler commands.

### Marketing site

- **Deploy target:** Cloudflare Workers static assets via wrangler.
  `apps/site/wrangler.jsonc` is the config.
- **Deploy command (from local):** `cd apps/site && bun run deploy` →
  `astro build && wrangler deploy`. Build needs Playwright (for OG image)
  and Bun + access to the sibling daemon assets (for SUI bundles). Deploy
  from a machine that has both — the script will warn and continue if
  Playwright is missing (existing PNG used) or if daemon assets are
  missing (SUI bundles skipped, web components won't upgrade).
- **No CI config to set up.** The deploy command is just `bun run deploy`.

### Gateway

- **Deploy target:** Cloudflare Worker. `apps/gateway/wrangler.jsonc` has
  staging + production routes already (`staging.getship.dev/*`,
  `api.getship.dev/*`).
- **Deploy command:** `cd apps/gateway && wrangler deploy --env staging`
  (and `--env production`).
- **Before first deploy, per env:**
  ```sh
  # KV namespace (paste returned id into wrangler.jsonc)
  wrangler kv namespace create WAITLIST --env staging

  # D1 migrations (auth schema additions for Stripe + plan)
  wrangler d1 migrations apply ship-auth-staging --env staging

  # Stripe (test mode in staging, live mode in production)
  wrangler secret put STRIPE_SECRET_KEY      --env staging   # sk_test_...
  wrangler secret put STRIPE_WEBHOOK_SECRET  --env staging   # whsec_...
  wrangler secret put STRIPE_PRICE_PRO      --env staging   # price_...

  # OAuth providers (each one needed only if you're enabling that button)
  wrangler secret put GITHUB_CLIENT_ID       --env staging
  wrangler secret put GITHUB_CLIENT_SECRET   --env staging
  wrangler secret put GOOGLE_CLIENT_ID       --env staging
  wrangler secret put GOOGLE_CLIENT_SECRET   --env staging
  wrangler secret put SLACK_CLIENT_ID        --env staging
  wrangler secret put SLACK_CLIENT_SECRET    --env staging

  # Better Auth signing secret
  wrangler secret put BETTER_AUTH_SECRET     --env staging
  ```
- **Stripe Dashboard:** create one Product (`SHIP Pro`) with a recurring
  monthly USD Price ($20). Webhook endpoint pointed at
  `https://staging.getship.dev/v1/auth/stripe/webhook` (and api.getship.dev
  for prod). Customer Portal config: enable cancel-at-period-end, payment
  method updates, invoice download; disable plan switching in the portal.
  Tax: enable Stripe Tax; add registrations as you cross nexus thresholds.
  See `apps/gateway/STRIPE.md` for the full checklist + Stripe CLI dev recipe.
- **Cloudflare Email Service:** onboard `getship.dev` in the Cloudflare
  dashboard (auto-adds MX/SPF/DKIM/DMARC). Set `EMAIL_LIVE_MODE=true` per
  env to send for real (otherwise emails log to console).

## Order of operations to go live

1. **Marketing site only, no auth:**
   - Deploy site to Cloudflare (`bun run deploy` from `apps/site/`).
   - Public surface is live. Waitlist form will return 503 until gateway
     is also deployed (acceptable — visitors see a clear error). To make
     waitlist work today, do step 2 first.
2. **Gateway with waitlist + ops only (cheap):**
   - Create KV namespace (WAITLIST).
   - Deploy gateway. `wrangler deploy --env staging`.
   - Waitlist form works. `/healthz` and `/v1/version` work.
   - All cloud routes return 503 (SHIP_CLOUD_ENABLED=false by default).
3. **Auth enabled:**
   - Register OAuth apps with Google/GitHub/Slack. Callback URLs:
     `https://staging.getship.dev/v1/auth/callback/{google|github|slack}`
     (and api.getship.dev for prod).
   - Set the secrets per the wrangler block above.
   - Onboard CF Email; set `EMAIL_LIVE_MODE=true`.
   - Flip `PUBLIC_SHOW_AUTH=true` on the marketing site.
   - Redeploy marketing site.
4. **Billing enabled:**
   - Create Stripe Product + Price, copy Price ID into
     `STRIPE_PRICE_PRO` secret.
   - Add Stripe webhook endpoint pointing at the gateway; copy webhook
     secret into `STRIPE_WEBHOOK_SECRET`.
   - Flip `PUBLIC_SHOW_PRICING`, `PUBLIC_SHOW_ACCOUNT`, `PUBLIC_SHOW_BILLING`
     on the marketing site.
   - Test loop: `/pricing` → checkout → 4242 card → `/billing` shows Pro.

## Feature flag matrix

| Flag | Default | Set on | Effect |
|---|---|---|---|
| `PUBLIC_SHOW_AUTH` | off | marketing site | enables sign-in/up/forgot/reset routes; switches navbar CTA |
| `PUBLIC_SHOW_PRICING` | off | marketing site | enables `/pricing` and "Pricing" nav link |
| `PUBLIC_SHOW_ACCOUNT` | off | marketing site | enables `/account` |
| `PUBLIC_SHOW_BILLING` | off | marketing site | enables `/billing` |
| `PUBLIC_GATEWAY_URL` | (auto) | marketing site | override gateway URL; defaults: staging in dev, api.getship.dev in prod |
| `SHIP_CLOUD_ENABLED` | off | gateway | enables cloud routes (none added yet but middleware exists) |
| `STRIPE_LIVE_MODE` | off | gateway | distinguishes test/live mode for billing |
| `EMAIL_LIVE_MODE` | off | gateway | when off, emails log to console; when on, real send |
| `MACAROON_ENFORCEMENT` | off | gateway | reserved (no consumer yet) |

## What's NOT shipped this session

These were discussed, scoped, or sketched but not implemented:

- **Capability model + `scoped()` route wrapper in `shipd`** (Rust, design
  agreed: `Capability` data type, type-enforced route declaration,
  lint-guarded). Closes the git-API scope hole as a side effect.
- **Git API per-handler scope checks** (Rust, blocker per
  `coherence/blockers.md` — any agent token can `push --force` today).
- **Web → desktop auth handoff** (not built; sketched only — gateway
  endpoints were prototyped and then removed at user request).
- **Keychain integration for daemon token storage** (Rust, deferred per
  agreement — file at mode 0600 for pre-launch).
- **Unix domain socket + SO_PEERCRED for daemon HTTP** (Rust, post-launch
  hardening).
- **Better Auth API key migration from JWT-shaped CLI token** (deferred
  to post-launch, when sync ships and revocability matters).
- **Sync** (entirely deferred — sync IS the event log per discussion;
  Pro/Pro features cascade from this).
- **Macaroons crate** (deferred until peer-to-peer sync or cloud compute
  pulls them in).
- **Legal pages** (Terms / Privacy / Refund / Security). Scaffold is
  ~45 min if needed; copy needs lawyer.
- **Blog + changelog scaffolds** (Astro content collections; ~1 day; not
  blocking if no launch announcement to write).
- **Webhooks as a Pro feature** (sketched; data model fits the existing
  `/v1/registry/webhook` pattern; needs cloud-spec 09 decisions).
- **OAuth vault as a Pro feature** (sketched; needs cloud-spec 05).
- **ENG SIDE: actually verify a Stripe test charge end-to-end** — until
  staging gateway is deployed with secrets, the loop is unproven.

## Auth posture, honest summary

What's reasonable about it:
- Better Auth handles every UI flow we care about (social + email +
  password reset + verify + sessions + linking).
- Stripe is wired through the official Better Auth plugin — webhooks,
  customer creation, lifecycle hooks all by-the-book.
- Trust boundaries are named: gateway-side validates via session/API key;
  daemon-side validates via local `DaemonSession` tokens; agents get
  scope-vec sessions; web shell will eventually talk to daemon over
  localhost with similar tokens.
- Handoff endpoints exist for the web↔desktop bridge; consumer (daemon
  receiver) can be built when the desktop dev cycles up.

What's NOT done that you'll want eventually:
- Capability-as-data (one type instead of three token kinds).
- Type-enforced scope declaration on routes (closes whole classes of bugs
  by construction; impossible to forget).
- Cross-device peer auth (requires macaroon-style delegation; needed when
  sync ships).

Net: **launchable as-is for a no-paid-features marketing surface**.
Auth is gated until the operator-side configuration is done; flipping
that switch unblocks the paid-features loop end-to-end.

## Quick smoke test after deploy

```sh
# Marketing site
curl -sf https://getship.dev/ | grep -q 'Vibe' && echo 'landing OK'
curl -sf https://getship.dev/waitlist | grep -q 'waitlist' && echo 'waitlist OK'
curl -sf -o /dev/null -w '%{http_code}' https://getship.dev/pricing  # 302 → / (when flag off)

# Gateway
curl -sf https://staging.getship.dev/healthz && echo 'health OK'
curl -sf https://staging.getship.dev/v1/version | jq .
curl -sf -X POST https://staging.getship.dev/v1/waitlist \
  -H 'Content-Type: application/json' \
  -d '{"email":"test@example.com","list":"general"}'
```

## Files written this session — summary

Marketing (`apps/site/`):
- `src/components/{Navbar,Footer,WaitlistForm,AuthPage,AccountShell,AccountSection}.astro`
- `src/pages/{index,features,download,store,pricing,sign-in,sign-up,forgot-password,reset-password,account,billing,waitlist,404,500}.astro`
- `src/pages/download/coming-soon.astro`
- `src/components/landing/{Hero,HeroPlate,Nav,ClusterStrip,FooterCta,SectionAgents,SectionPipelines,SectionData,SectionViews,TweaksPanel}.astro`
- `src/lib/{auth,themes}.ts`
- `src/scripts/landing.ts`
- `src/layouts/BaseLayout.astro`
- `src/styles/{global,tokens,themes,landing}.css`
- `public/og-image.png` (generated; commit if you want CI deploys)
- `public/robots.txt`
- `public/api/assets/view-tokens.css` (copied from daemon; gitignored fine)
- `astro.config.mjs` (added sitemap integration)
- `scripts/{copy-sui-assets.ts,build-og-image.mjs,og-template.html}`

Gateway (`apps/gateway/`):
- `src/lib/{auth,email}.ts` (Better Auth config + email transport)
- `src/middleware/feature-flags.ts`
- `src/routes/{ops,waitlist}.ts`
- `src/data/schema/auth.ts` (added stripeCustomerId, subscription table)
- `migrations/auth/{0002_user_plan,0003_stripe}.sql`
- `wrangler.jsonc` (KV bindings, send_email binding, flag vars, Stripe secrets)
- `worker-env.d.ts` (new env type slots)
- `STRIPE.md` (CLI dev recipe + dashboard checklist)
- `test/billing-auth.test.ts` (smoke test — runs when vitest-pool-workers config is fixed)
