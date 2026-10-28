# Farm docs
Docs site of Farm.

Install dependencies:
```bash
pnpm i
```

Development locally:
```bash
npm start
```

The default commands use the independent Farm Docs configuration,
`farm.config.ts`; it reads
`farm-docs.site.json` and `farm.tailwind.json`.

After building the matching workspace core, native plugins and public packages,
run these commands from `website`:

```bash
pnpm start
pnpm build
pnpm serve
```

`dev` and `dev:zh` start the same multilingual development server. The explicit
`start:farm`, `dev:farm`, `build:farm` and `preview:farm` aliases remain available.
Development uses port 1911; preview serves
the existing `build-farm` output on port 9000 and does not rebuild it. Both use
the independent Farm configuration. All configured languages share one server;
visit `/zh/` for Chinese instead of passing Docusaurus's `--locale` option.

The old Docusaurus build/configuration entry points have been removed.
The current running instructions are in
[Farm Docs usage](docs/farm-docs/usage.md). Historical migration and deployment
records are in [`docs/farm-docs`](../docs/farm-docs/); those records do not establish
the validation status of the current checkout. Local build and preview commands
do not publish or deploy the website.
