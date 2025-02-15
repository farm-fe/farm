import { moduleSystem } from "./module-system";
import { initModuleSystem as initModuleSystem1 } from "./modules/plugin";
import { initModuleSystem as initModuleSystem2 } from "./modules/module-helper";

initModuleSystem1(moduleSystem);
initModuleSystem2(moduleSystem);