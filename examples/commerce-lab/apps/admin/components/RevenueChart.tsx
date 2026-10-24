import React, { useEffect, useRef } from 'react';
import { init, use, type EChartsType } from 'echarts/core';
import { BarChart } from 'echarts/charts';
import { GridComponent, TooltipComponent } from 'echarts/components';
import { SVGRenderer } from 'echarts/renderers';
import { money } from '../../../packages/ui';

use([BarChart, GridComponent, TooltipComponent, SVGRenderer]);

export function RevenueChart({ values, firstDay }: { values: number[]; firstDay: number }) {
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
    chart.current?.setOption({
      animation: false,
      grid: { left: 72, right: 24, top: 24, bottom: 48 },
      tooltip: { trigger: 'axis', valueFormatter: (value: unknown) => money(Number(value) * 100) },
      xAxis: { type: 'category', data: values.map((_, index) => `Day ${firstDay + index}`) },
      yAxis: { type: 'value', axisLabel: { formatter: '${value}' } },
      series: [{ name: 'Revenue', type: 'bar', data: values.map(value => value / 100), itemStyle: { color: '#287957' } }],
    }, { notMerge: true });
  }, [values, firstDay]);
  return <>
    <div ref={element} data-testid="revenue-chart" role="img" aria-label="Daily revenue chart" style={{ height: 320, width: '100%' }} />
    <details><summary>View daily revenue data</summary>
      <table aria-label="Daily revenue data"><thead><tr><th>Day</th><th>Revenue</th></tr></thead>
        <tbody>{values.map((value, index) => <tr key={index}><td>Day {firstDay + index}</td><td>{money(value)}</td></tr>)}</tbody>
      </table>
    </details>
  </>;
}
