import React from 'react';
import type { CampaignQuote } from '../../../packages/sdk/src';
import { money } from '../../../packages/ui';
export function CampaignQuoteSummary({ quote }: { quote: CampaignQuote }) {
  const original = quote.items.reduce((sum, item) => sum + item.originalPrice, 0);
  const offered = quote.items.reduce((sum, item) => sum + item.salePrice, 0);
  return <dl><dt>Selected products</dt><dd data-testid="campaign-product-count">{quote.items.length}</dd><dt>Original unit basket</dt><dd>{money(original)}</dd><dt>Discounted unit basket</dt><dd data-testid="campaign-offer-total">{money(offered)}</dd><dt>Unit basket savings</dt><dd>{money(original - offered)}</dd><dd>One unit of every selected product. These totals are not a sales forecast.</dd></dl>;
}
