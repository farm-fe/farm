export interface CampaignProduct {
  id: string;
  tenantId: string;
  /** Integer cents. */
  price: number;
  stock: number;
}
export interface CampaignQuote {
  tenantId: string;
  discount: number;
  items: Array<{
    productId: string;
    originalPrice: number;
    salePrice: number;
    availableStock: number;
  }>;
}
