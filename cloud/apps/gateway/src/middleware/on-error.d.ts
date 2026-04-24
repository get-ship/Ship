import type { Context } from 'hono';
export declare class AppError extends Error {
    statusCode: number;
    code?: string | undefined;
    constructor(statusCode: number, message: string, code?: string | undefined);
}
export declare const errorHandler: (err: Error, c: Context) => (Response & import("hono").TypedResponse<{
    error: string;
    code: string | undefined;
}, 400 | 100 | 102 | 103 | 200 | 201 | 202 | 203 | 206 | 207 | 208 | 226 | 300 | 301 | 302 | 303 | 305 | 306 | 307 | 308 | 401 | 402 | 403 | 404 | 405 | 406 | 407 | 408 | 409 | 410 | 411 | 412 | 413 | 414 | 415 | 416 | 417 | 418 | 421 | 422 | 423 | 424 | 425 | 426 | 428 | 429 | 431 | 451 | 500 | 501 | 502 | 503 | 504 | 505 | 506 | 507 | 508 | 510 | 511 | -1, "json">) | (Response & import("hono").TypedResponse<{
    error: string;
}, 503, "json">) | (Response & import("hono").TypedResponse<{
    error: string;
}, 500, "json">);
