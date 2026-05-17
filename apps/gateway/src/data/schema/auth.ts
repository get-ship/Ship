import { integer, sqliteTable, text } from 'drizzle-orm/sqlite-core'

// ---------------------------------------------------------------------------
// Better Auth tables — column names are SACRED, do not rename.
// Better Auth manages these table definitions internally.
// ---------------------------------------------------------------------------

export const user = sqliteTable('user', {
  id: text('id').notNull().primaryKey(),
  name: text('name').notNull(),
  email: text('email').notNull().unique(),
  emailVerified: integer('emailVerified', { mode: 'boolean' }).notNull(),
  image: text('image'),
  createdAt: integer('createdAt', { mode: 'timestamp' }).notNull(),
  updatedAt: integer('updatedAt', { mode: 'timestamp' }).notNull(),
  // Custom Better Auth additionalFields — mirrored here so drizzle queries
  // (e.g. /api/auth/daemon-attest) can read it without raw SQL.
  plan: text('plan').notNull().default('local'),
  // Stripe customer link — created on signup by the better-auth/plugins/stripe
  // plugin (createCustomerOnSignUp: true).
  stripeCustomerId: text('stripeCustomerId'),
})

export const session = sqliteTable('session', {
  id: text('id').notNull().primaryKey(),
  expiresAt: integer('expiresAt', { mode: 'timestamp' }).notNull(),
  token: text('token').notNull().unique(),
  createdAt: integer('createdAt', { mode: 'timestamp' }).notNull(),
  updatedAt: integer('updatedAt', { mode: 'timestamp' }).notNull(),
  ipAddress: text('ipAddress'),
  userAgent: text('userAgent'),
  userId: text('userId')
    .notNull()
    .references(() => user.id),
})

export const account = sqliteTable('account', {
  id: text('id').notNull().primaryKey(),
  accountId: text('accountId').notNull(),
  providerId: text('providerId').notNull(),
  userId: text('userId')
    .notNull()
    .references(() => user.id),
  accessToken: text('accessToken'),
  refreshToken: text('refreshToken'),
  idToken: text('idToken'),
  accessTokenExpiresAt: integer('accessTokenExpiresAt', { mode: 'timestamp' }),
  refreshTokenExpiresAt: integer('refreshTokenExpiresAt', { mode: 'timestamp' }),
  scope: text('scope'),
  password: text('password'),
  createdAt: integer('createdAt', { mode: 'timestamp' }).notNull(),
  updatedAt: integer('updatedAt', { mode: 'timestamp' }).notNull(),
})

export const verification = sqliteTable('verification', {
  id: text('id').notNull().primaryKey(),
  identifier: text('identifier').notNull(),
  value: text('value').notNull(),
  expiresAt: integer('expiresAt', { mode: 'timestamp' }).notNull(),
  createdAt: integer('createdAt', { mode: 'timestamp' }),
  updatedAt: integer('updatedAt', { mode: 'timestamp' }),
})

// ---------------------------------------------------------------------------
// CLI auth tables
// ---------------------------------------------------------------------------

export const cliAuthState = sqliteTable('cli_auth_state', {
  state: text('state').notNull().primaryKey(),
  codeChallenge: text('code_challenge').notNull(),
  redirectUri: text('redirect_uri').notNull(),
  createdAt: integer('created_at').notNull(),
})

export const cliAuthCodes = sqliteTable('cli_auth_codes', {
  code: text('code').notNull().primaryKey(),
  userId: text('user_id').notNull(),
  codeChallenge: text('code_challenge').notNull(),
  createdAt: integer('created_at').notNull(),
  used: integer('used').notNull().default(0),
})

// ---------------------------------------------------------------------------
// Better Auth API Key plugin — column names are SACRED, do not rename.
// The plugin (@better-auth/api-key) reads/writes this table directly.
// Issued via `auth.api.createApiKey(...)` from server routes (PKCE exchange).
// Validated automatically by the plugin's request middleware.
// ---------------------------------------------------------------------------

export const apikey = sqliteTable('apikey', {
  id: text('id').notNull().primaryKey(),
  name: text('name'),
  start: text('start'),                         // first chars for UI display
  prefix: text('prefix'),                       // plaintext, e.g. 'ship_'
  key: text('key').notNull(),                   // hashed
  userId: text('userId').notNull().references(() => user.id),
  refillInterval: integer('refillInterval'),
  refillAmount: integer('refillAmount'),
  lastRefillAt: integer('lastRefillAt', { mode: 'timestamp' }),
  enabled: integer('enabled', { mode: 'boolean' }),
  rateLimitEnabled: integer('rateLimitEnabled', { mode: 'boolean' }),
  rateLimitTimeWindow: integer('rateLimitTimeWindow'),
  rateLimitMax: integer('rateLimitMax'),
  requestCount: integer('requestCount'),
  remaining: integer('remaining'),
  lastRequest: integer('lastRequest', { mode: 'timestamp' }),
  expiresAt: integer('expiresAt', { mode: 'timestamp' }),
  createdAt: integer('createdAt', { mode: 'timestamp' }).notNull(),
  updatedAt: integer('updatedAt', { mode: 'timestamp' }).notNull(),
  permissions: text('permissions'),             // JSON
  metadata: text('metadata'),                   // JSON
})

// ---------------------------------------------------------------------------
// Better Auth Stripe plugin — column names are SACRED, do not rename.
// The plugin (better-auth/plugins/stripe) reads/writes this table directly.
// ---------------------------------------------------------------------------

export const subscription = sqliteTable('subscription', {
  id: text('id').notNull().primaryKey(),
  plan: text('plan').notNull(),
  referenceId: text('referenceId').notNull(), // = user.id (when no organization plugin)
  stripeCustomerId: text('stripeCustomerId'),
  stripeSubscriptionId: text('stripeSubscriptionId'),
  status: text('status').notNull(),
  periodStart: integer('periodStart', { mode: 'timestamp' }),
  periodEnd: integer('periodEnd', { mode: 'timestamp' }),
  cancelAtPeriodEnd: integer('cancelAtPeriodEnd', { mode: 'boolean' }),
  cancelAt: integer('cancelAt', { mode: 'timestamp' }),
  canceledAt: integer('canceledAt', { mode: 'timestamp' }),
  endedAt: integer('endedAt', { mode: 'timestamp' }),
  seats: integer('seats'),
  trialStart: integer('trialStart', { mode: 'timestamp' }),
  trialEnd: integer('trialEnd', { mode: 'timestamp' }),
  billingInterval: text('billingInterval'),
  stripeScheduleId: text('stripeScheduleId'),
})
