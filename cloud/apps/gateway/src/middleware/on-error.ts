import type { Context } from 'hono';
import type { ContentfulStatusCode } from 'hono/utils/http-status';

export class AppError extends Error {
  constructor(
    public statusCode: number,
    message: string,
    public code?: string,
  ) {
    super(message);
    this.name = 'AppError';
  }
}

export const errorHandler = (err: Error, c: Context) => {
  console.error('Error occurred:', {
    name: err.name,
    message: err.message,
    stack: err.stack,
  });

  if (err instanceof AppError) {
    return c.json(
      { error: err.message, code: err.code },
      err.statusCode as ContentfulStatusCode,
    );
  }

  if (err.message.includes('Durable Object')) {
    return c.json({ error: 'Service temporarily unavailable' }, 503);
  }

  return c.json(
    { error: import.meta.env.PROD ? 'Internal server error' : err.message },
    500,
  );
};
