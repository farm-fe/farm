use farmfe_core::{
  error::{CompilationError, Result},
  serde_json,
};
use farmfe_docs_config::{ColorMode, PublicSiteConfig};
use farmfe_toolkit::hash::sha256;

pub(crate) struct Bootstrap {
  pub name: String,
  pub bytes: Vec<u8>,
  pub url: String,
  pub fixed_mode: Option<&'static str>,
}

impl Bootstrap {
  /// This self-contained browser resource must run before styles and React.
  /// Its only inputs are validated public configuration; no source paths leak.
  pub fn new(site: &PublicSiteConfig) -> Result<Self> {
    let (mode, fixed_mode) = match site.theme.color_mode {
      ColorMode::System => ("system", None),
      ColorMode::Light => ("light", Some("light")),
      ColorMode::Dark => ("dark", Some("dark")),
    };
    let key = serde_json::to_string(&format!("farm-docs:color-mode:{}", site.base_path))
      .map_err(|error| CompilationError::GenericError(format!("[DOCS_COLOR_MODE] {error}")))?;
    let code = format!(
      r#"(function(){{var mode="{mode}",key={key};try{{var saved=localStorage.getItem(key);if(saved==="system"||saved==="light"||saved==="dark")mode=saved;}}catch(error){{}}var resolved=mode;if(mode==="system"){{try{{resolved=matchMedia("(prefers-color-scheme: dark)").matches?"dark":"light";}}catch(error){{resolved="light";}}}}document.documentElement.setAttribute("data-theme",resolved);document.documentElement.style.colorScheme=resolved;}})();"#
    );
    let bytes = code.into_bytes();
    let name = format!("farm-docs-color-mode.{}.js", sha256(&bytes, 16));
    let url = format!("{}{name}", site.base_path);
    Ok(Self {
      name,
      bytes,
      url,
      fixed_mode,
    })
  }
}
