export type SsrErrorCode =
  | 'SSR_COMPILE_FAILED'
  | 'SSR_RENDER_FAILED'
  | 'SSR_TEMPLATE_MISSING'
  | 'SSR_TEMPLATE_PLACEHOLDER_MISSING'
  | 'SSR_ASSET_MANIFEST_MISSING'
  | 'SSR_ASSET_INJECT_FAILED'
  | 'SSR_RUNTIME_INTERNAL';

export type SsrError = {
  code: SsrErrorCode;
  message: string;
  cause?: unknown;
  debug?: {
    stack?: string;
    hint?: string;
  };
};

export function toSsrError(params: {
  code: SsrErrorCode;
  error: unknown;
  debug?: boolean;
  hint?: string;
}): SsrError {
  const message =
    params.error instanceof Error ? params.error.message : String(params.error);
  const debug =
    params.debug && params.error instanceof Error
      ? {
          stack: params.error.stack,
          hint: params.hint
        }
      : params.hint
        ? { hint: params.hint }
        : undefined;

  return {
    code: params.code,
    message,
    cause: params.error,
    debug
  };
}
