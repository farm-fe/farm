import type { CampaignProduct, CampaignQuote } from './models.js';
export type { CampaignProduct, CampaignQuote } from './models.js';

/** Prepare a tenant-scoped campaign preview without mutating its catalog. */
export function quoteCampaign(
  tenantId: string,
  products: readonly CampaignProduct[],
  discount: number,
): CampaignQuote {
  if (!tenantId) throw new Error('A merchant is required');
  if (!Number.isInteger(discount) || discount < 0 || discount > 100)
    throw new Error('Invalid campaign discount');
  return {
    tenantId,
    discount,
    items: products.map(product => {
      if (product.tenantId !== tenantId) throw new Error('Product belongs to another merchant');
      if (!Number.isSafeInteger(product.price) || product.price < 0)
        throw new Error('Invalid product price');
      if (!Number.isSafeInteger(product.stock) || product.stock < 0)
        throw new Error('Invalid product stock');
      return {
        productId: product.id,
        originalPrice: product.price,
        salePrice: Math.round(product.price * (100 - discount) / 100),
        availableStock: product.stock,
      };
    }),
  };
}
