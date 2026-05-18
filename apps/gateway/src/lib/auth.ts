import { getAuthDb } from './d1.js'
import { sendEmail, resetPasswordEmail, verifyEmailEmail, deleteAccountEmail } from './email.js'

// Origins permitted to call /v1/auth/* and receive Set-Cookie. The marketing
// site's local-dev origin is included so Astro dev can exercise the real
// signin flow against a locally-running or staging gateway.
const TRUSTED_ORIGINS = [
  'https://getship.dev',
  'https://staging.getship.dev',
  'http://localhost:3000',
  'http://localhost:4321',
]

// Single source of truth for plan IDs we mirror onto user.plan. Better Auth's
// Stripe plugin owns the `subscription` table; this column is the fast-read
// denormalization that daemon-attest and /v1/me return.
type Plan = 'local' | 'pro'

function planFromSubscriptionStatus(status: string): Plan {
  // Stripe statuses: active | trialing | past_due | canceled | unpaid |
  // incomplete | incomplete_expired | paused
  // We keep the user on Pro while they have an active, trialing, or
  // past-due (grace) subscription. Anything terminal drops them to local.
  return status === 'active' || status === 'trialing' || status === 'past_due'
    ? 'pro'
    : 'local'
}

async function setUserPlan(env: Env, userId: string, plan: Plan): Promise<void> {
  const db = getAuthDb(env)
  if (!db) return
  await db
    .prepare('UPDATE user SET plan = ?, updatedAt = ? WHERE id = ?')
    .bind(plan, Date.now(), userId)
    .run()
}

async function makeAuth(env: Env) {
  const { betterAuth } = await import('better-auth')
  const { stripe: stripePlugin } = await import('@better-auth/stripe')
  const { apiKey: apiKeyPlugin } = await import('@better-auth/api-key')
  const StripeMod = await import('stripe')
  const Stripe = StripeMod.default

  const d1 = getAuthDb(env)
  let database: ReturnType<typeof import('better-auth/adapters/drizzle').drizzleAdapter> | undefined

  if (d1) {
    const [{ drizzle }, { drizzleAdapter }, schema] = await Promise.all([
      import('drizzle-orm/d1'),
      import('better-auth/adapters/drizzle'),
      import('../data/schema/auth.js'),
    ])
    const db = drizzle(d1, { schema })
    database = drizzleAdapter(db, { provider: 'sqlite', schema })
  }

  // Stripe client is constructed with the Fetch http client so it works on
  // Cloudflare Workers (the default Node http client isn't available).
  // apiVersion is pinned — Stripe's API is dated; pinning prevents silent
  // behavior shifts when they roll the default version.
  const stripeClient = env.STRIPE_SECRET_KEY
    ? new Stripe(env.STRIPE_SECRET_KEY, {
        // Pinned to the version the installed SDK was built against. Bump
        // alongside `bun add stripe@...` and review the changelog.
        apiVersion: '2026-04-22.dahlia',
        httpClient: Stripe.createFetchHttpClient(),
      })
    : null

  const plugins: ReturnType<typeof stripePlugin>[] = []
  if (!stripeClient) {
    console.warn('[auth] STRIPE_SECRET_KEY not set — billing plugin disabled. /v1/auth/subscription/* will 404.')
  } else if (!env.STRIPE_WEBHOOK_SECRET) {
    console.warn('[auth] STRIPE_WEBHOOK_SECRET not set — billing plugin disabled (webhook verification would fail).')
  } else if (!env.STRIPE_PRICE_PRO) {
    console.warn('[auth] STRIPE_PRICE_PRO not set — billing plugin enabled but upgrade calls will fail with "unknown price".')
  }
  if (stripeClient && env.STRIPE_WEBHOOK_SECRET) {
    plugins.push(
      stripePlugin({
        stripeClient,
        stripeWebhookSecret: env.STRIPE_WEBHOOK_SECRET,
        // Create the Stripe customer when the user signs up so we have an
        // ID to pass to Checkout the first time they upgrade. Cheap — Stripe
        // doesn't charge for customers without subscriptions.
        createCustomerOnSignUp: true,
        subscription: {
          enabled: true,
          plans: [
            {
              name: 'pro',
              priceId: env.STRIPE_PRICE_PRO ?? '',
              // No trial — pay from day one per product decision.
            },
          ],
          // Add tax + promo handling to the Checkout Session at create time.
          // Stripe Tax must be enabled in the dashboard for `automatic_tax`
          // to actually charge; with no registrations it'll just label
          // sales "tax not collected" which is correct for an un-registered
          // seller.
          getCheckoutSessionParams: async () => ({
            params: {
              automatic_tax: { enabled: true },
              tax_id_collection: { enabled: true }, // EU B2B reverse-charge
              allow_promotion_codes: true,
              billing_address_collection: 'required',
            },
          }),
          onSubscriptionCreated: async ({ subscription }) => {
            await setUserPlan(env, subscription.referenceId, planFromSubscriptionStatus(subscription.status))
          },
          onSubscriptionUpdate: async ({ subscription }) => {
            await setUserPlan(env, subscription.referenceId, planFromSubscriptionStatus(subscription.status))
          },
          onSubscriptionDeleted: async ({ subscription }) => {
            await setUserPlan(env, subscription.referenceId, 'local')
          },
        },
      }),
    )
  }

  return betterAuth({
    ...(database ? { database } : {}),
    // Mount under /v1/auth/* — the host is already api.getship.dev, so the
    // legacy /api/auth/* prefix was redundant. All gateway paths now live
    // under /v1/* uniformly.
    basePath: '/v1/auth',
    trustedOrigins: TRUSTED_ORIGINS,
    emailAndPassword: {
      enabled: true,
      requireEmailVerification: false,
      // sendResetPassword is fired by Better Auth's /forget-password endpoint.
      // The marketing site's /forgot-password page calls authClient.forgetPassword
      // which hits that endpoint and triggers this hook.
      sendResetPassword: async ({ user, url }) => {
        await sendEmail(env, resetPasswordEmail({ to: user.email, url }))
      },
    },
    emailVerification: {
      // Fired when sign-up completes (if requireEmailVerification is true) or
      // when the user requests re-send. Wired now so flipping verification on
      // later is one config change, not a missed hookup.
      sendVerificationEmail: async ({ user, url }) => {
        await sendEmail(env, verifyEmailEmail({ to: user.email, url }))
      },
    },
    socialProviders: {
      github: {
        clientId: env.GITHUB_CLIENT_ID ?? '',
        clientSecret: env.GITHUB_CLIENT_SECRET ?? '',
        // Signup needs identity only. The `repo` scope (read+write to all
        // private repos) is way out of proportion to "create an account".
        // If a downstream feature ever needs private-repo access, request
        // it incrementally at that feature's call site via Better Auth's
        // scope-upgrade flow — don't widen the signup default.
        scope: ['read:user', 'user:email'],
      },
      google: {
        clientId: env.GOOGLE_CLIENT_ID ?? '',
        clientSecret: env.GOOGLE_CLIENT_SECRET ?? '',
      },
      slack: {
        clientId: env.SLACK_CLIENT_ID ?? '',
        clientSecret: env.SLACK_CLIENT_SECRET ?? '',
      },
    },
    account: {
      // If the same email signs in with two different trusted providers, link
      // them onto the same user record instead of refusing the second one.
      accountLinking: {
        enabled: true,
        trustedProviders: ['google', 'github', 'slack'],
      },
    },
    user: {
      // Surface `plan` in the session payload so the marketing site's billing
      // and /account UIs can render without a second round-trip. Stays opt-in
      // ('input: false' means signup forms can't set it) and defaults to
      // 'local' for every new account. The Stripe plugin lifecycle hooks
      // above keep this column in sync with the canonical `subscription`
      // table.
      additionalFields: {
        plan: {
          type: ['local', 'pro'] as const,
          required: false,
          defaultValue: 'local',
          input: false,
        },
      },
      // Account deletion. The endpoint stays disabled unless explicitly turned
      // on — the site's /account page calls authClient.deleteUser() and used
      // to silently 404 (BA returns NOT_FOUND when this is off), leaving the
      // user signed out but every row intact. Two-step confirm via email:
      // first call returns "Verification email sent", the link in the email
      // hits /v1/auth/delete-user/callback?token=... which actually deletes.
      deleteUser: {
        enabled: true,
        sendDeleteAccountVerification: async ({ user, url }) => {
          await sendEmail(env, deleteAccountEmail({ to: user.email, url }))
        },
        // beforeDelete fires inside the callback handler after token verify
        // and BEFORE the user row + dependents are removed. This is the
        // narrow window where stripeCustomerId is still readable. Stripe
        // deletion is irreversible — we only reach this point because the
        // user clicked the verification link in their inbox.
        beforeDelete: async (user) => {
          if (!stripeClient) return
          const customerId = (user as { stripeCustomerId?: string | null }).stripeCustomerId
          if (!customerId) return
          try {
            await stripeClient.customers.del(customerId)
          } catch (err) {
            // Don't block the deletion on a Stripe failure (customer may
            // already be gone, or Stripe may be down). Log and let the row
            // deletion proceed — orphaning a Stripe customer is recoverable
            // by hand; blocking the user from deleting their account isn't.
            console.error('[auth:deleteUser] stripe customer delete failed', {
              userId: user.id,
              customerId,
              err: err instanceof Error ? err.message : String(err),
            })
          }
        },
      },
    },
    advanced: {
      // In production both the marketing site (getship.dev) and the gateway
      // (api.getship.dev) live under getship.dev; share the session cookie
      // across them. Locally we run on different localhost ports, where this
      // option is a no-op.
      crossSubDomainCookies: {
        enabled: true,
        domain: '.getship.dev',
      },
    },
    plugins: [
      ...plugins,
      // API keys — used by the CLI / daemon for long-lived bearer auth.
      // `apiKeyHeaders` accepts both the standard `x-api-key` header and
      // `Authorization` (Bearer …) so the existing daemon HTTP client doesn't
      // need to change shape. `enableSessionForAPIKeys` lets requireSession
      // accept API-key-authed requests without a separate code path.
      apiKeyPlugin({
        apiKeyHeaders: ['x-api-key', 'authorization'],
        defaultPrefix: 'ship_',
        defaultKeyLength: 48,
        enableSessionForAPIKeys: true,
      }),
    ],
  })
}

type Auth = Awaited<ReturnType<typeof makeAuth>>
let _auth: Auth | undefined
let _authPromise: Promise<Auth> | undefined

export function getAuth(env: Env): Promise<Auth> {
  if (_auth) return Promise.resolve(_auth)
  if (!_authPromise) {
    _authPromise = makeAuth(env).then((a) => {
      _auth = a
      return a
    })
  }
  return _authPromise
}
