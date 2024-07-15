import {
  HMRBroadcasterClient,
  HMRPayload,
  InferCustomEventPayload
} from './type.js';

export interface HMRChannel {
  /**
   * Unique channel name
   */
  name: string;
  /**
   * Broadcast events to all clients
   */
  send(payload: HMRPayload): void;
  /**
   * Send custom event
   */
  send<T extends string>(event: T, payload?: InferCustomEventPayload<T>): void;
  /**
   * Handle custom event emitted by `import.meta.hot.send`
   */
  on<T extends string>(
    event: T,
    listener: (
      data: InferCustomEventPayload<T>,
      client: HMRBroadcasterClient,
      ...args: any[]
    ) => void
  ): void;
  on(event: 'connection', listener: () => void): void;
  /**
   * Unregister event listener
   */
  off(event: string, listener: Function): void;
  /**
   * Start listening for messages
   */
  listen(): void;
  /**
   * Disconnect all clients, called when server is closed or restarted.
   */
  close(): void;
}
