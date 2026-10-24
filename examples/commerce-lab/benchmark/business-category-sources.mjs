import { richCategorySources } from './rich-category-sources.mjs';

/** A category editor has operational rules, price bands and fulfillment routes. */
export function businessCategorySources(options) {
  const { id, title } = options;
  const files = richCategorySources(options);
  files[`${id}/rules.yaml`] = 'maxDiscount: 20\nreserveStock: 2\nchannels:\n  - retail\n  - wholesale\n';
  files[`${id}/tiers.csv`] = 'minQuantity,discount\n1,0\n5,10\n10,15\n';
  files[`${id}/shipping.tsv`] = 'destination\tcents\tfreeAbove\ndomestic\t500\t10000\nexport\t1500\t20000\n';
  files[`${id}/business.ts`] = `import rules from './rules.yaml';
import tiers from './tiers.csv';
import shipping from './shipping.tsv';
import { businessQuote } from '../../../../benchmark/business-quote.mjs';
import type { Variant } from './model';
export { rules };
export function quote(variant: Variant, quantity: number, discount: number, destination: string) {
  return businessQuote({ ...variant, quantity, discount, destination, rules, tiers, shipping });
}
export function validatePublication(variants: Variant[]) {
  return variants.some(variant => variant.stock > rules.reserveStock) ? [] : ['No sellable inventory after category reservation'];
}
`;
  files[`${id}/Form.tsx`] = files[`${id}/Form.tsx`]
    .replace("import Guide", "import { validatePublication } from './business';\nimport Guide")
    .replace('const next = validateAttributes(values, variants);', 'const next = [...validateAttributes(values, variants), ...validatePublication(variants)];')
    .replace(`className="extension-${id}"`, `className="extension-${id} grid grid-cols-1 gap-4 md:grid-cols-2"`)
    .replace('    </form>\n    {saved && <p role="status">{saved}</p>}', '      {saved && <p role="status">{saved}</p>}\n    </form>')
    .replace(/^(\s*)<VariantTable (.*) \/>$/m, '$1<div className="variant-table-scroll" role="region" aria-label="Variant inventory scroll area" tabIndex={0}><VariantTable $2 /></div>')
    .replace('<Guide />', `<section className="category-guide" aria-label="${title} guide"><Guide /></section>`);
  files[`${id}/PricingPreview.tsx`] = `import React, { useState } from 'react';
import { quote, rules } from './business';
import type { Variant } from './model';
export default function PricingPreview({ variants }: { variants: Variant[] }) {
  const [sku, setSku] = useState(variants[0].sku);
  const [quantity, setQuantity] = useState(1);
  const [discount, setDiscount] = useState(0);
  const [destination, setDestination] = useState('domestic');
  const selected = variants.find(variant => variant.sku === sku)!;
  let result: ReturnType<typeof quote> | undefined;
  let error = '';
  try { result = quote(selected, quantity, discount, destination); } catch (failure) { error = failure instanceof Error ? failure.message : 'Invalid quote'; }
  const money = (cents: number) => new Intl.NumberFormat('en-US', { style: 'currency', currency: 'USD' }).format(cents / 100);
  return <aside className="pricing-preview flex flex-col gap-4 rounded-lg p-4" data-discounted={discount > 0}>
    <h4>Promotion preview</h4><p>Category discount limit: {rules.maxDiscount}%; reserved stock: {rules.reserveStock}</p>
    <label>Preview size<select aria-label="Preview size" value={sku} onChange={event => setSku(event.target.value)}>{variants.map(variant => <option key={variant.sku} value={variant.sku}>{variant.size}</option>)}</select></label>
    <label>Preview quantity<input type="number" min="1" value={quantity} onChange={event => setQuantity(Number(event.target.value))} /></label>
    <label>Preview discount<input type="number" min="0" max={rules.maxDiscount} value={discount} onChange={event => setDiscount(Number(event.target.value))} /></label>
    <label>Delivery destination<select aria-label="Delivery destination" value={destination} onChange={event => setDestination(event.target.value)}><option value="domestic">Domestic</option><option value="export">Export</option></select></label>
    {error ? <p role="alert">{error}</p> : result && <><output aria-label="Applied discount">{result.discount}%</output><output aria-label="Shipping cost">{money(result.shipping)}</output><output aria-label="Promotion total">{money(result.total)}</output></>}
  </aside>;
}
`;
  files[`${id}/style.scss`] = `$accent: #245c40;
.extension-${id} {
  border-left: 4px solid $accent;
  padding: 16px;
  align-items: start;
  > h3 { grid-column: 1 / -1; display: flex; align-items: center; gap: 8px; margin: 0; color: $accent; }
  > form, > .pricing-preview { min-width: 0; }
  .variant-table-scroll { overflow-x: auto; min-width: 0; padding-bottom: 8px; }
  .category-guide { grid-column: 1 / -1; min-width: 0; h1, h2 { margin: 16px 0 8px; } p { margin: 8px 0 16px; } }
  label { display: block; margin: 12px 0; }
  .pricing-preview { background: #f0fdf4; &[data-discounted="true"] { outline: 2px solid $accent; } }
  .availability { padding: 4px 8px; border-radius: 4px; &.availability-low { color: #92400e; background: #fef3c7; } &.availability-unavailable { color: #991b1b; } }
}
`;
  files[`${id}/Guide.mdx`] = `# ${title} attribute guide\n\nReview category attributes before publishing. Two units of each SKU are reserved for support replacements.\n\n## Wholesale and fulfillment\n\nFive units unlock a 10% tier discount; ten unlock 15%. Additional promotions share the category's 20% cap. Domestic shipping costs $5, waived above $100 after discounts; export shipping costs $15, waived above $200.\n\n<table aria-label="Category review actions"><thead><tr><th scope="col">Review</th><th scope="col">Action</th></tr></thead><tbody><tr><td>Inventory</td><td>Keep sellable stock above the reservation</td></tr><tr><td>Promotions</td><td>Check tier and campaign discounts together</td></tr></tbody></table>\n`;
  return files;
}
