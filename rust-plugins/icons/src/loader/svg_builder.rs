use serde::Deserialize;
use serde_json::Value;
use xmltree::Element;
pub struct SvgBuilder {
  element: Element,
}

#[derive(Default, Deserialize)]
pub struct SvgCustomizations {
  pub fill: Option<String>,
  pub stroke: Option<String>,
  pub stroke_width: Option<String>,
  pub width: Option<String>,
  pub height: Option<String>,
  pub view_box: Option<String>,
  pub class: Option<String>,
  pub style: Option<Value>,
}

impl SvgBuilder {
  pub fn new(svg_content: &str) -> Self {
    let element = Element::parse(svg_content.as_bytes()).unwrap();
    SvgBuilder { element }
  }

  pub fn fill(mut self, fill: Option<String>) -> Self {
    if let Some(fill) = fill {
      self.element.attributes.insert("fill".to_string(), fill);
    }
    self
  }

  pub fn stroke(mut self, stroke: Option<String>) -> Self {
    if let Some(stroke) = stroke {
      self.element.attributes.insert("stroke".to_string(), stroke);
    }
    self
  }

  pub fn stroke_width(mut self, stroke_width: Option<String>) -> Self {
    if let Some(stroke_width) = stroke_width {
      self
        .element
        .attributes
        .insert("stroke-width".to_string(), stroke_width);
    }
    self
  }

  pub fn width(mut self, width: Option<String>) -> Self {
    if let Some(width) = width {
      self.element.attributes.insert("width".to_string(), width);
    }
    self
  }

  pub fn height(mut self, height: Option<String>) -> Self {
    if let Some(height) = height {
      self.element.attributes.insert("height".to_string(), height);
    }
    self
  }

  pub fn view_box(mut self, view_box: Option<String>) -> Self {
    if let Some(view_box) = view_box {
      self
        .element
        .attributes
        .insert("viewBox".to_string(), view_box);
    }
    self
  }

  pub fn class(mut self, class: Option<String>) -> Self {
    if let Some(class) = class {
      self.element.attributes.insert("class".to_string(), class);
    }
    self
  }

  pub fn style(mut self, style: Option<Value>) -> Self {
    if let Some(style) = style {
      let style_str = style.as_object().map_or(String::new(), |obj| {
        obj
          .iter()
          .map(|(key, value)| {
            format!(
              "{}:{};",
              key,
              value.as_str().unwrap_or("").replace("\"", "")
            )
          })
          .collect::<Vec<_>>()
          .join(" ")
      });
      self
        .element
        .attributes
        .insert("style".to_string(), style_str);
    }
    self
  }

  pub fn insert_customizations(self, customizations: SvgCustomizations) -> Self {
    self
      .fill(customizations.fill)
      .class(customizations.class)
      .height(customizations.height)
      .stroke(customizations.stroke)
      .stroke_width(customizations.stroke_width)
      .style(customizations.style)
      .view_box(customizations.view_box)
      .width(customizations.width)
  }

  pub fn build(self) -> String {
    let mut buffer = Vec::new();
    self.element.write(&mut buffer).unwrap();
    String::from_utf8(buffer).unwrap()
  }
}
