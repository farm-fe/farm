import type { CompilationContext } from '@farmfe/core';
import type Less from 'less';
export declare function createLessResolvePlugin(less: typeof Less, ctx: CompilationContext, resolvedPath: string): Less.Plugin;
