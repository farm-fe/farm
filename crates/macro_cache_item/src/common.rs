use proc_macro2::{Ident, Span};
use syn::parse::Parse;
use syn::punctuated::Punctuated;
use syn::{Expr, ExprParen, PatPath, Path, PathSegment, Token};

#[derive(Debug)]
pub enum Value {
  Ident(Ident),
  Expr(Expr),
}

impl Parse for Value {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    input
      .parse::<Expr>()
      .map(Value::Expr)
      .or_else(|_| input.parse::<Ident>().map(Value::Ident))
  }
}

#[derive(Debug)]
pub enum KeyVal {
  Expr(Expr),
  KeyVal(Ident, Option<Value>),
}

impl Parse for KeyVal {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let key = input.parse::<Value>()?;

    let key = match key {
      Value::Ident(ident) => ident,
      Value::Expr(expr) => return Ok(KeyVal::Expr(expr)),
    };

    if input.parse::<Token![=]>().is_err() {
      return Ok(KeyVal::KeyVal(key, None));
    };

    let value = input.parse::<Value>()?;

    Ok(KeyVal::KeyVal(key, Some(value)))
  }
}

pub struct CacheItemArgs {
  pub crate_name: Ident,
  pub derive: Option<Expr>,
}

impl Parse for CacheItemArgs {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let pairs: Punctuated<KeyVal, Token![,]> = input.parse_terminated(KeyVal::parse, Token![,])?;

    let mut result = CacheItemArgs {
      crate_name: Ident::new("crate", Span::call_site()),
      derive: None,
    };
    for pair in pairs {
      match pair {
        KeyVal::Expr(expr) => match expr {
          Expr::Call(callee) => {
            if let box Expr::Path(syn::ExprPath {
              path: Path { segments, .. },
              ..
            }) = &callee.func
              && let Some(PathSegment { ident, .. }) = segments.get(0)
            {
              if ident == "derive" {
                result.derive = Some(Expr::Call(callee));
              }
            }
          }
          Expr::Assign(assign) => match *assign.left {
            Expr::Path(path) => {
              if let Some(ident) = path.path.get_ident() {
                match ident.to_string().as_str() {
                  "crate_name" => {
                    if let Expr::Path(v) = *assign.right {
                      if let Some(v) = v.path.get_ident() {
                        result.crate_name = v.clone();
                      }
                    }
                  }
                  _ => {}
                }
              }
            }
            _ => {}
          },
          _ => {}
        },
        KeyVal::KeyVal(ident, value) => match ident.to_string().as_str() {
          "crate_name" => {
            if let Some(v) = value {
              if let Value::Ident(v) = v {
                result.crate_name = v;
              }
            }
          }
          _ => {}
        },
      }
      // match pair.key.to_string().as_str() {
      //   "crate_name" => {
      //     if let Some(v) = pair.val {
      //       if let Value::Ident(v) = v {
      //         result.crate_name = v;
      //       }
      //     }
      //   }
      //   "derive" => if let Some(v) = pair.val {},
      //   _ => {}
      // }
    }

    Ok(result)
  }
}
