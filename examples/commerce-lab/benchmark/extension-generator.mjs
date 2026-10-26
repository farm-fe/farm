import fs from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { extensionScales, scaleProfile } from './scale-profiles.mjs';
import { richCategorySources } from './rich-category-sources.mjs';
import { businessCategorySources } from './business-category-sources.mjs';

export { extensionScales };
export function extensionSources(count, { rich = false, business = false } = {}) {
  if (!Number.isInteger(count) || count < 1 || count > 4096) throw new Error('Invalid category count');
  const fields = [['Dimensions', 'Finish'], ['Material', 'Care instructions'], ['Size range', 'Colorway'], ['Origin', 'Lead time']];
  const files = {};
  const imports = [];
  const entries = [];
  for (let index = 0; index < count; index++) {
    const number = String(index + 1).padStart(3, '0');
    const id = `category-${number}`;
    const title = `Category ${number}`;
    const labels = fields[index % fields.length];
    imports.push(`import Form${index} from './${id}/Form';`);
    entries.push(`{ id: '${id}', title: '${title}', Form: Form${index} }`);
    files[`${id}/Form.tsx`] = `// Deterministically generated extension workload; separate from manual business baseline.
import React, { useState } from 'react';
import Guide from './Guide.mdx';
import Icon from './icon.svg';
import './style.scss';
export default function Form() {
  const [values, setValues] = useState<Record<string, string>>({});
  const [saved, setSaved] = useState('');
  return <section className="extension-${id}">
    <h3><Icon width={24} height={24} /> ${title} attributes</h3>
    <form onSubmit={event => { event.preventDefault(); setSaved('${title}: ' + Object.values(values).join(' / ')); }}>
      ${labels.map((label, field) => `<label>${label}<input required={${field === 0}} value={values['${label}'] ?? ''} onChange={event => setValues({ ...values, '${label}': event.target.value })} /></label>`).join('\n      ')}
      <button type="submit">Save attributes</button>
    </form>
    {saved && <p role="status">{saved}</p>}
    <Guide />
  </section>;
}
`;
    files[`${id}/Guide.mdx`] = `# ${title} attribute guide\n\nRecord ${labels[0].toLowerCase()} before publishing this collection. ${labels[1]} helps the operations team handle orders consistently.\n\n| Attribute | Review |\n| --- | --- |\n| ${labels[0]} | Required for this category |\n| ${labels[1]} | Optional fulfillment detail |\n`;
    files[`${id}/style.scss`] = `/* Generated category styling. */\n$accent: hsl(${(index * 37 + 42) % 360}, 35%, 35%);\n.extension-${id} { border-left: 4px solid $accent; padding: 16px; h3 { color: $accent; } label { display: block; margin: 12px 0; } }\n`;
    files[`${id}/icon.svg`] = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path fill="currentColor" d="M3 3h18v18H3zM${5 + index % 5} 5v14h2V5z"/></svg>\n`;
    if (business) Object.assign(files, businessCategorySources({ id, title, labels }));
    else if (rich) Object.assign(files, richCategorySources({ id, title, labels }));
  }
  files['index.ts'] = `${imports.join('\n')}\nexport const extensionCatalog = [${entries.join(',\n')}];\n`;
  return files;
}

export async function generateScale(scale, { preserveExisting = false } = {}) {
  if (!Object.hasOwn(extensionScales, scale)) throw new Error('Invalid extension scale');
  const root = fileURLToPath(new URL(`../packages/extensions/${scale}/`, import.meta.url));
  const sources = extensionSources(extensionScales[scale], scaleProfile(scale));
  for (const [name, content] of Object.entries(sources)) {
    const target = path.resolve(root, name);
    if (!target.startsWith(root)) throw new Error('Generated path escapes extension root');
    try {
      const existing = await fs.readFile(target, 'utf8');
      if (!preserveExisting && existing !== content) throw new Error(`Refusing to overwrite modified extension: ${target}`);
      continue;
    } catch (error) { if (error.code !== 'ENOENT') throw error; }
    await fs.mkdir(path.dirname(target), { recursive: true });
    try {
      await fs.writeFile(target, content, { flag: 'wx' });
    } catch (error) {
      if (error.code !== 'EEXIST') throw error;
      if (!preserveExisting && await fs.readFile(target, 'utf8') !== content)
        throw new Error(`Refusing to overwrite modified extension: ${target}`);
    }
  }
  return { scale, categories: extensionScales[scale], files: Object.keys(sources).length };
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  console.log(JSON.stringify(await generateScale(process.argv[2] ?? 'baseline')));
}
