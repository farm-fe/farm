import React from 'react';
import { Panel, Stat, money } from '../../../packages/ui';
import { metrics } from '../../../packages/domain/index.mjs';
import { RevenueChart } from './RevenueChart';
import { ChannelRevenueChart } from './ChannelRevenueChart';

type Totals = ReturnType<typeof metrics>;

export function RevenueSummary({ totals }: { totals: Totals }) {
  return <div className="stats" data-testid="overview-metrics">
    <Stat label="Net revenue" value={money(totals.revenue)} detail="Excludes refunded orders" />
    <Stat label="Orders" value={totals.orders} detail="Paid, shipped and completed" />
    <Stat label="Live products" value={totals.products} detail="Ready for your next campaign" />
    <Stat label="Campaigns" value={totals.campaigns} detail="Published for this merchant" />
  </div>;
}

export function DailyRevenuePanel({ totals, fromDay, toDay }: { totals: Totals; fromDay: number; toDay: number }) {
  return <Panel title="Daily revenue">
    <RevenueChart values={totals.days.slice(fromDay - 1, toDay)} firstDay={fromDay} />
    {!totals.orders && <p>No paid orders in this reporting range.</p>}
    <div className="chart-caption"><span>DAY {fromDay}</span><span>DAY {toDay}</span></div>
  </Panel>;
}

export function ChannelRevenuePanel({ channels, title }: { channels: Totals['channels']; title: string }) {
  return <Panel title={title}><ChannelRevenueChart channels={channels} /></Panel>;
}

export function MerchantWorkflowActions() {
  return <div className="grid-two">
    <Panel title="Next steps">
      <a href="#/import">Import a new collection →</a>
      <p>Validate your product sheet in a Worker, review the errors, then publish.</p>
    </Panel>
    <Panel title="Connected workflows">
      <p>Products → Inventory → Campaigns → Orders → Revenue</p>
      <p>Every action stays inside the selected merchant.</p>
    </Panel>
  </div>;
}
