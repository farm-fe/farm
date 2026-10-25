use crate::metadata::{DiagnosticSource, MetadataSource};
use crate::*;
use farmfe_docs_config::{CollectionKind, ContentFormat, ValidatedConfig};
use farmfe_docs_content::{
  ContentReference, HighlightOptions, ParsedContent, ReferenceKind, emit_content, parse_content,
};
use std::{
  collections::HashMap,
  fs,
  path::{Path, PathBuf},
  sync::Arc,
  time::{Duration, Instant},
};

pub(crate) fn highlight_options(config: &ValidatedConfig) -> HighlightOptions {
  HighlightOptions {
    light: config.code.light,
    dark: config.code.dark,
    tokens: config.code.tokens.clone(),
    representation: match config.build.highlight_representation {
      farmfe_docs_config::HighlightRepresentation::Jsx => {
        farmfe_docs_content::HighlightRepresentation::Jsx
      }
      farmfe_docs_config::HighlightRepresentation::StaticHtml => {
        farmfe_docs_content::HighlightRepresentation::StaticHtml
      }
    },
    ..HighlightOptions::default()
  }
}

pub fn farm_path(path: &Path) -> Result<String, String> {
  let path = path
    .to_str()
    .ok_or("[DOCS_CONTENT_PATH] paths must be UTF-8")?
    .replace('\\', "/");
  if let Some(path) = path.strip_prefix("//?/UNC/") {
    return Ok(format!("//{path}"));
  }
  Ok(path.strip_prefix("//?/").unwrap_or(&path).to_string())
}

fn collect(directory: &Path, files: &mut Vec<PathBuf>, scripts: bool) -> Result<(), String> {
  for entry in fs::read_dir(directory)
    .map_err(|error| format!("[DOCS_CONTENT_READ] {}: {error}", directory.display()))?
  {
    let entry = entry.map_err(|error| error.to_string())?;
    let kind = entry.file_type().map_err(|error| error.to_string())?;
    if scripts && entry.file_name().to_string_lossy().starts_with('_') {
      continue;
    }
    if kind.is_symlink() {
      return Err(format!("[DOCS_CONTENT_SYMLINK] {}", entry.path().display()));
    }
    if kind.is_dir() {
      if !entry
        .file_name()
        .to_string_lossy()
        .eq_ignore_ascii_case("node_modules")
        && !farmfe_docs_config::is_private_tooling_component(&entry.file_name().to_string_lossy())
      {
        collect(&entry.path(), files, scripts)?;
      }
    } else if kind.is_file()
      && entry.path().extension().is_some_and(|extension| {
        extension == "md"
          || extension == "mdx"
          || (scripts && matches!(extension.to_str(), Some("js" | "jsx" | "ts" | "tsx")))
      })
    {
      files.push(entry.path());
    }
  }
  Ok(())
}
pub(crate) fn content_listing(
  directory: &Path,
  scripts: bool,
) -> Result<(String, Vec<PathBuf>), String> {
  let mut files = vec![];
  collect(directory, &mut files, scripts)?;
  files.sort();
  let entries = files
    .iter()
    .map(|file| {
      let relative = file
        .strip_prefix(directory)
        .map_err(|error| error.to_string())?;
      farm_path(relative)
    })
    .collect::<Result<Vec<_>, String>>()?;
  let bytes = serde_json::to_vec(&entries).map_err(|error| error.to_string())?;
  Ok((hash(&bytes), files))
}

fn text_field(
  frontmatter: &serde_json::Value,
  field: &str,
  file: &MetadataSource<'_>,
) -> Result<Option<String>, String> {
  match frontmatter.get(field) {
    None | Some(serde_json::Value::Null) => Ok(None),
    Some(serde_json::Value::String(value)) if !value.trim().is_empty() => Ok(Some(value.clone())),
    _ => Err(format!(
      "[DOCS_FRONTMATTER] {}: {field} must be a nonempty string",
      file.at(field)
    )),
  }
}
fn localized_page_title(
  frontmatter: &serde_json::Value,
  locale: &str,
  locales: &[String],
  file: &MetadataSource<'_>,
) -> Result<Option<String>, String> {
  let Some(value) = frontmatter.get("titleByLocale") else {
    return Ok(None);
  };
  let invalid = |field: &str| {
    format!(
      "[DOCS_PAGE_METADATA] {}: titleByLocale requires declared locales and nonempty string titles",
      file.at(field)
    )
  };
  let values = value.as_object().ok_or_else(|| invalid("titleByLocale"))?;
  for (key, value) in values {
    if !locales.contains(key)
      || !value
        .as_str()
        .is_some_and(|text| !text.trim().is_empty() && !text.chars().any(char::is_control))
    {
      return Err(invalid(&format!("titleByLocale.{key}")));
    }
  }
  Ok(
    values
      .get(locale)
      .and_then(|value| value.as_str())
      .map(str::to_owned),
  )
}
pub(crate) fn safe_slug(value: &str) -> Result<String, String> {
  if value.starts_with("//")
    || value.contains(['%', ':', '?', '#', '\\'])
    || value
      .chars()
      .any(|ch| ch.is_control() || ch.is_whitespace())
    || value.split('/').any(|part| part == "." || part == "..")
  {
    return Err(format!(
      "[DOCS_ROUTE_PATH] unsafe or unsupported slug {value}"
    ));
  }
  Ok(value.trim_matches('/').into())
}
pub fn prepare(config: &ValidatedConfig) -> Result<DocPlan, String> {
  prepare_with_cache(config, &mut ContentCache::transient())
}

/// Reuse only unchanged original content; every derived plan is validated again.
pub fn prepare_with_cache(
  config: &ValidatedConfig,
  cache: &mut ContentCache,
) -> Result<DocPlan, String> {
  cache.begin();
  let result = prepare_inner(config, cache);
  if result.is_ok() {
    cache.commit();
  }
  cache.report_profile(result.is_ok());
  result
}

/// Earlier route consumers need private mutable trees. The final consumer can
/// move the original if no persistent cache or other collection still owns it.
fn take_document(
  documents: &mut HashMap<(String, String), Arc<ParsedContent>>,
  consumers: &mut HashMap<(String, String), usize>,
  key: &(String, String),
) -> Result<ParsedContent, String> {
  let remaining = consumers
    .get_mut(key)
    .ok_or("[DOCS_INDEX] missing content consumers")?;
  if *remaining == 0 {
    return Err("[DOCS_INDEX] exhausted content consumers".into());
  }
  *remaining -= 1;
  if *remaining == 0 {
    documents
      .remove(key)
      .map(Arc::unwrap_or_clone)
      .ok_or_else(|| "[DOCS_INDEX] missing parsed content".into())
  } else {
    documents
      .get(key)
      .map(|document| document.as_ref().clone())
      .ok_or_else(|| "[DOCS_INDEX] missing parsed content".into())
  }
}

/// Local aggregate only: no document paths, payloads, global state or per-page output.
struct PrepareProfile {
  started: Option<Instant>,
  stages: std::collections::BTreeMap<&'static str, (Duration, usize, usize)>,
  active: Option<&'static str>,
  complete: bool,
}
impl PrepareProfile {
  fn new() -> Self {
    Self {
      started: std::env::var("FARM_DOCS_VERIFY_PROFILE")
        .is_ok_and(|value| value == "1")
        .then(Instant::now),
      stages: Default::default(),
      active: None,
      complete: false,
    }
  }
  fn start(&mut self, stage: &'static str) -> Option<Instant> {
    self.started.map(|_| {
      self.active = Some(stage);
      Instant::now()
    })
  }
  fn end(&mut self, stage: &'static str, started: Option<Instant>, count: usize, bytes: usize) {
    if let Some(started) = started {
      let metric = self.stages.entry(stage).or_default();
      metric.0 += started.elapsed();
      metric.1 = metric.1.saturating_add(count);
      metric.2 = metric.2.saturating_add(bytes);
      self.active = None;
    }
  }
}
impl Drop for PrepareProfile {
  fn drop(&mut self) {
    if let Some(started) = self.started {
      let stages = self
        .stages
        .iter()
        .map(|(stage, (elapsed, count, bytes))| {
          serde_json::json!({"stage": stage, "milliseconds": elapsed.as_secs_f64() * 1000.0,
          "count": count, "bytes": bytes})
        })
        .collect::<Vec<_>>();
      eprintln!(
        "[FarmDocsPrepareProfile] {}",
        serde_json::json!({
          "component": "prepare", "complete": self.complete, "interruptedStage": self.active,
          "totalMilliseconds": started.elapsed().as_secs_f64() * 1000.0,
          "stages": stages, "finalSealInsidePrepare": false,
          "scope": "one serial prepare; cache lookup/parse reported separately; zero bytes means uncounted"
        })
      );
    }
  }
}

fn prepare_inner(config: &ValidatedConfig, cache: &mut ContentCache) -> Result<DocPlan, String> {
  let mut profile = PrepareProfile::new();
  let started = profile.start("static_scan_inclusive");
  let (static_files, static_inputs) = crate::static_files::scan(config)?;
  profile.end("static_scan_inclusive", started, static_files.len(), 0);
  let mut assets = crate::assets::Registry::with_static(&static_files);
  let mut plan = DocPlan {
    schema_version: 2,
    snapshot_id: String::new(),
    ssg: config.ssg.clone(),
    site: config.public.clone(),
    seo: Default::default(),
    not_found: vec![],
    routes: vec![],
    modules: vec![],
    styles: config
      .custom_css
      .iter()
      .map(|path| farm_path(path))
      .collect::<Result<_, _>>()?,
    theme_components: config
      .theme_components
      .as_deref()
      .map(farm_path)
      .transpose()?,
    diagnostics: vec![],
    public_diagnostics: vec![],
    sidebars: vec![],
    inputs: vec![],
    content_inputs: vec![],
    assets: vec![],
    static_files,
    static_inputs,
    blog_indexes: vec![],
  };
  for file in &plan.static_files {
    plan.inputs.push(PlanInput {
      path: file.source_path.clone(),
      hash: file.source_hash.clone(),
    });
  }
  if let Some(logo) = &mut plan.site.theme.logo {
    let reference = ContentReference {
      kind: ReferenceKind::Image,
      target: logo.src.clone(),
      file: "theme.logo.src".into(),
      line: 1,
      column: 1,
      snippet: "theme.logo.src".into(),
    };
    logo.src = assets.metadata_url(config, &reference)?;
  }
  let highlighting = highlight_options(config);
  let highlight_identity =
    serde_json::to_string(&highlighting).map_err(|e| format!("[DOCS_CODE_CONFIG] {e}"))?;
  let mut urls = HashMap::new();
  let mut documents = HashMap::new();
  for collection in &config.collections {
    let mut ids = HashMap::new();
    let scripts = collection.public.kind == CollectionKind::Page;
    let started = profile.start("content_listing_scan");
    let (entries_hash, files) = content_listing(&collection.directory, scripts)?;
    profile.end("content_listing_scan", started, files.len(), 0);
    plan.content_inputs.push(ContentDirectoryInput {
      path: farm_path(&collection.directory)?,
      scripts,
      entries_hash,
    });
    for file in files {
      if collection.excluded_files.contains(&file) {
        continue;
      }
      let relative = file
        .strip_prefix(&collection.directory)
        .map_err(|error| error.to_string())?;
      let mut stem = relative.to_path_buf();
      stem.set_extension("");
      let id = farm_path(&stem)?;
      let is_blog = collection.public.kind == CollectionKind::Blog;
      let (blog_slug, filename_date) = if is_blog {
        crate::blog::filename(&id).map_err(|error| format!("{error} ({})", file.display()))?
      } else {
        (String::new(), None)
      };
      let default_slug = if is_blog {
        blog_slug
      } else if stem.file_name().is_some_and(|name| name == "index") {
        farm_path(stem.parent().unwrap_or(Path::new("")))?
      } else {
        id.clone()
      };
      let source_path = farm_path(&file)?;
      let started = profile.start("source_read");
      let bytes =
        fs::read(&file).map_err(|error| format!("[DOCS_CONTENT_READ] {source_path}: {error}"))?;
      profile.end("source_read", started, 1, bytes.len());
      let started = profile.start("source_hash");
      let source_hash = hash(&bytes);
      profile.end("source_hash", started, 1, bytes.len());
      let source = String::from_utf8(bytes)
        .map_err(|error| format!("[DOCS_ENCODING] {source_path}: {error}"))?;
      let extension = file
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("");
      let is_script = collection.public.kind == CollectionKind::Page
        && matches!(extension, "js" | "jsx" | "ts" | "tsx");
      let started = profile.start("script_page_metadata_inclusive");
      let script = if is_script {
        crate::script_pages::parse(
          &source,
          &source_path,
          extension,
          &config.public.locales.items,
        )?
      } else {
        None
      };
      profile.end(
        "script_page_metadata_inclusive",
        started,
        usize::from(is_script),
        0,
      );
      if is_script && script.is_none() {
        continue;
      }
      let format = match collection.format {
        ContentFormat::Auto => farmfe_docs_content::ContentFormat::Auto,
        ContentFormat::Markdown => farmfe_docs_content::ContentFormat::Markdown,
        ContentFormat::Mdx => farmfe_docs_content::ContentFormat::Mdx,
      };
      let compiled = if is_script {
        None
      } else {
        Some(
          cache
            .shared_parse_with_options(
              &source_path,
              format,
              &source_hash,
              &highlight_identity,
              || {
                parse_content(
                  &source,
                  &mdxjs::Options {
                    filepath: Some(source_path.clone()),
                    jsx: true,
                    ..Default::default()
                  },
                  Some(&highlighting),
                  format,
                )
              },
            )
            .map_err(|error| format!("[DOCS_CONTENT_COMPILE] {source_path}: {error}"))?,
        )
      };
      let started = profile.start("route_metadata");
      let frontmatter = script
        .as_ref()
        .map(|script| script.metadata.clone())
        .or_else(|| compiled.as_ref().map(|content| content.frontmatter.clone()))
        .ok_or("[DOCS_INDEX] missing metadata")?;
      let metadata_source = MetadataSource {
        file: &source_path,
        content: compiled.as_deref(),
      };
      let toc = compiled
        .as_ref()
        .map(|content| content.toc.as_slice())
        .unwrap_or(&[]);
      let id = text_field(&frontmatter, "id", &metadata_source)?.unwrap_or(id);
      if is_blog && id.starts_with("@blog/") {
        return Err(format!(
          "[DOCS_BLOG_ID] {}: @blog/ is reserved for generated views",
          metadata_source.at("id")
        ));
      }
      let canonical_id = routes::logical_id(&id);
      if let Some(previous) = ids.insert(canonical_id.clone(), source_path.clone()) {
        return Err(format!(
          "[DOCS_CONTENT_ID] duplicate id {id:?} in collection {}: {previous} and {source_path}",
          collection.public.id
        ));
      }
      let slug =
        safe_slug(&text_field(&frontmatter, "slug", &metadata_source)?.unwrap_or(default_slug))
          .map_err(|error| format!("{error} ({})", metadata_source.at("slug")))?;
      if is_blog {
        crate::blog::validate_slug(&slug, &metadata_source.at("slug"))?;
      }
      let mut blog = if is_blog {
        Some(crate::blog::metadata(
          &frontmatter,
          filename_date,
          &metadata_source,
        )?)
      } else {
        None
      };
      if let Some(post) = &mut blog {
        for (index, author) in post.authors.iter_mut().enumerate() {
          if let Some(image) = &mut author.image {
            let field = if frontmatter
              .get("authors")
              .is_some_and(serde_json::Value::is_array)
            {
              format!("authors[{index}].image")
            } else {
              "authors.image".into()
            };
            let position = compiled
              .as_ref()
              .and_then(|content| content.metadata_locations.position(&field));
            let line = position.map(|point| point.line).unwrap_or(1);
            let column = position.map(|point| point.column).unwrap_or(1);
            let reference = ContentReference {
              kind: ReferenceKind::Image,
              target: image.clone(),
              file: source_path.clone(),
              line,
              column,
              snippet: source.lines().nth(line - 1).unwrap_or("").into(),
            };
            *image = assets.metadata_url(config, &reference)?;
          }
        }
      }
      let suffix = format!(
        "{}{slug}",
        collection.public.route_base.trim_start_matches('/')
      );
      let route_path = format!(
        "{}{}{}",
        config.public.base_path,
        suffix,
        if suffix.is_empty() || suffix.ends_with('/') {
          ""
        } else {
          "/"
        }
      );
      if let Some(previous) = urls.insert(routes::collision_key(&route_path), source_path.clone()) {
        return Err(format!(
          "[DOCS_ROUTE_COLLISION] {route_path}: {previous} and {source_path}"
        ));
      }
      let relative_output = route_path
        .strip_prefix(&config.public.base_path)
        .ok_or("[DOCS_ROUTE_PATH] base path mismatch")?;
      let output_path = format!("{relative_output}index.html");
      routes::validate_output(&output_path, &metadata_source.at("slug"))?;
      let key = routes::page_key(&collection.public, &canonical_id)?;
      let default_title = text_field(&frontmatter, "title", &metadata_source)?;
      let localized_title = if collection.public.kind == CollectionKind::Page {
        localized_page_title(
          &frontmatter,
          &collection.public.locale,
          &config.public.locales.items,
          &metadata_source,
        )?
      } else {
        None
      };
      let title = localized_title
        .or(default_title)
        .or_else(|| {
          toc
            .iter()
            .find(|heading| heading.depth == 1)
            .map(|heading| heading.title.clone())
        })
        .unwrap_or_else(|| id.clone());
      let description = text_field(&frontmatter, "description", &metadata_source)?;
      let headings = toc
        .iter()
        .map(|heading| PublicHeading {
          id: heading.id.clone(),
          title: heading.title.clone(),
          depth: heading.depth,
        })
        .collect();
      if let Some(content) = &compiled {
        plan.diagnostics.extend(content.diagnostics.clone());
        plan.public_diagnostics.extend(content.diagnostics.clone());
      }
      let layout = if collection.public.kind == CollectionKind::Page {
        frontmatter
          .get("layout")
          .filter(|value| !value.is_null())
          .map(|value| {
            serde_json::from_value::<PageLayout>(value.clone()).map_err(|cause| {
              format!(
                "[DOCS_FRONTMATTER] {}: layout: {cause}",
                metadata_source.at("layout")
              )
            })
          })
          .transpose()?
      } else {
        None
      };
      plan.routes.push(RenderRoute {
        page: PublicPage {
          markdown_url: None,
          markdown_sha256: None,
          key: key.clone(),
          id,
          collection: collection.public.id.clone(),
          kind: if is_blog {
            "blog"
          } else if collection.public.kind == CollectionKind::Page {
            "page"
          } else {
            "doc"
          }
          .into(),
          locale: collection.public.locale.clone(),
          version: collection.public.version.clone(),
          canonical_path: route_path.clone(),
          route_path,
          title,
          description,
          headings,
          alternates: vec![],
          fallback_source: None,
          previous: None,
          next: None,
          blog,
          blog_view: None,
          layout,
        },
        output_path,
      });
      plan.modules.push(PreparedModule {
        machine_markdown: None,
        key,
        source_path: source_path.clone(),
        source_hash,
        code: if is_script {
          source.clone()
        } else {
          String::new()
        }
        .into(),
        source_map: None,
        generated_source: None,
        language: script
          .as_ref()
          .map(|script| script.language)
          .unwrap_or_default(),
        static_anchors: script
          .as_ref()
          .map(|script| script.anchors.clone())
          .unwrap_or_default(),
      });
      if let Some(compiled) = compiled {
        documents.insert((source_path, collection.public.id.clone()), compiled);
      }
      profile.end("route_metadata", started, 1, 0);
    }
  }
  let started = profile.start("route_index_translation_blog");
  crate::translations::expand(config, &mut plan)?;
  crate::blog::index(config, &mut plan)?;
  crate::blog_views::generate(config, &mut plan, &mut documents)?;
  routes::apply_policy(config, &mut plan)?;
  if plan.routes.is_empty() {
    return Err("[DOCS_CONTENT_EMPTY] no documents found".into());
  }
  crate::translations::connect(config, &mut plan)?;
  profile.end(
    "route_index_translation_blog",
    started,
    plan.routes.len(),
    0,
  );
  let started = profile.start("sidebar");
  crate::sidebars::apply(config, &mut plan)?;
  profile.end("sidebar", started, plan.sidebars.len(), 0);
  let started = profile.start("link_index");
  let link_index = crate::links::Index::new(config, &plan, &documents)?;
  let brand_href = crate::brand_home::resolve_logo(&plan, &link_index)?;
  let mut exceptions = crate::link_exceptions::Registry::new(config, &plan)?;
  profile.end("link_index", started, plan.routes.len(), 0);
  let started = profile.start("consumer_index_and_cache_release");
  let mut emitted = vec![];
  let mut machine_documents = vec![];
  let mut consumers = HashMap::new();
  for (route, module) in plan.routes.iter().zip(&plan.modules) {
    if module.language == ContentLanguage::Markdown {
      let key = (
        module.source_path.clone(),
        route
          .page
          .fallback_source
          .as_ref()
          .map(|source| source.collection.clone())
          .unwrap_or_else(|| route.page.collection.clone()),
      );
      *consumers.entry(key).or_insert(0usize) += 1;
    }
  }
  // The link index has already captured anchors. A one-shot cache no longer
  // needs to retain an extra owner; persistent development caches still do.
  cache.release_transient_originals();
  profile.end(
    "consumer_index_and_cache_release",
    started,
    consumers.len(),
    0,
  );
  for (route, module) in plan.routes.iter().zip(&plan.modules) {
    if module.language != ContentLanguage::Markdown {
      emitted.push(None);
      machine_documents.push(None);
      continue;
    }
    let document_key = (
      module.source_path.clone(),
      route
        .page
        .fallback_source
        .as_ref()
        .map(|source| source.collection.clone())
        .unwrap_or_else(|| route.page.collection.clone()),
    );
    let started = profile.start("ast_ownership");
    let mut document = take_document(&mut documents, &mut consumers, &document_key)?;
    profile.end("ast_ownership", started, 1, 0);
    let started = profile.start("link_binding_and_labels");
    let collection = config
      .collections
      .iter()
      .find(|collection| collection.public.id == route.page.collection)
      .ok_or("[DOCS_INDEX] missing final content collection")?;
    document.apply_admonition_labels(&collection.admonition_labels);
    let mut bindings = vec![];
    document.rewrite_references(|reference| {
      let href = exceptions.resolve(&link_index, reference, &route.page)?;
      if link_index.is_resource_reference(reference, &href) {
        let asset = assets.resolve(config, reference)?;
        bindings.push((
          asset.url.clone(),
          format!("docs:asset:{}", asset.key),
          asset.suffix,
        ));
        return Ok(asset.url);
      }
      Ok(href)
    })?;
    for (url, source, suffix) in bindings {
      document.bind_resource(url, source, suffix);
    }
    profile.end("link_binding_and_labels", started, 1, 0);
    if route.page.kind == "doc" && config.public.ai.as_ref().is_some_and(|ai| ai.enabled) {
      let started = profile.start("ai_semantic_export");
      let body = document
        .machine_markdown()
        .map_err(|error| format!("{error} (document {})", route.page.id))?;
      let markdown = crate::machine_readable::document(&config.public, &route.page, &body)?;
      let url = crate::machine_readable::markdown_url(&config.public, &route.output_path)?;
      let bytes = markdown.len();
      machine_documents.push(Some((url, hash(markdown.as_bytes()), markdown)));
      profile.end("ai_semantic_export", started, 1, bytes);
    } else {
      machine_documents.push(None);
    }
    let started = profile.start("ast_emit_inclusive");
    let content = emit_content(document, true)?;
    let bytes = content.code.len() + content.source_map.as_ref().map_or(0, |map| map.len());
    emitted.push(Some(content));
    profile.end("ast_emit_inclusive", started, 1, bytes);
  }
  let started = profile.start("output_assembly_validation");
  let exception_diagnostics = exceptions.finish()?;
  for ((route, module), machine) in plan
    .routes
    .iter_mut()
    .zip(&mut plan.modules)
    .zip(machine_documents)
  {
    if let Some((url, digest, markdown)) = machine {
      route.page.markdown_url = Some(url);
      route.page.markdown_sha256 = Some(digest);
      module.machine_markdown = Some(markdown.into());
    }
  }
  for asset in assets.files.into_values() {
    let output = routes::collision_key(&asset.output_path);
    if plan.routes.iter().any(|route| {
      let page = routes::collision_key(&route.output_path);
      page == output
        || page.starts_with(&format!("{output}/"))
        || output.starts_with(&format!("{page}/"))
    }) {
      return Err(format!(
        "[DOCS_ASSET_COLLISION] {} conflicts with a page output",
        asset.output_path
      ));
    }
    if let Some(input) = plan
      .inputs
      .iter()
      .find(|input| input.path == asset.source_path)
    {
      if input.hash != asset.source_hash {
        return Err(format!("[DOCS_SNAPSHOT_CHANGED] {}", asset.source_path));
      }
    } else {
      plan.inputs.push(PlanInput {
        path: asset.source_path.clone(),
        hash: asset.source_hash.clone(),
      });
    }
    plan.assets.push(asset);
  }
  let mut output_names: Vec<_> = plan
    .routes
    .iter()
    .map(|route| route.output_path.clone())
    .collect();
  output_names.extend(
    crate::static_files::reserved(&plan.site)
      .iter()
      .map(|name| name.to_string()),
  );
  output_names.extend(
    plan
      .static_files
      .iter()
      .map(|file| file.output_path.clone()),
  );
  output_names.extend(crate::machine_readable::markdown_paths(&plan)?);
  output_names.extend(
    plan
      .assets
      .iter()
      .filter(|file| !file.is_static)
      .map(|file| file.output_path.clone()),
  );
  crate::static_files::validate_names(&output_names)?;
  for (module, content) in plan.modules.iter_mut().zip(emitted) {
    if let Some(content) = content {
      module.code = content.code.into();
      module.source_map = content.source_map.map(Into::into);
    }
  }
  plan.public_diagnostics.extend(exception_diagnostics.public);
  plan.diagnostics.extend(exception_diagnostics.private);
  if let (Some(logo), Some(href)) = (&mut plan.site.theme.logo, brand_href) {
    logo.href = href;
  }
  profile.end("output_assembly_validation", started, plan.modules.len(), 0);
  let started = profile.start("syndication_seo_not_found");
  crate::syndication::validate(&plan)?;
  plan.seo = crate::seo::heads(&plan)?;
  plan.not_found = crate::not_found::plan(&plan)?;
  profile.end("syndication_seo_not_found", started, plan.routes.len(), 0);
  let started = profile.start("plan_identity");
  plan.snapshot_id = identity(&plan)?;
  profile.end("plan_identity", started, plan.modules.len(), 0);
  profile.complete = true;
  Ok(plan)
}

#[cfg(test)]
mod ownership_tests {
  use super::*;

  fn document() -> Arc<ParsedContent> {
    Arc::new(
      parse_content(
        "# Original\n\n:::note\nBody\n:::\n",
        &mdxjs::Options::default(),
        None,
        farmfe_docs_content::ContentFormat::Mdx,
      )
      .unwrap(),
    )
  }

  #[test]
  fn last_single_consumer_moves_original_heading_allocation() {
    let key = ("/source.md".into(), "en".into());
    let original = document();
    let allocation = original.toc.as_ptr();
    let mut documents = [(key.clone(), original)].into();
    let mut consumers = [(key.clone(), 1)].into();
    let owned = take_document(&mut documents, &mut consumers, &key).unwrap();
    assert_eq!(
      allocation,
      owned.toc.as_ptr(),
      "single cold consumer must not deep-clone"
    );
    assert!(documents.is_empty());
    assert!(take_document(&mut documents, &mut consumers, &key).is_err());
  }

  #[test]
  fn fallback_consumers_are_private_and_the_last_moves_the_original() {
    let key = ("/source.md".into(), "en".into());
    let original = document();
    let allocation = original.toc.as_ptr();
    let mut documents = [(key.clone(), original)].into();
    let mut consumers = [(key.clone(), 2)].into();
    let mut first = take_document(&mut documents, &mut consumers, &key).unwrap();
    assert_ne!(allocation, first.toc.as_ptr());
    first.apply_admonition_labels(&[("note".into(), "First locale".into())].into());
    let mut last = take_document(&mut documents, &mut consumers, &key).unwrap();
    assert_eq!(allocation, last.toc.as_ptr());
    last.apply_admonition_labels(&[("note".into(), "Last locale".into())].into());
    let first_code = emit_content(first, false).unwrap().code;
    let last_code = emit_content(last, false).unwrap().code;
    assert!(first_code.contains("First locale") && !first_code.contains("Last locale"));
    assert!(last_code.contains("Last locale") && !last_code.contains("First locale"));
  }

  #[test]
  fn cold_single_source_parses_once_and_matches_persistent_preparation() {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir(root.path().join("docs")).unwrap();
    fs::write(
      root.path().join("docs/index.md"),
      "# One\n\n:::note\nBody\n:::\n",
    )
    .unwrap();
    let config = farmfe_docs_config::parse_config(
      &serde_json::json!({
        "title":"Ownership", "url":"https://example.com", "docs":{"collections":[
          {"id":"main", "dir":"docs", "routeBase":"/docs/"}
        ]}
      })
      .to_string(),
      root.path(),
    )
    .unwrap();
    let mut cold_cache = ContentCache::transient();
    let cold = prepare_with_cache(&config, &mut cold_cache).unwrap();
    assert_eq!(cold_cache.parse_calls(), 1);
    let mut warm_cache = ContentCache::default();
    for _ in 0..2 {
      let warm = prepare_with_cache(&config, &mut warm_cache).unwrap();
      assert_eq!(
        serde_json::to_value(&cold).unwrap(),
        serde_json::to_value(&warm).unwrap()
      );
    }
    assert_eq!(warm_cache.parse_calls(), 1);
  }

  #[test]
  fn shared_source_and_fallback_collections_parse_once_and_keep_rewrites_private() {
    for fallback in [false, true] {
      let root = tempfile::tempdir().unwrap();
      fs::create_dir(root.path().join("docs")).unwrap();
      fs::write(
        root.path().join("docs/index.md"),
        "# Index\n\n:::note\nBody\n:::\n\n[Target](./target.md)\n",
      )
      .unwrap();
      fs::write(root.path().join("docs/target.md"), "# Target\n").unwrap();
      let mut raw = serde_json::json!({
        "title":"Ownership", "url":"https://example.com", "locales":{"default":"en","items":["en","zh"]},
        "docs":{"collections":[
          {"id":"en","group":"docs","locale":"en","dir":"docs","routeBase":"/docs/","admonitionLabels":{"note":"English label"}},
          {"id":"zh","group":"docs","locale":"zh","dir":"docs","routeBase":"/zh/docs/","admonitionLabels":{"note":"Chinese label"}}
        ]}
      });
      if fallback {
        fs::create_dir(root.path().join("zh")).unwrap();
        raw["docs"]["collections"][1]["dir"] = serde_json::json!("zh");
        raw["docs"]["collections"][1]["fallbackLocale"] = serde_json::json!("en");
      }
      let config = farmfe_docs_config::parse_config(&raw.to_string(), root.path()).unwrap();
      let mut cold_cache = ContentCache::transient();
      let cold = prepare_with_cache(&config, &mut cold_cache).unwrap();
      assert_eq!(cold.routes.len(), 4);
      assert_eq!(
        cold_cache.parse_calls(),
        2,
        "same paths/options shared across two collections"
      );
      let mut warm_cache = ContentCache::default();
      for _ in 0..2 {
        let warm = prepare_with_cache(&config, &mut warm_cache).unwrap();
        assert_eq!(
          serde_json::to_value(&cold).unwrap(),
          serde_json::to_value(&warm).unwrap()
        );
        for route in warm.routes.iter().filter(|route| route.page.id == "index") {
          let module = warm
            .modules
            .iter()
            .find(|module| module.key == route.page.key)
            .unwrap();
          let (label, forbidden, target) = if route.page.locale == "en" {
            ("English label", "Chinese label", "/docs/target")
          } else {
            ("Chinese label", "English label", "/zh/docs/target")
          };
          assert!(module.code.contains(label) && !module.code.contains(forbidden));
          assert!(
            module.code.contains(target),
            "reference rewritten for final collection"
          );
        }
      }
      assert_eq!(
        warm_cache.parse_calls(),
        2,
        "unchanged warm generations retain originals"
      );
    }
  }
}
