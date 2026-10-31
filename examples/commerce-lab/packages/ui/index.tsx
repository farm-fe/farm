import React from 'react';
export const money = (cents: number) =>
  new Intl.NumberFormat('en-US', { style: 'currency', currency: 'USD' }).format(
    cents / 100,
  );
export function PageTitle({
  eyebrow,
  title,
  children,
}: {
  eyebrow: string;
  title: string;
  children?: React.ReactNode;
}) {
  return (
    <header className="page-title">
      <span className="eyebrow">{eyebrow}</span>
      <h1>{title}</h1>
      <p>{children}</p>
    </header>
  );
}
export function Stat({
  label,
  value,
  detail,
}: {
  label: string;
  value: string | number;
  detail: string;
}) {
  return (
    <article className="stat">
      <span>{label}</span>
      <strong>{value}</strong>
      <small>{detail}</small>
    </article>
  );
}
export function Badge({ children }: { children: React.ReactNode }) {
  return <span className={`badge badge-${children}`}>{children}</span>;
}
export function Empty({ children }: { children: React.ReactNode }) {
  return <div className="empty">{children}</div>;
}
export function Panel({
  title,
  children,
}: {
  title: string;
  children: React.ReactNode;
}) {
  return (
    <section className="panel">
      <h2>{title}</h2>
      {children}
    </section>
  );
}
export function MetricBars({ values, firstDay = 1 }: { values: number[]; firstDay?: number }) {
  const max = Math.max(1, ...values);
  return (
    <div className="bars" role="img" aria-label="Daily revenue chart">
      {values.map((value, i) => (
        <div
          key={i}
          title={`Day ${i + firstDay}: ${money(value)}`}
          style={{ height: `${Math.max(2, (value / max) * 100)}%` }}
        />
      ))}
    </div>
  );
}
