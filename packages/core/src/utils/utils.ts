import type { AddressInfo, Server } from 'node:net';
import os from 'node:os';

export interface ResolvedServerUrls {
  local: string[];
  network: string[];
}

export interface Hostname {
  host: string | undefined;
  name: string;
}

export const loopbackHosts = new Set([
  'localhost',
  '127.0.0.1',
  '::1',
  '0000:0000:0000:0000:0000:0000:0000:0001'
]);

export const wildcardHosts = new Set([
  '0.0.0.0',
  '::',
  '0000:0000:0000:0000:0000:0000:0000:0000'
]);

export async function resolveServerUrls(
  server: Server,
  options: any,
  config: any
): Promise<ResolvedServerUrls> {
  const address = server.address();

  const isAddressInfo = (x: any): x is AddressInfo => x?.address;
  if (!isAddressInfo(address)) {
    return { local: [], network: [] };
  }

  const local: string[] = [];
  const network: string[] = [];
  const hostname = await resolveHostname(options.host);

  const protocol = options.https ? 'https' : 'http';
  const port = address.port;
  const base = config.compilation.output.publicPath ?? '';

  if (hostname.host !== undefined && !wildcardHosts.has(hostname.host)) {
    let hostnameName = hostname.name;
    // ipv6 host
    if (hostnameName.includes(':')) {
      hostnameName = `[${hostnameName}]`;
    }
    const address = `${protocol}://${hostnameName}:${port}${base}`;
    if (loopbackHosts.has(hostname.host)) {
      local.push(address);
    } else {
      network.push(address);
    }
  } else {
    Object.values(os.networkInterfaces())
      .flatMap((nInterface) => nInterface ?? [])
      .filter(
        (detail) =>
          detail &&
          detail.address &&
          (detail.family === 'IPv4' ||
            // @ts-expect-error Node 18.0 - 18.3 returns number
            detail.family === 4)
      )
      .forEach((detail) => {
        let host = detail.address.replace('127.0.0.1', hostname.name);
        // ipv6 host
        if (host.includes(':')) {
          host = `[${host}]`;
        }
        const url = `${protocol}://${host}:${port}${base}`;
        if (detail.address.includes('127.0.0.1')) {
          local.push(url);
        } else {
          network.push(url);
        }
      });
  }
  return { local, network };
}

export async function resolveHostname(
  optionsHost: string | boolean | undefined
): Promise<Hostname> {
  let host: string | undefined;
  if (optionsHost === undefined || optionsHost === false) {
    // Use a secure default
    host = 'localhost';
  } else if (optionsHost === true) {
    // If passed --host in the CLI without arguments
    host = undefined; // undefined typically means 0.0.0.0 or :: (listen on all IPs)
  } else {
    host = optionsHost;
  }

  // Set host name to localhost when possible
  const name =
    host === undefined || wildcardHosts.has(host) ? 'localhost' : host;

  return { host, name };
}
