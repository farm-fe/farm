export type HMRPayload =
  | ConnectedPayload
  | UpdatePayload
  | FullReloadPayload
  | CustomPayload
  | ErrorPayload
  | PrunePayload

export interface ConnectedPayload {
  type: 'connected'
}

export interface UpdatePayload {
  type: 'update'
  updates: Update[]
}

export interface Update {
  type: 'js-update' | 'css-update'
  path: string
  acceptedPath: string
  timestamp: number
  /** @internal */
  explicitImportRequired?: boolean
  /** @internal */
  isWithinCircularImport?: boolean
  /** @internal */
  ssrInvalidates?: string[]
}

export interface PrunePayload {
  type: 'prune'
  paths: string[]
}

export interface FullReloadPayload {
  type: 'full-reload'
  path?: string
  /** @internal */
  triggeredBy?: string
}

export interface CustomPayload {
  type: 'custom'
  event: string
  data?: any
}

export interface ErrorPayload {
  type: 'error'
  err: {
    [name: string]: any
    message: string
    stack: string
    id?: string
    frame?: string
    plugin?: string
    pluginCode?: string
    loc?: {
      file?: string
      line: number
      column: number
    }
  }
}

export interface CustomEventMap {
  'vite:beforeUpdate': UpdatePayload
  'vite:afterUpdate': UpdatePayload
  'vite:beforePrune': PrunePayload
  'vite:beforeFullReload': FullReloadPayload
  'vite:error': ErrorPayload
  'vite:invalidate': InvalidatePayload
  'vite:ws:connect': WebSocketConnectionPayload
  'vite:ws:disconnect': WebSocketConnectionPayload
}

export interface WebSocketConnectionPayload {
  webSocket: WebSocket
}

export interface InvalidatePayload {
  path: string
  message: string | undefined
}

export type InferCustomEventPayload<T extends string> =
  T extends keyof CustomEventMap ? CustomEventMap[T] : any


export interface HMRBroadcasterClient {
  /**
   * Send event to the client
   */
  send(payload: HMRPayload): void
  /**
   * Send custom event
   */
  send(event: string, payload?: CustomPayload['data']): void
}
