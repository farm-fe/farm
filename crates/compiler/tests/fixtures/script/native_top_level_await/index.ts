const delay = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

// @ts-ignore
await delay(3000);