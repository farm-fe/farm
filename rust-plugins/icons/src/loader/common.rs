use crate::cache::HttpClient;
use farmfe_core::regex::{self, Regex};
use farmfe_toolkit::fs::read_file_utf8;
use serde_json::Value;
use std::{
  fs::File,
  io::BufReader,
  path::{Path, PathBuf},
  process::Command,
};
use tokio::runtime::Runtime;
use walkdir::WalkDir;

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
  let (path, _) = path.split_once(".").unzip();
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
  let icons_collection_path = Path::new(&project_dir).join(custom_collection_path);
  let walker = WalkDir::new(icons_collection_path).into_iter();
  let mut filtered_entries = walker.filter_map(Result::ok).filter(|e| {
    e.file_type().is_file()
      && e.path().extension().is_some()
      && e.path().extension().unwrap() == "svg"
  });

  if let Some(entry) = filtered_entries.next() {
    let path = entry.path();
    return read_file_utf8(path.to_str().unwrap()).unwrap();
  }

  String::new()
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
