use std::{collections::HashMap, fmt::Display, fs, io::Write, path, str::FromStr};

use crate::{
  package_manager::PackageManager,
  utils::{colors::*, lte},
};
use rust_embed::RustEmbed;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
#[derive(RustEmbed)]
#[folder = "templates"]
#[allow(clippy::upper_case_acronyms, non_camel_case_types)]
struct EMBEDDED_TEMPLATES;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TauriSubTemplate {
  React,
  Vue,
  Svelte,
  Vanilla,
  Solid,
  Preact,
}

impl Display for TauriSubTemplate {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      TauriSubTemplate::React => write!(f, "react"),
      TauriSubTemplate::Vue => write!(f, "vue"),
      TauriSubTemplate::Svelte => write!(f, "svelte"),
      TauriSubTemplate::Vanilla => write!(f, "vanilla"),
      TauriSubTemplate::Solid => write!(f, "solid"),
      TauriSubTemplate::Preact => write!(f, "preact"),
    }
  }
}

impl FromStr for TauriSubTemplate {
  type Err = String;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "react" => Ok(TauriSubTemplate::React),
      "vue" => Ok(TauriSubTemplate::Vue),
      "vanilla" => Ok(TauriSubTemplate::Vanilla),
      "svelte" => Ok(TauriSubTemplate::Svelte),
      "solid" => Ok(TauriSubTemplate::Solid),
      "preact" => Ok(TauriSubTemplate::Preact),
      _ => Err(format!("{s} is not a valid Tauri sub-template.")),
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Template {
  Vanilla,
  Vue3,
  Vue2,
  Svelte,
  React,
  Solid,
  Preact,
  Tauri(Option<TauriSubTemplate>),
}

impl Default for Template {
  fn default() -> Self {
    Template::Vanilla
  }
}

impl Display for Template {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Template::Vanilla => write!(f, "vanilla"),
      Template::Vue3 => write!(f, "vue"),
      Template::Vue2 => write!(f, "vue2"),
      Template::Svelte => write!(f, "svelte"),
      Template::React => write!(f, "react"),
      Template::Solid => write!(f, "solid"),
      Template::Preact => write!(f, "preact"),
      Template::Tauri(None) => write!(f, "tauri"),
      Template::Tauri(Some(sub_template)) => write!(f, "tauri-{}", sub_template),
    }
  }
}

impl FromStr for Template {
  type Err = String;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    if s == "tauri" {
      return Ok(Template::Tauri(None));
    } else if s.starts_with("tauri-") {
      let sub_template = s[6..].parse()?;
      return Ok(Template::Tauri(Some(sub_template)));
    }

    match s {
      "vanilla" => Ok(Template::Vanilla),
      "vue" => Ok(Template::Vue3),
      "vue2" => Ok(Template::Vue2),
      "svelte" => Ok(Template::Svelte),
      "react" => Ok(Template::React),
      "solid" => Ok(Template::Solid),
      "preact" => Ok(Template::Preact),
      "tauri" => Ok(Template::Tauri(None)),
      _ => Err(format!(
        "{YELLOW}{s}{RESET} is not a valid template. Valid templates are [{}]",
        Template::ALL
          .iter()
          .map(|e| format!("{GREEN}{e}{RESET}"))
          .collect::<Vec<_>>()
          .join(", ")
      )),
    }
  }
}

impl Template {
  pub const fn select_text<'a>(&self) -> &'a str {
    match self {
      Template::Vanilla => "\x1b[33mVanilla\x1b[39m",
      Template::React => "\x1b[36mReact - (https://react.dev/)\x1b[39m",
      Template::Vue3 => "\x1b[32mVue3 - (https://vuejs.org/)\x1b[39m",
      Template::Vue2 => "\x1b[38;2;180;0;100mVue2 - (https://v2.vuejs.org/)\x1b[39m",
      Template::Svelte => "\x1b[38;2;255;137;54mSvelte - (https://svelte.dev/)\x1b[39m",
      Template::Solid => "\x1b[38;2;68;206;246mSolid - (https://solidjs.com/)\x1b[39m",
      Template::Preact => "\x1b[36mPreact - (https://preactjs.com/)\x1b[36m",
      Template::Tauri(None) => "\x1b[38;2;255;215;0mTauri - (https://tauri.app/)\x1b[39m",
      Template::Tauri(Some(sub_template)) => match sub_template {
        TauriSubTemplate::React => "\x1b[38;2;255;215;0mTauri with React\x1b[39m",
        TauriSubTemplate::Vue => "\x1b[38;2;255;215;0mTauri with Vue\x1b[39m",
        TauriSubTemplate::Vanilla => "\x1b[38;2;255;215;0mTauri with Vanilla\x1b[39m",
        TauriSubTemplate::Svelte => "\x1b[38;2;255;215;0mTauri with Svelte\x1b[39m",
        TauriSubTemplate::Solid => "\x1b[38;2;255;215;0mTauri with Solid\x1b[39m",
        TauriSubTemplate::Preact => "\x1b[38;2;255;215;0mTauri with Preact\x1b[39m",
      },
      _ => unreachable!(),
    }
  }
}

impl<'a> Template {
  pub const ALL: &'a [Template] = &[
    Template::Vanilla,
    Template::Vue3,
    Template::Vue2,
    Template::Svelte,
    Template::React,
    Template::Solid,
    Template::Preact,
    Template::Tauri(None),
    Template::Tauri(Some(TauriSubTemplate::React)),
    Template::Tauri(Some(TauriSubTemplate::Vue)),
    Template::Tauri(Some(TauriSubTemplate::Vanilla)),
    Template::Tauri(Some(TauriSubTemplate::Svelte)),
    Template::Tauri(Some(TauriSubTemplate::Solid)),
    Template::Tauri(Some(TauriSubTemplate::Preact)),
  ];

  fn transform_to_pascal_case(s: String) -> String {
    let mut result = String::new();
    let mut capitalize_next = false;
    for (s, c) in s.chars().enumerate() {
      if s == 0 {
        result.push(c.to_ascii_uppercase());
      } else if capitalize_next {
        result.push(c.to_ascii_uppercase());
        capitalize_next = false;
      } else if ['_', '-'].contains(&c) {
        capitalize_next = true;
      } else {
        result.push(c);
      }
    }
    result
  }

  pub fn render(
    &self,
    target_dir: &path::Path,
    pkg_manager: PackageManager,
    project_name: &str,
    package_name: &str,
  ) -> anyhow::Result<()> {
    let lib_name = format!("{}_lib", package_name.replace('-', "_"));
    let project_name_pascal_case = Self::transform_to_pascal_case(project_name.to_string());

    let template_data: HashMap<&str, String> = [
      ("project_name", project_name.to_string()),
      (
        "project_name_pascal_case",
        project_name_pascal_case.to_string(),
      ),
      ("package_name", package_name.to_string()),
      ("lib_name", lib_name),
    ]
    .into();

    let write_file = |file: &str, template_data: HashMap<&str, String>| -> anyhow::Result<()> {
      // remove the first component, which is certainly the template directory they were in before getting embeded into the binary
      let p = path::PathBuf::from(file)
        .components()
        .skip(1)
        .collect::<Vec<_>>()
        .iter()
        .collect::<path::PathBuf>();

      let p = target_dir.join(p);
      let file_name = p.file_name().unwrap().to_string_lossy();

      let file_name = match &*file_name {
        "gitignore" => ".gitignore",
        // skip manifest
        name if name.starts_with("%(") && name[1..].contains(")%") => {
          let mut s = name.strip_prefix("%(").unwrap().split(")%");
          let (mut flags, name) = (
            s.next().unwrap().split('-').collect::<Vec<_>>(),
            s.next().unwrap(),
          );

          // skip writing this file
          return Ok(());
        }
        name => name,
      };

      let (file_data, file_name) = if let Some(new_name) = file_name.strip_suffix(".lte") {
        let data = EMBEDDED_TEMPLATES::get(file).unwrap().data.to_vec();
        let data = lte::render(data, &template_data)?.into_bytes();
        (data, new_name)
      } else {
        let data = EMBEDDED_TEMPLATES::get(file).unwrap().data.to_vec();
        (data, file_name)
      };

      let file_name = lte::render(file_name, &template_data)?;

      let parent = p.parent().unwrap();
      fs::create_dir_all(parent)?;
      fs::write(parent.join(file_name), file_data)?;
      Ok(())
    };

    for file in EMBEDDED_TEMPLATES::iter().filter(|e| {
      path::PathBuf::from(e.to_string())
        .components()
        .next()
        .unwrap()
        .as_os_str()
        == path::PathBuf::from(format!("{self}"))
    }) {
      write_file(&file, template_data.clone())?;
    }

    println!("✔️ Template copied Successfully!");

    Ok(())
  }
}
