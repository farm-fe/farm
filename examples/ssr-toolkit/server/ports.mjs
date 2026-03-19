import { createServer as createNetServer } from 'node:net';

const DEFAULT_MAX_PROBE = 30;
const DEFAULT_HMR_START_PORT = 9811;
const DEFAULT_HOST_START_PORT = 3011;

export function isPositiveInteger(value) {
  return Number.isInteger(value) && value > 0;
}

export async function isPortAvailable(portToCheck, hostname) {
  return new Promise((resolve) => {
    const server = createNetServer();
    server.unref();
    server.once('error', () => {
      resolve(false);
    });
    server.listen({ port: portToCheck, host: hostname }, () => {
      server.close(() => resolve(true));
    });
  });
}

async function resolvePortWithFallback(params) {
  const {
    label,
    envName,
    explicitPort,
    host,
    startPort,
    maxProbe,
    isPortAvailableFn
  } = params;

  if (explicitPort != null) {
    if (!isPositiveInteger(explicitPort)) {
      throw new Error(
        `[ssr-toolkit] invalid ${envName}="${process.env[envName]}", expected a positive integer.`
      );
    }

    const available = await isPortAvailableFn(explicitPort, host);
    if (!available) {
      throw new Error(`[ssr-toolkit] ${envName}=${explicitPort} is already in use.`);
    }

    return explicitPort;
  }

  for (let offset = 0; offset < maxProbe; offset++) {
    const candidate = startPort + offset;
    if (await isPortAvailableFn(candidate, host)) {
      return candidate;
    }
  }

  throw new Error(
    `[ssr-toolkit] failed to find an available ${label} port in range ${startPort}-${startPort + maxProbe - 1}.`
  );
}

export async function resolveOptionalDevHmrPort(params) {
  const {
    command,
    host,
    explicitHmrPort,
    startPort = DEFAULT_HMR_START_PORT,
    maxProbe = DEFAULT_MAX_PROBE,
    isPortAvailableFn = isPortAvailable
  } = params;

  if (command !== 'dev') {
    return undefined;
  }

  return resolvePortWithFallback({
    label: 'HMR',
    envName: 'SSR_HMR_PORT',
    explicitPort: explicitHmrPort,
    host,
    startPort,
    maxProbe,
    isPortAvailableFn
  });
}

export async function resolveHostPort(params) {
  const {
    host,
    explicitHostPort,
    startPort = DEFAULT_HOST_START_PORT,
    maxProbe = DEFAULT_MAX_PROBE,
    isPortAvailableFn = isPortAvailable
  } = params;

  return resolvePortWithFallback({
    label: 'host',
    envName: 'SSR_HOST_PORT',
    explicitPort: explicitHostPort,
    host,
    startPort,
    maxProbe,
    isPortAvailableFn
  });
}

