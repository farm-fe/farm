// import { URL } from 'url';
// import { IKoaProxiesOptions } from 'koa-proxies';
// import proxy from 'koa-proxies';
// import type { ServerOptions as HttpProxyServerOptions } from 'http-proxy';
export const proxyPlugin = ({ app, config }: any) => {
  console.log(config);
  console.log(app);

  if (!config.proxy) {
    return;
  }
  // const options = config.proxy;
};
