import type { GlobalSetupContext } from 'vitest/node';
import { chromium } from 'playwright-chromium';
import type { BrowserServer } from 'playwright-chromium';
import { createServer, Server } from 'http';

let browserServer: BrowserServer | undefined;
let client: Server | undefined;

let port = 23000;

function addPort() {
  return (port += 10);
}

function setPort(_port: number) {
  return (port = _port);
}

setPort(9100);

export async function setup({ provide }: GlobalSetupContext): Promise<void> {
  browserServer = await chromium.launchServer({
    headless: true
  });

  client = createServer((req, res) => {
    if (req.url.startsWith('/port')) {
      res.end(addPort().toString());
      return;
    }
    // not found path
    res.statusCode = 404;
    res.end();
  });

  client.listen(12306);

  // eslint-disable-next-line @typescript-eslint/ban-ts-comment
  // @ts-ignore
  provide('wsEndpoint', browserServer.wsEndpoint());
}

export async function teardown(): Promise<void> {
  await browserServer?.close();
  await new Promise((resolve, reject) => {
    client.close((err) => {
      if (err) {
        reject(err);
      } else {
        resolve(undefined);
      }
    });
  });
}
