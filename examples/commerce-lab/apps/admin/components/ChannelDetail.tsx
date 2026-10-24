import React from 'react';
import { Panel, Stat, money } from '../../../packages/ui';
import { RevenueChart } from './RevenueChart';

export function ChannelDetail({ channel, revenue, orders, days, firstDay }: {
  channel: string; revenue: number; orders: number; days: number[]; firstDay: number;
}) {
  return <div data-testid={`channel-detail-${channel}`}>
    <Panel title={`${channel} performance`}>
      <div className="grid-two">
        <Stat label="Revenue" value={money(revenue)} detail={`${orders} net orders`} />
        <Stat label="Average order value" value={money(orders ? Math.round(revenue / orders) : 0)} detail="Revenue divided by non-refunded orders" />
      </div>
      <RevenueChart values={days} firstDay={firstDay} />
      {!orders && <p>No paid orders for this channel in the selected range.</p>}
    </Panel>
  </div>;
}
