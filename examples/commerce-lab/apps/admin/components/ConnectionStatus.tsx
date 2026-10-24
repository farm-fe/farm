import React from 'react';

export type RequestStatus = { phase: 'pending' | 'success' | 'error'; message: string; code?: number };

export function ConnectionStatus({ status, onRetry }: { status: RequestStatus; onRetry: () => void }) {
  return <><p role={status.phase === 'error' ? 'alert' : 'status'}>
    {status.code && `HTTP ${status.code}: `}{status.message}</p>
    <button disabled={status.phase === 'pending'} onClick={onRetry}>Retry request</button></>;
}
