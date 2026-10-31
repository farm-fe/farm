/// A development document has no server-rendered page state. The native HTML
/// plugin follows the public client entry through its normal script dependency.
pub(crate) const VIRTUAL_ID: &str = "virtual:farm-docs:development";

pub(crate) fn html() -> String {
  concat!(
    "<!doctype html><html><head><meta charset=\"utf-8\">",
    "<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">",
    "<meta name=\"farm-docs-page-styles\" content=\"__FARM_DOCS_STYLE_LINKS__\">",
    "</head><body><div id=\"__FARM_DOCS_ROOT__\"></div>",
    "<script type=\"module\" src=\"@farmfe/docs/dev-client\"></script>",
    "</body></html>"
  )
  .into()
}
