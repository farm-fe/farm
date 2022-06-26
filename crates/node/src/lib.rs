#![deny(clippy::all)]

#[macro_use]
extern crate napi_derive;

fn sum(a: i32, b: i32) -> i32 {
  a + b
}

#[napi]
pub struct JsCompiler {}

#[napi]
impl JsCompiler {
  #[napi]
  pub fn js_sum(&self, a: i32, b: i32) -> napi::Result<i32> {
    Ok(sum(a, b))
  }
}
