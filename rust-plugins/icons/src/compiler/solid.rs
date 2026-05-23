use farmfe_core::regex::Regex;

use super::CompilerParams;

pub fn solid_compiler(parm: CompilerParams) -> String {
  let CompilerParams { svg, .. } = parm;
  let re_braces = Regex::new(r"([{}])").unwrap();
  let svg_with_props = re_braces.replace_all(&svg, "{'$1'}");
  let re_svg_tag = Regex::new(r"(<svg[\s\S]*?)(>)").unwrap();
  let svg_with_props = re_svg_tag.replace(&svg_with_props, "$1{...props}>");
  format!("export default (props = {{}}) => {svg_with_props}")
}
