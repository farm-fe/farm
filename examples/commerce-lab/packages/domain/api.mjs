import { queryProducts, metrics } from './index.mjs';

export class ApiError extends Error {
  constructor(status, message) { super(message); this.status = status; }
}

/** Local reproducible API simulator; no external service or credentials. */
export function createMockApi({ state, tenantId, role, mode = 'success', delay = 80 }) {
  return {
    /** @param {string} resource @param {{ signal?: AbortSignal, tenantId?: string }} [options] */
    async request(resource, { signal, tenantId: requestedTenant = tenantId } = {}) {
      await new Promise((resolve, reject) => {
        const abort = () => { clearTimeout(timer); reject(new DOMException('Request cancelled', 'AbortError')); };
        const timer = setTimeout(() => { signal?.removeEventListener('abort', abort); resolve(); }, delay);
        if (signal?.aborted) abort();
        else signal?.addEventListener('abort', abort, { once: true });
      });
      if (requestedTenant !== tenantId || mode === 'forbidden' || (role === 'viewer' && resource === 'campaigns')) throw new ApiError(403, 'Permission denied for this workspace');
      if (mode === 'timeout') throw new ApiError(408, 'The simulated request timed out. Retry with a healthy connection.');
      const current = state();
      if (!current.tenants.some(t => t.id === tenantId)) throw new ApiError(404, 'Merchant not found');
      if (resource === 'products') return queryProducts(current, tenantId);
      if (resource === 'metrics') return metrics(current, tenantId);
      if (['orders', 'campaigns'].includes(resource)) return current[resource].filter(row => row.tenantId === tenantId);
      throw new ApiError(404, 'Resource not found');
    },
  };
}
