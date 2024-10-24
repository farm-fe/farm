/**
 * The following is modified based on source found in
 * https://github.com/vitejs/vite
 *
 * MIT Licensed
 * Copyright (c) 2019-present, (Evan) You and Vite contributors
 * https://github.com/vitejs/vite/blob/main/LICENSE
 */

import { promises as dns } from 'node:dns';
import type { AddressInfo, Server } from 'node:net';
import os from 'node:os';
import { ResolvedUserConfig } from '../config/types.js';

export interface ResolvedServerUrls {
  local: string[];
  network: string[];
}

export interface Hostname {
  host: string | undefined;
  name: string;
}

export const urlRegex = /^(https?:)?\/\/([^/]+)/;

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
  config: ResolvedUserConfig,
  type: 'dev' | 'preview' = 'dev'
): Promise<ResolvedServerUrls> {
  const address = server.address();
  const isAddressInfo = (x: any): x is AddressInfo => x?.address;

  if (!isAddressInfo(address)) {
    return { local: [], network: [] };
  }
  const serverOptions = type == 'dev' ? config.server : config.preview;
  const local: string[] = [];
  const network: string[] = [];
  const hostname = await resolveHostname(serverOptions.host);
  const protocol = serverOptions.https ? 'https' : 'http';
  const port = address.port;
  const base = config.compilation.output.publicPath;

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
  let name = host === undefined || wildcardHosts.has(host) ? 'localhost' : host;

  if (host === 'localhost') {
    // See #8647 for more details.
    const localhostAddr = await getLocalhostAddressIfDiffersFromDNS();
    if (localhostAddr) {
      name = localhostAddr;
    }
  }

  return { host, name };
}

function getAddressHostnamePort(server: AddressInfo): {
  host: string;
  port: number;
} {
  const hostname = server.address || 'localhost';
  const port = server.port;
  return { host: hostname, port };
}

function createServerUrl(
  protocol: string,
  hostname: string,
  port: number,
  publicPath: string
): string {
  const hostnameName = hostname.includes(':') ? `[${hostname}]` : hostname;
  return `${protocol}://${hostnameName}:${port}${publicPath}`;
}

export const teardownSIGTERMListener = (
  callback: () => Promise<void>
): void => {
  process.off('SIGTERM', callback);
  if (process.env.CI !== 'true') {
    process.stdin.off('end', callback);
  }
};

/**
 * Returns resolved localhost address when `dns.lookup` result differs from DNS
 *
 * `dns.lookup` result is same when defaultResultOrder is `verbatim`.
 * Even if defaultResultOrder is `ipv4first`, `dns.lookup` result maybe same.
 * For example, when IPv6 is not supported on that machine/network.
 */
export async function getLocalhostAddressIfDiffersFromDNS(): Promise<
  string | undefined
> {
  const [nodeResult, dnsResult] = await Promise.all([
    dns.lookup('localhost'),
    dns.lookup('localhost', { verbatim: true })
  ]);
  const isSame =
    nodeResult.family === dnsResult.family &&
    nodeResult.address === dnsResult.address;
  return isSame ? undefined : nodeResult.address;
}
