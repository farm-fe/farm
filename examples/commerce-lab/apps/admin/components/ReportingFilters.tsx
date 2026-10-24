import React from 'react';

export function ReportingFilters({ fromDay, toDay, channel, onChange }: {
  fromDay: number; toDay: number; channel: string;
  onChange: (value: { fromDay: number; toDay: number; channel: string }) => void;
}) {
  return <div className="toolbar">
    <label>From day<select aria-label="From day" value={fromDay} onChange={event => {
      const from = Number(event.target.value);
      onChange({ fromDay: from, toDay: Math.max(from, toDay), channel });
    }}>{Array.from({ length: 30 }, (_, index) => <option key={index} value={index + 1}>{index + 1}</option>)}</select></label>
    <label>To day<select aria-label="To day" value={toDay} onChange={event => {
      const to = Number(event.target.value);
      onChange({ fromDay: Math.min(fromDay, to), toDay: to, channel });
    }}>{Array.from({ length: 30 }, (_, index) => <option key={index} value={index + 1}>{index + 1}</option>)}</select></label>
    <label>Sales channel<select aria-label="Sales channel" value={channel} onChange={event => onChange({ fromDay, toDay, channel: event.target.value })}>
      <option value="all">All channels</option>
      {['Store', 'Social', 'Marketplace'].map(value => <option key={value}>{value}</option>)}
    </select></label>
  </div>;
}
