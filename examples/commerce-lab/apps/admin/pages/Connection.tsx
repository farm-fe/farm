import React, { useEffect, useState } from 'react';
import { useStore } from '../store';
import { createMockApi } from '../../../packages/domain/api.mjs';
import { PageTitle, Panel } from '../../../packages/ui';
import { ConnectionControls } from '../components/ConnectionControls';
import { ConnectionStatus, type RequestStatus } from '../components/ConnectionStatus';
import { ConnectionResult, type ConnectionValue } from '../components/ConnectionResult';
export default function Connection() {
  const { state, tenant, role } = useStore();
  const [mode, setMode] = useState('success');
  const [resource, setResource] = useState('products');
  const [attempt, setAttempt] = useState(0);
  const [status, setStatus] = useState<RequestStatus>({ phase: 'pending', message: 'Connecting…' });
  const [result, setResult] = useState<ConnectionValue | null>(null);
  useEffect(() => {
    const controller = new AbortController();
    setStatus({ phase: 'pending', message: 'Connecting…' });
    setResult(null);
    createMockApi({ state: () => state, tenantId: tenant, role, mode }).request(resource, { signal: controller.signal }).then(data => {
      if (controller.signal.aborted) return;
      const value = Array.isArray(data)
        ? { resource, count: data.length }
        : { resource, count: data.orders, revenue: data.revenue };
      setResult(value);
      setStatus({ phase: 'success', message: `Connected: ${value.count} merchant ${resource === 'metrics' ? 'non-refunded orders' : resource}` });
    }).catch(error => {
      if (!controller.signal.aborted && error.name !== 'AbortError') setStatus({ phase: 'error', message: error.message, code: error.status });
    });
    return () => controller.abort();
  }, [tenant, role, mode, resource, attempt, state]);
  return <><PageTitle eyebrow="Settings / Connection" title="Recover with confidence">Exercise fixed API outcomes without depending on an external server.</PageTitle>
    <Panel title="API simulator"><ConnectionControls mode={mode} resource={resource} onMode={setMode} onResource={setResource} />
      <ConnectionStatus status={status} onRetry={() => setAttempt(value => value + 1)} />
      {result && <ConnectionResult value={result} />}
    </Panel></>;
}
