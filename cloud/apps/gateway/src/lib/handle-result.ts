import type { SerializedResult } from '@ship-cloud/types';
import type { Context } from 'hono';
import type { ContentfulStatusCode } from 'hono/utils/http-status';

export function handleResult<T>(
  result: SerializedResult<T>,
  c: Context,
  successStatus: ContentfulStatusCode = 200,
) {
  if (result.isFailure) {
    const status = result.errorCode === 'ENTITY_NOT_FOUND' ? 404 : 400;

    if (status >= 500) {
      console.error(`[Result Failure ${status}] ${result.errorCode}: ${result.errorMessage}`);
    } else {
      console.warn(`[Result Failure ${status}] ${result.errorCode}: ${result.errorMessage}`);
    }

    return c.json(
      { error: result.errorMessage, code: result.errorCode },
      status as ContentfulStatusCode,
    );
  }

  return c.json<T>(result.value, successStatus);
}
