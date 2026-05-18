# Stripe + Billing — gateway setup

The gateway uses the [Better Auth Stripe plugin](https://better-auth.com/docs/plugins/stripe)
to expose Subscription / Checkout / Customer-Portal endpoints under the
existing `/v1/auth/*` namespace. The marketing site calls these via
`authClient.subscription.upgrade()` / `.cancel()` / `.restore()` /
`.billingPortal()` / `.list()`.

This doc covers (1) **local development** with the Stripe CLI, (2) the
**Stripe Dashboard configuration** the plugin expects, and (3) the
**deploy checklist** for staging + production.

## 1. Local development

### Install the Stripe CLI

```sh
# macOS
brew install stripe/stripe-cli/stripe

# Linux
# https://docs.stripe.com/stripe-cli#install — download the binary or use
# your package manager.

stripe --version    # confirm
stripe login        # authorizes the CLI against your Stripe account
```

### Get test-mode keys

Once logged in:

```sh
stripe config --list
# Test-mode secret key lives in ~/.config/stripe/config.toml after login.

# Or grab it from the dashboard:
#   https://dashboard.stripe.com/test/apikeys  → "Secret key" (sk_test_…)
```

### Forward webhooks to a local gateway

Run two terminals:

```sh
# Terminal A — gateway running on localhost:8787
cd apps/gateway
bun wrangler dev

# Terminal B — Stripe CLI forwards webhook events to your local gateway.
# The webhook URL the plugin registers is /v1/auth/stripe/webhook.
stripe listen --forward-to localhost:8787/v1/auth/stripe/webhook
```

The `stripe listen` command prints a **webhook signing secret** that's
only valid for this CLI session (`whsec_…`). Set it as a local secret:

```sh
cd apps/gateway
# Local dev — put it in .dev.vars (gitignored). wrangler reads this on
# `bun wrangler dev`:
cat >> .dev.vars <<EOT
STRIPE_SECRET_KEY=sk_test_xxx_from_dashboard
STRIPE_WEBHOOK_SECRET=whsec_xxx_from_stripe_listen_output
STRIPE_PRICE_TEAM=price_xxx_from_dashboard
EOT
```

Restart `bun wrangler dev` after editing `.dev.vars`.

### Point the marketing site at your local gateway

```sh
cd apps/site
PUBLIC_GATEWAY_URL=http://localhost:8787 bun dev
```

Now /pricing → "Start with Team" → Stripe Checkout (test mode) → 4242
card → /billing reflects the live subscription. The `stripe listen`
terminal will show every webhook event as it fires.

### Test cards

| Card | What it does |
|---|---|
| `4242 4242 4242 4242` | Always succeeds |
| `4000 0025 0000 3155` | Requires 3-D Secure auth |
| `4000 0000 0000 9995` | Insufficient funds (declined) |
| `4000 0000 0000 0341` | Succeeds, later disputed as fraud |
| `4000 0027 6000 3184` | 3-D Secure auth fails |

Any 3-digit CVC, any future expiry, any postal code.

Full list: https://docs.stripe.com/testing#cards

### Trigger arbitrary events from the CLI

Useful when debugging webhook handlers without going through Checkout:

```sh
stripe trigger checkout.session.completed
stripe trigger customer.subscription.updated
stripe trigger customer.subscription.deleted
stripe trigger invoice.payment_failed
```

The plugin writes lifecycle changes into our `subscription` table and
fires the `onSubscription{Created,Update,Deleted}` callbacks defined in
`apps/gateway/src/lib/auth.ts`, which keep `user.plan` denormalized for
fast reads.

## 2. Stripe Dashboard configuration

The plugin assumes a specific Stripe Dashboard setup. Once per Stripe
account (test mode + live mode separately):

### Products and prices

1. Dashboard → **Products** → "+ Add product"
2. Name: `SHIP Team`
3. Pricing model: `Standard pricing` → Recurring, monthly, USD, $20.00
4. Save.
5. Copy the **Price ID** (`price_…`) into `STRIPE_PRICE_TEAM` for that env.

(Add an annual price later when we ship the annual toggle — separate
Price on the same Product with `billing_scheme: per_unit`, `interval: year`,
discounted price.)

### Customer Portal

Dashboard → **Settings** → **Billing** → **Customer Portal**:

- **Subscriptions**: enable "Customers can cancel subscriptions" with
  "Cancellation method: Cancel at end of billing period". Disable
  "Customers can switch plans" — we own the upgrade UI in /billing.
- **Payment methods**: enable "Customers can update payment methods".
- **Invoice history**: enable.
- **Business information**: set headline, support email, branding so the
  portal looks like SHIP.
- **Default return URL**: `https://getship.dev/billing` (or staging/local).

### Stripe Tax

Dashboard → **Tax**:

- Toggle Stripe Tax **on**.
- Add registrations only for jurisdictions where you're registered to
  collect tax. For a US-person DBA with no incorporated entity, that
  usually means: register your home state when you cross economic
  nexus ($100k or 200 transactions/year), then add it here. Until then
  leave registrations empty — Stripe will mark all customers as
  "tax not collected", which is the correct behavior for an
  unregistered seller.
- When you incorporate in Switzerland, add Swiss VAT registration and
  use the OSS scheme for EU sales.

The plugin always sends `automatic_tax: { enabled: true }` and
`tax_id_collection: { enabled: true }` on the Checkout Session, so EU
B2B customers get reverse-charge handling automatically once you have
the registrations in place.

### Webhook endpoint (production / staging)

For each deployed env (staging + prod), add one webhook endpoint:

Dashboard → **Developers** → **Webhooks** → "+ Add endpoint"

- URL: `https://staging.getship.dev/v1/auth/stripe/webhook`
  (or `https://api.getship.dev/v1/auth/stripe/webhook` for prod)
- Description: `SHIP gateway — staging` (or `production`)
- Events to send: select these (do not select "all events"):
  - `checkout.session.completed`
  - `customer.subscription.created`
  - `customer.subscription.updated`
  - `customer.subscription.deleted`
  - `invoice.payment_failed`
  - `invoice.paid`
- Click **Add endpoint**.
- Reveal **Signing secret** (`whsec_…`) — set it via wrangler secret
  put as `STRIPE_WEBHOOK_SECRET` for that env.

You do **not** need a webhook endpoint for local dev — `stripe listen`
forwards events through the CLI session.

## 3. Deploy checklist

### One-time per environment

```sh
cd apps/gateway

# Staging
wrangler secret put STRIPE_SECRET_KEY --env staging       # sk_test_… (test mode)
wrangler secret put STRIPE_WEBHOOK_SECRET --env staging   # whsec_… from staging webhook endpoint
wrangler secret put STRIPE_PRICE_TEAM --env staging       # price_… for test-mode Team price

# Production
wrangler secret put STRIPE_SECRET_KEY --env production    # sk_live_…
wrangler secret put STRIPE_WEBHOOK_SECRET --env production # whsec_… from prod webhook
wrangler secret put STRIPE_PRICE_TEAM --env production    # price_… for live Team price
```

### Schema migration

Two migrations support billing:

- `migrations/auth/0002_user_plan.sql` — adds the `user.plan` column
- `migrations/auth/0003_stripe.sql` — adds `user.stripeCustomerId` and
  the `subscription` table

Apply both:

```sh
wrangler d1 migrations apply ship-auth-staging --env staging
wrangler d1 migrations apply ship-auth --env production
```

### Deploy

```sh
wrangler deploy --env staging
wrangler deploy --env production
```

Check the Workers log for the `[auth]` warnings — if any Stripe env
var is missing, the plugin disables itself and logs which one. Don't
rely on this in production; verify with a real test charge after
deploy.

## 4. Smoke-testing the deployed loop

Against staging, with test-mode keys:

1. Visit `https://staging.getship.dev/pricing`
2. Click **Start with Team** → expect redirect to Stripe Checkout
3. Pay with `4242 4242 4242 4242`
4. Expect redirect to `https://staging.getship.dev/billing?upgrade=success`
5. /billing should show "Team — Active — Renews [date]" within a second
   or two (we re-fetch once to absorb webhook lag)
6. Click **Open billing portal** → Stripe Customer Portal opens in same
   tab, return URL goes back to /billing
7. Cancel subscription via portal → /billing shows "Team — Cancelling —
   Access ends [date]"
8. From /billing click **Resume subscription** → status flips back to
   Active
9. Confirm `user.plan` updates in D1:
   ```sh
   wrangler d1 execute ship-auth-staging --env staging \
     --command "SELECT email, plan FROM user WHERE email='your@email.com'"
   ```

## 5. Known limitations and rough edges

- **Tests can't run.** `apps/gateway/test/billing-auth.test.ts` is a
  smoke test for the plugin wiring, but the existing
  `@cloudflare/vitest-pool-workers` config has a broken `./config`
  import (pre-existing in the repo, not introduced by billing).
  Once that's fixed, the smoke test will run.
- **Webhook race on success.** After Checkout, Stripe redirects
  immediately while the `customer.subscription.created` webhook is
  asynchronous. /billing waits 1.5s and refreshes once if the
  subscription hasn't landed. If it still hasn't, the user sees Local;
  reload fixes. Real fix: poll Stripe directly on the redirect.
- **Paused subscriptions.** Treated as `local` for billing privileges
  (no Team access) but the row is still displayed on /billing with a
  "Paused" pill so the user can resume via Customer Portal. We don't
  expose pause from our UI — only Stripe Portal can pause.
- **Annual billing.** Not enabled at launch. To add: create an annual
  Price in Dashboard, set `STRIPE_PRICE_TEAM_ANNUAL` env, add a second
  plan entry with `annualDiscountPriceId`, add a toggle on /pricing.
- **Email transport.** Better Auth's dunning emails (payment failed,
  trial ending) are silently no-op'd because we haven't registered an
  email transport. Stripe sends its own customer-facing emails out of
  the box (invoice.paid, payment_failed) — that covers the critical
  ones until we add Resend/SES.
