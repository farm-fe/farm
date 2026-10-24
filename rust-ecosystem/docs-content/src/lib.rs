//! Content semantics shared by native Docs and MDX plugins, without plugin ABI exports.
mod code_blocks;
mod comments;
mod headings;
mod jsx_attributes;
mod machine_markdown;
mod markdown_html;
mod metadata;
mod preprocess;
mod references;
mod resource_bindings;
mod source_map;
mod static_highlight;

use markdown::Location;
use mdxjs::{Options, hast};
use serde_json::Value;

pub use farmfe_syntax_highlight::{HighlightOptions, HighlightRepresentation};
pub use headings::Heading;
pub use metadata::{MetadataLocations, MetadataPosition};
pub use references::{ContentReference, ReferenceKind};

/// Rust-only intermediate form: metadata and body share the same retained syntax tree.
#[derive(Clone)]
pub struct ParsedContent {
  pub frontmatter: Value,
  pub metadata_locations: MetadataLocations,
  pub toc: Vec<Heading>,
  pub diagnostics: Vec<String>,
  tree: hast::Node,
  options: Options,
  source: String,
  prepared_source: String,
  resource_bindings: resource_bindings::ResourceBindings,
  default_admonitions: std::collections::HashMap<usize, String>,
  static_code_blocks: static_highlight::Blocks,
}
impl ParsedContent {
  /// Export the retained content tree without executing MDX or scraping HTML.
  /// Unsupported dynamic content is an error; partial Markdown is never returned.
  pub fn machine_markdown(&self) -> Result<String, String> {
    machine_markdown::export(self)
  }
  /// Apply the final collection's defaults once to this parsed-content clone.
  /// Fallback routes clone the original tree before selecting target labels.
  pub fn apply_admonition_labels(&mut self, labels: &std::collections::BTreeMap<String, String>) {
    let defaults = std::mem::take(&mut self.default_admonitions);
    preprocess::apply_admonition_labels(&mut self.tree, &defaults, labels);
  }
  pub fn bind_resource(&mut self, url: String, source: String, suffix: String) {
    self.resource_bindings.0.insert(url, (source, suffix));
  }
  pub fn anchors(&self) -> Result<std::collections::HashSet<String>, String> {
    references::anchors(&self.tree, &self.options, &self.source)
  }
  pub fn references(&self) -> Vec<ContentReference> {
    references::collect(&self.tree, &self.options, &self.source)
  }
  pub fn rewrite_references(
    &mut self,
    mut resolve: impl FnMut(&ContentReference) -> Result<String, String>,
  ) -> Result<(), String> {
    references::rewrite(&mut self.tree, &self.options, &self.source, &mut resolve)
  }
}

#[derive(Clone, Copy, Debug, Default, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ContentFormat {
  #[default]
  Auto,
  Markdown,
  Mdx,
}

pub struct CompiledContent {
  pub code: String,
  pub frontmatter: Value,
  pub toc: Vec<Heading>,
  pub source_map: Option<String>,
  pub diagnostics: Vec<String>,
}

/// Local, opt-in verification instrumentation; never includes paths or content.
struct VerifyProfile {
  operation: &'static str,
  started: std::time::Instant,
  phase_started: std::time::Instant,
  phase: &'static str,
  phases_ms: Vec<(&'static str, f64)>,
  source_bytes: usize,
  output_bytes: usize,
  source_map_bytes: usize,
  heading_count: usize,
  diagnostic_count: usize,
  completed: bool,
}

impl VerifyProfile {
  fn start(operation: &'static str, phase: &'static str, source_bytes: usize) -> Option<Self> {
    if std::env::var_os("FARM_DOCS_VERIFY_PROFILE").as_deref() != Some(std::ffi::OsStr::new("1")) {
      return None;
    }
    let started = std::time::Instant::now();
    Some(Self {
      operation,
      started,
      phase_started: started,
      phase,
      phases_ms: Vec::new(),
      source_bytes,
      output_bytes: 0,
      source_map_bytes: 0,
      heading_count: 0,
      diagnostic_count: 0,
      completed: false,
    })
  }

  fn mark(profile: &mut Option<Self>, phase: &'static str) {
    if let Some(profile) = profile {
      let now = std::time::Instant::now();
      profile.phases_ms.push((
        profile.phase,
        now.duration_since(profile.phase_started).as_secs_f64() * 1000.0,
      ));
      profile.phase_started = now;
      profile.phase = phase;
    }
  }
}

impl Drop for VerifyProfile {
  fn drop(&mut self) {
    use std::io::Write;
    let now = std::time::Instant::now();
    self.phases_ms.push((
      self.phase,
      now.duration_since(self.phase_started).as_secs_f64() * 1000.0,
    ));
    let record = serde_json::json!({
      "event": "farmDocsContentProfile",
      "operation": self.operation,
      "completed": self.completed,
      "totalMs": now.duration_since(self.started).as_secs_f64() * 1000.0,
      "phasesMs": self.phases_ms,
      "sourceBytes": self.source_bytes,
      "outputBytes": self.output_bytes,
      "sourceMapBytes": self.source_map_bytes,
      "headingCount": self.heading_count,
      "diagnosticCount": self.diagnostic_count,
    });
    if let Ok(bytes) = serde_json::to_vec(&record) {
      let mut stderr = std::io::stderr().lock();
      let _ = stderr.write_all(&bytes);
      let _ = stderr.write_all(b"\n");
    }
  }
}

/// Parse once; derive the body and metadata from the same syntax tree.
pub fn compile_content(source: &str, options: &Options) -> Result<CompiledContent, String> {
  compile_content_with_map(source, options, false)
}

pub fn compile_content_with_map(
  source: &str,
  options: &Options,
  emit_source_map: bool,
) -> Result<CompiledContent, String> {
  compile_content_with_features(source, options, emit_source_map, None)
}

pub fn compile_content_with_features(
  source: &str,
  options: &Options,
  emit_source_map: bool,
  highlight: Option<&HighlightOptions>,
) -> Result<CompiledContent, String> {
  compile_content_with_format(
    source,
    options,
    emit_source_map,
    highlight,
    ContentFormat::Auto,
  )
}

pub fn compile_content_with_format(
  source: &str,
  options: &Options,
  emit_source_map: bool,
  highlight: Option<&HighlightOptions>,
  format: ContentFormat,
) -> Result<CompiledContent, String> {
  emit_content(
    parse_content(source, options, highlight, format)?,
    emit_source_map,
  )
}

pub fn parse_content(
  source: &str,
  options: &Options,
  highlight: Option<&HighlightOptions>,
  format: ContentFormat,
) -> Result<ParsedContent, String> {
  let mut profile = VerifyProfile::start("parse", "preprocess", source.len());
  let allow_html_comments = options
    .filepath
    .as_deref()
    .is_some_and(|path| path.ends_with(".md"));
  let markdown_format = match format {
    ContentFormat::Auto => allow_html_comments,
    ContentFormat::Markdown => true,
    ContentFormat::Mdx => false,
  };
  let mut prepared = preprocess::prepare(
    source,
    allow_html_comments || markdown_format,
    markdown_format,
    options.filepath.as_deref(),
  )?;
  let mut options = options.clone();
  options.parse.constructs.gfm_table = true;
  options.parse.constructs.gfm_strikethrough = true;
  options.parse.constructs.gfm_task_list_item = true;
  VerifyProfile::mark(&mut profile, "mdastParse");
  let mut mdast = if markdown_format {
    markdown::to_mdast(&prepared.source, &markdown::ParseOptions::gfm())
      .map_err(|error| error.to_string())?
  } else {
    mdxjs::mdast_util_from_mdx(&prepared.source, &options).map_err(|error| error.to_string())?
  };
  VerifyProfile::mark(&mut profile, "restoreHtml");
  if markdown_format {
    preprocess::restore_raw_html(&mut mdast, source, &mut prepared.directives)?;
  }
  VerifyProfile::mark(&mut profile, "hastConversion");
  let mut tree = if markdown_format {
    markdown_html::to_hast(&mdast, &prepared.source)?
  } else {
    mdxjs::mdast_util_to_hast(&mdast)
  };
  VerifyProfile::mark(&mut profile, "highlight");
  let highlighted = if let Some(highlight) = highlight {
    code_blocks::apply(&mdast, &mut tree, highlight, &prepared.source)?
  } else {
    Default::default()
  };
  VerifyProfile::mark(&mut profile, "headingsAdmonitions");
  let toc = headings::annotate(&mut tree, &prepared.heading_ids)?;
  preprocess::wrap_admonitions(&mut tree, &prepared.directives);
  VerifyProfile::mark(&mut profile, "finish");
  let parsed = ParsedContent {
    frontmatter: prepared.frontmatter,
    metadata_locations: prepared.metadata_locations,
    toc,
    diagnostics: highlighted.diagnostics,
    tree,
    options,
    source: source.into(),
    prepared_source: prepared.source,
    resource_bindings: Default::default(),
    default_admonitions: preprocess::default_admonitions(&prepared.directives),
    static_code_blocks: highlighted.static_blocks,
  };
  if let Some(profile) = &mut profile {
    profile.heading_count = parsed.toc.len();
    profile.diagnostic_count = parsed.diagnostics.len();
    profile.completed = true;
  }
  Ok(parsed)
}

pub fn emit_content(
  parsed: ParsedContent,
  emit_source_map: bool,
) -> Result<CompiledContent, String> {
  let mut profile = VerifyProfile::start("emit", "hastToSwc", parsed.source.len());
  let ParsedContent {
    frontmatter: data,
    metadata_locations: _,
    toc,
    diagnostics,
    tree,
    options,
    source,
    prepared_source,
    resource_bindings,
    default_admonitions: _,
    static_code_blocks,
  } = parsed;
  let location = Location::new(prepared_source.as_bytes());
  let mut explicit_jsxs = Default::default();
  let mut program = mdxjs::hast_util_to_swc(&tree, &options, Some(&location), &mut explicit_jsxs)
    .map_err(|error| error.to_string())?;
  VerifyProfile::mark(&mut profile, "recmaStaticRewrite");
  mdxjs::mdx_plugin_recma_document(&mut program, &options, Some(&location))
    .map_err(|error| error.to_string())?;
  // The public rewrite helper also lowers JSX when options.jsx is false.
  // Attach the generated property before that lowering, in either mode.
  static_highlight::apply(&mut program.module, static_code_blocks)?;
  mdxjs::mdx_plugin_recma_jsx_rewrite(&mut program, &options, Some(&location), &explicit_jsxs)
    .map_err(|error| error.to_string())?;
  VerifyProfile::mark(&mut profile, "attributeResourceBindings");
  jsx_attributes::preserve(&mut program.module, &tree, |span| {
    explicit_jsxs.contains(&span)
  });
  resource_bindings::apply(&mut program.module, resource_bindings)?;
  VerifyProfile::mark(&mut profile, "serializeSourceMap");
  let frontmatter = serde_json::to_string(&data).map_err(|error| error.to_string())?;
  let headings = serde_json::to_string(&toc).map_err(|error| error.to_string())?;
  let (body, source_map) = if emit_source_map {
    let (code, map) = source_map::emit(
      &program.module,
      &program.comments,
      &source,
      options.filepath.as_deref().unwrap_or("document.mdx"),
    )?;
    (code, Some(map))
  } else {
    (program.serialize(), None)
  };
  let code =
    format!("{body}\nexport const frontmatter = {frontmatter};\nexport const toc = {headings};\n");
  let compiled = CompiledContent {
    code,
    frontmatter: data,
    toc,
    source_map,
    diagnostics,
  };
  if let Some(profile) = &mut profile {
    profile.output_bytes = compiled.code.len();
    profile.source_map_bytes = compiled.source_map.as_ref().map_or(0, String::len);
    profile.heading_count = compiled.toc.len();
    profile.diagnostic_count = compiled.diagnostics.len();
    profile.completed = true;
  }
  Ok(compiled)
}

fn element(
  name: &str,
  properties: Vec<(String, hast::PropertyValue)>,
  children: Vec<hast::Node>,
) -> hast::Node {
  hast::Node::Element(hast::Element {
    tag_name: name.into(),
    properties,
    children,
    position: None,
  })
}
