import React, { useState } from 'react';
import { useStore } from '../store';
import { PageTitle } from '../../../packages/ui';
import { metrics } from '../../../packages/domain/index.mjs';
import { ReportingFilters } from '../components/ReportingFilters';
import { ChannelRevenuePanel } from '../components/ReportingPanels';
import { ChannelDetail } from '../components/ChannelDetail';
export default function Channels() {
  const { state, tenant } = useStore();
  const [filters, setFilters] = useState({ fromDay: 1, toDay: 30, channel: 'all' });
  const totals = metrics(state, tenant, filters);
  const channels = filters.channel === 'all' ? totals.channels : totals.channels.filter(row => row.channel === filters.channel);
  return <>
    <PageTitle eyebrow="Analytics / Channels" title="See where growth begins">Compare store, social and marketplace revenue.</PageTitle>
    <ReportingFilters {...filters} onChange={setFilters} />
    <ChannelRevenuePanel channels={totals.channels} title="Channel mix" />
    {channels.map(row => {
      const detail = metrics(state, tenant, { ...filters, channel: row.channel });
      return <ChannelDetail key={row.channel} {...row} days={detail.days.slice(filters.fromDay - 1, filters.toDay)} firstDay={filters.fromDay} />;
    })}
  </>;
}
