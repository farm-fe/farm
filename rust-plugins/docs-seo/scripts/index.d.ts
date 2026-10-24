export type SeoOptions = { planPath: string; development?: false } | { development: true; planPath?: never };
export default function seo(options: SeoOptions): [string, SeoOptions];
