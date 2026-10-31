pub use farmfe_core::Cacheable;

#[farmfe_macro_cache_item::cache_item]
#[derive(Debug, PartialEq)]
pub struct Payload {
    value: u32,
    label: String,
}

#[farmfe_macro_plugin::farm_plugin]
pub struct SdkPlugin;

impl SdkPlugin {
    pub fn new(_: &farmfe_core::config::Config, _: String) -> Self {
        Self
    }
}
impl farmfe_core::plugin::Plugin for SdkPlugin {
    fn name(&self) -> &str {
        "independent-sdk-probe"
    }
}

#[test]
fn real_core_plugin_macro_and_abi() {
    assert_eq!(_core_version(), "0.5.4");
    let plugin = _plugin_create(&farmfe_core::config::Config::default(), String::new());
    assert_eq!(plugin.name(), "independent-sdk-probe");
}

#[test]
fn real_core_cache_round_trip() {
    let expected = Payload { value: 42, label: "SDK".into() };
    let bytes = expected.serialize_bytes().unwrap();
    let actual = Payload::deserialize_bytes(bytes).unwrap();
    assert_eq!(actual.downcast_ref::<Payload>().unwrap(), &expected);
}

#[test]
fn compiler_toolkit_and_glob_public_surfaces() {
    assert!(std::mem::size_of::<Option<farmfe_compiler::Compiler>>() > 0);
    let atom: farmfe_toolkit::swc_atoms::Atom = "sdk".into();
    let text: &str = atom.as_ref();
    assert_eq!(text, "sdk");
    let glob = farmfe_core::glob::Glob::new("src/{main,lib}.rs").unwrap();
    assert!(glob.is_match("src/main.rs").unwrap());
    assert!(!glob.is_match("src/other.rs").unwrap());
}
