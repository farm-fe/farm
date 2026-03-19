const SSR_HOST = '127.0.0.1';

function parseOptionalNumber(raw) {
  if (raw == null || raw === '') {
    return undefined;
  }

  return Number(raw);
}

export function resolveSsrCommand(env = process.env) {
  if (env.SSR_COMMAND === 'preview' || env.SSR_MODE === 'preview') {
    return 'preview';
  }

  return 'dev';
}

export function resolveSsrEnvMode(command, env = process.env) {
  return env.SSR_ENV_MODE ?? (command === 'preview' ? 'production' : 'development');
}

export function resolveTemplateMode(env = process.env) {
  return env.SSR_TEMPLATE_MODE === 'ejs' ? 'ejs' : 'html';
}

export function createRuntimeConfig(env = process.env) {
  const command = resolveSsrCommand(env);
  return {
    host: SSR_HOST,
    command,
    mode: resolveSsrEnvMode(command, env),
    templateMode: resolveTemplateMode(env),
    explicitHmrPort: parseOptionalNumber(env.SSR_HMR_PORT),
    explicitHostPort: parseOptionalNumber(env.SSR_HOST_PORT)
  };
}

