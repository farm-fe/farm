import { queryProducts } from './index.mjs';
export function filterProducts(state, tenant, { search = '', category = 'all', status = 'all' } = {}) {
  if (!state.tenants.some(row => row.id === tenant)) throw new Error('Unknown merchant.');
  if (!['all', 'draft', 'published'].includes(status)) throw new Error('Invalid product status.');
  return queryProducts(state, tenant, search.trim()).filter(product => (category === 'all' || product.category === category) && (status === 'all' || product.status === status));
}
