import path from 'node:path';
import {fileURLToPath} from 'node:url';
import federation from '@farmfe/js-plugin-module-federation';
const base = path.dirname(fileURLToPath(import.meta.url));
export function configurations(mode = 'production') {
return [['remote-a','remoteA',4101,'/deploy/a/'],['remote-b','remoteB',4102,'/deploy/b/'],['host','host',4100,'/']].map(([dir, name, port, prefix]) => {
  const remote = dir !== 'host';
  return {root:path.join(base,dir), configFile:false, mode,host:'127.0.0.1',port,minify:mode === 'production',outDir:path.join(base,'dist',dir),
    server:{host:'127.0.0.1',port,strictPort:true,hmr:false,cors:true,origin:`http://127.0.0.1:${port}`},
    compilation:{
      ...(remote ? {} : {input:{index:'./index.html'}}),
      output:{path:path.join(base,'dist',dir),targetEnv:'browser-esnext',publicPath:mode === 'development' ? prefix : `http://127.0.0.1:${port}${prefix}`},
      persistentCache:false,lazyCompilation:false,css:{transformToScript:false}, minify:mode === 'production', sourcemap:true,
      presetEnv:false
    },
    plugins:[{name:'probe-loopback-only',configureServer(server) {
      server.serverOptions.hostname = {host:'127.0.0.1',name:'127.0.0.1'};
    }},federation({name,
      ...(remote ? {exposes:{'./Button':'./Button.js'}} : {remotes:{
        remote:{name:'remoteA',entry:'http://127.0.0.1:4101/deploy/a/remoteEntry.js'},
        other:{name:'remoteB',entry:'http://127.0.0.1:4102/deploy/b/remoteEntry.js'},
        missing:{name:'missingRemote',entry:'http://127.0.0.1:4101/missing/remoteEntry.js'}
      }})
    })]
  };
});
}
