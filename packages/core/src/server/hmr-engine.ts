import { HmrOptions } from '../config/index.js';
import type { JsUpdateResult } from '../types/binding.js';
import { HmrBroadcaster } from './hmr/hmrBroadcaster.js';
import { HmrCoordinator } from './hmr/hmrCoordinator.js';
import { HmrErrorHandler } from './hmr/hmrErrorHandler.js';
import type { Server as FarmDevServer } from './index.js';

/**
 * Refactored HMR Engine
 * Separation of concerns: Coordinator handles update flow, Broadcaster handles message sending, Error Handler handles exceptions
 */
export class HmrEngine {
  private coordinator: HmrCoordinator;
  private broadcaster: HmrBroadcaster;
  private errorHandler: HmrErrorHandler;
  private _onUpdates: ((result: JsUpdateResult) => void)[] = [];
  private lastUpdateResult?: JsUpdateResult;

  constructor(private readonly devServer: FarmDevServer) {
    // Initialize broadcaster
    this.broadcaster = new HmrBroadcaster({
      ws: devServer.ws,
      hmrOptions: devServer.config?.server?.hmr as HmrOptions
    });

    // Initialize error handler
    this.errorHandler = new HmrErrorHandler({
      logger: devServer.logger,
      onError: (error) => {
        // Broadcast error to clients
        this.broadcaster.broadcastError(error);
      }
    });

    // Initialize coordinator
    this.coordinator = new HmrCoordinator({
      compiler: devServer.compiler,
      broadcaster: this.broadcaster,
      logger: devServer.logger,
      writeToDisk: devServer.config?.server?.writeToDisk
    });

    // Register update completion callback
    this.coordinator.onUpdateFinish((result) => {
      this.lastUpdateResult = result;
      this.callUpdates(result);
    });
  }

  /**
   * Trigger HMR update
   */
  async hmrUpdate(absPath: string | string[], force = false): Promise<void> {
    try {
      await this.coordinator.triggerUpdate(absPath, force);
    } catch (error) {
      const { shouldRetry, formattedError } =
        await this.errorHandler.handleError(error as Error, {
          module: Array.isArray(absPath) ? absPath[0] : absPath
        });

      if (shouldRetry) {
        // Retry after delay
        setTimeout(() => {
          this.hmrUpdate(absPath, force);
        }, 1000);
      } else {
        // Log error and notify clients
        this.devServer.logger.error(formattedError, { exit: true });
      }
    }
  }

  /**
   * Compatibility method for old recompileAndSendResult
   * @deprecated Use hmrUpdate instead
   */
  async recompileAndSendResult(): Promise<JsUpdateResult | void> {
    // This method is kept for backward compatibility
    // Delegate to coordinator to process the queue
    try {
      // Get pending paths from coordinator's queue
      const pendingPaths = this.coordinator.getPendingPaths();
      if (pendingPaths.length === 0) {
        return;
      }

      // Process the update queue
      await this.coordinator.triggerUpdate(pendingPaths, true);

      // Return the last update result for compatibility
      return this.lastUpdateResult;
    } catch (error) {
      this.devServer.logger.error(`recompileAndSendResult failed: ${error}`);
      return;
    }
  }

  /**
   * Register update completion callback
   */
  onUpdateFinish(cb: (result: JsUpdateResult) => void): void {
    this._onUpdates.push(cb);
  }

  /**
   * Call all update callbacks
   */
  callUpdates(result: JsUpdateResult): void {
    this._onUpdates?.forEach((cb) => cb(result));
  }

  /**
   * Get error statistics
   */
  getErrorStats(): Map<string, number> {
    return this.errorHandler.getErrorStats();
  }

  /**
   * Cleanup resources
   */
  dispose(): void {
    this.coordinator.dispose();
    this.errorHandler.clearHistory();
    this._onUpdates = [];
  }
}
