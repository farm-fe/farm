import React, { useState } from 'react';
import { ReportingFilters } from '../components/ReportingFilters';
import { RevenueSummary, DailyRevenuePanel, ChannelRevenuePanel, MerchantWorkflowActions } from '../components/ReportingPanels';
import { useStore } from '../store';
import { metrics } from '../../../packages/domain/index.mjs';
import { PageTitle } from '../../../packages/ui';

export default function Overview() {
  const { state, tenant } = useStore();
  const [filters, setFilters] = useState({ fromDay: 1, toDay: 30, channel: 'all' });
  const totals = metrics(state, tenant, filters);
  return <>
    <PageTitle eyebrow="Merchant intelligence" title="A clearer view of your business">
      Thirty days of trading, one connected workspace.
    </PageTitle>
    <ReportingFilters {...filters} onChange={setFilters} />
    <RevenueSummary totals={totals} />
    <DailyRevenuePanel totals={totals} fromDay={filters.fromDay} toDay={filters.toDay} />
    <ChannelRevenuePanel channels={totals.channels} title="Channel revenue distribution" />
    <MerchantWorkflowActions />
  </>;
}