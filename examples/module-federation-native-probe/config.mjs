import path from "node:path";
import { fileURLToPath } from "node:url";
import federation from "@farmfe/plugin-module-federation";
const base = path.dirname(fileURLToPath(import.meta.url));
export function configurations(mode = "production") {
  return [
    ["remote-a", "providerA", 4101, "/deploy/a/"],
    ["remote-b", "providerB", 4102, "/deploy/b/"],
    ["host", "host", 4100, "/"],
  ].map(([dir, name, port, prefix]) => {
    const remote = dir !== "host";
    return {
      root: path.join(base, dir),
      configFile: false,
      mode,
      host: "127.0.0.1",
      port,
      minify: mode === "production",
      outDir: path.join(base, "dist", dir),
      server: {
        host: "127.0.0.1",
        port,
        strictPort: true,
        hmr:
          process.env.MF_HMR === "1"
            ? { host: "127.0.0.1", port, overlay: false }
            : false,
        allowedHosts:
          process.env.MF_HMR === "1" ? ["http://127.0.0.1:4100"] : [],
        cors: true,
        origin: `http://127.0.0.1:${port}`,
      },
      compilation: {
        input: remote
          ? { remoteEntry: "farm-mf-native:container" }
          : {
              index: process.env.MF_HMR === "1" ? "./hmr.html" : "./index.html",
            },
        output: {
          path: path.join(base, "dist", dir),
          targetEnv: "browser-esnext",
          publicPath:
            mode === "development"
              ? prefix
              : `http://127.0.0.1:${port}${prefix}`,
        },
        persistentCache: false,
        lazyCompilation: false,
        css: { transformToScript: process.env.MF_HMR === "1" },
        minify: mode === "production",
        sourcemap: false,
        presetEnv: false,
      },
      plugins: [
        {
          name: "probe-loopback-only",
          configureServer(server) {
            server.serverOptions.hostname = {
              host: "127.0.0.1",
              name: "127.0.0.1",
            };
            if (process.env.MF_HMR === "1")
              server.middlewares.use((req, res, next) => {
                if (req.url === "/__mf_hmr_test_disconnect") {
                  for (const client of server.ws.wss.clients)
                    client.terminate();
                  res.end("disconnected");
                } else next();
              });
          },
        },
        federation({
          name,
          hot: process.env.MF_HMR === "1",
          publicPath: `http://127.0.0.1:${port}${prefix}`,
          shared: {
            "react/jsx-runtime": {
              version: "18.3.1",
              requiredVersion: "^18.3.1",
              singleton: true,
              strictVersion: true,
              fallback: fileURLToPath(import.meta.resolve("react/jsx-runtime")),
            },
            react: {
              version: "18.3.1",
              requiredVersion: "^18.3.1",
              singleton: true,
              strictVersion: true,
              fallback: fileURLToPath(import.meta.resolve("react")),
            },
          },
          format: process.env.MF_FORMAT || "global",
          globalName:
            dir === "remote-a"
              ? "remoteA"
              : dir === "remote-b"
                ? "remoteB"
                : "host",
          ...(remote
            ? {
                exposes: { "./Button": "./Button.js", "./Link": "./Link.js" },
                remotes: {
                  peer: {
                    name: dir === "remote-a" ? "providerB" : "providerA",
                    globalName: dir === "remote-a" ? "remoteB" : "remoteA",
                    format: process.env.MF_FORMAT || "global",
                    entry:
                      dir === "remote-a"
                        ? "http://127.0.0.1:4102/deploy/b/remoteEntry.js"
                        : "http://127.0.0.1:4101/deploy/a/remoteEntry.js",
                  },
                },
              }
            : {
                remotes: {
                  remote: {
                    name: "providerA",
                    format: process.env.MF_FORMAT || "global",
                    globalName: "remoteA",
                    entry: "http://127.0.0.1:4101/deploy/a/remoteEntry.js",
                  },
                  other: {
                    name: "providerB",
                    format: process.env.MF_FORMAT || "global",
                    globalName: "remoteB",
                    entry: "http://127.0.0.1:4102/deploy/b/remoteEntry.js",
                  },
                  missing: {
                    name: "missingRemote",
                    format: process.env.MF_FORMAT || "global",
                    entry: "http://127.0.0.1:4101/missing/remoteEntry.js",
                  },
                },
              }),
        }),
      ],
    };
  });
}
