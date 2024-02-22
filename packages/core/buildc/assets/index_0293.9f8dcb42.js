(function (modules) {
            for (var key in modules) {
              modules[key].__farm_resource_pot__ = 'index_0293.js';
                (globalThis || window || global)['8631a49814b6940e6ec3522bbe70b0e5'].__farm_module_system__.register(key, modules[key]);
            }
        })({"5e369e13": function(module, exports, farmRequire, farmDynamicRequire) {
/**
 * { data-analysis:  ['read', 'write'] }
 */ "use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "default", {
    enumerable: true,
    get: function() {
        return _default;
    }
});
const judge = (actions, perm)=>{
    if (!perm || !perm.length) {
        return false;
    }
    if (perm.join("") === "*") {
        return true;
    }
    return actions.every((action)=>perm.includes(action));
};
const auth = (params, userPermission)=>{
    const { resource, actions = [] } = params;
    if (resource instanceof RegExp) {
        const permKeys = Object.keys(userPermission);
        const matchPermissions = permKeys.filter((item)=>item.match(resource));
        return matchPermissions.every((key)=>{
            const perm = userPermission[key];
            return judge(actions, perm);
        });
    }
    const perm = userPermission[resource];
    return judge(actions, perm);
};
const _default = (params, userPermission)=>{
    const { requiredPermissions, oneOfPerm } = params;
    if (Array.isArray(requiredPermissions) && requiredPermissions.length) {
        let count = 0;
        for (const rp of requiredPermissions){
            if (auth(rp, userPermission)) {
                count++;
            }
        }
        return oneOfPerm ? count > 0 : count === requiredPermissions.length;
    }
    return true;
};

},});