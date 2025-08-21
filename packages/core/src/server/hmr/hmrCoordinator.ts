import { isAbsolute, relative } from 'node:path';
import { bold, cyan, green } from '@farmfe/utils/colors';
import type { Compiler } from '../../compiler/index.js';
import type { JsUpdateResult } from '../../types/binding.js';
import type { Logger } from '../../utils/logger.js';
import { HmrBroadcaster } from './hmrBroadcaster.js';
import { UpdateQueue } from './updateQueue.js';

export interface HmrCoordinatorOptions {
  compiler: Compiler;
  broadcaster: HmrBroadcaster;
  logger: Logger;
  writeToDisk?: boolean;
}

/**
 * HMR Coordinator - Responsible for coordinating the update process
 */
export class HmrCoordinator {
  private updateQueue: UpdateQueue;
  private updateCallbacks: ((result: JsUpdateResult) => void)[] = [];
  private pendingUpdate: Promise<void> | null = null;

  constructor(private readonly options: HmrCoordinatorOptions) {
    this.updateQueue = new UpdateQueue({
      maxRetries: 3,
      batchDelay: 50,
      maxBatchSize: 100
    });
  }

  /**
   * Trigger HMR update
   */
  async triggerUpdate(paths: string | string[], force = false): Promise<void> {
    const pathArray = Array.isArray(paths) ? paths : [paths];
    const validPaths = pathArray.filter((path) =>
      this.options.compiler.hasModule(path)
    );

    if (validPaths.length === 0) return;

    // Add to queue
    await this.updateQueue.add(validPaths, 0, force);

    // Start processing if compiler is idle and queue has content
    if (!this.options.compiler.compiling && this.updateQueue.size > 0) {
      await this.processUpdateQueue();
    }
  }

  /**
   * Process update queue
   */
  private async processUpdateQueue(): Promise<void> {
    // Avoid concurrent processing
    if (this.pendingUpdate) {
      return this.pendingUpdate;
    }

    this.pendingUpdate = this.doProcessUpdate();
    try {
      await this.pendingUpdate;
    } finally {
      this.pendingUpdate = null;
    }
  }

  /**
   * Execute actual update processing
   */
  private async doProcessUpdate(): Promise<void> {
    const batch = this.updateQueue.getNextBatch();
    if (batch.length === 0) return;

    const paths = batch.map((item) => item.path);
    const startTime = performance.now();

    try {
      // Register completion callback
      this.options.compiler.onUpdateFinish(async () => {
        // Continue processing if there are more updates
        if (this.updateQueue.size > 0) {
          await this.processUpdateQueue();
        }

        // Write to disk
        if (this.options.writeToDisk) {
          this.options.compiler.writeResourcesToDisk();
        }
      });

      // Execute compilation update
      const result = await this.options.compiler.update(paths);

      // Log update
      this.logUpdate(paths, performance.now() - startTime);

      // Broadcast update results
      await this.options.broadcaster.broadcastUpdate(result);

      // Trigger callbacks
      this.notifyUpdateCallbacks(result);
    } catch (error) {
      // Retry failed items
      this.updateQueue.retry(batch);

      // Broadcast error
      await this.options.broadcaster.broadcastError(error as Error);

      throw error;
    }
  }

  /**
   * Log update information
   */
  private logUpdate(paths: string[], duration: number): void {
    const root = this.options.compiler.config.root;
    let updatedFilesStr = paths
      .map((path) => {
        if (isAbsolute(path)) {
          return relative(root, path);
        } else {
          const resolvedPath = this.options.compiler.transformModulePath(
            root,
            path
          );
          return relative(root, resolvedPath);
        }
      })
      .join(', ');

    if (updatedFilesStr.length >= 100) {
      updatedFilesStr =
        updatedFilesStr.slice(0, 100) + `...(${paths.length} files)`;
    }

    this.options.logger.info(
      `${bold(cyan(updatedFilesStr))} updated in ${bold(green(this.options.logger.formatTime(duration)))}`
    );
  }

  /**
   * Register update completion callback
   */
  onUpdateFinish(callback: (result: JsUpdateResult) => void): void {
    this.updateCallbacks.push(callback);
  }

  /**
   * Notify all update callbacks
   */
  private notifyUpdateCallbacks(result: JsUpdateResult): void {
    this.updateCallbacks.forEach((cb) => cb(result));
  }

  /**
   * Get pending paths from the update queue
   */
  getPendingPaths(): string[] {
    return this.updateQueue.getPendingPaths();
  }

  /**
   * Cleanup resources
   */
  dispose(): void {
    this.updateQueue.clear();
    this.updateCallbacks = [];
    this.pendingUpdate = null;
  }
}
