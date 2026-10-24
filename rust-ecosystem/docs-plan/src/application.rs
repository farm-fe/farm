use crate::DocPlan;

/// Generate only the public application graph. The private prepared plan is never serialized here.
pub fn application(plan: &DocPlan) -> Result<String, String> {
  generate(plan, false)
}

/// Browser content is loaded only through explicit Rust-generated import edges.
pub fn browser_application(plan: &DocPlan) -> Result<String, String> {
  generate(plan, true)
}

fn generate(plan: &DocPlan, lazy: bool) -> Result<String, String> {
  let encode = |value: &str| serde_json::to_string(value).map_err(|error| error.to_string());
  let mut output = String::from(
    "import * as React from 'react';\nimport { DocsProvider, NotFoundProvider } from '@farmfe/docs/client';\nimport { resolveTheme } from '@farmfe/docs/theme';\nimport * as DefaultTheme from '@farmfe/docs-theme-default';\n",
  );
  output.push_str(&format!(
    "export const home = {};\n",
    encode(crate::brand_home::href(plan)?)?
  ));
  let mut aliases = std::collections::BTreeMap::new();
  for route in &plan.routes {
    let bare = route.page.route_path.trim_end_matches('/');
    let paths = if bare.is_empty() {
      vec!["/".to_string()]
    } else {
      vec![bare.to_string(), format!("{bare}/")]
    };
    for path in paths {
      if let Some(previous) = aliases.insert(path.clone(), route.page.key.clone()) {
        if previous != route.page.key {
          return Err(format!("[DOCS_RUNTIME_ROUTE_COLLISION] {path}"));
        }
      }
    }
  }
  output.push_str(&format!(
    "export const routeAliases = {};\n",
    serde_json::to_string(&aliases).map_err(|error| error.to_string())?
  ));
  output.push_str(&format!(
    "export const seoByKey = {};\n",
    serde_json::to_string(&plan.seo).map_err(|error| error.to_string())?
  ));
  output.push_str(&format!(
    "export const notFoundPages = {};\n",
    serde_json::to_string(
      &plan
        .not_found
        .iter()
        .map(|page| serde_json::json!({
          "title": page.title,
            "discoveryLinks": page.discovery_links,
          "description": page.description,
          "locale": page.locale,
          "routeBase": page.route_base,
          "home": page.home,
          "heading": page.heading,
          "homeLabel": page.home_label,
          "robots": crate::not_found::ROBOTS
        }))
        .collect::<Vec<_>>()
    )
    .map_err(|error| error.to_string())?
  ));
  output.push_str("export function getNotFound(path) {\nconst pathname = new URL(path, 'https://farm-docs.invalid').pathname;\nreturn notFoundPages.filter(page => { const mount = new URL(page.routeBase, 'https://farm-docs.invalid').pathname; return pathname === mount.slice(0, -1) || pathname.startsWith(mount); }).sort((a, b) => b.routeBase.length - a.routeBase.length)[0] || notFoundPages[0];\n}\n");
  if let Some(theme) = plan.theme_components.as_deref() {
    output.push_str(&format!("import * as ThemeOverrides from {};\nconst theme = resolveTheme(DefaultTheme, ThemeOverrides);\n", encode(theme)?));
  } else {
    output.push_str("const theme = resolveTheme(DefaultTheme);\n");
  }
  output.push_str("import '@farmfe/docs-theme-default/style.css';\nimport '@farmfe/plugin-syntax-highlight/style.css';\n");
  for style in &plan.styles {
    output.push_str(&format!("import {};\n", encode(style)?));
  }
  for (index, module) in plan.modules.iter().enumerate().filter(|_| !lazy) {
    output.push_str(&format!(
      "import Content{index} from {};\n",
      encode(&format!("docs:content:{}", module.key))?
    ));
  }
  output.push_str(&format!(
    "export const sidebars = {};\n",
    serde_json::to_string(&plan.sidebars).map_err(|error| error.to_string())?
  ));
  output.push_str(&format!(
    "export const blogIndexes = {};\n",
    serde_json::to_string(&plan.blog_indexes).map_err(|error| error.to_string())?
  ));
  output.push_str(&format!("export const site = {};\nexport const snapshotId = {};\nexport const pages = {};\nconst contents = {{\n",
    serde_json::to_string(&plan.site).map_err(|error| error.to_string())?,
    encode(&plan.snapshot_id)?,
    serde_json::to_string(&plan.routes.iter().map(|route| &route.page).collect::<Vec<_>>()).map_err(|error| error.to_string())?));
  for (index, module) in plan.modules.iter().enumerate().filter(|_| !lazy) {
    output.push_str(&format!("{}: Content{index},\n", encode(&module.key)?));
  }
  output.push_str("};\n");
  let mut blog_content_keys = std::collections::BTreeMap::new();
  for route in &plan.routes {
    let Some(view) = route
      .page
      .blog_view
      .as_ref()
      .filter(|view| view.full_content)
    else {
      continue;
    };
    if view.kind != "blog-index" {
      return Err("[DOCS_BLOG_CONTENT] full content requires a blog index".into());
    }
    for key in &view.posts {
      if !plan.routes.iter().any(|post| {
        post.page.key == *key
          && post.page.collection == route.page.collection
          && post.page.kind == "blog"
          && post.page.blog_view.is_none()
      }) {
        return Err(format!(
          "[DOCS_BLOG_CONTENT] invalid article identity: {key}"
        ));
      }
    }
    blog_content_keys.insert(route.page.key.clone(), view.posts.clone());
  }
  output.push_str(&format!(
    "const blogContentKeys = {};\n",
    serde_json::to_string(&blog_content_keys).map_err(|error| error.to_string())?
  ));
  if lazy {
    output.push_str("const loaders = {\n");
    for module in &plan.modules {
      output.push_str(&format!(
        "{}: () => import({}),\n",
        encode(&module.key)?,
        encode(&format!("docs:page:{}", module.key))?
      ));
    }
    output.push_str("};\nfunction loadContent(key) {\nif (contents[key]) return Promise.resolve();\nconst load = loaders[key];\nif (!load) return Promise.reject(new Error('[DOCS_ROUTE] unknown page: ' + key));\nreturn load().then(module => {\nif (!module.default) throw new Error('[DOCS_CONTENT] missing page component: ' + key);\nconst posts = (blogContentKeys[key] || []).map((postKey, index) => {\nconst Post = module['listingPost' + index];\nif (!Post) throw new Error('[DOCS_BLOG_CONTENT] missing article export: ' + postKey);\nreturn [postKey, Post];\n});\nfor (const [postKey, Post] of posts) contents[postKey] = Post;\ncontents[key] = module.default;\n});\n}\nexport function loadPage(key) { return loadContent(key); }\n");
  } else {
    output.push_str("export function loadPage(key) { return contents[key] ? Promise.resolve() : Promise.reject(new Error('[DOCS_ROUTE] unknown page: ' + key)); }\n");
  }
  output.push_str("export function renderPage(key) {\nconst page = pages.find(page => page.key === key);\nconst Content = contents[key];\nif (!page || !Content) throw new Error('[DOCS_ROUTE] unknown or unloaded page: ' + key);\nconst body = React.createElement(Content, { components: theme.contentComponents });\nconst listingPosts = blogContentKeys[key]?.map(postKey => {\nconst post = pages.find(candidate => candidate.key === postKey);\nconst Post = contents[postKey];\nif (!post || !Post) throw new Error('[DOCS_BLOG_CONTENT] unloaded article: ' + postKey);\nreturn { page: post, children: React.createElement(DocsProvider, { site, page: post }, React.createElement(Post, { components: theme.contentComponents })) };\n});\nreturn React.createElement(DocsProvider, { site, page }, page.layout === 'none' ? body : React.createElement(theme.components.Layout, { site, page, data: { home, pages, sidebars, blogIndexes }, components: theme.components, listingPosts }, body));\n}\n");
  output.push_str("export function renderNotFound(locale = site.locales.default) {\nconst notFound = notFoundPages.find(page => page.locale === locale);\nif (!notFound) throw new Error('[DOCS_NOT_FOUND_LOCALE] unknown locale: ' + locale);\nreturn React.createElement(NotFoundProvider, { site, locale }, React.createElement(theme.components.NotFound, { site, data: { home: notFound.home, notFound, pages, sidebars, blogIndexes }, components: theme.components }));\n}\n");
  Ok(output)
}
