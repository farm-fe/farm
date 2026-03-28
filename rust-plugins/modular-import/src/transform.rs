use farmfe_core::{
  error::Result as HookResult,
  plugin::{PluginTransformHookParam, PluginTransformHookResult},
};
use regex::Regex;
// use mod::{ CompilerParams};
/**
 * Replaces the import module path with the specified library name and converts the component name to camel case or kebab case based on the configuration options.
 *
 * @param options - The configuration options for the plugin.
 * @param param - The parameters for the transform hook.
 * @return If the content is modified, returns a `PluginTransformHookResult` containing the modified content; otherwise, returns `None`.
 */
pub fn transform(
  options: &crate::options::Options,
  param: &PluginTransformHookParam,
) -> HookResult<Option<PluginTransformHookResult>> {
  // Clone the content to avoid ownership issues
  let content = param.content.clone();

  // Check if the library_name configuration exists
  if let Some(name) = &options.library_name {
    // Create a regex pattern to match modules imported from the specified library_name
    let import_regex_pattern = format!(
      r#"import\s*\{{\s*(\w+)\s*\}}\s*from\s*['"]{}['"]\s*;?"#,
      regex::escape(name)
    );
    // Create a Regex instance using the regex pattern
    let import_regex = Regex::new(&import_regex_pattern).expect("Failed to create regex");

    if import_regex.is_match(&content) {
      // Replace all matching import statements using the regex
      let modified_content = import_regex.replace_all(&content, |caps: &regex::Captures| {
        // Get the component name
        let component_name = &caps[1];

        // Convert the component name to camel case or kebab case based on the configuration options
        let formatted_component_name = if options.camel2_dash.unwrap_or(true) {
          format!(
            "{}{}",
            &component_name[..1].to_uppercase(),
            &component_name[1..]
          )
        } else {
          format!(
            "{}{}",
            &component_name[..1].to_lowercase(),
            &component_name[1..]
          )
        };

        // Build the path to the style file
        let style_path = build_style_path(
          name,
          &options.style_lib_dir,
          &options.style_library_name,
          &formatted_component_name,
          &options.style_library_path,
        );

        // Generate the new import statement, including the import of the component and the style file
        format!(
          "import {} from '{}/{}/{}';\nimport '{}';\n",
          component_name,
          name,
          options.lib_dir.as_deref().unwrap_or_default(),
          formatted_component_name,
          style_path
        )
      });

      // println!("Modified content with replacement: {}", modified_content);
      // Return a `PluginTransformHookResult` containing the modified content
      return Ok(Some(PluginTransformHookResult {
        content: modified_content.to_string(),
        module_type: Some(param.module_type.clone()),
        source_map: None,
        ignore_previous_source_map: true,
      }));
    }
  } else {
    // If the library_name configuration does not exist, print an error message and terminate the program
    eprintln!("\x1B[31mError: @farmfe/plugin-modular-import library_name is missing\x1B[0m");
    panic!();
  }
  // If the content is not modified, return `None`
  Ok(None)
}

/**
 * Builds the path to the style file based on the configuration options.
 *
 * @param name - The library name.
 * @param style_lib_dir - The directory where the style library is located.
 * @param style_library_name - The name of the style library.
 * @param formatted_component_name - The formatted component name.
 * @param style_library_path - The path to the style file within the style library.
 * @return The path to the style file.
 */
fn build_style_path(
  name: &str,
  style_lib_dir: &Option<String>,
  style_library_name: &Option<String>,
  formatted_component_name: &str,
  style_library_path: &Option<String>,
) -> String {
  if let Some(style_library_name) = style_library_name {
    if is_dot_path(style_library_path) {
      format!(
        "{}/{}/{}/{}",
        name,
        style_lib_dir.as_deref().unwrap_or_default(),
        style_library_name,
        format!(
          "{}{}",
          formatted_component_name,
          style_library_path.as_deref().unwrap_or_default()
        )
      )
    } else {
      format!(
        "{}/{}/{}/{}/{}",
        name,
        style_lib_dir.as_deref().unwrap_or_default(),
        style_library_name,
        formatted_component_name,
        style_library_path.as_deref().unwrap_or_default()
      )
    }
  } else {
    if is_dot_path(&style_library_path) {
      format!(
        "{}/{}/{}",
        name,
        style_lib_dir.as_deref().unwrap_or_default(),
        format!(
          "{}{}",
          formatted_component_name,
          style_library_path.as_deref().unwrap_or_default()
        )
      )
    } else {
      format!(
        "{}/{}/{}/{}",
        name,
        style_lib_dir.as_deref().unwrap_or_default(),
        formatted_component_name,
        style_library_path.as_deref().unwrap_or_default()
      )
    }
  }
}

/**
 * Checks if the given path starts with a dot.
 *
 * @param path - The path to check.
 * @return True if the path starts with a dot, false otherwise.
 */
fn is_dot_path(path: &Option<String>) -> bool {
  path.as_ref().map_or(false, |s| s.starts_with('.'))
}
