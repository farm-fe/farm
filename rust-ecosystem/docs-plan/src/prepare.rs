use crate::metadata::{DiagnosticSource, MetadataSource};
use crate::*;
use farmfe_docs_config::{CollectionKind, ContentFormat, ValidatedConfig};
use farmfe_docs_content::{
  emit_content, parse_content, ContentReference, HighlightOptions, ReferenceKind,
};
use std::{
  collections::HashMap,
  fs,
  path::{Path, PathBuf},
};

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
  prepare_with_cache(config, &mut ContentCache::default())
}

/// Reuse only unchanged original content; every derived plan is validated again.
pub fn prepare_with_cache(
  config: &ValidatedConfig,
  cache: &mut ContentCache,
) -> Result<DocPlan, String> {
  cache.begin();
  let plan = prepare_inner(config, cache)?;
  cache.commit();
  Ok(plan)
}

fn prepare_inner(config: &ValidatedConfig, cache: &mut ContentCache) -> Result<DocPlan, String> {
  let (static_files, static_inputs) = crate::static_files::scan(config)?;
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
  let mut urls = HashMap::new();
  let mut documents = HashMap::new();
  for collection in &config.collections {
    let mut ids = HashMap::new();
    let scripts = collection.public.kind == CollectionKind::Page;
    let (entries_hash, files) = content_listing(&collection.directory, scripts)?;
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
      let bytes =
        fs::read(&file).map_err(|error| format!("[DOCS_CONTENT_READ] {source_path}: {error}"))?;
      let source_hash = hash(&bytes);
      let source = String::from_utf8(bytes)
        .map_err(|error| format!("[DOCS_ENCODING] {source_path}: {error}"))?;
      let extension = file
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("");
      let is_script = collection.public.kind == CollectionKind::Page
        && matches!(extension, "js" | "jsx" | "ts" | "tsx");
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
            .parse(&source_path, format, &source_hash, || {
              parse_content(
                &source,
                &mdxjs::Options {
                  filepath: Some(source_path.clone()),
                  jsx: true,
                  ..Default::default()
                },
                Some(&HighlightOptions::default()),
                format,
              )
            })
            .map_err(|error| format!("[DOCS_CONTENT_COMPILE] {source_path}: {error}"))?,
        )
      };
      let frontmatter = script
        .as_ref()
        .map(|script| script.metadata.clone())
        .or_else(|| compiled.as_ref().map(|content| content.frontmatter.clone()))
        .ok_or("[DOCS_INDEX] missing metadata")?;
      let metadata_source = MetadataSource {
        file: &source_path,
        content: compiled.as_ref(),
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
        key,
        source_path: source_path.clone(),
        source_hash,
        code: if is_script {
          source.clone()
        } else {
          String::new()
        },
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
    }
  }
  crate::translations::expand(config, &mut plan)?;
  crate::blog::index(config, &mut plan)?;
  crate::blog_views::generate(config, &mut plan, &mut documents)?;
  routes::apply_policy(config, &mut plan)?;
  if plan.routes.is_empty() {
    return Err("[DOCS_CONTENT_EMPTY] no documents found".into());
  }
  crate::translations::connect(config, &mut plan)?;
  crate::sidebars::apply(config, &mut plan)?;
  let link_index = crate::links::Index::new(config, &plan, &documents)?;
  let brand_href = crate::brand_home::resolve_logo(&plan, &link_index)?;
  let mut exceptions = crate::link_exceptions::Registry::new(config, &plan)?;
  let mut emitted = vec![];
  for (route, module) in plan.routes.iter().zip(&plan.modules) {
    if module.language != ContentLanguage::Markdown {
      emitted.push(None);
      continue;
    }
    let mut document = documents
      .get(&(
        module.source_path.clone(),
        route
          .page
          .fallback_source
          .as_ref()
          .map(|source| source.collection.clone())
          .unwrap_or_else(|| route.page.collection.clone()),
      ))
      .cloned()
      .ok_or("[DOCS_INDEX] missing parsed content")?;
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
    emitted.push(Some(emit_content(document, true)?));
  }
  let exception_diagnostics = exceptions.finish()?;
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
      module.code = content.code;
      module.source_map = content.source_map;
    }
  }
  plan.public_diagnostics.extend(exception_diagnostics.public);
  plan.diagnostics.extend(exception_diagnostics.private);
  if let (Some(logo), Some(href)) = (&mut plan.site.theme.logo, brand_href) {
    logo.href = href;
  }
  crate::syndication::validate(&plan)?;
  plan.seo = crate::seo::heads(&plan)?;
  plan.not_found = crate::not_found::plan(&plan)?;
  plan.snapshot_id = identity(&plan)?;
  Ok(plan)
}
