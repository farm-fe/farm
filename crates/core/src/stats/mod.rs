use std::collections::HashMap;

use parking_lot::RwLock;

use crate::{
  module::{
    module_graph::{ModuleGraph, ModuleGraphEdge},
    ModuleId, ModuleType,
  },
  plugin::PluginHookContext,
};

pub struct Stats {
  /// First compilation flow stats
  pub initial_compilation_flow_stats: RwLock<CompilationStats>,
  /// Most 10 recent compilation flow stats
  pub hmr_compilation_flow_stats: RwLock<Vec<CompilationStats>>,
}

macro_rules! handle_compilation_stats {
  ($stats:expr, $func:expr) => {{
    let mut flow_stats = $stats.hmr_compilation_flow_stats.write();

    let compilation_stats = if flow_stats.len() > 0 {
      flow_stats.last_mut().unwrap()
    } else {
      &mut $stats.initial_compilation_flow_stats.write()
    };

    $func(compilation_stats)
  }};
}

impl ToString for Stats {
  fn to_string(&self) -> String {
    // return json string
    let initial_compilation_flow_stats: &CompilationStats =
      &self.initial_compilation_flow_stats.read();
    let hmr_compilation_flow_stats: &Vec<CompilationStats> =
      &self.hmr_compilation_flow_stats.read();

    format!(
      "{{ \n  \"initialCompilationFlowStats\": {},\n  \"hmrCompilationFlowStats\": {}\n}}",
      serde_json::to_string(initial_compilation_flow_stats).unwrap(),
      serde_json::to_string(hmr_compilation_flow_stats).unwrap()
    )
  }
}

impl Stats {
  pub fn new() -> Self {
    Self {
      initial_compilation_flow_stats: RwLock::new(CompilationStats::new()),
      hmr_compilation_flow_stats: RwLock::new(vec![]),
    }
  }

  pub fn set_start_time(&self) {
    handle_compilation_stats!(self, |compilation_stats: &mut CompilationStats| {
      compilation_stats.set_start_time();
    })
  }

  pub fn set_build_end_time(&self) {
    handle_compilation_stats!(self, |compilation_stats: &mut CompilationStats| {
      compilation_stats.set_build_end_time();
    })
  }

  pub fn set_end_time(&self) {
    handle_compilation_stats!(self, |compilation_stats: &mut CompilationStats| {
      compilation_stats.set_end_time();
    })
  }

  pub fn add_hmr_compilation_stats(&self) {
    let mut flow_stats = self.hmr_compilation_flow_stats.write();
    flow_stats.push(CompilationStats::new());

    if flow_stats.len() > 10 {
      flow_stats.remove(0);
    }
  }

  pub fn add_plugin_hook_stats(&self, hook_stats: CompilationPluginHookStats) {
    handle_compilation_stats!(self, |compilation_stats: &mut CompilationStats| {
      compilation_stats.add_plugin_hook_stats(hook_stats)
    })
  }

  pub fn set_module_graph_stats(&self, module_graph: &ModuleGraph) {
    handle_compilation_stats!(self, |compilation_stats: &mut CompilationStats| {
      compilation_stats.set_module_graph_stats(module_graph);
    })
  }

  pub fn set_entries(&self, entries: Vec<ModuleId>) {
    handle_compilation_stats!(self, |compilation_stats: &mut CompilationStats| {
      compilation_stats.entries = entries;
    })
  }
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompilationStats {
  pub entries: Vec<ModuleId>,
  pub hook_stats_map: HashMap<String, Vec<CompilationPluginHookStats>>,
  pub module_graph_stats: CompilationModuleGraphStats,
  pub duration: u128,
  pub start_time: u128,
  pub build_end_time: u128,
  pub end_time: u128,
}

impl CompilationStats {
  pub fn new() -> Self {
    Self {
      entries: vec![],
      hook_stats_map: HashMap::new(),
      module_graph_stats: CompilationModuleGraphStats::new(),
      duration: 0,
      start_time: 0,
      build_end_time: 0,
      end_time: 0,
    }
  }

  pub fn set_start_time(&mut self) {
    self.start_time = std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap()
      .as_millis();
  }

  pub fn set_build_end_time(&mut self) {
    self.build_end_time = std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap()
      .as_millis();
  }

  pub fn set_end_time(&mut self) {
    self.end_time = std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap()
      .as_millis();
    self.duration = self.end_time - self.start_time;
  }

  pub fn add_plugin_hook_stats(&mut self, hook_stats: CompilationPluginHookStats) {
    let hook_stats_vec = self
      .hook_stats_map
      .entry(hook_stats.hook_name.clone())
      .or_insert(vec![]);
    hook_stats_vec.push(hook_stats);
  }

  pub fn set_module_graph_stats(&mut self, module_graph: &ModuleGraph) {
    self.module_graph_stats = module_graph.into();
  }
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompilationPluginHookStats {
  pub plugin_name: String,
  pub hook_name: String,
  pub module_id: ModuleId,
  pub hook_context: Option<PluginHookContext>,
  /// JSON string of input of the hook
  pub input: String,
  /// JSON string of output of the hook
  pub output: String,
  /// Duration of the hook in ms
  pub duration: u128,
  pub start_time: u128,
  pub end_time: u128,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompilationModuleGraphStats {
  pub modules: HashMap<ModuleId, CompilationModuleStats>,
  pub edges: HashMap<ModuleId, Vec<(ModuleId, ModuleGraphEdge)>>,
}

impl CompilationModuleGraphStats {
  pub fn new() -> Self {
    Self {
      modules: HashMap::new(),
      edges: HashMap::new(),
    }
  }
}

impl From<&ModuleGraph> for CompilationModuleGraphStats {
  fn from(module_graph: &ModuleGraph) -> Self {
    let mut stats = Self::new();
    for module in module_graph.modules() {
      stats.modules.insert(
        module.id.clone(),
        CompilationModuleStats {
          module_id: module.id.clone(),
          module_type: module.module_type.clone(),
        },
      );
      for (dep_module_id, edge) in module_graph.dependencies(&module.id) {
        let edges = stats.edges.entry(module.id.clone()).or_insert(Vec::new());
        edges.push((dep_module_id.clone(), edge.clone()));
      }
    }
    stats
  }
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompilationModuleStats {
  pub module_id: ModuleId,
  pub module_type: ModuleType,
}
