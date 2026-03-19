import type { SsrError } from './errors.js';
import type { SsrRuntimeCommand, SsrRuntimeMeta } from './runtime-types.js';

export type SsrRuntimeHooks = {
  onCompileStart?: (ctx: { kind: 'client' | 'server' }) => void;
  onCompileEnd?: (ctx: { kind: 'client' | 'server'; ms: number }) => void;
  onInvalidate?: (ctx: {
    kind: 'client' | 'server';
    reason: 'update' | 'rebuild';
    added: number;
    changed: number;
    removed: number;
    ms?: number;
  }) => void;
  onRenderStart?: (ctx: {
    requestId: string;
    url: string;
    command: SsrRuntimeCommand;
    mode: string;
    runtime: SsrRuntimeMeta;
  }) => void;
  onRenderEnd?: (ctx: {
    requestId: string;
    url: string;
    command: SsrRuntimeCommand;
    mode: string;
    runtime: SsrRuntimeMeta;
    ms: number;
    error?: SsrError;
  }) => void;
  onAssetInject?: (ctx: {
    requestId: string;
    url: string;
    css: number;
    preload: number;
    scripts: number;
  }) => void;
};
