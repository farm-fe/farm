#[allow(unused)]
#[napi_derive::napi]
fn run(args: Vec<String>, bin_name: Option<String>, pkg_manager: Option<String>) {
  create_farm::run(args, bin_name, pkg_manager);
}
