/**
 * Transactional email via Cloudflare Email Service (beta).
 *
 * Binding: `EMAIL` (configured in wrangler.jsonc as `send_email`).
 * Sender:  no-reply@getship.dev (verified via Cloudflare DNS auto-config).
 *
 * If EMAIL binding isn't present (local dev without `wrangler dev --remote`),
 * we log the message and return as if sent. Better Auth's reset flow then
 * "succeeds" from the user's perspective; the developer sees the message
 * in the worker logs and clicks the URL manually.
 *
 * Beta caveat: Cloudflare's API may change pre-GA. Each template here is a
 * single function and the binding is one line — swap-out cost is small.
 */

import { flagEnabled } from '../middleware/feature-flags.js'

const FROM = 'SHIP <no-reply@getship.dev>'

export interface SendInput {
  to: string
  subject: string
  html: string
  text: string
}

interface EmailBinding {
  send(input: { to: string; from: string; subject: string; html: string; text: string }): Promise<unknown>
}

export async function sendEmail(env: Env, input: SendInput): Promise<void> {
  const email = (env as unknown as { EMAIL?: EmailBinding }).EMAIL
  if (!email || !flagEnabled(env, 'email_live')) {
    // Dev mode: log the message so the developer can see the URL.
    console.log('[email:dev-mode]', {
      to: input.to,
      subject: input.subject,
      text: input.text,
    })
    return
  }
  await email.send({
    from: FROM,
    to: input.to,
    subject: input.subject,
    html: input.html,
    text: input.text,
  })
}

// ── Templates ─────────────────────────────────────────────────────────────
// Inline HTML + text. Each one returns a SendInput. Kept dumb on purpose —
// no template engine, no React Email, no Handlebars. Templates are short,
// rarely change, and stay readable side-by-side with the rest of the code.

export function resetPasswordEmail(args: { to: string; url: string }): SendInput {
  const safeUrl = args.url
  return {
    to: args.to,
    subject: 'Reset your SHIP password',
    text: [
      'Someone (hopefully you) asked to reset your SHIP password.',
      '',
      `Click this link to set a new password (expires in 1 hour):`,
      safeUrl,
      '',
      `If you didn't request this, you can safely ignore this email.`,
      '',
      '— SHIP',
    ].join('\n'),
    html: htmlEnvelope(
      `<h1>Reset your password</h1>` +
      `<p>Someone (hopefully you) asked to reset your SHIP password. Click the button below to set a new one — the link expires in 1 hour.</p>` +
      ctaButton('Set a new password', safeUrl) +
      `<p class="muted">If you didn't request this, you can safely ignore this email.</p>`,
    ),
  }
}

export function verifyEmailEmail(args: { to: string; url: string }): SendInput {
  return {
    to: args.to,
    subject: 'Verify your SHIP email',
    text: [
      'Welcome to SHIP. Verify your email to finish setting up your account:',
      args.url,
      '',
      '— SHIP',
    ].join('\n'),
    html: htmlEnvelope(
      `<h1>Welcome to SHIP</h1>` +
      `<p>Verify your email to finish setting up your account.</p>` +
      ctaButton('Verify email', args.url),
    ),
  }
}

// ── HTML helpers ──────────────────────────────────────────────────────────

function htmlEnvelope(body: string): string {
  return `<!doctype html>
<html><head><meta charset="utf-8"><meta name="viewport" content="width=device-width">
<title>SHIP</title>
<style>
  body { margin: 0; background: #f5f5f4; color: #1c1917; font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif; line-height: 1.55; }
  .wrap { max-width: 560px; margin: 32px auto; padding: 32px 28px; background: #fff; border-radius: 14px; box-shadow: 0 1px 3px rgba(0,0,0,0.06); }
  .brand { font-family: "Syne", system-ui, sans-serif; font-weight: 700; letter-spacing: 0.04em; font-size: 18px; color: #1c1917; }
  h1 { font-family: "Syne", system-ui, sans-serif; font-weight: 700; font-size: 26px; letter-spacing: -0.02em; margin: 16px 0 12px; }
  p { font-size: 15px; margin: 12px 0; }
  .muted { color: #78716c; font-size: 13px; margin-top: 24px; }
  .cta { display: inline-block; padding: 12px 22px; background: #c2410c; color: #fff !important; text-decoration: none; border-radius: 10px; font-weight: 600; }
  .footer { max-width: 560px; margin: 16px auto; padding: 0 28px; color: #a8a29e; font-size: 12px; text-align: center; }
</style></head>
<body>
  <div class="wrap">
    <div class="brand">SHIP</div>
    ${body}
  </div>
  <div class="footer">SHIP · getship.dev</div>
</body></html>`
}

function ctaButton(label: string, href: string): string {
  return `<p style="margin: 24px 0;"><a class="cta" href="${href}">${label}</a></p>` +
         `<p class="muted">Or paste this URL into your browser:<br/><span style="font-family: ui-monospace, SF Mono, monospace; font-size: 12px; word-break: break-all;">${href}</span></p>`
}
