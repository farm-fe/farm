use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{parse_macro_input, parse_quote, Item};

#[proc_macro_attribute]
pub fn cache_item(attr: TokenStream, input: TokenStream) -> TokenStream {
  let item: Item = parse_macro_input!(input);
  let mut crate_name: Ident = Ident::new("crate", Span::call_site());

  if !attr.is_empty() {
    crate_name = parse_macro_input!(attr);
  }

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
    #item

    impl #crate_name::Cacheable for #item_ident {
      fn serialize_bytes(&self) -> std::result::Result<Vec<u8>, String> {
        let bytes = rkyv::to_bytes::<_, 256>(self).unwrap();
        Ok(bytes.into_vec())
      }

      fn deserialize_bytes(&self, bytes: Vec<u8>) -> std::result::Result<Box<dyn #crate_name::Cacheable>, String> {
        let archived = unsafe { rkyv::archived_root::<#item_ident>(&bytes[..]) };
        let deserialized: #item_ident = archived
          .deserialize(&mut rkyv::de::deserializers::SharedDeserializeMap::new())
          .unwrap();
        Ok(Box::new(deserialized))
      }
    }
  };

  derives.into()
}
