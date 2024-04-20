# Farm Vue SSR 例子

Vue + Vue Router + SSR.

## 启动

```sh
npm start; # 启动开发服务器
npm run watch; # 以 development 打包 SSR 服务端入口
```

打开 `http://localhost:9000` 查看页面

编译服务端入口时注意将 farm 配置中 `compilation.lazyCompilation` 设置为 `false`

```js
export default {
  input: {
    index: 'server.ts'
  },
  compilation: {
    lazyCompilation: false
  }
};
```

## 打包生产环境

同时打包 SSR 客户端和服务端

```sh
npm run build && npm run build:server
```

启动生产服务器:

```sh
NODE_ENV=production node server.js
```

打开 `http://localhost:3000` 访问页面
