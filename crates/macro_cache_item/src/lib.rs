use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{parse_macro_input, parse_quote, Ident, Item};

#[proc_macro_attribute]
pub fn cache_item(_attr: TokenStream, input: TokenStream) -> TokenStream {
  let item: Item = parse_macro_input!(input);

  let derives = quote! {
    use rkyv::*;

    #[derive(rkyv::Serialize, rkyv::Deserialize, rkyv::Archive)]
    #item
  };

  derives.into()
}

#[proc_macro_attribute]
pub fn custom_meta_data(attr: TokenStream, input: TokenStream) -> TokenStream {
  let item: Item = parse_macro_input!(input);
  let args: Ident = if !attr.is_empty() {
    parse_macro_input!(attr)
  } else {
    Ident::new("CustomModuleMetaData", Span::call_site())
  };

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
    use rkyv::*;

    #[derive(rkyv::Serialize, rkyv::Deserialize, rkyv::Archive)]
    #[archive_attr(derive(TypeName))]
    #item

    #[archive_dyn(deserialize)]
    impl #args for #item_ident {}
    impl #args for rkyv::Archived<#item_ident> {}
  };

  derives.into()
}
