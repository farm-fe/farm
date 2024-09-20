import { ResolvedUserConfig } from '../config/types.js';
import { Compiler } from './index.js';

export function createCompiler(resolvedUserConfig: ResolvedUserConfig) {
  const {
    jsPlugins,
    rustPlugins,
    compilation: compilationConfig,
    logger
  } = resolvedUserConfig;

  const compiler = new Compiler(
    {
      config: compilationConfig,
      jsPlugins,
      rustPlugins
    },
    logger
  );
  return compiler;
}

// export async function createBundleHandler(
//   resolvedUserConfig: ResolvedUserConfig,
//   logger: Logger,
//   watchMode = false
// ) {
//   const compiler = await createCompiler(resolvedUserConfig, logger);

//   await compilerHandler(
//     async () => {
//       if (resolvedUserConfig.compilation?.output?.clean) {
//         compiler.removeOutputPathDir();
//       }

//       try {
//         await compiler.compile();
//       } catch (err) {
//         // throw new Error(logError(err) as unknown as string);
//         throw new Error(err as unknown as string);
//       }
//       compiler.writeResourcesToDisk();
//     },
//     resolvedUserConfig,
//     logger
//   );

//   if (resolvedUserConfig.compilation?.watch || watchMode) {
//     const watcher = new FileWatcher(compiler, resolvedUserConfig, logger);
//     await watcher.watch();
//     return watcher;
//   }
// }
