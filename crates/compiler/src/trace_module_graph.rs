use std::collections::HashMap;

use farmfe_core::serde::{Deserialize, Serialize};

use crate::Compiler;

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "farmfe_core::serde", rename_all = "camelCase")]
pub struct TracedModule {
  pub id: String,
  pub content_hash: String,
  pub package_name: String,
  pub package_version: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "farmfe_core::serde", rename_all = "camelCase")]
pub struct TracedModuleGraph {
  pub root: String,
  pub modules: Vec<TracedModule>,
  pub edges: HashMap<String, Vec<String>>,
  pub reverse_edges: HashMap<String, Vec<String>>,
}

impl TracedModuleGraph {
  pub fn new(root: String) -> Self {
    Self {
      root,
      modules: vec![],
      edges: HashMap::new(),
      reverse_edges: HashMap::new(),
    }
  }

  pub fn add_module(&mut self, module: TracedModule) {
    self.modules.push(module);
  }

  pub fn add_edge(&mut self, from: String, to: String) {
    self.edges.entry(from.clone()).or_default().push(to.clone());
    self.reverse_edges.entry(to).or_default().push(from);
  }
}

impl Compiler {
  pub fn trace_module_graph(&self) -> farmfe_core::error::Result<TracedModuleGraph> {
    self.build()?;

    let mut graph = TracedModuleGraph::new(self.context.config.root.clone());
    let module_graph = self.context.module_graph.read();

    for module in module_graph.modules() {
      let t_module = TracedModule {
        id: module.id.to_string(),
        content_hash: module.content_hash.to_string(),
        package_name: module.package_name.to_string(),
        package_version: module.package_version.to_string(),
      };
      graph.add_module(t_module);
    }

    let watch_graph = self.context.watch_graph.read();

    for module in module_graph.modules() {
      let deps = module_graph.dependencies(&module.id);

      for (dep, _) in deps {
        graph.add_edge(module.id.to_string(), dep.to_string());
      }
      // watch graph added by scss, less, etc.
      for dep in watch_graph.dependencies(&module.id) {
        graph.add_edge(module.id.to_string(), dep.to_string());
      }
    }

    Ok(graph)
  }
}
