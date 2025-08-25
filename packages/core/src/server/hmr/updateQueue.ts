import { existsSync } from 'node:fs';
import { stat } from 'node:fs/promises';

export interface UpdateItem {
  path: string;
  priority: number;
  timestamp: number;
  retryCount?: number;
}

export interface UpdateQueueOptions {
  maxRetries?: number;
  batchDelay?: number;
  maxBatchSize?: number;
}

/**
 * Manages HMR update queue with support for batching, priority, and deduplication
 */
export class UpdateQueue {
  private queue: Map<string, UpdateItem> = new Map();
  private processing = false;
  private batchTimer: NodeJS.Timeout | null = null;
  private lastModifiedCache: Map<string, string> = new Map();

  constructor(
    private readonly options: UpdateQueueOptions = {
      maxRetries: 3,
      batchDelay: 50,
      maxBatchSize: 100
    }
  ) {}

  /**
   * Add files to the update queue
   */
  async add(
    paths: string | string[],
    priority = 0,
    force = false
  ): Promise<void> {
    const pathArray = Array.isArray(paths) ? paths : [paths];

    for (const path of pathArray) {
      if (!force && existsSync(path)) {
        const lastModified = this.lastModifiedCache.get(path);
        const currentTimestamp = (await stat(path)).mtime.toISOString();

        if (lastModified === currentTimestamp) {
          continue;
        }

        this.lastModifiedCache.set(path, currentTimestamp);
      }

      // Add or update queue item
      const existing = this.queue.get(path);
      if (!existing || existing.priority < priority) {
        this.queue.set(path, {
          path,
          priority,
          timestamp: Date.now(),
          retryCount: existing?.retryCount || 0
        });
      }
    }
  }

  /**
   * Get the next batch of updates to process
   */
  getNextBatch(): UpdateItem[] {
    if (this.queue.size === 0) return [];

    // Sort by priority and timestamp
    const sorted = Array.from(this.queue.values())
      .sort((a, b) => {
        if (a.priority !== b.priority) {
          return b.priority - a.priority;
        }
        return a.timestamp - b.timestamp;
      })
      .slice(0, this.options.maxBatchSize);

    // Remove from queue
    sorted.forEach((item) => this.queue.delete(item.path));

    return sorted;
  }

  /**
   * Retry failed update items
   */
  retry(items: UpdateItem[]): void {
    for (const item of items) {
      if ((item.retryCount || 0) < (this.options.maxRetries || 3)) {
        this.queue.set(item.path, {
          ...item,
          retryCount: (item.retryCount || 0) + 1,
          timestamp: Date.now()
        });
      }
    }
  }

  /**
   * Clear the queue
   */
  clear(): void {
    this.queue.clear();
    if (this.batchTimer) {
      clearTimeout(this.batchTimer);
      this.batchTimer = null;
    }
  }

  /**
   * Get queue size
   */
  get size(): number {
    return this.queue.size;
  }

  /**
   * Check if processing
   */
  get isProcessing(): boolean {
    return this.processing;
  }

  /**
   * Set processing status
   */
  setProcessing(value: boolean): void {
    this.processing = value;
  }

  /**
   * Get all pending paths
   */
  getPendingPaths(): string[] {
    return Array.from(this.queue.keys());
  }
}
