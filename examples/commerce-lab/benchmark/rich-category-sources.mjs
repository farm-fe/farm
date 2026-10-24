/** Real interacting catalog editor modules; all imports are local or existing React. */
export function richCategorySources({ id, title, labels }) {
  return {
    [`${id}/Form.tsx`]: `import React, { useState } from 'react';
import Guide from './Guide.mdx';
import Icon from './icon.svg';
import './style.scss';
import AttributeFields from './AttributeFields';
import ValidationSummary from './ValidationSummary';
import VariantTable from './VariantTable';
import PricingPreview from './PricingPreview';
import { initialVariants, validateAttributes } from './model';
import type { Variant } from './model';
export default function Form() {
  const [values, setValues] = useState<Record<string, string>>({});
  const [variants, setVariants] = useState<Variant[]>(initialVariants);
  const [errors, setErrors] = useState<string[]>([]);
  const [saved, setSaved] = useState('');
  return <section className="extension-${id}">
    <h3><Icon width={24} height={24} /> ${title} attributes</h3>
    <form onSubmit={event => { event.preventDefault(); const next = validateAttributes(values, variants); setErrors(next); if (!next.length) setSaved('${title}: ' + Object.values(values).join(' / ')); }}>
      <AttributeFields values={values} errors={errors} onChange={(field, value) => { setValues(current => ({ ...current, [field]: value })); setSaved(''); }} />
      <VariantTable variants={variants} onChange={(sku, patch) => { setVariants(current => current.map(variant => variant.sku === sku ? { ...variant, ...patch } : variant)); setSaved(''); }} />
      <ValidationSummary errors={errors} />
      <button type="submit">Save attributes</button>
    </form>
    {saved && <p role="status">{saved}</p>}
    <PricingPreview variants={variants} />
    <Guide />
  </section>;
}
`,
    [`${id}/AttributeFields.tsx`]: `import React from 'react';
import { fieldLabels } from './model';
type Props = { values: Record<string, string>; errors: string[]; onChange: (field: string, value: string) => void };
export default function AttributeFields({ values, errors, onChange }: Props) {
  return <fieldset><legend>Product attributes</legend>{fieldLabels.map((label, index) => <label key={label}>{label}<input required={index === 0} maxLength={200} aria-invalid={errors.some(error => error.startsWith(label))} value={values[label] ?? ''} onChange={event => onChange(label, event.target.value)} /></label>)}</fieldset>;
}
`,
    [`${id}/ValidationSummary.tsx`]: `import React from 'react';
export default function ValidationSummary({ errors }: { errors: string[] }) {
  return errors.length ? <div role="alert"><strong>Review product details</strong><ul>{errors.map(error => <li key={error}>{error}</li>)}</ul></div> : null;
}
`,
    [`${id}/VariantTable.tsx`]: `import React from 'react';
import AvailabilityBadge from './AvailabilityBadge';
import type { Variant } from './model';
type Props = { variants: Variant[]; onChange: (sku: string, patch: Partial<Variant>) => void };
export default function VariantTable({ variants, onChange }: Props) {
  return <table><caption>Variant inventory</caption><thead><tr><th>Size</th><th>Price</th><th>Stock</th><th>Availability</th></tr></thead><tbody>{variants.map(variant => <tr key={variant.sku}><th>{variant.size}</th><td><input aria-label={'Price ' + variant.size} type="number" min="0.01" step="0.01" value={variant.price / 100} onChange={event => onChange(variant.sku, { price: Math.round(Number(event.target.value) * 100) })} /></td><td><input aria-label={'Stock ' + variant.size} type="number" min="0" step="1" value={variant.stock} onChange={event => onChange(variant.sku, { stock: Number(event.target.value) })} /></td><td><AvailabilityBadge stock={variant.stock} /></td></tr>)}</tbody></table>;
}
`,
    [`${id}/AvailabilityBadge.jsx`]: `import React from 'react';
export default function AvailabilityBadge({ stock }) {
  const status = stock === 0 ? 'unavailable' : stock < 5 ? 'low' : 'available';
  return <span className={'availability availability-' + status} data-stock-status={status}>{stock === 0 ? 'Out of stock' : stock < 5 ? 'Low stock' : 'In stock'}</span>;
}
`,
    [`${id}/PricingPreview.tsx`]: `import React, { useMemo, useState } from 'react';
import { quoteVariant } from './model';
import type { Variant } from './model';
export default function PricingPreview({ variants }: { variants: Variant[] }) {
  const [sku, setSku] = useState(variants[0].sku);
  const [quantity, setQuantity] = useState(1);
  const [discount, setDiscount] = useState(0);
  const selected = variants.find(variant => variant.sku === sku)!;
  const total = useMemo(() => quoteVariant(Math.max(1, selected.price), quantity, discount), [selected.price, quantity, discount]);
  const money = new Intl.NumberFormat('en-US', { style: 'currency', currency: 'USD' });
  return <aside className="pricing-preview" data-discounted={discount > 0}><h4>Promotion preview</h4><label>Preview size<select value={sku} onChange={event => setSku(event.target.value)}>{variants.map(variant => <option key={variant.sku} value={variant.sku}>{variant.size}</option>)}</select></label><label>Preview quantity<input type="number" min="1" value={quantity} onChange={event => setQuantity(Math.max(1, Math.floor(Number(event.target.value)) || 1))} /></label><label>Preview discount<input type="number" min="0" max="50" value={discount} onChange={event => setDiscount(Math.max(0, Math.min(50, Number(event.target.value) || 0)))} /></label><output aria-label="Promotion total">{money.format(total / 100)}</output>{quantity > selected.stock && <p>Requested quantity exceeds available stock.</p>}</aside>;
}
`,
    [`${id}/model.ts`]: `export type Variant = { sku: string; size: string; price: number; stock: number };
export const fieldLabels = ${JSON.stringify(labels)} as const;
export const initialVariants: Variant[] = [
  { sku: '${id}-small', size: 'S', price: 1999, stock: 12 },
  { sku: '${id}-medium', size: 'M', price: 2499, stock: 4 },
  { sku: '${id}-large', size: 'L', price: 2999, stock: 0 },
];
export function validateAttributes(values: Record<string, string>, variants: Variant[] = []): string[] {
  const errors: string[] = [];
  if (!values[fieldLabels[0]]?.trim()) errors.push(fieldLabels[0] + ' is required');
  for (const label of fieldLabels) if ((values[label]?.length ?? 0) > 200) errors.push(label + ' exceeds 200 characters');
  for (const variant of variants) {
    if (!Number.isSafeInteger(variant.price) || variant.price <= 0) errors.push(variant.sku + ': invalid price');
    if (!Number.isSafeInteger(variant.stock) || variant.stock < 0) errors.push(variant.sku + ': invalid stock');
  }
  return errors;
}
export function quoteVariant(price: number, quantity: number, discount: number): number {
  if (!Number.isSafeInteger(price) || price <= 0 || !Number.isSafeInteger(quantity) || quantity < 1 || !Number.isSafeInteger(price * quantity) || !Number.isFinite(discount) || discount < 0 || discount > 50) throw new RangeError('Invalid promotion quote');
  return Math.round(price * quantity * (100 - discount) / 100);
}
`,
  };
}
