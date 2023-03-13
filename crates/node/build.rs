extern crate napi_build;

fn main() {
  // println!("cargo:rustc-cdylib-link-arg=-rdynamic");
  // println!("cargo:rustc-cdylib-link-arg=-symbolic");
  napi_build::setup();
}
