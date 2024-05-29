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

impl TauriSubTemplate {
  pub fn to_simple_string(&self) -> &str {
    match self {
      TauriSubTemplate::React => "react",
      TauriSubTemplate::Vue => "vue",
      TauriSubTemplate::Svelte => "svelte",
      TauriSubTemplate::Vanilla => "vanilla",
      TauriSubTemplate::Solid => "solid",
      TauriSubTemplate::Preact => "preact",
    }
  }
}

impl Display for TauriSubTemplate {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      TauriSubTemplate::React => write!(f, "\x1b[36mReact - (https://react.dev/)\x1b[39m"),
      TauriSubTemplate::Vue => write!(f, "\x1b[32mVue3 - (https://vuejs.org/)\x1b[39m"),
      TauriSubTemplate::Svelte => write!(
        f,
        "\x1b[38;2;255;137;54mSvelte - (https://svelte.dev/)\x1b[39m"
      ),
      TauriSubTemplate::Vanilla => write!(f, "\x1b[33mVanilla\x1b[39m"),
      TauriSubTemplate::Solid => write!(
        f,
        "\x1b[38;2;68;206;246mSolid - (https://solidjs.com/)\x1b[39m"
      ),
      TauriSubTemplate::Preact => write!(f, "\x1b[36mPreact - (https://preactjs.com/)\x1b[36m"),
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
      _ => Err(format!("{s} is not a valid Tauri template.")),
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Template {
  Vanilla,
  React,
  Vue3,
  Vue2,
  Lit,
  Svelte,
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
      Template::React => write!(f, "react"),
      Template::Vue3 => write!(f, "vue"),
      Template::Vue2 => write!(f, "vue2"),
      Template::Svelte => write!(f, "svelte"),
      Template::Lit => write!(f, "lit"),
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
    match s {
      "vanilla" => Ok(Template::Vanilla),
      "react" => Ok(Template::React),
      "vue" => Ok(Template::Vue3),
      "vue2" => Ok(Template::Vue2),
      "lit" => Ok(Template::Lit),
      "svelte" => Ok(Template::Svelte),
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
      Template::Vue2 => "\x1b[38;2;255;102;102mVue2 - (https://v2.vuejs.org/)\x1b[39m",
      Template::Svelte => "\x1b[38;2;255;137;54mSvelte - (https://svelte.dev/)\x1b[39m",
      Template::Solid => "\x1b[38;2;68;206;246mSolid - (https://solidjs.com/)\x1b[39m",
      Template::Lit => "\x1b[38;2;255;102;102mLit - (https://lit.dev/)\x1b[39m",
      Template::Preact => "\x1b[36mPreact - (https://preactjs.com/)\x1b[36m",
      Template::Tauri(None) => "\x1b[38;2;255;137;54mTauri - (https://tauri.app/)\x1b[39m",
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
    Template::React,
    Template::Vue3,
    Template::Vue2,
    Template::Lit,
    Template::Svelte,
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

    let write_file =
      |file: &str, template_data: HashMap<&str, String>, skip_count: usize| -> anyhow::Result<()> {
        // remove the first component, which is certainly the template directory they were in before getting embeded into the binary
        let p = path::PathBuf::from(file)
          .components()
          .skip(skip_count)
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

    let current_template_name = match self {
      Template::Tauri(None) => "tauri".to_string(),
      Template::Tauri(Some(sub_template)) => format!("tauri/{}", sub_template.to_simple_string()),
      _ => self.to_string(),
    };

    let skip_count = current_template_name.matches('/').count() + 1;
    for file in EMBEDDED_TEMPLATES::iter().filter(|e| {
      let path = path::PathBuf::from(e.to_string());
      let components: Vec<_> = path.components().collect();
      let path_str = path.to_string_lossy();
      // let template_name = components.first().unwrap().as_os_str().to_str().unwrap();
      path_str.starts_with(&current_template_name)
    }) {
      write_file(&file, template_data.clone(), skip_count)?;
    }

    handle_brand_text("\n ✔️ Template copied Successfully! \n");
    Ok(())
  }
}
