//index.js:
 function defineExportStar(to, from) {
    Object.keys(from).forEach(function(k) {
        if (k !== "default" && !Object.prototype.hasOwnProperty.call(to, k)) {
            Object.defineProperty(to, k, {
                value: from[k],
                enumerable: true,
                configurable: true
            });
        }
    });
    return from;
}
import * as utils_external_all_farm_internal_ from "/external/utils";
import * as color_external_all_farm_internal_ from "/external/color";
; // module_id: server.ts
class Server {
    constructor(){
        green('server constructor');
    }
}
; // module_id: compiler.ts
class Compiler {
    constructor(){
        console.log('Compiler constructor');
    }
}
; // module_id: color.ts
function green(str) {
    console.log('green', str);
}
var color_ts_namespace_farm_internal_ = {
    green: green,
    __esModule: true
};
defineExportStar(color_ts_namespace_farm_internal_, color_external_all_farm_internal_);
defineExportStar(color_ts_namespace_farm_internal_, utils_external_all_farm_internal_);
; // module_id: index.ts
var bold = color_ts_namespace_farm_internal_.bold;
console.log(bold('hello'));
export { Compiler as Compiler, Server as Server, green as green };
export * from "/external/color";
