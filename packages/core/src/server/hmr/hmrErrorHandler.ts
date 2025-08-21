import { convertErrorMessage } from '../../utils/error.js';
import type { Logger } from '../../utils/logger.js';

export interface HmrErrorHandlerOptions {
  logger: Logger;
  onError?: (error: Error) => void;
}

export interface ErrorContext {
  module?: string;
  timestamp?: number;
  retryCount?: number;
}

/**
 * HMR Error Handler - Unified error handling and recovery
 */
export class HmrErrorHandler {
  private errorHistory: Map<string, ErrorContext[]> = new Map();
  private readonly maxErrorHistory = 10;
  private readonly maxRetries = 3;

  constructor(private readonly options: HmrErrorHandlerOptions) {}

  /**
   * Handle HMR errors
   */
  async handleError(
    error: Error,
    context?: ErrorContext
  ): Promise<{ shouldRetry: boolean; formattedError: string }> {
    const formattedError = this.formatError(error);

    // Record error history
    this.recordError(error, context);

    // Log error
    this.options.logger.error(formattedError);

    // Trigger error callback
    this.options.onError?.(error);

    // Determine if should retry
    const shouldRetry = this.shouldRetry(error, context);

    return {
      shouldRetry,
      formattedError
    };
  }

  /**
   * Format error message
   */
  private formatError(error: Error): string {
    return convertErrorMessage(error);
  }

  /**
   * Record error history
   */
  private recordError(error: Error, context?: ErrorContext): void {
    const key = error.message;
    const history = this.errorHistory.get(key) || [];

    history.push({
      ...context,
      timestamp: Date.now()
    });

    // Limit history size
    if (history.length > this.maxErrorHistory) {
      history.shift();
    }

    this.errorHistory.set(key, history);
  }

  /**
   * Determine if should retry
   */
  private shouldRetry(error: Error, context?: ErrorContext): boolean {
    // Don't retry if max retries reached
    if ((context?.retryCount || 0) >= this.maxRetries) {
      return false;
    }

    // Some error types should not be retried
    const nonRepeatingErrors = [
      'ENOENT', // File not found
      'EACCES', // Permission denied
      'EMFILE' // Too many open files
    ];

    const errorMessage = error.message || '';
    if (nonRepeatingErrors.some((code) => errorMessage.includes(code))) {
      return false;
    }

    return true;
  }

  /**
   * Get error statistics
   */
  getErrorStats(): Map<string, number> {
    const stats = new Map<string, number>();

    for (const [key, history] of this.errorHistory) {
      stats.set(key, history.length);
    }

    return stats;
  }

  /**
   * Clear error history
   */
  clearHistory(): void {
    this.errorHistory.clear();
  }

  /**
   * Cleanup expired error records
   */
  cleanupOldErrors(maxAge = 3600000): void {
    // Default 1 hour
    const now = Date.now();

    for (const [key, history] of this.errorHistory) {
      const filtered = history.filter(
        (ctx) => now - (ctx.timestamp || 0) < maxAge
      );

      if (filtered.length === 0) {
        this.errorHistory.delete(key);
      } else {
        this.errorHistory.set(key, filtered);
      }
    }
  }
}
