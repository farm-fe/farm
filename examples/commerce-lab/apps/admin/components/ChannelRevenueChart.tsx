import React, { useEffect, useRef } from 'react';
import { init, use, type EChartsType } from 'echarts/core';
import { PieChart } from 'echarts/charts';
import { LegendComponent, TooltipComponent } from 'echarts/components';
import { SVGRenderer } from 'echarts/renderers';
import { money } from '../../../packages/ui';

use([PieChart, LegendComponent, TooltipComponent, SVGRenderer]);
type Channel = { channel: string; revenue: number; orders: number };
export function ChannelRevenueChart({ channels }: { channels: Channel[] }) {
  const element = useRef<HTMLDivElement>(null);
  const chart = useRef<EChartsType | null>(null);
  useEffect(() => {
    if (!element.current) return;
    const instance = init(element.current, undefined, { renderer: 'svg' });
    chart.current = instance;
    const observer = new ResizeObserver(() => instance.resize());
    observer.observe(element.current);
    return () => { observer.disconnect(); instance.dispose(); chart.current = null; };
  }, []);
  useEffect(() => {
    chart.current?.setOption({ animation: false,
      color: ['#287957', '#466ca6', '#cc9745'],
      tooltip: { trigger: 'item', valueFormatter: (value: unknown) => money(Number(value) * 100) },
      legend: { bottom: 0 },
      series: [{ name: 'Channel revenue', type: 'pie', radius: ['38%', '62%'],
        stillShowZeroSum: false, label: { formatter: '{b}: {d}%' },
        data: channels.filter(row => row.revenue > 0).map(row => ({ name: row.channel, value: row.revenue / 100 })) }],
    }, { notMerge: true });
  }, [channels]);
  return <>
    <div data-testid="channel-revenue-chart">
      <div ref={element} role="img" aria-label="Channel revenue distribution" style={{ height: 320, width: '100%' }} />
      {!channels.some(row => row.revenue > 0) && <p>No channel revenue in this range.</p>}
    </div>
    <table aria-label="Channel revenue data"><thead><tr><th>Channel</th><th>Revenue</th><th>Orders</th></tr></thead>
      <tbody>{channels.map(row => <tr key={row.channel}><td>{row.channel}</td><td>{money(row.revenue)}</td><td>{row.orders}</td></tr>)}</tbody>
    </table>
  </>;
}
