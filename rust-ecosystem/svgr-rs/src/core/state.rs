use std::path::Path;

use lazy_static::lazy_static;
use regex::Regex;

#[derive(Debug, Clone, Default)]
pub struct Caller {
  pub name: Option<String>,
  pub previous_export: Option<String>,
}

#[derive(Debug, Clone)]
/// The state linked to the transformation.
pub struct Config {
  /// The name of the file that is generated, mainly used to find runtime config file to apply.
  pub file_path: Option<String>,

  /// The name of the component that will be used in the generated component.
  pub component_name: Option<String>,

  /// If you create a tool based on SVGR, it is always better to specify `state.caller`.
  /// It permits the inter-operability betweens plugins.
  /// If someone create a SVGR plugin it could adapt it specifically to your tool.
  pub caller: Option<Caller>,
}

impl Default for Config {
  fn default() -> Self {
    Config {
      file_path: None,
      component_name: Some("SvgComponent".to_string()),
      caller: None,
    }
  }
}

#[derive(Debug)]
pub struct InternalConfig {
  #[allow(dead_code)]
  pub file_path: Option<String>,
  pub component_name: String,
  pub caller: Option<Caller>,
}

impl Default for InternalConfig {
  fn default() -> Self {
    InternalConfig {
      file_path: None,
      component_name: "SvgComponent".to_string(),
      caller: None,
    }
  }
}

fn uppercase_first_letter(s: &str) -> String {
  let mut cs = s.chars();
  match cs.next() {
    None => String::new(),
    Some(f) => f.to_uppercase().chain(cs).collect(),
  }
}

const IDENTIFIER: &str = r"([\p{Alpha}\p{N}_]|$)";
const SEPARATORS: &str = r"[_.\- ]+";

fn pascal_case(input: &str) -> String {
  lazy_static! {
    static ref SEPARATORS_AND_IDENTIFIER_REGEX: Regex =
      Regex::new(&format!("{}{}", SEPARATORS, IDENTIFIER)).unwrap();
    static ref NUMBERS_AND_IDENTIFIER_REGEX: Regex =
      Regex::new(&format!("(\\d+){}", IDENTIFIER)).unwrap();
  }

  let result = SEPARATORS_AND_IDENTIFIER_REGEX
    .replace_all(input, |caps: &regex::Captures| {
      let identifier = caps.get(1).unwrap().as_str();
      identifier.to_uppercase()
    })
    .to_string();
  let result = NUMBERS_AND_IDENTIFIER_REGEX
    .replace_all(&result, |caps: &regex::Captures| {
      let num = caps.get(1).unwrap().as_str();
      let identifier = caps.get(2).unwrap().as_str();
      format!("{}{}", num, identifier.to_uppercase())
    })
    .to_string();
  uppercase_first_letter(&result)
}

fn get_component_name(file_path: &str) -> String {
  lazy_static! {
    static ref VALID_CHAR_REGEX_REGEX: Regex = Regex::new(r"[^a-zA-Z0-9 _-]").unwrap();
  }

  let file_name = VALID_CHAR_REGEX_REGEX
    .replace_all(
      Path::new(file_path)
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap(),
      "",
    )
    .to_string();
  let pascal_case_file_name = pascal_case(&file_name);
  format!("Svg{}", pascal_case_file_name)
}

pub fn expand_state(state: &Config) -> InternalConfig {
  InternalConfig {
    file_path: state.file_path.clone(),
    component_name: match state.component_name.clone() {
      Some(component_name) => component_name,
      None => match state.file_path.clone() {
        None => "SvgComponent".to_string(),
        Some(path) => get_component_name(&path),
      },
    },
    caller: state.caller.clone(),
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_1() {
    let internal_config = expand_state(&Default::default());
    assert_eq!(internal_config.component_name, "SvgComponent");
  }

  #[test]
  fn test_2() {
    let input = Config {
      file_path: Some("hello.svg".to_string()),
      component_name: None,
      caller: None,
    };
    let internal_config = expand_state(&input);
    assert_eq!(internal_config.file_path.unwrap(), "hello.svg");
    assert_eq!(internal_config.component_name, "SvgHello");
  }

  #[test]
  fn test_3() {
    let input = Config {
      file_path: Some("hello-you.svg".to_string()),
      component_name: None,
      caller: None,
    };
    let internal_config = expand_state(&input);
    assert_eq!(internal_config.file_path.unwrap(), "hello-you.svg");
    assert_eq!(internal_config.component_name, "SvgHelloYou");
  }

  #[test]
  fn test_4() {
    let input = Config {
      file_path: Some("hello_you.svg".to_string()),
      component_name: None,
      caller: None,
    };
    let internal_config = expand_state(&input);
    assert_eq!(internal_config.file_path.unwrap(), "hello_you.svg");
    assert_eq!(internal_config.component_name, "SvgHelloYou");
  }

  #[test]
  fn test_5() {
    let input = Config {
      file_path: Some("1_big_svg.svg".to_string()),
      component_name: None,
      caller: None,
    };
    let internal_config = expand_state(&input);
    assert_eq!(internal_config.file_path.unwrap(), "1_big_svg.svg");
    assert_eq!(internal_config.component_name, "Svg1BigSvg");
  }

  #[test]
  fn test_6() {
    let input = Config {
      file_path: Some("a&b~c-d_e.svg".to_string()),
      component_name: None,
      caller: None,
    };
    let internal_config = expand_state(&input);
    assert_eq!(internal_config.file_path.unwrap(), "a&b~c-d_e.svg");
    assert_eq!(internal_config.component_name, "SvgAbcDE");
  }

  #[test]
  fn test_7() {
    let input = Config {
      file_path: Some("Arrow up.svg".to_string()),
      component_name: None,
      caller: None,
    };
    let internal_config = expand_state(&input);
    assert_eq!(internal_config.file_path.unwrap(), "Arrow up.svg");
    assert_eq!(internal_config.component_name, "SvgArrowUp");
  }

  #[test]
  fn test_8() {
    let input = Config {
      file_path: Some("Arrow up.svg".to_string()),
      component_name: Some("MyComponent".to_string()),
      caller: None,
    };
    let internal_config = expand_state(&input);
    assert_eq!(internal_config.file_path.unwrap(), "Arrow up.svg");
    assert_eq!(internal_config.component_name, "MyComponent");
  }
}
