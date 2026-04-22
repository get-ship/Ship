import type { Context, Next } from 'hono';
/**
 * Stubbed auth middleware.
 * For now, extracts project-id and user-id from headers.
 * Will be replaced with Better Auth session extraction.
 */
export declare const authMiddleware: (c: Context, next: Next) => Promise<(Response & import("hono").TypedResponse<{
    error: string;
}, 400, "json">) | undefined>;
