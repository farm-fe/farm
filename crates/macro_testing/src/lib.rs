use std::path::{Path, PathBuf};

use heck::AsSnakeCase;
use proc_macro::TokenStream;
use proc_macro_error::abort;
use quote::quote;

struct Testing {
  pattern: syn::ExprLit,
  handler: syn::Path,
}

#[derive(Debug)]
struct WalkFiles {
  file: PathBuf,
  #[allow(dead_code)]
  cwd: PathBuf,
  base_dir: PathBuf,
}

fn safe_test_name(file: &Path) -> String {
  use regex::Regex;

  let replace_valid_syntax = Regex::new("[^a-zA-Z0-9_]+").unwrap();
  let replace_start_syntax = Regex::new("^[^a-zA-Z]").unwrap();

  replace_start_syntax
    .replace_all(
      &replace_valid_syntax.replace_all(file.to_string_lossy().as_ref(), "_"),
      "_",
    )
    .to_string()
}

impl syn::parse::Parse for Testing {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let path: syn::ExprLit = input.parse()?;
    let _: syn::Token!(,) = input.parse()?;
    let handler: syn::Path = input.parse()?;

    Ok(Testing {
      pattern: path,
      handler,
    })
  }
}

impl Testing {
  fn files(&self) -> Result<Vec<WalkFiles>, &str> {
    // let files = vec![];
    let pattern = match &self.pattern.lit {
      syn::Lit::Str(str) => str.value().to_string(),
      _ => abort!(self.pattern, "expected string literal"),
    };

    let mut files: Vec<WalkFiles> = vec![];

    let base_dir = PathBuf::from(
      std::env::var("CARGO_MANIFEST_DIR").map_err(|_| "failed to get CARGO_MANIFEST_DIR")?,
    );

    let pattern = base_dir.join(&pattern);

    for file in
      glob::glob(pattern.to_string_lossy().as_ref()).map_err(|_| "failed match files")?
    {
      match file {
        Ok(path) => {
          files.push(WalkFiles {
            file: path.clone(),
            cwd: path.parent().unwrap().to_path_buf(),
            base_dir: base_dir.clone(),
          });
        }
        Err(e) => {
          abort!(e.to_string(), "{:?}", e.to_string());
        }
      }
    }

    Ok(files)
  }
}

impl Testing {
  fn to_tokens(&self) -> Result<proc_macro2::TokenStream, &str> {
    let files = self.files()?;

    let mut output = proc_macro2::TokenStream::new();

    let f = &self.handler;

    for WalkFiles { file, base_dir, .. } in files {
      let relative = file.strip_prefix(&base_dir).unwrap();
      let test_name = syn::Ident::new(
        &AsSnakeCase(safe_test_name(relative)).to_string(),
        self.pattern.lit.span(),
      );
      let file = file.to_string_lossy().to_string();
      let base_dir = base_dir.to_string_lossy().to_string();

      output.extend(quote! {
        #[test]
        pub fn #test_name() {
          let test_file = #file;
          let base_dir = #base_dir;

          #f(test_file.to_string(), base_dir.to_string());
        }
      });
    }

    Ok(output)
  }
}

impl From<Testing> for TokenStream {
  fn from(testing: Testing) -> TokenStream {
    match testing.to_tokens() {
      Ok(tokens) => tokens.into(),
      Err(err) => abort!(err, "{}", err),
    }
  }
}

#[proc_macro]
pub fn testing(input: TokenStream) -> TokenStream {
  let testing = syn::parse_macro_input!(input as Testing);

  testing.into()
}
