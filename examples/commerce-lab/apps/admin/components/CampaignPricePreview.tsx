import React, { useState } from 'react';
import type { CampaignQuote } from '../../../packages/sdk/src';
import { CampaignQuoteSummary } from './CampaignQuoteSummary';
import { CampaignPriceTable } from './CampaignPriceTable';
import { Pagination } from '../../../packages/ui/Pagination';
export function CampaignPricePreview({ quote, products }: { quote: CampaignQuote; products: Array<{ id: string; name: string; sku: string }> }) {
  const [page, setPage] = useState(0);
  const pages = Math.max(1, Math.ceil(quote.items.length / 10));
  const visiblePage = Math.min(page, pages - 1);
  return <><CampaignQuoteSummary quote={quote} /><CampaignPriceTable items={quote.items.slice(visiblePage * 10, (visiblePage + 1) * 10)} products={products} />
    {quote.items.length > 10 && <Pagination page={visiblePage} pages={pages} label="campaign prices" onChange={setPage} />}</>;
}
