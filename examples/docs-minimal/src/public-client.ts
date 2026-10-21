import * as client from '@farmfe/docs/client';
// Keep every browser export reachable for the boundary integration test.
export const publicApi = Object.keys(client);
