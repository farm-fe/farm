use rkyv::Deserialize;
use std::{path::PathBuf, sync::Arc};

use farmfe_core::{
  cache::cache_store::CacheStoreKey,
  context::CompilationContext,
  deserialize,
  enhanced_magic_string::collapse_sourcemap::{collapse_sourcemap_chain, CollapseSourcemapOptions},
  module::{ModuleId, ModuleMetaData, ModuleSystem, ModuleType, ScriptModuleMetaData},
  rayon::prelude::*,
  serialize,
  swc_common::Mark,
  swc_css_ast::Stylesheet,
  swc_ecma_ast::EsVersion,
  swc_ecma_parser::Syntax,
};
use farmfe_toolkit::{
  common::{create_swc_source_map, Source},
  css::codegen_css_stylesheet,
  hash::base64_encode,
  script::{parse_module, swc_try_with::try_with},
  sourcemap::SourceMap,
  swc_ecma_transforms_base::resolver,
  swc_ecma_visit::VisitMutWith,
};
use farmfe_utils::{hash::sha256, relative};

use crate::source_replace;

pub fn transform_css_to_script_modules(
  module_ids: Vec<ModuleId>,
  context: &Arc<CompilationContext>,
) -> farmfe_core::error::Result<()> {
  module_ids
    .into_par_iter()
    .filter(|id| {
      let g = context.module_graph.read();
      let m = g.module(id).unwrap();
      matches!(m.module_type, ModuleType::Css)
    })
    .try_for_each(|module_id: ModuleId| {
      let mut cache_store_key = None;

      if context.config.persistent_cache.enabled() {
        let content_hash = {
          let module_graph = context.module_graph.read();
          let m = module_graph.module(&module_id).unwrap();
          m.content_hash.clone()
        };
        // try read custom css transform cache
        let store_key = CacheStoreKey {
          name: module_id.to_string() + "-transform_css_to_script_modules",
          key: sha256(
            format!("{}{}", content_hash, module_id.to_string()).as_bytes(),
            32,
          ),
        };
        let cache_manager = &context.cache_manager;

        if cache_manager.custom.has_cache(&store_key.name)
          && !cache_manager.custom.is_cache_changed(&store_key)
        {
          let cache = cache_manager.custom.read_cache(&store_key.name).unwrap();
          let meta = deserialize!(&cache, ModuleMetaData);
          let mut module_graph = context.module_graph.write();
          let module = module_graph.module_mut(&module_id).unwrap();
          module.meta = meta;
          module.module_type = ModuleType::Js;
          return Ok(());
        }

        cache_store_key = Some(store_key);
      }

      let stylesheet = transform_css_stylesheet(&module_id, context);
      let css_deps = transform_css_deps(&module_id, context);

      // let source_map_enabled = context.config.sourcemap.enabled();
      let module_graph = context.module_graph.read();
      let m = module_graph.module(&module_id).unwrap();
      let (css_code, mut src_map) = codegen_css_stylesheet(
        &stylesheet,
        if context.config.sourcemap.enabled(m.immutable) {
          Some(Source {
            path: PathBuf::from(module_id.resolved_path_with_query(&context.config.root)),
            content: m.content.clone(),
          })
        } else {
          None
        },
        context.config.minify,
      );
      let mut source_map_chain = m.source_map_chain.clone();
      drop(module_graph);
      if let Some(sm) = src_map {
        let root = context.config.root.clone();
        source_map_chain.push(Arc::new(sm));
        let map = collapse_sourcemap_chain(
          source_map_chain
            .into_iter()
            .map(|s| SourceMap::from_slice(s.as_bytes()).unwrap())
            .collect(),
          CollapseSourcemapOptions {
            remap_source: Some(Box::new(move |src| format!("/{}", relative(&root, src)))),
            inline_content: true,
          },
        );
        let mut buf = vec![];
        map.to_writer(&mut buf).expect("failed to write sourcemap");
        src_map = Some(String::from_utf8(buf).unwrap());

        context
          .module_graph
          .write()
          .module_mut(&module_id)
          .unwrap()
          .source_map_chain = vec![];
      }

      let css_code = wrapper_style_load(&css_code, module_id.to_string(), &css_deps, src_map);
      let css_code = Arc::new(css_code);
      let (cm, _) = create_swc_source_map(Source {
        path: PathBuf::from(module_id.to_string()),
        content: css_code.clone(),
      });
      {
        context
          .module_graph
          .write()
          .module_mut(&module_id)
          .unwrap()
          .content = css_code.clone();
      }

      try_with(cm.clone(), &context.meta.script.globals, || {
        let mut ast = parse_module(
          &module_id.to_string(),
          &css_code,
          Syntax::default(),
          EsVersion::default(),
        )
        .unwrap();
        let top_level_mark = Mark::new();
        let unresolved_mark = Mark::new();

        ast.visit_mut_with(&mut resolver(unresolved_mark, top_level_mark, false));

        let mut module_graph = context.module_graph.write();
        let module = module_graph.module_mut(&module_id).unwrap();

        module.meta = ModuleMetaData::Script(ScriptModuleMetaData {
          ast,
          top_level_mark: top_level_mark.as_u32(),
          unresolved_mark: unresolved_mark.as_u32(),
          module_system: ModuleSystem::EsModule,
          hmr_self_accepted: true,
          hmr_accepted_deps: Default::default(),
        });

        module.module_type = ModuleType::Js;

        if context.config.persistent_cache.enabled() {
          let store_key = cache_store_key.unwrap();
          let bytes = serialize!(&module.meta);
          context
            .cache_manager
            .custom
            .write_single_cache(store_key, bytes)
            .expect("failed to write css transform cache");
        }
      })
    })
}

pub fn transform_css_stylesheet(
  module_id: &ModuleId,
  context: &Arc<CompilationContext>,
) -> Stylesheet {
  let mut module_graph = context.module_graph.write();

  let mut stylesheet = {
    let module = module_graph.module_mut(module_id).unwrap();
    module.meta.as_css_mut().take_ast()
  };

  let resources_map = context.resources_map.lock();
  source_replace(&mut stylesheet, module_id, &module_graph, &resources_map);

  stylesheet
}

pub fn transform_css_deps(module_id: &ModuleId, context: &Arc<CompilationContext>) -> String {
  let module_graph = context.module_graph.read();
  let mut load_statements = Vec::new();
  let dep_modules = module_graph.dependencies(module_id);

  for (module, _) in dep_modules {
    let relative_path = module.id(context.config.mode.clone()).to_string();
    let load_statement = format!(
      "farmRequire(\"{}\");",
      if cfg!(windows) {
        relative_path.replace('\\', "\\\\")
      } else {
        relative_path.to_string()
      }
    );
    load_statements.push(load_statement);
  }
  load_statements.join(" ")
}

pub fn wrapper_style_load(
  code: &str,
  id: String,
  css_deps: &String,
  src_map: Option<String>,
) -> String {
  format!(
    r#"
const cssCode = `{}`;
const farmId = '{}';
{}
const previousStyle = document.querySelector(`style[data-farm-id="${{farmId}}"]`);
const style = document.createElement('style');
style.setAttribute('data-farm-id', farmId);
style.innerHTML = cssCode;
if (previousStyle) {{
previousStyle.replaceWith(style);
}} else {{
document.head.appendChild(style);
}}

if (module.meta.hot) {{
  module.meta.hot.accept();
  module.meta.hot.prune(() => {{
    style.remove();
  }});
}}
"#,
    format!(
      "{}\n{}",
      code.replace('`', "'").replace('\\', "\\\\"),
      if let Some(src_map) = src_map {
        format!(
          r#"/*# sourceMappingURL=data:application/json;charset=utf-8;base64,{} */"#,
          base64_encode(src_map.as_bytes())
        )
      } else {
        "".to_string()
      }
    ),
    id.replace('\\', "\\\\"),
    css_deps,
  )
}
