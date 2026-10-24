use crate::cache::HttpClient;
use farmfe_core::regex::{self, Regex};
use farmfe_toolkit::fs::read_file_utf8;
use serde_json::Value;
use std::{
  fs::File,
  io::BufReader,
  path::{Component, Path, PathBuf},
  process::Command,
};
use tokio::runtime::Runtime;

pub const URL_PREFIXES: [&str; 4] = ["/~icons/", "~icons/", "virtual:icons/", "virtual/icons/"];

pub fn is_icon_path(path: &str) -> bool {
  let icon_path_re = regex::Regex::new(
    &URL_PREFIXES
      .iter()
      .map(|v| format!("^{v}"))
      .collect::<Vec<String>>()
      .join("|"),
  )
  .unwrap();
  icon_path_re.is_match(path)
}
#[derive(Debug)]
pub struct ResolveResult {
  pub collection: String,
  pub icon: String,
}

pub fn remove_prefix(path: &str) -> String {
  let mut path = path.to_string();
  for prefix in URL_PREFIXES.iter() {
    if path.starts_with(prefix) {
      path = path.replacen(prefix, "", 1);
      break;
    }
  }
  path
}

pub fn resolve_icons_path(path: &str) -> ResolveResult {
  let path = remove_prefix(path);
  let (path, _) = path.rsplit_once(".").unzip();
  let (collection, icon) = path.unwrap().split_once("/").unzip();

  ResolveResult {
    collection: collection.unwrap().to_owned(),
    icon: icon.unwrap().to_owned(),
  }
}

pub struct PathMate {
  pub base_path: String,
  pub query: String,
}

pub fn get_path_meta(path: &str) -> PathMate {
  let normalized_id = remove_prefix(path);
  let query_index = normalized_id.find('?').unwrap_or(normalized_id.len());

  let re_extension = Regex::new(r"\.\w+$").unwrap();
  let re_leading_slash = Regex::new(r"^/").unwrap();

  let base = if query_index < normalized_id.len() {
    &normalized_id[..query_index]
  } else {
    &normalized_id
  };

  let base = re_extension.replace(base, "").to_string();
  let base = re_leading_slash.replace(&base, "").to_string();

  let query = if query_index < normalized_id.len() {
    format!("?{}", &normalized_id[query_index + 1..])
  } else {
    "".to_string()
  };

  PathMate {
    base_path: base,
    query,
  }
}

#[derive(Debug)]
pub struct GetSvgByCustomCollectionsParams {
  pub custom_collection_path: String,
  pub icon: String,
  pub project_dir: String,
}

pub struct GetIconPathDataParams {
  pub path: String,
  pub project_dir: String,
  pub auto_install: bool,
}

pub fn get_svg_by_custom_collections(
  http_client: &HttpClient,
  opt: GetSvgByCustomCollectionsParams,
) -> String {
  let GetSvgByCustomCollectionsParams {
    custom_collection_path,
    icon,
    project_dir,
  } = opt;
  if is_valid_icon_path(&custom_collection_path) {
    let mut svg_raw = String::new();
    let custom_collection_path = custom_collection_path.replace("[iconname]", &icon);
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
      if let Ok(res) = http_client.fetch_data(&custom_collection_path).await {
        svg_raw = res;
      }
    });
    return svg_raw;
  }
  read_local_custom_icon(&Path::new(&project_dir).join(custom_collection_path), &icon)
    .unwrap_or_default()
}

fn read_local_custom_icon(collection: &Path, icon: &str) -> Option<String> {
  if icon.is_empty()
    || Path::new(icon)
      .components()
      .any(|component| !matches!(component, Component::Normal(_)))
  {
    return None;
  }
  let root = collection.canonicalize().ok()?;
  let selected = if root.is_file() {
    if root.extension()? != "svg" || root.file_stem()?.to_str()? != icon {
      return None;
    }
    root.clone()
  } else {
    root.join(format!("{icon}.svg")).canonicalize().ok()?
  };
  if !selected.is_file() || !selected.starts_with(&root) {
    return None;
  }
  read_file_utf8(selected.to_str()?).ok()
}

fn is_valid_icon_path(icon_path: &str) -> bool {
  icon_path.contains("[iconname]") && icon_path.contains("http")
}
pub fn get_svg_by_local_path(path: &str) -> String {
  read_file_utf8(path).unwrap()
}

pub fn get_icon_data_by_iconify(opt: GetIconPathDataParams) -> Value {
  let ResolveResult { collection, icon } = resolve_icons_path(&opt.path);
  let all_icon_path = build_icon_path(&opt.project_dir, "@iconify/json/json");
  let icons_path = build_icon_path(&opt.project_dir, &format!("@iconify-json/{collection}"));

  if !all_icon_path.exists() && !icons_path.exists() {
    if opt.auto_install {
      install_icon_package(&opt.project_dir, &collection);
    } else {
      return Value::Null;
    }
  }

  let icon_collection_path_by_sign = icons_path.join("icons.json");
  let icon_collection_path = all_icon_path.join(format!("{collection}.json"));

  if let Some(body) = get_icon_body(&icon_collection_path_by_sign, &icon) {
    return body;
  } else if let Some(body) = get_icon_body(&icon_collection_path, &icon) {
    return body;
  }

  Value::Null
}

fn get_icon_body(path: &std::path::Path, icon: &str) -> Option<Value> {
  if path.exists() {
    let json = read_json_from_file(path.to_str().unwrap());
    if let Some(body) = json.get("icons").and_then(|icons| icons.get(icon)) {
      let mut body = body.clone();
      if body.get("height").is_none() {
        let default_height = json.get("height").and_then(|v| v.as_i64());
        body["height"] = Value::Number(default_height.unwrap_or(24).into());
      }
      if body.get("width").is_none() {
        let default_width = json.get("width").and_then(|v| v.as_i64());
        body["width"] = Value::Number(default_width.unwrap_or(24).into());
      }
      return Some(body);
    }
  }
  None
}
fn read_json_from_file(file_path: &str) -> serde_json::Value {
  let file = File::open(file_path).expect("Failed to open file");
  let reader = BufReader::new(file);
  serde_json::from_reader(reader).expect("Failed to read JSON")
}

fn build_icon_path(project_dir: &str, sub_path: &str) -> PathBuf {
  Path::new(project_dir).join("node_modules").join(sub_path)
}

fn install_icon_package(project_dir: &str, collection: &str) {
  let pkg_manager = get_package_manager(project_dir);
  let cmd = match pkg_manager.as_str() {
    "npm" => format!("npm install @iconify-json/{collection}"),
    "pnpm" => format!("pnpm add @iconify-json/{collection}"),
    "yarn" => format!("yarn add @iconify-json/{collection}"),
    _ => panic!("Unknown package manager"),
  };
  let output = Command::new("sh")
    .arg("-c")
    .arg(cmd)
    .output()
    .expect("Failed to execute command");
  if !output.status.success() {
    panic!(
      "Command execution failed: {}",
      String::from_utf8_lossy(&output.stderr)
    );
  }
}

pub fn get_package_manager(project_dir: &str) -> String {
  find_package_manager_in_current_or_parent(Path::new(project_dir))
    .unwrap_or_else(|| "pnpm".to_string())
}

fn find_package_manager_in_current_or_parent(dir: &Path) -> Option<String> {
  if let Some(manager) = check_lock_files(dir) {
    return Some(manager);
  }

  if let Some(parent) = dir.parent() {
    return check_lock_files(parent);
  }

  None
}

fn check_lock_files(dir: &Path) -> Option<String> {
  let npm_lock = dir.join("package-lock.json");
  let pnpm_lock = dir.join("pnpm-lock.yaml");
  let yarn_lock = dir.join("yarn.lock");

  if npm_lock.exists() {
    Some("npm".to_string())
  } else if pnpm_lock.exists() {
    Some("pnpm".to_string())
  } else if yarn_lock.exists() {
    Some("yarn".to_string())
  } else {
    None
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::{
    fs,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
  };

  struct Fixture {
    root: PathBuf,
    parent: PathBuf,
  }

  impl Fixture {
    fn new() -> Self {
      static NEXT: AtomicU64 = AtomicU64::new(0);
      let parent = std::env::temp_dir().canonicalize().unwrap();
      let root = parent.join(format!(
        "farm-icons-local-{}-{}-{}",
        std::process::id(),
        SystemTime::now()
          .duration_since(UNIX_EPOCH)
          .unwrap()
          .as_nanos(),
        NEXT.fetch_add(1, Ordering::Relaxed)
      ));
      fs::create_dir(&root).unwrap();
      Self { root, parent }
    }
  }

  impl Drop for Fixture {
    fn drop(&mut self) {
      let resolved = self.root.canonicalize().unwrap();
      assert_eq!(resolved.parent(), Some(self.parent.as_path()));
      assert!(
        resolved
          .file_name()
          .unwrap()
          .to_string_lossy()
          .starts_with("farm-icons-local-")
      );
      fs::remove_dir_all(resolved).unwrap();
    }
  }

  #[test]
  fn directory_loads_requested_icon_instead_of_first_svg() {
    let fixture = Fixture::new();
    fs::write(fixture.root.join("a-first.svg"), "<svg>inventory</svg>").unwrap();
    fs::write(fixture.root.join("launch.svg"), "<svg>launch</svg>").unwrap();
    assert_eq!(
      read_local_custom_icon(&fixture.root, "launch").as_deref(),
      Some("<svg>launch</svg>")
    );
    assert_eq!(
      read_local_custom_icon(&fixture.root, "a-first").as_deref(),
      Some("<svg>inventory</svg>")
    );
  }

  #[test]
  fn missing_icon_has_no_arbitrary_fallback() {
    let fixture = Fixture::new();
    fs::write(fixture.root.join("other.svg"), "<svg>other</svg>").unwrap();
    assert!(read_local_custom_icon(&fixture.root, "missing").is_none());
  }

  #[test]
  fn nested_and_dotted_names_keep_the_requested_identity() {
    let fixture = Fixture::new();
    fs::create_dir(fixture.root.join("nested")).unwrap();
    fs::write(
      fixture.root.join("nested/launch.v2.svg"),
      "<svg>nested</svg>",
    )
    .unwrap();
    assert_eq!(
      read_local_custom_icon(&fixture.root, "nested/launch.v2").as_deref(),
      Some("<svg>nested</svg>")
    );
    let resolved = resolve_icons_path("~icons/custom/nested/launch.v2.jsx");
    assert_eq!(resolved.collection, "custom");
    assert_eq!(resolved.icon, "nested/launch.v2");
  }

  #[test]
  fn collection_boundary_rejects_parent_and_absolute_paths() {
    let fixture = Fixture::new();
    let collection = fixture.root.join("collection");
    fs::create_dir(&collection).unwrap();
    fs::write(fixture.root.join("outside.svg"), "<svg>outside</svg>").unwrap();
    for name in ["../outside", "/outside", "", ".", "nested/../outside"] {
      assert!(
        read_local_custom_icon(&collection, name).is_none(),
        "{name}"
      );
    }
    assert!(
      read_local_custom_icon(&collection, fixture.root.join("outside").to_str().unwrap()).is_none()
    );
  }

  #[test]
  fn explicit_file_collection_only_matches_its_name() {
    let fixture = Fixture::new();
    let file = fixture.root.join("launch.svg");
    fs::write(&file, "<svg>launch</svg>").unwrap();
    assert_eq!(
      read_local_custom_icon(&file, "launch").as_deref(),
      Some("<svg>launch</svg>")
    );
    assert!(read_local_custom_icon(&file, "inventory").is_none());
  }

  #[cfg(unix)]
  #[test]
  fn nested_symlink_cannot_escape_the_collection() {
    let fixture = Fixture::new();
    let collection = fixture.root.join("collection");
    let outside = fixture.root.join("outside");
    fs::create_dir(&collection).unwrap();
    fs::create_dir(&outside).unwrap();
    fs::write(outside.join("secret.svg"), "<svg>outside</svg>").unwrap();
    std::os::unix::fs::symlink(&outside, collection.join("linked")).unwrap();
    assert!(read_local_custom_icon(&collection, "linked/secret").is_none());
  }
}
