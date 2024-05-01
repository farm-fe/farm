import { chromium } from "playwright-chromium";
import type { BrowserServer } from "playwright-chromium";
import type { GlobalSetupContext } from "vitest/node";

let browserServer: BrowserServer | undefined;

export async function setup({ provide }: GlobalSetupContext): Promise<void> {
  browserServer = await chromium.launchServer({
    headless: true
  });

  // eslint-disable-next-line @typescript-eslint/ban-ts-comment
  // @ts-ignore
  provide("wsEndpoint", browserServer.wsEndpoint());
}

export async function teardown(): Promise<void> {
  await browserServer?.close();
}
