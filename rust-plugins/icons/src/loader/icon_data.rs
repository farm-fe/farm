use super::struct_config::IconifyLoaderOptions;
use super::svg_builder::SvgBuilder;
use super::{
  icon_to_svg::{icon_to_svg, IconifyIconBuildResult},
  struct_config::IconifyIcon,
};

pub fn gen_svg_for_icon_data(
  icon_data: IconifyIcon,
  options: IconifyLoaderOptions,
) -> Option<String> {
  let IconifyIconBuildResult {
    mut attributes,
    body,
    ..
  } = icon_to_svg(icon_data, None);
  if let Some(s) = options.scale {
    if s != 0.0 {
      attributes.height = Some(format!("{s}em"));
      attributes.width = Some(format!("{s}em"));
    }
  }
  let svg_content = format!("<svg>{body}</svg>");
  let svg = SvgBuilder::new(&svg_content)
    .width(attributes.width)
    .height(attributes.height)
    .view_box(Some(attributes.view_box))
    .insert_customizations(options.customizations.unwrap_or_default())
    .build();

  Some(svg)
}
