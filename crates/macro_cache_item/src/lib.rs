use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;
use syn::{parse_macro_input, parse_quote, Item};

#[proc_macro_attribute]
pub fn cache_item(attr: TokenStream, input: TokenStream) -> TokenStream {
  let item: Item = parse_macro_input!(input);

  if !attr.is_empty() {
    let args: Ident = parse_macro_input!(attr);

    let item_ident = match &item {
      Item::Enum(e) => e.ident.clone(),
      Item::Struct(s) => s.ident.clone(),
      _ => {
        let ts: proc_macro2::TokenStream = parse_quote! {
          compile_error!("#[cache_item] can only be used on struct or enum")
        };
        return ts.into();
      }
    };
    let derives = quote! {
      #[derive(rkyv::Serialize, rkyv::Deserialize, rkyv::Archive)]
      #[archive_attr(derive(TypeName))]
      #item

      #[archive_dyn(deserialize)]
      impl #args for #item_ident {}
      impl #args for rkyv::Archived<#item_ident> {}
    };

    return derives.into();
  }

  let derives = quote! {
    #[derive(rkyv::Serialize, rkyv::Deserialize, rkyv::Archive)]
    #item
  };

  derives.into()
}
