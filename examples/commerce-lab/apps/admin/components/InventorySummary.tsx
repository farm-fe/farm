import React from 'react';
import { Stat, money } from '../../../packages/ui';

export function InventorySummary({ summary }: {
  summary: { totalUnits: number; stockValue: number; lowStock: number };
}) {
  return <div className="stats"><Stat label="Available units" value={summary.totalUnits} detail="All merchant products" />
    <Stat label="Stock value" value={money(summary.stockValue)} detail="At current sale prices" />
    <Stat label="Low stock products" value={summary.lowStock} detail="Fewer than 25 units" /></div>;
}
