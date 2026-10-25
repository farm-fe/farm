use std::sync::Arc;

use farmfe_core::{
  error::{CompilationError, Result},
  resource::Resource,
  swc_html_ast::Child,
};
use farmfe_docs_plan::resource_owner;
use farmfe_toolkit::html::{
  codegen_html_document_with_quoted_attributes, create_attribute, create_element,
  parse_html_document,
};

const BOOTSTRAP_META: &str = "farm-docs:html-bootstrap";
const BOOTSTRAP_OWNER: &str = "FarmPluginDocs:color-mode";

/// The HTML plugin appends entry CSS after source head children. Place the
/// per-page placeholder last only after that native injection has completed.
pub(crate) fn order_page_styles(
  resource: &mut Resource,
  minify: bool,
  theme: Option<&super::color_mode::Bootstrap>,
) -> Result<()> {
  let source = String::from_utf8(resource.bytes.clone())
    .map_err(|error| CompilationError::GenericError(format!("[DOCS_TEMPLATE] {error}")))?;
  let mut document = parse_html_document(&resource.name, Arc::new(source))?;
  let html = document.children.iter_mut().find_map(|child| match child {
    Child::Element(element) if element.tag_name == "html" => Some(element),
    _ => None,
  });
  let html =
    html.ok_or_else(|| CompilationError::GenericError("[DOCS_TEMPLATE] missing html".into()))?;
  if let Some(theme) = theme {
    html
      .attributes
      .retain(|attribute| attribute.name != "data-theme");
    if let Some(mode) = theme.fixed_mode {
      html
        .attributes
        .push(create_attribute("data-theme", Some(mode)));
    }
  }
  let head = {
    html.children.iter_mut().find_map(|child| match child {
      Child::Element(element) if element.tag_name == "head" => Some(element),
      _ => None,
    })
  }
  .ok_or_else(|| CompilationError::GenericError("[DOCS_TEMPLATE] missing head".into()))?;
  let positions = head
    .children
    .iter()
    .enumerate()
    .filter_map(|(index, child)| match child {
      Child::Element(element)
        if element.tag_name == "meta"
          && element.attributes.iter().any(|attribute| {
            attribute.name == "name" && attribute.value.as_deref() == Some("farm-docs-page-styles")
          }) =>
      {
        Some(index)
      }
      _ => None,
    })
    .collect::<Vec<_>>();
  if positions.len() > 1 {
    return Err(CompilationError::GenericError(
      "[DOCS_TEMPLATE] duplicate page styles marker".into(),
    ));
  }
  if let Some(position) = positions.first() {
    let marker = head.children.remove(*position);
    head.children.push(marker);
  }
  if let Some(theme) = theme {
    let mut previous = None;
    for (index, child) in head.children.iter().enumerate() {
      let Child::Element(element) = child else {
        continue;
      };
      if !element
        .attributes
        .iter()
        .any(|attribute| attribute.name == "data-farm-docs-color-mode")
      {
        continue;
      }
      let src = element
        .attributes
        .iter()
        .find(|attribute| attribute.name == "src")
        .and_then(|attribute| attribute.value.as_deref());
      let owned = element.tag_name == "script"
        && element.children.is_empty()
        && element.attributes.len() == 2
        && element.attributes.iter().any(|attribute| {
          attribute.name == "data-farm-docs-color-mode"
            && attribute.value.as_deref().unwrap_or("").is_empty()
        })
        && src.is_some_and(|src| {
          resource_owner::is_owned(
            resource.meta.get(BOOTSTRAP_META).map(String::as_str),
            BOOTSTRAP_OWNER,
            &resource.name,
            src.as_bytes(),
          )
        });
      if !owned || previous.is_some() {
        return Err(CompilationError::GenericError(
          "[DOCS_TEMPLATE] reserved color mode marker".into(),
        ));
      }
      previous = Some(index);
    }
    if let Some(index) = previous {
      head.children.remove(index);
    }
    let script = create_element(
      "script",
      None,
      vec![("src", &theme.url), ("data-farm-docs-color-mode", "")],
    );
    // Preserve charset declarations before the script, but never place it
    // after a stylesheet or application/runtime script.
    let position = head.children.iter().position(|child| matches!(child, Child::Element(element) if element.tag_name == "script" || element.tag_name == "link" || element.tag_name == "style")).unwrap_or(head.children.len());
    head.children.insert(position, Child::Element(script));
  }
  // Attribute placeholders contain no spaces before SSG. Their eventual escaped
  // values may, so letting minification remove their quotes changes DOM meaning.
  let marker = theme
    .map(|theme| resource_owner::stamp(BOOTSTRAP_OWNER, &resource.name, theme.url.as_bytes()))
    .transpose()
    .map_err(|error| CompilationError::GenericError(format!("[DOCS_TEMPLATE] {error}")))?;
  resource.bytes = codegen_html_document_with_quoted_attributes(&document, minify).into_bytes();
  if let Some(marker) = marker {
    resource.meta.insert(BOOTSTRAP_META.into(), marker);
  }
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  fn bootstrap(url: &str) -> super::super::color_mode::Bootstrap {
    super::super::color_mode::Bootstrap {
      name: "bootstrap.js".into(),
      bytes: vec![],
      url: url.into(),
      fixed_mode: None,
    }
  }

  #[test]
  fn repeated_bootstrap_injection_survives_html_reencoding_and_changes_url() {
    for minify in [false, true] {
      let mut resource = Resource { name: "client.html".into(), bytes: b"<!doctype html><html><head><meta charset=\"utf-8\"><link rel=\"stylesheet\" href=\"app.css\"></head><body></body></html>".to_vec(), ..Default::default() };
      order_page_styles(&mut resource, minify, Some(&bootstrap("/old.js"))).unwrap();
      // Core may reencode HTML after finalization; provenance cannot depend on
      // a digest of the entire HTML document.
      order_page_styles(&mut resource, !minify, None).unwrap();
      order_page_styles(&mut resource, minify, Some(&bootstrap("/new.js")))
        .expect("replace only the previously injected bootstrap");
      let html = String::from_utf8(resource.bytes).unwrap();
      assert_eq!(html.matches("data-farm-docs-color-mode").count(), 1);
      assert!(!html.contains("/old.js"));
      assert!(html.contains("/new.js"));
      assert!(html.find("/new.js").unwrap() < html.find("app.css").unwrap());
    }
  }

  #[test]
  fn user_reserved_bootstrap_marker_is_rejected_without_mutation() {
    let mut resource = Resource { name: "client.html".into(), bytes: b"<html><head><script src=\"/user.js\" data-farm-docs-color-mode></script></head><body></body></html>".to_vec(), ..Default::default() };
    let before = resource.bytes.clone();
    assert!(order_page_styles(&mut resource, false, Some(&bootstrap("/new.js"))).is_err());
    assert_eq!(resource.bytes, before);
  }

  #[test]
  fn altered_bootstrap_or_missing_record_is_rejected_without_mutation() {
    for mutation in [
      "metadata",
      "src",
      "inline",
      "attribute",
      "duplicate",
      "name",
    ] {
      let mut resource = Resource {
        name: "client.html".into(),
        bytes: b"<html><head></head><body></body></html>".to_vec(),
        ..Default::default()
      };
      order_page_styles(&mut resource, false, Some(&bootstrap("/old.js"))).unwrap();
      let html = String::from_utf8(resource.bytes.clone()).unwrap();
      match mutation {
        "metadata" => resource.meta.clear(),
        "src" => resource.bytes = html.replace("/old.js", "/foreign.js").into_bytes(),
        "inline" => resource.bytes = html.replace("</script>", "alert(1)</script>").into_bytes(),
        "attribute" => resource.bytes = html.replace("<script ", "<script async ").into_bytes(),
        "duplicate" => {
          resource.bytes = html
            .replace(
              "</head>",
              "<script src=\"/old.js\" data-farm-docs-color-mode></script></head>",
            )
            .into_bytes()
        }
        "name" => resource.name = "other.html".into(),
        _ => unreachable!(),
      }
      let before = (resource.bytes.clone(), resource.meta.clone());
      assert!(
        order_page_styles(&mut resource, false, Some(&bootstrap("/new.js"))).is_err(),
        "{mutation}"
      );
      assert_eq!((resource.bytes, resource.meta), before, "{mutation}");
    }
  }

  #[test]
  fn template_attribute_placeholders_remain_quoted_in_both_modes() {
    for minify in [true, false] {
      let mut resource = Resource {
        name: "client.html".into(),
        bytes: br#"<!doctype html><html lang="__FARM_DOCS_LANG__"><head><title>__FARM_DOCS_TITLE__</title><meta name="description" content="__FARM_DOCS_DESCRIPTION__"><link rel="canonical" href="__FARM_DOCS_CANONICAL__"></head><body></body></html>"#.to_vec(),
        ..Default::default()
      };
      order_page_styles(&mut resource, minify, None).expect("valid template");
      let html = String::from_utf8(resource.bytes).expect("UTF-8");
      for attribute in [
        "lang=\"__FARM_DOCS_LANG__\"",
        "content=\"__FARM_DOCS_DESCRIPTION__\"",
        "href=\"__FARM_DOCS_CANONICAL__\"",
      ] {
        assert!(html.contains(attribute), "minify={minify}: {html}");
      }
    }
  }
}
