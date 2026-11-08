use std::{path::PathBuf, process::Command};

use farmfe_compiler::Compiler;
use farmfe_core::{
  HashMap,
  config::{ModuleFormat, ModuleFormatConfig, TargetEnv, bool_or_obj::BoolOrObj},
};

mod common;

#[test]
fn string_module_names_preserve_live_bindings() {
  let crate_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  let root = crate_path.join("tests/fixtures/string_module_names");
  for commonjs in [false, true] {
    for tree_shaking in [false, true] {
      for concatenate in [false, true] {
        for minify in [false, true] {
          let output = std::env::temp_dir().join(format!(
            "farm-string-names-{commonjs}-{tree_shaking}-{concatenate}-{minify}-{}",
            std::process::id()
          ));
          let mut config = common::create_config(root.clone(), crate_path.clone());
          config.input = HashMap::from_iter([(
            "main".into(),
            root.join("main.js").to_string_lossy().into_owned(),
          )]);
          config.runtime = common::generate_runtime(crate_path.clone(), true);
          config.concatenate_modules = concatenate;
          config.tree_shaking = Box::new(BoolOrObj::Bool(tree_shaking));
          config.output.target_env = TargetEnv::Node;
          config.output.format = ModuleFormatConfig::Single(if commonjs {
            ModuleFormat::CommonJs
          } else {
            ModuleFormat::EsModule
          });
          let extension = if commonjs { "cjs" } else { "mjs" };
          config.output.filename = format!("[resourceName].{extension}");
          config.output.entry_filename = format!("[entryName].{extension}");
          config.output.path = output.to_string_lossy().into_owned();
          config.minify = Box::new(BoolOrObj::Bool(minify));
          let compiler = Compiler::new(config, vec![]).unwrap();
          compiler.compile().unwrap();
          compiler.write_resources_to_disk().unwrap();
          let script = "import assert from 'node:assert/strict'; import { pathToFileURL } from 'node:url'; import { createRequire } from 'node:module'; const m = process.argv[1].endsWith('.cjs') ? createRequire(import.meta.url)(process.argv[1]) : await import(pathToFileURL(process.argv[1])); assert.deepEqual(m.values(), [1,1,1,1,1,1]); assert.equal(m['public-name'], 1); assert.deepEqual(await m.boundaryValues(), [1,1,1,1,1,1]); m.increment(); assert.deepEqual(m.values(), [2,2,2,2,2,2]); assert.deepEqual(await m.boundaryValues(), [2,2,2,2,2,2]); console.log('string-names-loaded');";
          let result = Command::new("node")
            .args(["--input-type=module", "--eval", script])
            .arg(output.join(format!("main.{extension}")))
            .output()
            .expect("node must be installed for runtime regression tests");
          assert!(
            result.status.success(),
            "commonjs={commonjs}, tree_shaking={tree_shaking}, concatenate={concatenate}, minify={minify}: {}",
            String::from_utf8_lossy(&result.stderr)
          );
          assert!(String::from_utf8_lossy(&result.stdout).contains("string-names-loaded"));
        }
      }
    }
  }
}

#[test]
fn library_existing_external_import_and_string_star_reexports() {
  use farmfe_core::config::config_regex::ConfigRegex;
  let crate_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  let root = crate_path.join("tests/fixtures/string_module_names/library");
  let output = std::env::temp_dir().join(format!("farm-string-library-{}", std::process::id()));
  let mut config = common::create_config(root.clone(), crate_path.clone());
  config.input = HashMap::from_iter([(
    "main".into(),
    root.join("main.js").to_string_lossy().into_owned(),
  )]);
  config.runtime = common::generate_runtime(crate_path, true);
  config.concatenate_modules = true;
  config.tree_shaking = Box::new(BoolOrObj::Bool(true));
  config.minify = Box::new(BoolOrObj::Bool(false));
  config.external = vec![ConfigRegex::new(r"external\.mjs$")];
  config.output.target_env = TargetEnv::Library;
  config.output.format = ModuleFormatConfig::Single(ModuleFormat::EsModule);
  config.output.filename = "[resourceName].mjs".into();
  config.output.entry_filename = "[entryName].mjs".into();
  config.output.path = output.to_string_lossy().into_owned();
  let compiler = Compiler::new(config, vec![]).unwrap();
  compiler.compile().unwrap();
  compiler.write_resources_to_disk().unwrap();
  std::fs::copy(root.join("external.mjs"), output.join("external.mjs")).unwrap();
  let script = "import assert from 'node:assert/strict'; import { pathToFileURL } from 'node:url'; const m = await import(pathToFileURL(process.argv[1])); assert.equal(m.read(), 1); for (const key of ['s-k','1-name','123','']) assert.equal(m[key], 1); m.increment(); assert.equal(m.read(), 2); for (const key of ['s-k','1-name','123','']) assert.equal(m[key], 2);";
  let result = Command::new("node")
    .args(["--input-type=module", "--eval", script])
    .arg(output.join("main.mjs"))
    .output()
    .unwrap();
  assert!(
    result.status.success(),
    "{}",
    String::from_utf8_lossy(&result.stderr)
  );
}

#[test]
fn materialized_internal_namespace_preserves_live_bindings() {
  let crate_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  let root = crate_path.join("tests/fixtures/string_module_names/internal-namespace");
  for commonjs in [false, true] {
    for tree_shaking in [false, true] {
      for concatenate in [false, true] {
        for minify in [false, true] {
          let output = std::env::temp_dir().join(format!(
            "farm-internal-namespace-{commonjs}-{tree_shaking}-{concatenate}-{minify}-{}",
            std::process::id()
          ));
          let mut config = common::create_config(root.clone(), crate_path.clone());
          config.input = HashMap::from_iter([(
            "main".into(),
            root.join("main.js").to_string_lossy().into_owned(),
          )]);
          config.runtime = common::generate_runtime(crate_path.clone(), true);
          config.concatenate_modules = concatenate;
          config.tree_shaking = Box::new(BoolOrObj::Bool(tree_shaking));
          config.output.target_env = TargetEnv::Node;
          config.output.format = ModuleFormatConfig::Single(if commonjs {
            ModuleFormat::CommonJs
          } else {
            ModuleFormat::EsModule
          });
          let extension = if commonjs { "cjs" } else { "mjs" };
          config.output.filename = format!("[resourceName].{extension}");
          config.output.entry_filename = format!("[entryName].{extension}");
          config.output.path = output.to_string_lossy().into_owned();
          config.minify = Box::new(BoolOrObj::Bool(minify));
          let compiler = Compiler::new(config, vec![]).unwrap();
          compiler.compile().unwrap();
          compiler.write_resources_to_disk().unwrap();
          let script = "import assert from 'node:assert/strict'; import {pathToFileURL} from 'node:url'; import {createRequire} from 'node:module'; const m = process.argv[1].endsWith('.cjs') ? createRequire(import.meta.url)(process.argv[1]) : await import(pathToFileURL(process.argv[1])); assert.deepEqual(m.verify(), [1,2,2,true]); console.log('string-names-loaded');";
          let result = Command::new("node")
            .args(["--input-type=module", "--eval", script])
            .arg(output.join(format!("main.{extension}")))
            .output()
            .expect("node must be installed for runtime regression tests");
          assert!(
            result.status.success(),
            "commonjs={commonjs}, tree_shaking={tree_shaking}, concatenate={concatenate}, minify={minify}: {}",
            String::from_utf8_lossy(&result.stderr)
          );
          assert!(String::from_utf8_lossy(&result.stdout).contains("string-names-loaded"));
        }
      }
    }
  }
}

#[test]
fn entry_facade_cjs_live_bindings() {
  assert_entry_facade_live_bindings(true, false);
}

#[test]
fn entry_facade_esm_live_bindings() {
  assert_entry_facade_live_bindings(false, false);
}

#[test]
fn entry_facade_esm_direct_live_bindings() {
  assert_entry_facade_live_bindings(false, true);
}

fn assert_entry_facade_live_bindings(commonjs: bool, direct: bool) {
  let crate_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  let root = crate_path.join("tests/fixtures/string_module_names");
  let mut failures = vec![];
  for tree_shaking in [false, true] {
    for concatenate in [false, true] {
      for minify in [false, true] {
        let output = std::env::temp_dir().join(format!(
          "farm-entry-live-{commonjs}-{direct}-{tree_shaking}-{concatenate}-{minify}-{}",
          std::process::id()
        ));
        let mut config = common::create_config(root.clone(), crate_path.clone());
        config.input = HashMap::from_iter([(
          "main".into(),
          root
            .join(if direct { "value.js" } else { "main.js" })
            .to_string_lossy()
            .into_owned(),
        )]);
        config.input.insert(
          "default-function".into(),
          root
            .join("entry-defaults/function.js")
            .to_string_lossy()
            .into_owned(),
        );
        config.input.insert(
          "default-zero".into(),
          root
            .join("entry-defaults/zero.js")
            .to_string_lossy()
            .into_owned(),
        );
        for name in [
          "primitive-named",
          "function-prototype",
          "frozen-named",
          "null-named",
          "sealed-named",
          "nonextensible-named",
        ] {
          config.input.insert(
            name.into(),
            root
              .join(format!("entry-defaults/{name}.js"))
              .to_string_lossy()
              .into_owned(),
          );
        }
        config.runtime = common::generate_runtime(crate_path.clone(), true);
        config.concatenate_modules = concatenate;
        config.tree_shaking = Box::new(BoolOrObj::Bool(tree_shaking));
        config.output.target_env = TargetEnv::Node;
        config.output.format = ModuleFormatConfig::Single(if commonjs {
          ModuleFormat::CommonJs
        } else {
          ModuleFormat::EsModule
        });
        let extension = if commonjs { "cjs" } else { "mjs" };
        config.output.filename = format!("[resourceName].{extension}");
        config.output.entry_filename = format!("[entryName].{extension}");
        config.output.path = output.to_string_lossy().into_owned();
        config.minify = Box::new(BoolOrObj::Bool(minify));
        let compiler = Compiler::new(config, vec![]).unwrap();
        compiler.compile().unwrap();
        compiler.write_resources_to_disk().unwrap();
        let consumer = if direct {
          "import assert from 'node:assert/strict'; import * as m from './main.mjs'; import {'s-k' as named} from './main.mjs'; assert.equal(named,1); m.increment(); assert.equal(named,2); assert.equal(m['s-k'],2);"
        } else if commonjs {
          "import assert from 'node:assert/strict'; import {createRequire} from 'node:module'; const m = createRequire(import.meta.url)('./main.cjs'); assert.equal(m['public-name'], 1); m.increment(); assert.equal(m['public-name'], 2); assert.deepEqual(m.values(), [2,2,2,2,2,2]); const require = createRequire(import.meta.url); const read = require('./default-function.cjs'); assert.equal(typeof read, 'function'); assert.equal(read.value, 1); read.increment(); assert.equal(read.value, 2); assert.equal(read['mutable-value'], 2); assert.equal(read(), 2); assert.equal(require('./default-zero.cjs'), 0); assert.equal(require('./primitive-named.cjs'), 7); const ctor = require('./function-prototype.cjs'); assert.equal(typeof ctor, 'function'); assert.equal(ctor.prototype.marker, 1); assert.equal(Object.getOwnPropertyDescriptor(ctor, 'prototype').configurable, false); const frozen = require('./frozen-named.cjs'); assert.equal(frozen.existing, 4); assert.equal(frozen.added, undefined); assert.equal(Object.isFrozen(frozen), true); const sealed = require('./sealed-named.cjs'); assert.equal(sealed.existing, 9); assert.equal(sealed.added, undefined); const closed = require('./nonextensible-named.cjs'); assert.equal(closed.existing, 9); assert.equal(closed.added, undefined); assert.equal(Object.isExtensible(closed), false); assert.equal(typeof Object.getOwnPropertyDescriptor(closed, 'existing').get, 'function'); closed.increment(); assert.equal(closed.existing, 10); assert.throws(() => require('./null-named.cjs'), TypeError);"
        } else {
          "import assert from 'node:assert/strict'; import * as m from './main.mjs'; import {'public-name' as named} from './main.mjs'; assert.equal(named, 1); assert.equal(m['public-name'], 1); m.increment(); assert.equal(named, 2); assert.equal(m['public-name'], 2); assert.deepEqual(m.values(), [2,2,2,2,2,2]);"
        };
        let file = output.join("external-consumer.mjs");
        std::fs::write(&file, consumer).unwrap();
        let result = Command::new("node").arg(file).output().unwrap();
        if !result.status.success() {
          failures.push(format!("commonjs={commonjs}, tree_shaking={tree_shaking}, concatenate={concatenate}, minify={minify}: {}", String::from_utf8_lossy(&result.stderr)));
        }
      }
    }
  }
  assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn entry_mutations_execute_real_concat_and_preserve_write_semantics() {
  let crate_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  let root = crate_path.join("tests/fixtures/string_module_names/entry-mutations");
  for concatenate in [false, true] {
    for minify in [false, true] {
      let output = std::env::temp_dir().join(format!(
        "farm-entry-patterns-{concatenate}-{minify}-{}",
        std::process::id()
      ));
      let mut config = common::create_config(root.clone(), crate_path.clone());
      config.input = HashMap::from_iter([(
        "main".into(),
        root.join("main.js").to_string_lossy().into_owned(),
      )]);
      config.runtime = common::generate_runtime(crate_path.clone(), true);
      config.concatenate_modules = concatenate;
      config.tree_shaking = Box::new(BoolOrObj::Bool(true));
      config.output.target_env = TargetEnv::Node;
      config.output.format = ModuleFormatConfig::Single(ModuleFormat::EsModule);
      config.output.filename = "[resourceName].mjs".into();
      config.output.entry_filename = "[entryName].mjs".into();
      config.output.path = output.to_string_lossy().into_owned();
      config.minify = Box::new(BoolOrObj::Bool(minify));
      let compiler = Compiler::new(config, vec![]).unwrap();
      compiler.compile().unwrap();
      compiler.write_resources_to_disk().unwrap();
      {
        let pots = compiler.context().resource_pot_map.read();
        let value_pot = pots
          .resource_pots()
          .into_iter()
          .find(|pot| pot.modules().iter().any(|id| id.to_string() == "value.js"))
          .expect("value module resource pot");
        use farmfe_toolkit::swc_ecma_visit::{Visit, VisitWith};
        struct FactoryFinder {
          key: String,
          found: bool,
        }
        impl Visit for FactoryFinder {
          fn visit_key_value_prop(&mut self, prop: &farmfe_core::swc_ecma_ast::KeyValueProp) {
            use farmfe_core::swc_ecma_ast::{Expr, PropName};
            let key = match &prop.key {
              PropName::Str(value) => value.value.to_string_lossy().into_owned(),
              PropName::Ident(value) => value.sym.to_string(),
              _ => String::new(),
            };
            if key == self.key && matches!(&*prop.value, Expr::Fn(_)) {
              self.found = true;
            }
            prop.visit_children_with(self);
          }
        }
        let value_id = value_pot
          .modules()
          .into_iter()
          .find(|id| id.to_string() == "value.js")
          .cloned()
          .unwrap();
        let mut finder = FactoryFinder {
          key: value_id.id(compiler.context().config.mode),
          found: false,
        };
        value_pot.meta.as_js().ast.visit_with(&mut finder);
        assert_eq!(
          finder.found, !concatenate,
          "must exercise actual merged-module factory rendering, not only the config switch"
        );
      }
      let consumer = "import assert from 'node:assert/strict'; import * as m from './main.mjs'; import {value as named} from './main.mjs'; let calls=0; m.mutate((value,big,rest)=>{calls++;assert.equal(named,value);assert.equal(m.value,value);assert.equal(m.alias,value);assert.equal(m.big,big);assert.deepEqual(m.rest,rest);assert.equal(m.snapshot,1);assert.equal(m.other,100);});assert.ok(calls>=16);m.incrementOther();assert.equal(m.other,101);assert.equal(named,43);";
      let file = output.join("consumer.mjs");
      std::fs::write(&file, consumer).unwrap();
      let result = Command::new("node").arg(file).output().unwrap();
      assert!(
        result.status.success(),
        "concat={concatenate} minify={minify}: {}",
        String::from_utf8_lossy(&result.stderr)
      );
    }
  }
}

#[test]
fn native_entry_slots_recover_compiled_hmr_candidates_and_no_write_replacements() {
  use farmfe_core::plugin::UpdateType;
  let crate_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  let root = std::env::temp_dir().join(format!("farm-entry-hmr-source-{}", std::process::id()));
  std::fs::create_dir_all(&root).unwrap();
  let root = root.canonicalize().unwrap();
  std::fs::write(
    root.join("state.js"),
    "export let value=1;export function increment(){value++;}",
  )
  .unwrap();
  std::fs::write(
    root.join("main.js"),
    "export {value,increment} from './state.js';",
  )
  .unwrap();
  std::fs::write(
    root.join("alias.js"),
    "export {value as alias} from './state.js';",
  )
  .unwrap();
  let output = root.join("output");
  let mut config = common::create_config(root.clone(), crate_path.clone());
  config.input = HashMap::from_iter([
    (
      "main".into(),
      root.join("main.js").to_string_lossy().into_owned(),
    ),
    (
      "alias".into(),
      root.join("alias.js").to_string_lossy().into_owned(),
    ),
  ]);
  config.runtime = common::generate_runtime(crate_path.clone(), true);
  config.mode = farmfe_core::config::Mode::Development;
  config.output.target_env = TargetEnv::Node;
  config.output.format = ModuleFormatConfig::Single(ModuleFormat::EsModule);
  config.output.filename = "[resourceName].mjs".into();
  config.output.entry_filename = "[entryName].mjs".into();
  config.output.path = output.to_string_lossy().into_owned();
  let compiler = Compiler::new(config, vec![]).unwrap();
  compiler.compile().unwrap();
  compiler.write_resources_to_disk().unwrap();
  let state_id = farmfe_core::module::ModuleId::from("state.js").id(compiler.context().config.mode);
  for (name, source) in [
    (
      "candidate",
      "export let value=11;export function increment(){value++;}",
    ),
    (
      "constant",
      "export const value=20;export function increment(){}",
    ),
  ] {
    std::fs::write(root.join("state.js"), source).unwrap();
    let update = compiler
      .update(
        vec![(
          root.join("state.js").to_string_lossy().into_owned(),
          UpdateType::Updated,
        )],
        || {},
        true,
        true,
      )
      .unwrap();
    assert!(
      update
        .updated_module_ids
        .iter()
        .any(|id| id.to_string() == "state.js")
    );
    if name == "constant" {
      assert!(
        update.mutable_resources.contains("entryBindings.seed"),
        "no-write replacement must still publish its initialized origin"
      );
    }
    std::fs::write(
      output.join(format!("{name}-update.mjs")),
      format!(
        "{}\n{}",
        update.immutable_resources, update.mutable_resources
      ),
    )
    .unwrap();
  }
  let repo = crate_path.parent().unwrap().parent().unwrap();
  let repo_json = farmfe_core::serde_json::to_string(&repo.to_string_lossy()).unwrap();
  let id_json = farmfe_core::serde_json::to_string(&state_id).unwrap();
  let consumer = format!(
    r#"
import assert from 'node:assert/strict';
import fs from 'node:fs';import {{createRequire}} from 'node:module';
import * as main from './main.mjs';import {{value as named}} from './main.mjs';import * as second from './alias.mjs';
const repo={repo_json};const ts=createRequire(repo+'/package.json')('typescript');
const source=fs.readFileSync(repo+'/packages/runtime-plugin-hmr/src/update-transaction.ts','utf8');
const js=ts.transpileModule(source,{{compilerOptions:{{module:ts.ModuleKind.ESNext,target:ts.ScriptTarget.ES2022}}}}).outputText;
const {{captureUpdateSnapshot}}=await import('data:text/javascript;base64,'+Buffer.from(js).toString('base64'));
const ms=globalThis.__farm_default_namespace__.m;const id={id_json};
assert.equal(named,1);const old=main.increment;old();assert.equal(named,2);assert.equal(second.alias,2);
const restore=captureUpdateSnapshot(ms,new Map(),[],new Map());
await import('./candidate-update.mjs');ms.a(id);const candidate=ms.r(id);
assert.equal(named,11);assert.equal(second.alias,11);candidate.increment();assert.equal(named,12);
try {{throw new Error('candidate validation failed');}} catch {{restore();}}
assert.equal(named,2);assert.equal(main.value,2);assert.equal(second.alias,2);
candidate.increment();assert.equal(named,2);old();assert.equal(named,3);
await import('./constant-update.mjs');ms.a(id);ms.r(id);
assert.equal(named,20);assert.equal(second.alias,20);old();assert.equal(named,20);
console.log('compiled-hmr-native-slots-restored');
"#
  );
  let file = output.join("consumer.mjs");
  std::fs::write(&file, consumer).unwrap();
  let result = Command::new("node").arg(file).output().unwrap();
  assert!(
    result.status.success(),
    "{}",
    String::from_utf8_lossy(&result.stderr)
  );
  assert!(String::from_utf8_lossy(&result.stdout).contains("compiled-hmr-native-slots-restored"));
}

#[test]
fn native_entry_bindings_survive_disk_cache_and_source_contract_changes() {
  use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
  };
  struct RenderCounter(Arc<AtomicUsize>);
  impl farmfe_core::plugin::Plugin for RenderCounter {
    fn name(&self) -> &str {
      "EntryRenderCounter"
    }
    fn priority(&self) -> i32 {
      10000
    }
    fn render_resource_pot(
      &self,
      pot: &farmfe_core::resource::resource_pot::ResourcePot,
      _context: &Arc<farmfe_core::context::CompilationContext>,
      _hook: &farmfe_core::plugin::PluginHookContext,
    ) -> farmfe_core::error::Result<Option<farmfe_core::resource::meta_data::ResourcePotMetaData>>
    {
      if pot.modules().iter().any(|id| id.to_string() == "state.js") {
        self.0.fetch_add(1, Ordering::SeqCst);
      }
      Ok(None)
    }
  }
  let crate_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  let root = std::env::temp_dir().join(format!("farm-entry-warm-{}", std::process::id()));
  std::fs::create_dir_all(&root).unwrap();
  let root = root.canonicalize().unwrap();
  let output = root.join("output");
  std::fs::write(
    root.join("state.js"),
    "export let value=1;export function increment(){value++;}",
  )
  .unwrap();
  std::fs::write(
    root.join("main.js"),
    "export {value,increment} from './state.js';",
  )
  .unwrap();
  let mut config = common::create_config(root.clone(), crate_path.clone());
  config.input = HashMap::from_iter([(
    "main".into(),
    root.join("main.js").to_string_lossy().into_owned(),
  )]);
  config.runtime = common::generate_runtime(crate_path, true);
  config.output.target_env = TargetEnv::Node;
  config.output.format = ModuleFormatConfig::Single(ModuleFormat::EsModule);
  config.output.filename = "[resourceName].mjs".into();
  config.output.entry_filename = "[entryName].mjs".into();
  config.output.path = output.to_string_lossy().into_owned();
  config.persistent_cache =
    Box::new(farmfe_core::config::persistent_cache::PersistentCacheConfig::Bool(true));
  for phase in 0..5 {
    if phase == 2 {
      std::fs::write(
        root.join("main.js"),
        "export {value as renamed,increment} from './state.js';",
      )
      .unwrap();
    }
    if phase == 3 {
      std::fs::write(
        root.join("state.js"),
        "export const value=5;export function increment(){}",
      )
      .unwrap();
    }
    if phase == 4 {
      std::fs::write(
        root.join("state.js"),
        "export let value=7;export function increment(){value++;}",
      )
      .unwrap();
    }
    let count = Arc::new(AtomicUsize::new(0));
    let compiler =
      Compiler::new(config.clone(), vec![Arc::new(RenderCounter(count.clone()))]).unwrap();
    compiler.compile().unwrap();
    compiler.flush_cache().unwrap();
    compiler.write_resources_to_disk().unwrap();
    let renders = count.load(Ordering::SeqCst);
    if phase == 0 {
      assert!(renders > 0, "cold compilation must render the source");
    }
    if phase == 1 {
      assert_eq!(
        renders, 0,
        "second compiler must really reuse disk resource cache"
      );
    }
    let name = if phase < 2 { "value" } else { "renamed" };
    let before = if phase < 3 {
      1
    } else if phase == 3 {
      5
    } else {
      7
    };
    let after = if phase == 3 { 5 } else { before + 1 };
    let consumer = format!(
      "import assert from 'node:assert/strict';import * as m from './main.mjs';import {{{name} as named}} from './main.mjs';assert.equal(named,{before});m.increment();assert.equal(named,{after});assert.equal(m.{name},{after});"
    );
    let file = output.join(format!("consumer-{phase}.mjs"));
    std::fs::write(&file, consumer).unwrap();
    let result = Command::new("node").arg(file).output().unwrap();
    assert!(
      result.status.success(),
      "phase={phase}, renders={renders}: {}",
      String::from_utf8_lossy(&result.stderr)
    );
    println!("entry cache phase={phase} source renders={renders}");
  }
}

#[test]
fn native_library_mutable_exports_do_not_receive_runtime_bridges() {
  let crate_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  let root = std::env::temp_dir().join(format!("farm-library-live-domain-{}", std::process::id()));
  std::fs::create_dir_all(&root).unwrap();
  let root = root.canonicalize().unwrap();
  std::fs::write(
    root.join("state.js"),
    "export let value=1;export function increment(){value++;}",
  )
  .unwrap();
  std::fs::write(
    root.join("main.js"),
    "export {value,increment} from './state.js';",
  )
  .unwrap();
  for multiple in [false, true] {
    let output = root.join(format!("output-{multiple}"));
    let mut config = common::create_config(root.clone(), crate_path.clone());
    config.input = HashMap::from_iter([(
      "main".into(),
      root.join("main.js").to_string_lossy().into_owned(),
    )]);
    config.runtime = common::generate_runtime(crate_path.clone(), true);
    config.output.target_env = TargetEnv::Library;
    config.output.format = if multiple {
      ModuleFormatConfig::Multiple(vec![ModuleFormat::EsModule, ModuleFormat::CommonJs])
    } else {
      ModuleFormatConfig::Single(ModuleFormat::EsModule)
    };
    config.output.filename = "[resourceName].js".into();
    config.output.entry_filename = "[entryName].js".into();
    config.output.path = output.to_string_lossy().into_owned();
    config.concatenate_modules = true;
    config.minify = Box::new(BoolOrObj::Bool(false));
    let compiler = Compiler::new(config, vec![]).unwrap();
    compiler.compile().unwrap();
    for resource in compiler.context().resources_map.lock().values() {
      let source = String::from_utf8_lossy(&resource.bytes);
      assert!(
        !source.contains("entryBindings"),
        "library resource {} acquired runtime ABI",
        resource.name
      );
      if resource.name.ends_with(".js") {
        let esm = !resource.name.starts_with("cjs/");
        std::fs::create_dir_all(&output).unwrap();
        let file = output.join(if esm { "library.mjs" } else { "library.cjs" });
        std::fs::write(&file, &resource.bytes).unwrap();
        let script = "import assert from 'node:assert/strict';import {pathToFileURL} from 'node:url';import {createRequire} from 'node:module';const p=process.argv[1];const m=p.endsWith('.cjs')?createRequire(import.meta.url)(p):await import(pathToFileURL(p));assert.equal(m.value,1);m.increment();assert.equal(m.value,2);";
        let result = Command::new("node")
          .args(["--input-type=module", "--eval", script])
          .arg(file)
          .output()
          .unwrap();
        assert!(
          result.status.success(),
          "multiple={multiple}, resource={}: {}",
          resource.name,
          String::from_utf8_lossy(&result.stderr)
        );
      }
    }
  }
}

#[test]
fn html_and_mixed_entries_switch_bridge_domain_before_disk_cache_lookup() {
  let crate_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  let root = std::env::temp_dir().join(format!("farm-html-live-domain-{}", std::process::id()));
  std::fs::create_dir_all(&root).unwrap();
  let root = root.canonicalize().unwrap();
  std::fs::write(
    root.join("state.js"),
    "export let value=1;export function increment(){value++;}",
  )
  .unwrap();
  std::fs::write(
    root.join("second.js"),
    "export let other=1;export function step(){other++;}",
  )
  .unwrap();
  std::fs::write(root.join("main.js"), "export {value,increment} from './state.js';import {value,increment} from './state.js';import {other,step} from './second.js';globalThis.domainRead=()=>value;globalThis.domainIncrement=increment;globalThis.domainOther=()=>other;globalThis.domainStep=step;").unwrap();
  std::fs::write(root.join("index.html"), "<!DOCTYPE html><html><head></head><body><script type=module src='./main.js'></script></body></html>").unwrap();
  for (phase, mixed) in [false, true, true, false, false].into_iter().enumerate() {
    let mut config = common::create_config(root.clone(), crate_path.clone());
    config.input = HashMap::from_iter([(
      "index".into(),
      root.join("index.html").to_string_lossy().into_owned(),
    )]);
    if mixed {
      config.input.insert(
        "main".into(),
        root.join("main.js").to_string_lossy().into_owned(),
      );
    }
    config.runtime = common::generate_runtime(crate_path.clone(), true);
    // Match the core configuration's always-installed import-meta runtime plugin.
    config.runtime.plugins.push(
      crate_path
        .join("../../packages/runtime-plugin-import-meta/src/index.ts")
        .to_string_lossy()
        .into_owned(),
    );
    config.define.insert("FARM_PROCESS_ENV".into(), "{}".into());
    config.output.target_env = TargetEnv::Browser;
    config.output.format = ModuleFormatConfig::Single(ModuleFormat::EsModule);
    config.output.path = root.join("output").to_string_lossy().into_owned();
    config.output.filename = "[resourceName].mjs".into();
    config.output.entry_filename = "[entryName].mjs".into();
    config.minify = Box::new(BoolOrObj::Bool(false));
    config.persistent_cache =
      Box::new(farmfe_core::config::persistent_cache::PersistentCacheConfig::Bool(true));
    let compiler = Compiler::new(config, vec![]).unwrap();
    compiler.compile().unwrap();
    compiler.flush_cache().unwrap();
    compiler.write_resources_to_disk().unwrap();
    let resources = compiler.context().resources_map.lock();
    let calls = resources
      .values()
      .map(|resource| {
        String::from_utf8_lossy(&resource.bytes)
          .matches(".entryBindings.bind(")
          .count()
      })
      .sum::<usize>();
    assert_eq!(calls > 0, mixed, "phase={phase}, calls={calls}");
    if mixed {
      let script = "import assert from 'node:assert/strict';import {pathToFileURL} from 'node:url';globalThis.window=globalThis;globalThis.location=pathToFileURL(process.argv[1]);const m=await import(pathToFileURL(process.argv[1]));assert.equal(m.value,1);m.increment();assert.equal(m.value,2);assert.equal(globalThis.domainRead(),2);";
      let result = Command::new("node")
        .args(["--input-type=module", "--eval", script])
        .arg(root.join("output/main.mjs"))
        .output()
        .unwrap();
      assert!(
        result.status.success(),
        "mixed phase={phase}: {}",
        String::from_utf8_lossy(&result.stderr)
      );
    }
    if !mixed {
      assert!(
        !compiler
          .context()
          .custom
          .lock()
          .contains_key("farm.entry-live-scans.v1"),
        "HTML-only must skip origin scans, not merely omit generated calls"
      );
      let script_resource = resources
        .values()
        .find(|resource| {
          matches!(
            resource.resource_type,
            farmfe_core::resource::ResourceType::Html
          )
        })
        .expect("HTML entry resource");
      // Execute the exact emitted script order with local URL globals. This is
      // a bootstrap simulation, not browser DOM/network acceptance.
      let script = r#"import assert from 'node:assert/strict';import {pathToFileURL} from 'node:url';import fs from 'node:fs';import path from 'node:path';import vm from 'node:vm';const file=process.argv[1];globalThis.window=globalThis;globalThis.location=pathToFileURL(file);const html=fs.readFileSync(file,'utf8');for(const tag of html.matchAll(/<script\b([^>]*)>([\s\S]*?)<\/script>/g)){const source=tag[1].match(/\bsrc=(?:"([^"]+)"|'([^']+)'|([^\s>]+))/);let code=tag[2];if(source){const url=source[1]||source[2]||source[3];assert.ok(url.startsWith('/')&&!url.startsWith('//')&&!url.includes('..'));code=fs.readFileSync(path.join(path.dirname(file),url.slice(1)),'utf8');}vm.runInThisContext(code);}assert.equal(globalThis.domainRead(),1);globalThis.domainIncrement();assert.equal(globalThis.domainRead(),2);assert.equal(globalThis.domainOther(),1);globalThis.domainStep();assert.equal(globalThis.domainOther(),2);"#;
      let result = Command::new("node")
        .args(["--input-type=module", "--eval", script])
        .arg(root.join("output").join(&script_resource.name))
        .output()
        .unwrap();
      assert!(
        result.status.success(),
        "HTML phase={phase}: {}",
        String::from_utf8_lossy(&result.stderr)
      );
      for resource in resources.values() {
        let text = String::from_utf8_lossy(&resource.bytes);
        assert!(
          !text.contains(".entryBindings.seed(") && !text.contains(".entryBindings.complete(")
        );
      }
    }
  }
}

#[test]
fn native_entry_bridge_upgrades_shared_pre_bridge_runtime_without_losing_cache() {
  fn copy_sources(from: &std::path::Path, to: &std::path::Path) {
    std::fs::create_dir_all(to).unwrap();
    for entry in std::fs::read_dir(from).unwrap() {
      let entry = entry.unwrap();
      if entry.file_type().unwrap().is_dir() {
        copy_sources(&entry.path(), &to.join(entry.file_name()));
      } else {
        std::fs::copy(entry.path(), to.join(entry.file_name())).unwrap();
      }
    }
  }
  let crate_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  for tree_shaking in [false, true] {
    for concatenate in [false, true] {
      for minify in [false, true] {
        let root = std::env::temp_dir().join(format!(
          "farm-old-runtime-live-{tree_shaking}-{concatenate}-{minify}-{}",
          std::process::id()
        ));
        std::fs::create_dir_all(&root).unwrap();
        let root = root.canonicalize().unwrap();
        let runtime = crate_path.join("../../packages/runtime");
        let old_runtime = root.join("old-runtime");
        copy_sources(&runtime.join("src"), &old_runtime.join("src"));
        std::fs::copy(
          runtime.join("package.json"),
          old_runtime.join("package.json"),
        )
        .unwrap();
        // Frozen production helper from 593dc94f; all other runtime sources are unchanged.
        std::fs::copy(
          crate_path
            .join("tests/fixtures/string_module_names/runtime-compat/pre-entry-bridge-helper.ts"),
          old_runtime.join("src/modules/module-helper.ts"),
        )
        .unwrap();
        std::fs::write(root.join("old.js"), "export const identity={old:true};").unwrap();
        std::fs::write(
          root.join("new.js"),
          "export let value=1;export function increment(){value++;}",
        )
        .unwrap();
        for old in [true, false] {
          let entry = if old { "old" } else { "new" };
          let mut config = common::create_config(root.clone(), crate_path.clone());
          config.input = HashMap::from_iter([(
            entry.into(),
            root
              .join(format!("{entry}.js"))
              .to_string_lossy()
              .into_owned(),
          )]);
          config.runtime = common::generate_runtime(crate_path.clone(), true);
          if old {
            config.runtime.path = old_runtime.to_string_lossy().into_owned();
          }
          config.preset_env =
            Box::new(farmfe_core::config::preset_env::PresetEnvConfig::Bool(true));
          config.output.target_env = TargetEnv::Node;
          config.output.format = ModuleFormatConfig::Single(ModuleFormat::EsModule);
          config.output.filename = "[resourceName].mjs".into();
          config.output.entry_filename = "[entryName].mjs".into();
          config.output.path = root.join(entry).to_string_lossy().into_owned();
          config.minify = Box::new(BoolOrObj::Bool(minify));
          config.concatenate_modules = concatenate;
          config.tree_shaking = Box::new(BoolOrObj::Bool(tree_shaking));
          let compiler = Compiler::new(config, vec![]).unwrap();
          compiler.compile().unwrap();
          compiler.write_resources_to_disk().unwrap();
        }
        let script = "import assert from 'node:assert/strict';import {pathToFileURL} from 'node:url';const root=process.argv[1];const old=await import(pathToFileURL(root+'/old/old.mjs'));const identity=old.identity;const host=globalThis.__farm_default_namespace__.m;const cache=host.c();const helper=cache['@farm-runtime/module-helper'];assert.equal(host.r.entryBindings,undefined);const m=await import(pathToFileURL(root+'/new/new.mjs'));assert.equal(globalThis.__farm_default_namespace__.m,host);assert.equal(host.c()['@farm-runtime/module-helper'],helper);assert.equal(old.identity,identity);assert.equal(m.value,1);m.increment();assert.equal(m.value,2);const restore=host.r.entryBindings.snapshot();m.increment();assert.equal(m.value,3);restore();assert.equal(m.value,2);m.increment();assert.equal(m.value,3);";
        let result = Command::new("node")
          .args(["--input-type=module", "--eval", script])
          .arg(&root)
          .output()
          .unwrap();
        assert!(
          result.status.success(),
          "{}",
          String::from_utf8_lossy(&result.stderr)
        );
      }
    }
  }
}

#[test]
fn readonly_origin_assignment_keeps_failure_and_snapshot_never_writes_const() {
  let crate_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  let root = std::env::temp_dir().join(format!("farm-readonly-live-{}", std::process::id()));
  std::fs::create_dir_all(&root).unwrap();
  let root = root.canonicalize().unwrap();
  std::fs::write(root.join("main.js"), "const value=1;export {value};export function attempt(){try{value=2;return false;}catch{return true;}}").unwrap();
  for concatenate in [false, true] {
    for minify in [false, true] {
      let output = root.join(format!("output-{concatenate}-{minify}"));
      let mut config = common::create_config(root.clone(), crate_path.clone());
      config.input = HashMap::from_iter([(
        "main".into(),
        root.join("main.js").to_string_lossy().into_owned(),
      )]);
      config.runtime = common::generate_runtime(crate_path.clone(), true);
      config.concatenate_modules = concatenate;
      config.minify = Box::new(BoolOrObj::Bool(minify));
      config.output.target_env = TargetEnv::Node;
      config.output.format = ModuleFormatConfig::Single(ModuleFormat::EsModule);
      config.output.filename = "[resourceName].mjs".into();
      config.output.entry_filename = "[entryName].mjs".into();
      config.output.path = output.to_string_lossy().into_owned();
      let compiler = Compiler::new(config, vec![]).unwrap();
      compiler.compile().unwrap();
      compiler.write_resources_to_disk().unwrap();
      let script = "import assert from 'node:assert/strict';import {pathToFileURL} from 'node:url';const m=await import(pathToFileURL(process.argv[1]));assert.equal(m.attempt(),true);assert.equal(m.value,1);const restore=globalThis.__farm_default_namespace__.m.r.entryBindings.snapshot();restore();assert.equal(m.value,1);";
      let result = Command::new("node")
        .args(["--input-type=module", "--eval", script])
        .arg(output.join("main.mjs"))
        .output()
        .unwrap();
      assert!(
        result.status.success(),
        "concat={concatenate} minify={minify}: {}",
        String::from_utf8_lossy(&result.stderr)
      );
    }
  }
}

#[test]
fn native_entry_upgrade_rejects_unclosed_factory_dependencies() {
  fn copy_sources(from: &std::path::Path, to: &std::path::Path) {
    std::fs::create_dir_all(to).unwrap();
    for entry in std::fs::read_dir(from).unwrap() {
      let entry = entry.unwrap();
      if entry.file_type().unwrap().is_dir() {
        copy_sources(&entry.path(), &to.join(entry.file_name()));
      } else {
        std::fs::copy(entry.path(), to.join(entry.file_name())).unwrap();
      }
    }
  }
  let crate_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  let root = std::env::temp_dir().join(format!("farm-unclosed-bridge-{}", std::process::id()));
  std::fs::create_dir_all(&root).unwrap();
  let root = root.canonicalize().unwrap();
  let runtime = crate_path.join("../../packages/runtime");
  let helper = std::fs::read_to_string(runtime.join("src/modules/module-helper.ts")).unwrap();
  std::fs::write(
    root.join("main.js"),
    "export let value=1;export function increment(){value++;}",
  )
  .unwrap();
  for preset in [false, true] {
    for module_capture in [false, true] {
      let custom = root.join(format!("runtime-{preset}-{module_capture}"));
      copy_sources(&runtime.join("src"), &custom.join("src"));
      std::fs::copy(runtime.join("package.json"), custom.join("package.json")).unwrap();
      let edited = if module_capture {
        format!(
          "const capturedBridgeValue = globalThis.unavailable;\n{}",
          helper.replace(
            "const origins = new Map<string, Origin>();",
            "const origins = capturedBridgeValue;"
          )
        )
      } else {
        helper.replace(
          "const origins = new Map<string, Origin>();",
          "const origins = FARM_UNKNOWN_BRIDGE_DEPENDENCY();",
        )
      };
      assert_ne!(edited, helper);
      std::fs::write(custom.join("src/modules/module-helper.ts"), edited).unwrap();
      let mut config = common::create_config(root.clone(), crate_path.clone());
      config.input = HashMap::from_iter([(
        "main".into(),
        root.join("main.js").to_string_lossy().into_owned(),
      )]);
      config.runtime = common::generate_runtime(crate_path.clone(), true);
      config.runtime.path = custom.to_string_lossy().into_owned();
      config.output.target_env = TargetEnv::Node;
      config.output.format = ModuleFormatConfig::Single(ModuleFormat::EsModule);
      config.preset_env = Box::new(farmfe_core::config::preset_env::PresetEnvConfig::Bool(
        preset,
      ));
      let compiler = Compiler::new(config, vec![]).unwrap();
      let error = compiler
        .compile()
        .expect_err("unclosed bridge must fail compilation");
      assert!(
        format!("{error:?}").contains("unclosed dependencies"),
        "unexpected error: {error:?}"
      );
    }
  }
}
