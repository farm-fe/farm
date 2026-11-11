import { SpecRunner } from "../../e2e/runner.mjs";
import fs from "node:fs/promises";
const results = await new SpecRunner().run(
  new URL("./e2e.spec.mjs", import.meta.url).href,
  "module-federation-native-probe",
);
await fs.writeFile(
  new URL("./e2e-results/summary.json", import.meta.url),
  JSON.stringify(
    results.map((r) => ({ ...r, error: r.error?.stack })),
    null,
    2,
  ),
);
if (results.some((r) => !r.passed && !r.skipped)) process.exitCode = 1;
