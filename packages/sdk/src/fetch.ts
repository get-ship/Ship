// Browser-facing fetch helper that pairs with the Rust-generated route
// catalog (`./generated/routes`). All Ship daemon URLs are relative; the
// `ship_session` cookie authenticates requests automatically.

export class ShipApiError extends Error {
  readonly status: number;
  readonly body: string;
  constructor(status: number, body: string) {
    super(`Ship API ${status}: ${body}`);
    this.status = status;
    this.body = body;
  }
}

/**
 * Fetch a Ship daemon endpoint, parse JSON, and throw `ShipApiError` on
 * non-2xx responses. Credentials (cookies) are always included so the
 * browser session cookie reaches the daemon.
 *
 * Typical call:
 * ```ts
 * import { listJobs, shipFetch } from '@ship/sdk'
 * const jobs = await shipFetch<JobRecord[]>(listJobs())
 * ```
 */
export async function shipFetch<T>(
  url: string,
  init?: RequestInit,
): Promise<T> {
  const res = await fetch(url, {
    credentials: 'include',
    headers: { 'Content-Type': 'application/json', ...(init?.headers || {}) },
    ...init,
  });
  if (!res.ok) {
    const text = await res.text().catch(() => '');
    throw new ShipApiError(res.status, text);
  }
  return res.json() as Promise<T>;
}
