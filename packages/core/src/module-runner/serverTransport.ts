import type { Server } from '../server/index.js';
import type { JsUpdateResult } from '../types/binding.js';
import type {
  ModuleRunnerInvokeHandlers,
  ModuleRunnerTransport,
  RunnerHotPayload,
  RunnerHotUpdate
} from './types.js';

export type ModuleRunnerHotBus = {
  subscribe: (cb: (payload: RunnerHotPayload) => void) => () => void;
};

export type ModuleRunnerTransportDeps = {
  invokeHandlers: ModuleRunnerInvokeHandlers;
  hotBus: ModuleRunnerHotBus;
};

export function createModuleRunnerTransportFromInvokeHandlers(
  deps: ModuleRunnerTransportDeps
): ModuleRunnerTransport {
  let disconnectHandler: (() => void) | null = null;

  return {
    connect({ onMessage, onDisconnection }) {
      onMessage({ type: 'connected' });

      let active = true;
      const unsubscribe = deps.hotBus.subscribe((payload) => {
        if (active) {
          onMessage(payload);
        }
      });

      disconnectHandler = () => {
        if (!active) {
          return;
        }
        active = false;
        unsubscribe();
        onDisconnection();
      };
    },
    disconnect() {
      disconnectHandler?.();
      disconnectHandler = null;
    },
    async invoke(name, data) {
      const handler = deps.invokeHandlers[name] as (
        ...args: unknown[]
      ) => Promise<unknown>;
      return (await handler(...(data as unknown[]))) as never;
    }
  };
}

export function createServerModuleRunnerTransport(
  server: Server
): ModuleRunnerTransport {
  let disconnectHandler: (() => void) | null = null;

  return {
    connect({ onMessage, onDisconnection }) {
      onMessage({ type: 'connected' });

      let active = true;
      const updateListener = (result: JsUpdateResult) => {
        if (!active) {
          return;
        }

        const changedModules = [
          ...result.changed,
          ...result.added,
          ...result.extraWatchResult.add
        ];

        if (changedModules.length > 0) {
          const updates: RunnerHotUpdate[] = changedModules.map((item) => ({
            type: 'js-update',
            path: item,
            acceptedPath: item,
            timestamp: Date.now()
          }));

          onMessage({
            type: 'update',
            updates
          } satisfies RunnerHotPayload);
          return;
        }

        if (result.removed.length > 0) {
          onMessage({ type: 'full-reload' });
        }
      };

      server.hmrEngine?.onUpdateFinish(updateListener);

      disconnectHandler = () => {
        active = false;
        onDisconnection();
      };
    },
    disconnect() {
      disconnectHandler?.();
      disconnectHandler = null;
    },
    invoke(name, data) {
      return server.invokeModuleRunner(name, data as never);
    }
  };
}
