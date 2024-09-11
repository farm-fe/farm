/**
 * The following is modified based on source found in
 * https://github.com/vitejs/vite
 *
 * MIT Licensed
 * Copyright (c) 2019-present, (Evan) You and Vite contributors
 * https://github.com/vitejs/vite/blob/main/LICENSE
 */

import type { AddressInfo, Server } from 'node:net';
import os from 'node:os';
import { UserServerConfig } from '../index.js';

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
  options: UserServerConfig,
  publicPath?: string
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
  const { port } = getAddressHostnamePort(address);
  const base = publicPath || '';

  if (hostname.host !== undefined && !wildcardHosts.has(hostname.host)) {
    const url = createServerUrl(protocol, hostname.name, port, base);
    if (loopbackHosts.has(hostname.host)) {
      local.push(url);
    } else {
      network.push(url);
    }
  } else {
    const networkInterfaces = Object.values(os.networkInterfaces()).flatMap(
      (nInterface) => nInterface || []
    );
    networkInterfaces
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
        host = host.includes(':') ? `[${host}]` : host;
        const url = createServerUrl(protocol, host, port, base);
        detail.address.includes('127.0.0.1')
          ? local.push(url)
          : network.push(url);
      });
  }

  return { local, network };
}

export async function resolveHostname(
  optionsHost: string | boolean | undefined
): Promise<Hostname> {
  let host: string | undefined;
  if (optionsHost === undefined || optionsHost === false) {
    host = 'localhost';
  } else if (optionsHost === true) {
    host = undefined;
  } else {
    host = optionsHost;
  }

  const name =
    host === undefined || wildcardHosts.has(host) ? 'localhost' : host;

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
