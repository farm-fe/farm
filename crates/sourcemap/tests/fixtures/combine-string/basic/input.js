import { dep, formatTargetDir } from './modules/dep';
import { DepNoInlineSources } from './modules/dep-no-inline-sources';
export function init() {
    const targetDir = formatTargetDir('targetDir');
    console.log(targetDir);
    const depNoInlineSources = new DepNoInlineSources();
    console.log(depNoInlineSources.getA());
    depNoInlineSources.setA(dep);
    console.log(depNoInlineSources.getA());
}
//# sourceMappingURL=input.js.map