import type { Resource } from '@farmfe/runtime';
import { HmrOptions } from '../../config/index.js';
import type { JsUpdateResult } from '../../types/binding.js';
import type { WsServer } from '../ws.js';

export interface HmrBroadcasterOptions {
  ws: WsServer;
  hmrOptions?: HmrOptions;
}

/**
 * HMR Broadcaster - Responsible for broadcasting update messages to clients
 */
export class HmrBroadcaster {
  constructor(private readonly options: HmrBroadcasterOptions) {}

  /**
   * Broadcast update results
   */
  async broadcastUpdate(result: JsUpdateResult): Promise<void> {
    const message = this.formatUpdateMessage(result);
    this.broadcast(message);
  }

  /**
   * Broadcast error messages
   */
  async broadcastError(error: Error): Promise<void> {
    const serialization = error.message.replace(/\x1b\[[0-9;]*m/g, '');
    const errorMessage = JSON.stringify({
      type: 'error',
      err: { message: serialization },
      overlay: this.options.hmrOptions?.overlay ?? true
    });

    this.broadcast(errorMessage);
  }

  /**
   * Format update message
   */
  private formatUpdateMessage(result: JsUpdateResult): string {
    const dynamicResourcesMap = this.formatDynamicResources(
      result.dynamicResourcesMap
    );

    const {
      added,
      changed,
      removed,
      immutableModules,
      mutableModules,
      boundaries
    } = result;

    const messageObj = {
      type: 'farm-update',
      result: {
        added: this.formatModuleArray(added),
        changed: this.formatModuleArray(changed),
        removed: this.formatModuleArray(removed),
        immutableModules: immutableModules.trim(),
        mutableModules: mutableModules.trim(),
        boundaries,
        dynamicResourcesMap
      }
    };

    return JSON.stringify(messageObj);
  }

  /**
   * Format dynamic resources mapping
   */
  private formatDynamicResources(
    resourcesMap: Record<string, any> | null
  ): Record<string, Resource[]> | null {
    if (!resourcesMap) return null;

    const formatted: Record<string, Resource[]> = {};

    for (const [key, value] of Object.entries(resourcesMap)) {
      if (Array.isArray(value)) {
        formatted[key] = value.map((r) => {
          if (Array.isArray(r) && r.length >= 2) {
            return {
              path: r[0],
              type: r[1] as 'script' | 'link'
            };
          }
          // Handle other formats
          return r;
        });
      }
    }

    return formatted;
  }

  /**
   * Format module array, handle backslashes in paths
   */
  private formatModuleArray(modules: string[]): string {
    return modules
      .map((item) => `'${item.replaceAll('\\', '\\\\')}'`)
      .join(', ');
  }

  /**
   * Broadcast message to all clients
   */
  private broadcast(message: string): void {
    // Check if WebSocket server exists
    if (!this.options.ws || !this.options.ws.wss) {
      console.warn('[HMR] WebSocket server not initialized');
      return;
    }

    this.options.ws.wss.clients.forEach((client) => {
      if (client.readyState === 1) {
        client.send(message);
      }
    });
  }

  /**
   * Send custom message
   */
  sendCustomMessage(type: string, data: any): void {
    const message = JSON.stringify({ type, data });
    this.broadcast(message);
  }
}
