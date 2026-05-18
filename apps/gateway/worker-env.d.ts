declare namespace Cloudflare {
  interface GlobalProps {
    mainModule: typeof import('./src/index');
  }
}

interface RateLimit {
  limit(options: { key: string }): Promise<{ success: boolean }>
}

interface Env {
  PLATFORM: DurableObjectNamespace<import('../workers/platform/src/index').Platform>;
  PROJECT: DurableObjectNamespace<import('../workers/project/src/index').Project>;
  AUTH_DB: D1Database;
  REGISTRY_DB: D1Database;
  RATE_LIMITER_PUBLISH: RateLimit;
  RATE_LIMITER_CLAIM: RateLimit;
  RATE_LIMITER_INSTALL: RateLimit;
  RATE_LIMITER_SEARCH: RateLimit;
  RATE_LIMITER_STAR: RateLimit;
  RATE_LIMITER_AUTH: RateLimit;
  GITHUB_CLIENT_ID: string;
  GITHUB_CLIENT_SECRET: string;
  GITHUB_WEBHOOK_SECRET: string;
  GOOGLE_CLIENT_ID: string;
  GOOGLE_CLIENT_SECRET: string;
  SLACK_CLIENT_ID: string;
  SLACK_CLIENT_SECRET: string;
  STRIPE_SECRET_KEY: string;
  STRIPE_WEBHOOK_SECRET: string;
  STRIPE_PRICE_PRO: string;
  SHIP_CLOUD_ENABLED?: string;
  STRIPE_LIVE_MODE?: string;
  EMAIL_LIVE_MODE?: string;
  MACAROON_ENFORCEMENT?: string;
  GATEWAY_VERSION?: string;
  BUILD_SHA?: string;
  WAITLIST?: KVNamespace;
  EMAIL?: { send(input: { to: string; from: string; subject: string; html: string; text: string }): Promise<unknown> };
  SHIP_JWT_PRIVATE_KEY: string;
  SHIP_JWT_PRIVATE_KEY_ID: string;
  SHIP_JWT_PUBLIC_KEY: string;
  SHIP_JWT_PUBLIC_JWKS: string;
  BETTER_AUTH_SECRET: string;
  /** Issuer claim for /api/auth/daemon-attest JWTs. Brand domain, not API host. */
  SHIP_ATTESTATION_ISSUER?: string;
}
