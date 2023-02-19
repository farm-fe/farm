use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemStruct};

#[proc_macro_attribute]
pub fn farm_plugin(_attr: TokenStream, item: TokenStream) -> TokenStream {
  let item_struct: ItemStruct = parse_macro_input!(item);
  let struct_name = item_struct.ident.clone();

  let ts = quote! {
    #[no_mangle]
    pub fn _plugin_create(config: &farmfe_core::config::Config, options: String) -> std::sync::Arc<dyn farmfe_core::plugin::Plugin> {
      std::sync::Arc::new(#struct_name::new(config, options))
    }

    #[no_mangle]
    pub fn _core_version() -> std::string::String {
      farmfe_core::VERSION.to_string()
    }

    #item_struct
  };

  ts.into()
}
