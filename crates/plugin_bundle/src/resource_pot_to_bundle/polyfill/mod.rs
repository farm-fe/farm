use std::collections::HashSet;

use farmfe_core::{error::Result, farm_profile_scope, swc_ecma_ast::ModuleItem};

use super::common::parse_module_item;
pub mod cjs;

// TODO: global polyfill
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum Polyfill {
  ///
  /// ```ts
  /// // export.ts
  /// module.export.name = 'shulan';
  /// module.export.age = 18;
  /// ```
  /// =>
  /// ```ts
  /// __commonJs({
  ///   "export.ts": function(module, exports) {
  ///     module.export.name = 'shulan';
  ///     module.export.age = 18;
  ///   }
  /// })
  /// ```
  ///
  WrapCommonJs,
  ///
  /// ```ts
  /// export * from 'node:fs';
  /// const name = 'shulan', age = 18;
  /// export { name, age }
  /// // =>
  /// import * as node_fs from "node:fs";
  /// _mergeNamespaces({ name, age }, [node_fs])
  /// ```
  ///
  MergeNamespace,
  /// compatible require and esm
  Wildcard,
  /// use esm in cjs `export * from 'node:fs'`
  /// =>
  /// ```ts
  /// import node_fs from "node:fs";
  ///
  /// (function (module, exports) {
  ///   _export_star(node_fs, exports);
  /// })
  /// ```
  ExportStar,
  /// use esm in cjs `import fs from "node:fs"`
  /// =>
  /// ```ts
  /// import node_fs from "node:fs";
  /// const fs = _interop_require_default(node_fs);
  /// ```
  InteropRequireDefault,
  ///
  /// support use require in esm
  ///
  /// ```ts
  /// // esm pre
  /// import __farmNodeModule from 'module';
  /// global.nodeRequire = __farmNodeModule.createRequire(import.meta.url);
  /// ```
  ///
  NodeEsmGlobalRequireHelper,

  ///
  /// browser external load
  /// ```ts
  /// const events = require("events");
  /// // =>
  /// loadExternalRequire('events');
  /// ```
  ///
  BrowserExternalRequire,
}

impl Polyfill {
  fn to_str(&self) -> Vec<String> {
    match self {
      Polyfill::WrapCommonJs => vec![
        (r#"
function __commonJs(mod) {
  var module;
  return () => {
    if (module) {
      return module.exports;
    }
    module = {
      exports: {},
    };
    if(typeof mod === "function") {
      mod(module, module.exports);
    }else {
      mod[Object.keys(mod)[0]](module, module.exports);
    }
    return module.exports;
  };
}
      "#),
      ],
      Polyfill::MergeNamespace => vec![
        (r#"
function _mergeNamespaces(n, m) {
    m.forEach(function (e) {
        e && typeof e !== 'string' && !Array.isArray(e) && Object.keys(e).forEach(function (k) {
            if (k !== 'default' && !(k in n)) {
                var d = Object.getOwnPropertyDescriptor(e, k);
                Object.defineProperty(n, k, d.get ? d : {
                    enumerable: true,
                    get: function () { return e[k]; }
                });
            }
        });
    });
    return Object.freeze(n);
}
"#),
      ],
      Polyfill::Wildcard => vec![
        (r#"
function _getRequireWildcardCache(nodeInterop) {
    if (typeof WeakMap !== "function") return null;
    var cacheBabelInterop = new WeakMap();
    var cacheNodeInterop = new WeakMap();
    return (_getRequireWildcardCache = function(nodeInterop) {
        return nodeInterop ? cacheNodeInterop : cacheBabelInterop;
    })(nodeInterop);
}
"#),
        (r#"
function _interop_require_wildcard(obj, nodeInterop) {
    if (!nodeInterop && obj && obj.__esModule) return obj;
    if (obj === null || typeof obj !== "object" && typeof obj !== "function") return {
        default: obj
    };
    var cache = _getRequireWildcardCache(nodeInterop);
    if (cache && cache.has(obj)) return cache.get(obj);
    var newObj = {
        __proto__: null
    };
    var hasPropertyDescriptor = Object.defineProperty && Object.getOwnPropertyDescriptor;
    for(var key in obj){
        if (key !== "default" && Object.prototype.hasOwnProperty.call(obj, key)) {
            var desc = hasPropertyDescriptor ? Object.getOwnPropertyDescriptor(obj, key) : null;
            if (desc && (desc.get || desc.set)) Object.defineProperty(newObj, key, desc);
            else newObj[key] = obj[key];
        }
    }
    newObj.default = obj;
    if (cache) cache.set(obj, newObj);
    return newObj;
}
        "#),
      ],
      Polyfill::ExportStar => vec![
        (r#"
function _export_star(from, to) {
    Object.keys(from).forEach(function(k) {
        if (k !== "default" && !Object.prototype.hasOwnProperty.call(to, k)) {
            Object.defineProperty(to, k, {
                enumerable: true,
                get: function() {
                    return from[k];
                }
            });
        }
    });
    return from;
}
      "#),
      ],
      Polyfill::InteropRequireDefault => vec![
        (r#"
function _interop_require_default(obj) {
    return obj && obj.__esModule ? obj : {
        default: obj
    };
}
"#),
      ],
      Polyfill::NodeEsmGlobalRequireHelper => vec![
        r#"
import __farmNodeModule from 'module';
global.nodeRequire = __farmNodeModule.createRequire(import.meta.url);
"#,
      ],
      Polyfill::BrowserExternalRequire => vec![
        r#"
function loadExternalRequire(name) {
  var _g = (window || {});
  var m = _g[name];
  var assign = function() {
    var args = Array.prototype.slice.call(arguments);
    var target = args.shift();
    var hasOwnProperty = Object.hasOwnProperty;
    for(var i = 0; i < args.length; i ++) {
      for(var key in args[i]) {
        if(!hasOwnProperty.call(target, key)) {
          target[key] = args[i][key];
        }
      }
    }
    return target;
  }
  return m ? m.default && !m.__esModule ? assign({}, m, {__esModule: true}) : (assign({}, m)) : m;
};
        "#,
      ],
    }
    .into_iter()
    .map(|item| item.trim().into())
    .collect()
  }

  fn to_ast(&self) -> Result<Vec<ModuleItem>> {
    let mut result = vec![];

    for item in self.to_str() {
      result.push(parse_module_item(&item)?);
    }

    Ok(result)
  }

  fn dependents(&self) -> Vec<Polyfill> {
    vec![]
  }

  fn name(&self) -> Vec<String> {
    (match self {
      Polyfill::WrapCommonJs => vec!["__commonJs"],
      Polyfill::MergeNamespace => vec!["_mergeNamespaces"],
      Polyfill::Wildcard => vec!["_getRequireWildcardCache", "_interop_require_wildcard"],
      Polyfill::ExportStar => vec!["_export_star"],
      Polyfill::InteropRequireDefault => vec!["_interop_require_default"],
      Polyfill::NodeEsmGlobalRequireHelper => vec!["__farmNodeModule"],
      Polyfill::BrowserExternalRequire => vec!["loadExternalRequire"],
    })
    .into_iter()
    .map(|item| item.into())
    .collect()
  }
}

#[derive(Debug, Default, Clone)]
pub struct SimplePolyfill {
  polyfills: HashSet<Polyfill>,
}

impl SimplePolyfill {
  pub fn new(polyfill: Vec<Polyfill>) -> Self {
    let mut polyfills = HashSet::new();

    polyfills.extend(polyfill);

    Self { polyfills }
  }

  pub fn add(&mut self, polyfill: Polyfill) {
    if self.polyfills.contains(&polyfill) {
      return;
    }

    let dependents = polyfill.dependents();

    self.polyfills.insert(polyfill);

    dependents.into_iter().for_each(|dep| self.add(dep));
  }

  pub fn contain(&self, polyfill: &Polyfill) -> bool {
    self.polyfills.contains(polyfill)
  }

  pub fn to_ast(&self) -> Result<Vec<ModuleItem>> {
    farm_profile_scope!("polyfill to ast");
    let mut asts = vec![];

    let mut polyfills = self.polyfills.iter().collect::<Vec<_>>();

    polyfills.sort();

    for polyfill in &polyfills {
      asts.extend(polyfill.to_ast()?)
    }

    Ok(asts)
  }

  pub fn to_str(&self) -> Vec<String> {
    farm_profile_scope!("polyfill to str");
    let mut str_list = vec![];

    let mut polyfills = self.polyfills.iter().collect::<Vec<_>>();

    polyfills.sort();

    for polyfill in &polyfills {
      str_list.extend(polyfill.to_str())
    }

    str_list
  }

  pub fn is_empty(&self) -> bool {
    self.polyfills.is_empty()
  }

  pub fn reserved_word() -> Vec<String> {
    vec![
      Polyfill::WrapCommonJs,
      Polyfill::MergeNamespace,
      Polyfill::Wildcard,
      Polyfill::ExportStar,
      Polyfill::InteropRequireDefault,
      Polyfill::NodeEsmGlobalRequireHelper,
    ]
    .into_iter()
    .flat_map(|polyfill| polyfill.name())
    .collect()
  }

  pub fn extends(&mut self, polyfill: &SimplePolyfill) {
    self.polyfills.extend(polyfill.polyfills.clone());
  }
}
