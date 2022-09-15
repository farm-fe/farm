import merge from 'lodash.merge';

export interface DevServerOptions {
  port?: number;
  https?: boolean;
  // http2?: boolean;
  writeToDisk?: boolean;
}

export type NormalizedDevServerOptions = Required<DevServerOptions>;

export const DEFAULT_DEV_SERVER_OPTIONS: NormalizedDevServerOptions = {
  port: 9000,
  https: false,
  // http2: false,
  writeToDisk: false,
};

export function normalizeDevServerOptions(
  options: DevServerOptions = {}
): NormalizedDevServerOptions {
  return merge(DEFAULT_DEV_SERVER_OPTIONS, options);
}
