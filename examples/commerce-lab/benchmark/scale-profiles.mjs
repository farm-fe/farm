export const scaleProfiles = Object.freeze({
  'business-smoke': { categories: 4, rich: true, business: true },
  'business-256': { categories: 256, rich: true, business: true },
  'business-512': { categories: 512, rich: true, business: true },
  'business-1024': { categories: 1024, rich: true, business: true },
  'business-2048': { categories: 2048, rich: true, business: true },
  'business-4096': { categories: 4096, rich: true, business: true },
  baseline: { categories: 4, rich: false },
  medium: { categories: 32, rich: false },
  large: { categories: 128, rich: false },
  'catalog-256': { categories: 256, rich: true },
  'catalog-512': { categories: 512, rich: true },
  'catalog-1024': { categories: 1024, rich: true },
  'catalog-2048': { categories: 2048, rich: true },
  'catalog-4096': { categories: 4096, rich: true },
});
export const extensionScales = Object.freeze(Object.fromEntries(Object.entries(scaleProfiles).map(([name, profile]) => [name, profile.categories])));
export function scaleProfile(name) {
  if (!Object.hasOwn(scaleProfiles, name)) throw new Error('Invalid extension scale');
  return scaleProfiles[name];
}
