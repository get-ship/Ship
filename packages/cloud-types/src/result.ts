/**
 * Serialized result for transport across DO/RPC boundaries.
 */
export type SerializedResult<T> =
  | {
    isSuccess: true;
    isFailure: false;
    value: T;
  }
  | {
    isSuccess: false;
    isFailure: true;
    errorMessage: string;
    errorCode?: string;
  };

/**
 * Result type for explicit error handling without exceptions.
 */
export class Result<T, E = Error | Error[]> {
  private constructor(
    public readonly isSuccess: boolean,
    private readonly _value?: T,
    private readonly _error?: E,
  ) {}

  static ok<T>(value?: T): Result<T, never> {
    return new Result<T, never>(true, value);
  }

  static fail<E = Error>(error: E): Result<never, E> {
    return new Result<never, E>(false, undefined, error);
  }

  serialize(): SerializedResult<T> {
    if (this.isSuccess) {
      return {
        isSuccess: true,
        isFailure: false,
        value: this._value!,
      };
    }

    const error: any = this._error;
    const errorCode = error?.code || error?.errorCode;

    return {
      isSuccess: false,
      isFailure: true,
      errorMessage: this.errorMessage,
      errorCode: typeof errorCode === 'string' ? errorCode : undefined,
    };
  }

  toJSON(): SerializedResult<T> {
    return this.serialize();
  }

  get value(): T {
    if (!this.isSuccess) {
      throw new Error('Cannot get value from failed result');
    }
    return this._value!;
  }

  get error(): E {
    if (this.isSuccess) {
      throw new Error('Cannot get error from successful result');
    }
    return this._error!;
  }

  get isFailure(): boolean {
    return !this.isSuccess;
  }

  get errorMessage(): string {
    if (this.isSuccess) {
      throw new Error('Cannot get error message from successful result');
    }
    const error: any = this._error;
    if (Array.isArray(error)) {
      return error[0]?.message ?? 'An unknown error occurred.';
    }
    return error?.message ?? 'An unknown error occurred.';
  }

  map<U>(fn: (value: T) => U): Result<U, E> {
    if (this.isFailure) {
      return Result.fail(this.error);
    }
    return Result.ok(fn(this.value));
  }

  static combine<E = Error>(results: Result<unknown, E>[]): Result<true, E[]> {
    const failedResults = results.filter((r) => r.isFailure);

    if (failedResults.length > 0) {
      const errors = failedResults.map((r) => r.error);
      return Result.fail(errors);
    }

    return Result.ok(true);
  }
}
