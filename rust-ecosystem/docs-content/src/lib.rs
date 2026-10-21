//! Content semantics shared by native Docs and MDX plugins, without plugin ABI exports.
mod code_blocks;
mod comments;
mod headings;
mod jsx_attributes;
mod markdown_html;
mod metadata;
mod preprocess;
mod references;
mod resource_bindings;
mod source_map;

use markdown::Location;
use mdxjs::{Options, hast};
use serde_json::Value;

pub use farmfe_syntax_highlight::HighlightOptions;
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
}
impl ParsedContent {
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
  let mut mdast = if markdown_format {
    markdown::to_mdast(&prepared.source, &markdown::ParseOptions::gfm())
      .map_err(|error| error.to_string())?
  } else {
    mdxjs::mdast_util_from_mdx(&prepared.source, &options).map_err(|error| error.to_string())?
  };
  if markdown_format {
    preprocess::restore_raw_html(&mut mdast, source, &mut prepared.directives)?;
  }
  let mut tree = if markdown_format {
    markdown_html::to_hast(&mdast, &prepared.source)?
  } else {
    mdxjs::mdast_util_to_hast(&mdast)
  };
  let diagnostics = if let Some(highlight) = highlight {
    code_blocks::apply(&mdast, &mut tree, highlight, &prepared.source)?
  } else {
    vec![]
  };
  let toc = headings::annotate(&mut tree, &prepared.heading_ids)?;
  preprocess::wrap_admonitions(&mut tree, &prepared.directives);
  Ok(ParsedContent {
    frontmatter: prepared.frontmatter,
    metadata_locations: prepared.metadata_locations,
    toc,
    diagnostics,
    tree,
    options,
    source: source.into(),
    prepared_source: prepared.source,
    resource_bindings: Default::default(),
    default_admonitions: preprocess::default_admonitions(&prepared.directives),
  })
}

pub fn emit_content(
  parsed: ParsedContent,
  emit_source_map: bool,
) -> Result<CompiledContent, String> {
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
  } = parsed;
  let location = Location::new(prepared_source.as_bytes());
  let mut explicit_jsxs = Default::default();
  let mut program = mdxjs::hast_util_to_swc(&tree, &options, Some(&location), &mut explicit_jsxs)
    .map_err(|error| error.to_string())?;
  mdxjs::mdx_plugin_recma_document(&mut program, &options, Some(&location))
    .map_err(|error| error.to_string())?;
  mdxjs::mdx_plugin_recma_jsx_rewrite(&mut program, &options, Some(&location), &explicit_jsxs)
    .map_err(|error| error.to_string())?;
  jsx_attributes::preserve(&mut program.module, &tree, |span| {
    explicit_jsxs.contains(&span)
  });
  resource_bindings::apply(&mut program.module, resource_bindings)?;
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
  Ok(CompiledContent {
    code,
    frontmatter: data,
    toc,
    source_map,
    diagnostics,
  })
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
