import { dep, formatTargetDir } from './modules/dep';
import { DepNoInlineSources } from './modules/dep-no-inline-sources';

export function init(): void {
  type str = string;
  const targetDir: str = formatTargetDir('targetDir');
  console.log(targetDir);
  const depNoInlineSources = new DepNoInlineSources();
  console.log(depNoInlineSources.getA());
  depNoInlineSources.setA(dep);
  console.log(depNoInlineSources.getA());
}