use super::CompilerParams;
use farmfe_toolkit::resolve::package_json_loader::{Options, PackageJsonLoader};
use once_cell::sync::Lazy;
use std::path::Path;
use std::sync::Mutex;

static SVELTE_RUNES: Lazy<Mutex<Option<bool>>> = Lazy::new(|| Mutex::new(None));

pub fn svelte_compiler(param: CompilerParams) -> String {
  let CompilerParams { svg, root_path, .. } = param;
  let mut svelte_runes = SVELTE_RUNES.lock().unwrap();
  if svelte_runes.is_none() {
    *svelte_runes = match get_svelte_version(&root_path.unwrap()) {
      Some(version) => Some(version >= 5),
      None => Some(false),
    };
  }

  let svelte_runes = svelte_runes.unwrap();
  let open_tag_end = svg.find('>').unwrap_or(0);
  let close_tag_start = svg.rfind("</svg").unwrap_or(svg.len());
  let mut sfc = format!(
    "{} {{...{}}}>",
    &svg[..open_tag_end],
    if svelte_runes { "p" } else { "$$props" }
  );

  if svelte_runes {
    sfc.push_str(&svg[open_tag_end + 1..close_tag_start]);
  } else {
    sfc.push_str(&format!(
      "@html `{}`",
      escape_svelte(&svg[open_tag_end + 1..close_tag_start])
    ));
  }

  sfc.push_str(&svg[close_tag_start..]);
  if svelte_runes {
    format!("<script>const{{...p}}=$props()</script>{sfc}")
  } else {
    sfc
  }
}

fn get_svelte_version(root_path: &str) -> Option<u8> {
  let loader = PackageJsonLoader::new();
  let package_path = Path::new(root_path).join("node_modules/svelte/compiler/package.json");
  match loader.load(
    package_path.clone(),
    Options {
      follow_symlinks: false,
      resolve_ancestor_dir: false,
    },
  ) {
    Ok(package_json) => package_json
      .raw_map()
      .get("version")
      .and_then(|v| v.as_str())
      .and_then(|v| v.split('.').next())
      .and_then(|v| v.parse().ok()),
    Err(_) => None,
  }
}

pub fn escape_svelte(str: &str) -> String {
  str
    .replace("{", "&#123;")
    .replace("}", "&#125;")
    .replace("`", "&#96;")
    .replace("\\t", " ")
    .replace("\\r", " ")
    .replace("\\n", " ")
}
