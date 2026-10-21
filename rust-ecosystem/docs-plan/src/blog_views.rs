use crate::*;
use farmfe_docs_config::{CollectionKind, ResolvedCollection, ValidatedConfig};
use farmfe_docs_content::{parse_content, ContentFormat, ParsedContent};
use std::collections::{HashMap, HashSet};

fn escape(value: &str) -> String {
  let mut output = String::new();
  for ch in value.chars() {
    if ch.is_ascii_punctuation() {
      output.push('\\');
    }
    if ch == '\n' || ch == '\r' {
      output.push(' ');
    } else {
      output.push(ch);
    }
  }
  output
}
fn post_list(
  keys: &[String],
  pages: &HashMap<String, PublicPage>,
  no_posts: &str,
) -> Result<String, String> {
  if keys.is_empty() {
    return Ok(format!("{}\n", escape(no_posts)));
  }
  let mut source = String::new();
  for key in keys {
    let page = pages
      .get(key)
      .ok_or("[DOCS_BLOG_VIEW] indexed article is absent")?;
    source.push_str(&format!(
      "- [{}](<{}>)",
      escape(&page.title),
      page.route_path
    ));
    if let Some(date) = page.blog.as_ref().and_then(|post| post.date.as_ref()) {
      source.push_str(&format!(" — {date}"));
    }
    source.push('\n');
  }
  Ok(source)
}

struct View {
  id: String,
  kind: &'static str,
  suffix: String,
  title: String,
  body: String,
  posts: Vec<String>,
  tag: Option<String>,
}
pub fn generate(
  config: &ValidatedConfig,
  plan: &mut DocPlan,
  documents: &mut HashMap<(String, String), ParsedContent>,
) -> Result<(), String> {
  let pages: HashMap<_, _> = plan
    .routes
    .iter()
    .map(|route| (route.page.key.clone(), route.page.clone()))
    .collect();
  let mut urls: HashSet<_> = plan
    .routes
    .iter()
    .map(|route| routes::collision_key(&route.page.route_path))
    .collect();
  for collection in config
    .collections
    .iter()
    .filter(|collection| collection.public.kind == CollectionKind::Blog)
  {
    let index = plan
      .blog_indexes
      .iter()
      .find(|index| index.collection == collection.public.id)
      .cloned()
      .ok_or("[DOCS_BLOG_VIEW] missing blog index")?;
    let root = format!(
      "{}{}",
      config.public.base_path,
      collection.public.route_base.trim_start_matches('/')
    );
    let mut archive = String::new();
    let labels = &collection.blog_labels;
    for year in &index.years {
      archive.push_str(&format!(
        "## {}\n\n{}\n",
        year.year,
        post_list(&year.posts, &pages, &labels.no_posts)?
      ));
    }
    if !index.undated.is_empty() {
      archive.push_str(&format!(
        "## {}\n\n{}",
        escape(&labels.undated),
        post_list(&index.undated, &pages, &labels.no_posts)?
      ));
    }
    if archive.is_empty() {
      archive = format!("{}\n", escape(&labels.no_posts));
    }
    let mut views = vec![
      View {
        id: "@blog/index".into(),
        kind: "blog-index",
        suffix: String::new(),
        title: labels.title.clone(),
        body: post_list(&index.posts, &pages, &labels.no_posts)?,
        posts: index.posts.clone(),
        tag: None,
      },
      View {
        id: "@blog/archive".into(),
        kind: "blog-archive",
        suffix: "archive/".into(),
        title: labels.archive.clone(),
        body: archive,
        posts: index.posts.clone(),
        tag: None,
      },
    ];
    if !index.tags.is_empty() {
      let body = index
        .tags
        .iter()
        .map(|tag| format!("- [{}](<{}tags/{}/>)\n", escape(&tag.label), root, tag.id))
        .collect::<String>();
      views.push(View {
        id: "@blog/tags".into(),
        kind: "blog-tags",
        suffix: "tags/".into(),
        title: labels.tags.clone(),
        body,
        posts: index.posts.clone(),
        tag: None,
      });
      for tag in &index.tags {
        views.push(View {
          id: format!("@blog/tag/{}", tag.id),
          kind: "blog-tag",
          suffix: format!("tags/{}/", tag.id),
          title: tag.label.clone(),
          body: post_list(&tag.posts, &pages, &labels.no_posts)?,
          posts: tag.posts.clone(),
          tag: Some(tag.id.clone()),
        });
      }
    }
    for view in views {
      append(config, collection, plan, documents, &mut urls, &root, view)?;
    }
  }
  Ok(())
}
fn append(
  config: &ValidatedConfig,
  collection: &ResolvedCollection,
  plan: &mut DocPlan,
  documents: &mut HashMap<(String, String), ParsedContent>,
  urls: &mut HashSet<String>,
  root: &str,
  view: View,
) -> Result<(), String> {
  let route_path = format!("{root}{}", view.suffix);
  if !urls.insert(routes::collision_key(&route_path)) {
    return Err(format!(
      "[DOCS_ROUTE_COLLISION] generated blog page {route_path} conflicts with an existing page"
    ));
  }
  let key = routes::page_key(&collection.public, &view.id)?;
  let source_path = format!("farm-docs:generated:{key}");
  let source = format!("# {}\n\n{}", escape(&view.title), view.body);
  let parsed = parse_content(
    &source,
    &mdxjs::Options {
      filepath: Some(source_path.clone()),
      jsx: true,
      ..Default::default()
    },
    None,
    ContentFormat::Markdown,
  )?;
  let headings = parsed
    .toc
    .iter()
    .map(|heading| PublicHeading {
      id: heading.id.clone(),
      title: heading.title.clone(),
      depth: heading.depth,
    })
    .collect();
  let output_path = format!(
    "{}index.html",
    route_path
      .strip_prefix(&config.public.base_path)
      .ok_or("[DOCS_BLOG_VIEW] invalid base path")?
  );
  routes::validate_output(&output_path, &source_path)?;
  plan.routes.push(RenderRoute {
    page: PublicPage {
      key: key.clone(),
      id: view.id,
      collection: collection.public.id.clone(),
      kind: view.kind.into(),
      locale: collection.public.locale.clone(),
      version: None,
      canonical_path: route_path.clone(),
      route_path,
      title: view.title,
      description: None,
      headings,
      alternates: vec![],
      fallback_source: None,
      previous: None,
      next: None,
      blog: None,
      blog_view: Some(PublicBlogView {
        full_content: view.kind == "blog-index"
          && matches!(
            collection.blog_list_content,
            farmfe_docs_config::BlogListContent::Full
          ),
        kind: view.kind.into(),
        posts: view.posts,
        tag: view.tag,
      }),
      layout: None,
    },
    output_path,
  });
  plan.modules.push(PreparedModule {
    key,
    source_path: source_path.clone(),
    source_hash: hash(source.as_bytes()),
    code: String::new(),
    source_map: None,
    generated_source: Some(source),
    language: ContentLanguage::Markdown,
    static_anchors: vec![],
  });
  documents.insert((source_path, collection.public.id.clone()), parsed);
  Ok(())
}
