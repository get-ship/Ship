# Gateway — setup

Cloudflare Worker. Hono. Better Auth + Stripe + Cloudflare Email + KV.
This file lists every environment variable, binding, and dashboard step
needed to bring a fresh deployment online.

For **dev work locally** scroll to [Local dev](#local-dev).
For **deploying to staging/prod** scroll to [Deploy checklist](#deploy-checklist).

---

## 1. Cloudflare bindings (in `wrangler.jsonc`)

| Binding name | Kind | Purpose |
|---|---|---|
| `AUTH_DB` | D1 | Better Auth tables + Stripe subscription + CLI auth state |
| `REGISTRY_DB` | D1 | Marketplace registry packages |
| `WAITLIST` | KV | Email signups (per-list, hashed key) |
| `EMAIL` | send_email | Cloudflare Email transport (beta) |
| `PLATFORM` | Durable Object | Cross-project platform routing |
| `PROJECT` | Durable Object | Per-project sync state |
| `RATE_LIMITER_*` | Rate Limit (×5) | Throttle classes (CLAIM, PUBLISH, SEARCH, INSTALL, STAR) |

The D1, KV, and send_email bindings need to be **created** in the
Cloudflare dashboard (or via CLI) before `wrangler deploy` will succeed.
Durable Objects + Rate Limits auto-bind from the wrangler config.

### Create the KV namespace

```sh
cd apps/gateway

# Per env (you need one for each env you deploy to)
wrangler kv namespace create WAITLIST --env staging
wrangler kv namespace create WAITLIST --env production

# Paste the returned id into wrangler.jsonc → kv_namespaces[0].id
# (it's currently the literal string "REPLACE_WITH_KV_NAMESPACE_ID")
```

### Create the D1 databases

The wrangler.jsonc already references named D1 databases per env. Confirm
they exist in your Cloudflare account, then apply migrations:

```sh
wrangler d1 migrations apply ship-auth-staging --env staging
wrangler d1 migrations apply ship-registry-staging --env staging

wrangler d1 migrations apply ship-auth --env production
wrangler d1 migrations apply ship-registry --env production
```

Three auth migrations exist as of this writing:
- `0001_initial.sql` — Better Auth core tables (user, session, account, verification) + CLI PKCE
- `0002_user_plan.sql` — adds `user.plan` (defaults to `'local'`)
- `0003_stripe.sql` — adds `user.stripeCustomerId` + the `subscription` table

### Cloudflare Email binding

The `send_email` binding doesn't need creation — but the **sender domain
has to be onboarded** to Cloudflare Email for it to actually send.

In the Cloudflare dashboard:
1. Add `getship.dev` to your account (if not already there).
2. Email → Email Service → enable.
3. Add the verified sender address(es). The current code sends from
   `no-reply@getship.dev` (see `src/lib/email.ts`).
4. Cloudflare auto-adds MX, SPF, DKIM, DMARC records to your DNS.

Note: Cloudflare Email Service is **beta**. APIs may change. If a wave
of mail bounces, swap to Resend/SES — the `sendEmail()` wrapper in
`src/lib/email.ts` is one function.

---

## 2. Environment variables

### Always required (set in `wrangler.jsonc` → `vars`, or per-env)

| Var | What it does |
|---|---|
| `GATEWAY_VERSION` | Returned by `/api/version`. Bump in CI on each deploy. Default `dev` |
| `BUILD_SHA` | Returned by `/api/version`. CI should stamp commit SHA. Default `unknown` |

### Feature flags (default OFF — flip per env as features go live)

| Var | When to enable |
|---|---|
| `SHIP_CLOUD_ENABLED` | When you ship cloud routes that should be live (none gated by this today, but middleware exists) |
| `STRIPE_LIVE_MODE` | Production. Flips Stripe SDK to expect live keys / live webhook secret |
| `EMAIL_LIVE_MODE` | When sender domain is verified and you want real sends. Off = logs to console |
| `MACAROON_ENFORCEMENT` | Reserved. No consumer yet |

### Secrets — `wrangler secret put` per env

These must NOT live in `wrangler.jsonc` (it's in git):

**Better Auth core:**
```sh
wrangler secret put BETTER_AUTH_SECRET --env staging
# Generate with: openssl rand -base64 32
```

**OAuth providers** (each one is optional — only set the ones you've
registered apps for; missing ones cause that provider's button to silently
fail at sign-in):
```sh
wrangler secret put GITHUB_CLIENT_ID     --env staging
wrangler secret put GITHUB_CLIENT_SECRET --env staging
wrangler secret put GOOGLE_CLIENT_ID     --env staging
wrangler secret put GOOGLE_CLIENT_SECRET --env staging
wrangler secret put SLACK_CLIENT_ID      --env staging
wrangler secret put SLACK_CLIENT_SECRET  --env staging

# GitHub webhook secret (for registry package indexing webhook)
wrangler secret put GITHUB_WEBHOOK_SECRET --env staging
```

**Stripe** (per env — test-mode keys in staging, live keys in production):
```sh
wrangler secret put STRIPE_SECRET_KEY     --env staging   # sk_test_… (staging) / sk_live_… (production)
wrangler secret put STRIPE_WEBHOOK_SECRET --env staging   # whsec_…
wrangler secret put STRIPE_PRICE_PRO      --env staging   # price_… (the Pro plan's recurring monthly Price ID)
```

**Daemon attestation JWT signing** (Ed25519 — used by `/api/auth/daemon-attest` to mint capability JWTs the daemon presents to peer services; verified via the JWKS endpoint at `/api/jwks`):
```sh
wrangler secret put SHIP_JWT_PRIVATE_KEY     --env staging
wrangler secret put SHIP_JWT_PRIVATE_KEY_ID  --env staging
wrangler secret put SHIP_JWT_PUBLIC_KEY      --env staging
wrangler secret put SHIP_JWT_PUBLIC_JWKS     --env staging
```

The JWT carries `iss: "https://getship.dev"` (the brand domain) regardless
of which env host minted it. Override the issuer per env only if you have
a reason to:

```sh
# Optional — defaults to https://getship.dev
wrangler secret put SHIP_ATTESTATION_ISSUER --env staging
```

The daemon side resolves issuer and JWKS URL **separately** (the issuer
claim is the brand identity; JWKS is hosted at the API host). Daemon
defaults are `auth.attestation_issuer = "https://getship.dev"` and
`auth.attestation_jwks_url = "https://api.getship.dev/api/jwks"`,
overridable via `~/.ship/config.toml`.

---

## 3. Better Auth — per-provider OAuth app setup

For each social provider you want enabled, register an OAuth app and set
the **callback URL** to:

- Staging: `https://staging.getship.dev/api/auth/callback/{provider}`
- Production: `https://api.getship.dev/api/auth/callback/{provider}`

Where `{provider}` is one of: `github`, `google`, `slack`.

| Provider | Where to register | Notes |
|---|---|---|
| GitHub | https://github.com/settings/developers → New OAuth App | Scopes needed: `repo user:email` (the wider repo scope is for the future CLI flow — narrow to `user:email` if CLI isn't shipped yet) |
| Google | https://console.cloud.google.com → APIs & Services → Credentials → OAuth client ID (Web app) | Authorized JS origins: `https://api.getship.dev` + `https://getship.dev` |
| Slack | https://api.slack.com/apps → Create New App → OAuth & Permissions | Scopes auto-selected via Better Auth's OIDC profile |

If you skip a provider, its button on `/sign-in` will surface a friendly
error at click time — won't break anything else.

---

## 4. Stripe dashboard setup

See `apps/gateway/STRIPE.md` for the full walkthrough. Short version:

1. Create one Product (`SHIP Pro`) with one Price (monthly, USD, $20).
2. Copy the Price ID into `STRIPE_PRICE_PRO` per env.
3. Add webhook endpoint at `https://{api,staging}.getship.dev/api/auth/stripe/webhook`
   with events: `checkout.session.completed`, `customer.subscription.{created,updated,deleted}`,
   `invoice.{paid,payment_failed}`.
4. Configure Customer Portal: enable cancellation (at period end),
   payment method update, invoice download. Disable plan switching
   (we own that in `/billing`).
5. Stripe Tax: enable on, register jurisdictions as you cross nexus.

---

## 5. Local dev

```sh
# Terminal A — gateway
cd apps/gateway
bun wrangler dev
# defaults to http://localhost:8787

# Terminal B — marketing site (gateway URL auto-defaults to localhost:8787 in dev)
cd apps/site
bun dev
# defaults to http://localhost:4321

# Optional terminal C — Stripe webhook forwarding
stripe listen --forward-to localhost:8787/api/auth/stripe/webhook
# Paste the printed whsec_… into .dev.vars as STRIPE_WEBHOOK_SECRET
```

For local secrets, create `apps/gateway/.dev.vars` (gitignored):

```sh
# .dev.vars — local-only secrets (NEVER commit)
BETTER_AUTH_SECRET=local-dev-secret-32-bytes-min
STRIPE_SECRET_KEY=sk_test_…
STRIPE_WEBHOOK_SECRET=whsec_…
STRIPE_PRICE_PRO=price_…
GITHUB_CLIENT_ID=…
GITHUB_CLIENT_SECRET=…
# … etc. for whichever providers you want to test locally
```

Cloudflare emails will log to console in dev (since `EMAIL_LIVE_MODE` is
off and likely no `EMAIL` binding is mocked). Click the log lines to
follow the password-reset / verification URLs manually.

---

## 6. Deploy checklist

Once-per-env, in order:

1. **D1 databases** — create in dashboard if they don't exist, then
   `wrangler d1 migrations apply` (both auth + registry).
2. **KV namespace** — `wrangler kv namespace create WAITLIST --env <env>`,
   paste id into `wrangler.jsonc`.
3. **Cloudflare Email** — onboard `getship.dev` domain, verify sender.
4. **Secrets** — all of the section above via `wrangler secret put`.
5. **OAuth apps** — register one per provider, set callback URLs, capture
   client ID + secret into secrets.
6. **Stripe** — create Product + Price, register webhook, capture Price ID
   + webhook secret into secrets.
7. **`wrangler.jsonc` review** — confirm KV ids are pasted, env-specific
   D1 ids look right.
8. **Deploy** — `wrangler deploy --env staging` (then `--env production`).
9. **Smoke test** — `curl https://staging.getship.dev/healthz`,
   `/api/version`, `POST /api/waitlist` with a test email.

---

## 7. What's NOT done yet (real list)

- **Email transport unverified.** The code path is wired but no end-to-end
  send has been tested. First real signup with reset will exercise it.
- **Stripe checkout loop end-to-end** — never test-charged. Verifies on
  first real upgrade attempt.
- **Account deletion.** The frontend `/account` page calls
  `authClient.deleteUser()`. Better Auth may or may not expose this
  endpoint by default; needs verification.
- **No production health monitoring.** Cloudflare Workers Observability
  is enabled in `wrangler.jsonc` (logs ON) but no alerting is wired —
  no UptimeRobot, no Sentry, etc.
- **Macaroon enforcement.** Middleware reserved (`MACAROON_ENFORCEMENT`
  flag exists) but no consumer routes use macaroons yet. Lands when
  sync / cloud compute pulls them in.
