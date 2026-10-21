use crate::metadata::{DiagnosticSource, MetadataSource};
use crate::{
  routes, DocPlan, PageNavigation, PublicBlogAuthor, PublicBlogIndex, PublicBlogPost,
  PublicBlogTag, PublicBlogTagIndex, PublicBlogYearIndex,
};
use farmfe_docs_config::{CollectionKind, ValidatedConfig};
use serde::Deserialize;
use serde_json::Value;
use std::collections::{BTreeMap, HashMap, HashSet};

fn error(
  code: &str,
  file: &(impl DiagnosticSource + ?Sized),
  field: &str,
  message: impl std::fmt::Display,
) -> String {
  format!("[{code}] {} ({field}): {message}", file.at(field))
}
fn valid_date(value: &str) -> bool {
  let bytes = value.as_bytes();
  if bytes.len() != 10
    || bytes[4] != b'-'
    || bytes[7] != b'-'
    || bytes
      .iter()
      .enumerate()
      .any(|(index, byte)| index != 4 && index != 7 && !byte.is_ascii_digit())
  {
    return false;
  }
  let year = value[..4].parse::<u32>().unwrap_or(0);
  let month = value[5..7].parse::<u32>().unwrap_or(0);
  let day = value[8..].parse::<u32>().unwrap_or(0);
  let days = match month {
    1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
    4 | 6 | 9 | 11 => 30,
    2 if year % 400 == 0 || (year % 4 == 0 && year % 100 != 0) => 29,
    2 => 28,
    _ => 0,
  };
  year > 0 && day > 0 && day <= days
}
pub fn filename(stem: &str) -> Result<(String, Option<String>), String> {
  // Unlike docs/index, the top-level blog/index is an article, not the listing.
  let path = stem.strip_suffix("/index").unwrap_or(stem);
  let (parent, name) = path.rsplit_once('/').unwrap_or(("", path));
  if name.as_bytes().get(10) == Some(&b'-')
    && name.get(..10).is_some_and(|value| {
      value.as_bytes().get(4) == Some(&b'-') && value.as_bytes().get(7) == Some(&b'-')
    })
  {
    let date = &name[..10];
    if !valid_date(date) || name.len() == 11 {
      return Err(error(
        "DOCS_BLOG_DATE",
        stem,
        "filename",
        "invalid dated filename",
      ));
    }
    let slug = format!(
      "{}{}{}/{}/{}/{}",
      parent,
      if parent.is_empty() { "" } else { "/" },
      &date[..4],
      &date[5..7],
      &date[8..],
      &name[11..]
    );
    return Ok((slug, Some(date.into())));
  }
  Ok((path.into(), None))
}
pub fn validate_slug(slug: &str, file: &str) -> Result<(), String> {
  if slug.is_empty()
    || ["archive", "tags", "page"]
      .iter()
      .any(|reserved| slug == *reserved || slug.starts_with(&format!("{reserved}/")))
  {
    return Err(error(
      "DOCS_BLOG_ROUTE",
      file,
      "slug",
      "blog listing/archive/tag/pagination paths are reserved",
    ));
  }
  Ok(())
}
fn text(value: &str, code: &str, file: &MetadataSource<'_>, field: &str) -> Result<String, String> {
  if value.trim().is_empty() || value.chars().any(char::is_control) {
    return Err(error(
      code,
      file,
      field,
      "require nonempty text without control characters",
    ));
  }
  Ok(value.trim().into())
}
fn entries(
  value: Option<&Value>,
  code: &str,
  file: &MetadataSource<'_>,
  field: &str,
) -> Result<Vec<Value>, String> {
  match value {
    None | Some(Value::Null) => Ok(vec![]),
    Some(Value::Array(values)) => Ok(values.clone()),
    Some(value @ (Value::String(_) | Value::Object(_))) => Ok(vec![value.clone()]),
    _ => Err(error(
      code,
      file,
      field,
      "require a string, object or array",
    )),
  }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AuthorRecord {
  name: String,
  url: Option<String>,
  image: Option<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TagRecord {
  id: String,
  label: String,
}
fn record<T: serde::de::DeserializeOwned>(
  value: Value,
  allowed: &[&str],
  required: &[&str],
  code: &str,
  file: &MetadataSource<'_>,
  field: &str,
) -> Result<T, String> {
  let object = value
    .as_object()
    .ok_or_else(|| error(code, file, field, "require a string or object"))?;
  for key in object.keys() {
    if !allowed.contains(&key.as_str()) {
      return Err(error(
        code,
        file,
        &format!("{field}.{key}"),
        "unknown field",
      ));
    }
  }
  for key in allowed {
    match object.get(*key) {
      Some(Value::String(_)) => {}
      None | Some(Value::Null) if !required.contains(key) => {}
      _ => {
        return Err(error(
          code,
          file,
          &format!("{field}.{key}"),
          "require a string",
        ));
      }
    }
  }
  serde_json::from_value(value).map_err(|cause| error(code, file, field, cause))
}
fn entry_field(frontmatter: &Value, field: &str, index: usize) -> String {
  if frontmatter.get(field).is_some_and(Value::is_array) {
    format!("{field}[{index}]")
  } else {
    field.into()
  }
}

pub fn metadata(
  frontmatter: &Value,
  filename_date: Option<String>,
  file: &MetadataSource<'_>,
) -> Result<PublicBlogPost, String> {
  let (date, published_at) = match frontmatter.get("date") {
    None | Some(Value::Null) => (filename_date, None),
    Some(Value::String(value)) if valid_date(value) => (Some(value.clone()), None),
    Some(Value::String(value)) => {
      let timestamp = chrono::DateTime::parse_from_rfc3339(value)
        .map_err(|cause| error("DOCS_BLOG_DATE", file, "date", cause))?;
      let date = timestamp.date_naive().to_string();
      if !valid_date(&date) {
        return Err(error(
          "DOCS_BLOG_DATE",
          file,
          "date",
          "publication day must be within years 0001..9999",
        ));
      }
      (Some(date), Some(value.clone()))
    }
    _ => {
      return Err(error(
        "DOCS_BLOG_DATE",
        file,
        "date",
        "require a valid Gregorian YYYY-MM-DD date or RFC3339 timestamp",
      ));
    }
  };
  let mut authors = vec![];
  for (index, value) in entries(
    frontmatter.get("authors"),
    "DOCS_BLOG_AUTHOR",
    file,
    "authors",
  )?
  .into_iter()
  .enumerate()
  {
    let field = entry_field(frontmatter, "authors", index);
    let is_record = value.is_object();
    let author = match value {
      Value::String(name) => AuthorRecord {
        name,
        url: None,
        image: None,
      },
      value => record(
        value,
        &["name", "url", "image"],
        &["name"],
        "DOCS_BLOG_AUTHOR",
        file,
        &field,
      )?,
    };
    let name_field = if is_record {
      format!("{field}.name")
    } else {
      field.clone()
    };
    let name = text(&author.name, "DOCS_BLOG_AUTHOR", file, &name_field)?;
    for (property, value) in [("url", &author.url), ("image", &author.image)] {
      if let Some(value) = value {
        farmfe_docs_config::public_link(value, &field).map_err(|cause| {
          error(
            "DOCS_BLOG_AUTHOR",
            file,
            &format!("{field}.{property}"),
            cause,
          )
        })?;
        if value.starts_with('#') {
          return Err(error(
            "DOCS_BLOG_AUTHOR",
            file,
            &format!("{field}.{property}"),
            "fragment-only author URLs have no context",
          ));
        }
      }
    }
    authors.push(PublicBlogAuthor {
      name,
      url: author.url,
      image: author.image,
    });
  }
  let mut tags = vec![];
  let mut ids = HashSet::new();
  for (index, value) in entries(frontmatter.get("tags"), "DOCS_BLOG_TAG", file, "tags")?
    .into_iter()
    .enumerate()
  {
    let field = entry_field(frontmatter, "tags", index);
    let id_field = if value.is_object() {
      format!("{field}.id")
    } else {
      field.clone()
    };
    let (id, label) = match value {
      Value::String(label) => {
        let label = text(&label, "DOCS_BLOG_TAG", file, &field)?;
        let mut id = String::new();
        for ch in routes::logical_id(&label)
          .chars()
          .flat_map(char::to_lowercase)
        {
          if ch.is_alphanumeric() {
            id.push(ch);
          } else if !id.ends_with('-') && !id.is_empty() {
            id.push('-');
          }
        }
        (id.trim_end_matches('-').into(), label)
      }
      value => {
        let tag: TagRecord = record(
          value,
          &["id", "label"],
          &["id", "label"],
          "DOCS_BLOG_TAG",
          file,
          &field,
        )?;
        (
          routes::logical_id(&tag.id.to_lowercase()),
          text(&tag.label, "DOCS_BLOG_TAG", file, &format!("{field}.label"))?,
        )
      }
    };
    if id.is_empty()
      || matches!(id.as_str(), "." | "..")
      || !id
        .chars()
        .all(|ch| ch.is_alphanumeric() || matches!(ch, '-' | '_' | '.'))
      || !ids.insert(id.clone())
    {
      return Err(error(
        "DOCS_BLOG_TAG",
        file,
        &id_field,
        "invalid or duplicate tag ID; use distinct explicit IDs when labels normalize alike",
      ));
    }
    routes::validate_output(&format!("tags/{id}/index.html"), file.file)
      .map_err(|cause| error("DOCS_BLOG_TAG", file, &id_field, cause))?;
    tags.push(PublicBlogTag { id, label });
  }
  Ok(PublicBlogPost {
    date,
    published_at,
    authors,
    tags,
  })
}

fn instant(post: &PublicBlogPost) -> Option<(i64, u32)> {
  if let Some(value) = &post.published_at {
    chrono::DateTime::parse_from_rfc3339(value)
      .ok()
      .map(|value| (value.timestamp(), value.timestamp_subsec_nanos()))
  } else {
    chrono::NaiveDate::parse_from_str(post.date.as_deref()?, "%Y-%m-%d")
      .ok()?
      .and_hms_opt(0, 0, 0)
      .map(|value| (value.and_utc().timestamp(), 0))
  }
}

pub fn index(config: &ValidatedConfig, plan: &mut DocPlan) -> Result<(), String> {
  let mut indexes = vec![];
  let mut navigation = HashMap::new();
  for collection in config
    .collections
    .iter()
    .filter(|collection| collection.public.kind == CollectionKind::Blog)
  {
    let mut posts: Vec<_> = plan
      .routes
      .iter()
      .enumerate()
      .filter(|(_, route)| {
        route.page.collection == collection.public.id && route.page.blog.is_some()
      })
      .collect();
    posts.sort_by(|(_, a), (_, b)| {
      b.page
        .blog
        .as_ref()
        .and_then(instant)
        .cmp(&a.page.blog.as_ref().and_then(instant))
        .then(a.page.route_path.cmp(&b.page.route_path))
    });
    let mut tags: BTreeMap<String, PublicBlogTagIndex> = BTreeMap::new();
    let mut years: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut undated = vec![];
    let order: Vec<_> = posts
      .iter()
      .map(|(_, route)| PageNavigation {
        key: route.page.key.clone(),
        id: route.page.id.clone(),
        label: route.page.title.clone(),
        href: route.page.route_path.clone(),
      })
      .collect();
    for (position, (index, route)) in posts.iter().enumerate() {
      let blog = route
        .page
        .blog
        .as_ref()
        .ok_or("[DOCS_BLOG_INDEX] missing blog metadata")?;
      if let Some(date) = &blog.date {
        years
          .entry(date[..4].into())
          .or_default()
          .push(route.page.key.clone());
      } else {
        undated.push(route.page.key.clone());
      }
      for tag in &blog.tags {
        let entry = tags
          .entry(tag.id.clone())
          .or_insert_with(|| PublicBlogTagIndex {
            id: tag.id.clone(),
            label: tag.label.clone(),
            posts: vec![],
          });
        if routes::collision_key(&entry.label) != routes::collision_key(&tag.label) {
          return Err(error(
            "DOCS_BLOG_TAG",
            &plan.modules[*index].source_path,
            "tags",
            format!(
              "tag {:?} has conflicting labels {:?} and {:?}",
              tag.id, entry.label, tag.label
            ),
          ));
        }
        entry.posts.push(route.page.key.clone());
      }
      navigation.insert(
        route.page.key.clone(),
        (
          position
            .checked_sub(1)
            .map(|previous| order[previous].clone()),
          order.get(position + 1).cloned(),
        ),
      );
    }
    indexes.push(PublicBlogIndex {
      collection: collection.public.id.clone(),
      posts: order.iter().map(|page| page.key.clone()).collect(),
      tags: tags.into_values().collect(),
      years: years
        .into_iter()
        .rev()
        .map(|(year, posts)| PublicBlogYearIndex { year, posts })
        .collect(),
      undated,
    });
  }
  for route in &mut plan.routes {
    if let Some((previous, next)) = navigation.remove(&route.page.key) {
      route.page.previous = previous;
      route.page.next = next;
    }
  }
  plan.blog_indexes = indexes;
  Ok(())
}
