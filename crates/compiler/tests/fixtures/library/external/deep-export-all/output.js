//index.js:
 import * as color_ambiguous_export_all_farm_internal_ from "/external/color";
export * from "/external/color";
import * as utils_ambiguous_export_all_farm_internal_ from "/external/utils";
export * from "/external/utils";
var color_bold = color_ambiguous_export_all_farm_internal_.bold || utils_ambiguous_export_all_farm_internal_.bold;
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
; // module_id: index.ts
console.log(color_bold('hello'));
export { Compiler as Compiler, Server as Server, green as green };
