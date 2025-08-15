use std::{borrow::Cow, collections::HashMap, fmt::Display, fs, path, str::FromStr};

use crate::{
  lang::Lang,
  package_manager::PackageManager,
  utils::{colors::*, lte},
};
use rust_embed::RustEmbed;
use std::any::TypeId;
use std::mem::transmute;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
#[derive(RustEmbed)]
#[folder = "templates"]
#[allow(clippy::upper_case_acronyms, non_camel_case_types)]
struct EMBEDDED_TEMPLATES;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
#[derive(RustEmbed)]
#[folder = "js-templates"]
#[allow(clippy::upper_case_acronyms, non_camel_case_types)]
struct EMBEDDED_JS_TEMPLATES;

pub(crate) trait Displayable {
  fn display_text(&self) -> &str;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElectronSubTemplate {
  React,
  Vue,
  Svelte,
  Vanilla,
  Solid,
  Preact,
}

impl ElectronSubTemplate {
  pub(crate) fn to_simple_string(&self) -> &str {
    match self {
      ElectronSubTemplate::React => "react",
      ElectronSubTemplate::Vue => "vue",
      ElectronSubTemplate::Svelte => "svelte",
      ElectronSubTemplate::Vanilla => "vanilla",
      ElectronSubTemplate::Solid => "solid",
      ElectronSubTemplate::Preact => "preact",
    }
  }
}

impl Displayable for ElectronSubTemplate {
  fn display_text(&self) -> &'static str {
    match self {
      ElectronSubTemplate::React => "\x1b[36mReact - (https://reactjs.org/)\x1b[0m",
      ElectronSubTemplate::Vue => "\x1b[32mVue - (https://vuejs.org/)\x1b[0m",
      ElectronSubTemplate::Svelte => "\x1b[38;2;255;137;54mSvelte - (https://svelte.dev/)\x1b[0m",
      ElectronSubTemplate::Vanilla => "\x1b[33mVanilla\x1b[0m",
      ElectronSubTemplate::Solid => "\x1b[38;2;68;206;246mSolid - (https://solidjs.com/)\x1b[0m",
      ElectronSubTemplate::Preact => "\x1b[36mPreact - (https://preactjs.com/)\x1b[0m",
    }
  }
}

impl Display for ElectronSubTemplate {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    fmt_sub_template(self, f)
  }
}

impl FromStr for ElectronSubTemplate {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let map: HashMap<&str, ElectronSubTemplate> = HashMap::from([
      ("react", ElectronSubTemplate::React),
      ("vue", ElectronSubTemplate::Vue),
      ("vanilla", ElectronSubTemplate::Vanilla),
      ("svelte", ElectronSubTemplate::Svelte),
      ("solid", ElectronSubTemplate::Solid),
      ("preact", ElectronSubTemplate::Preact),
    ]);
    match map.get(s) {
      Some(template) => Ok(*template),
      None => Err(format!("{s} is not a valid Electron template.")),
    }
  }
}

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
  pub(crate) fn to_simple_string(&self) -> &str {
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

impl Displayable for TauriSubTemplate {
  fn display_text(&self) -> &'static str {
    match self {
      TauriSubTemplate::React => "\x1b[36mReact - (https://react.dev/)\x1b[0m",
      TauriSubTemplate::Vue => "\x1b[32mVue - (https://vuejs.org/)\x1b[0m",
      TauriSubTemplate::Svelte => "\x1b[38;2;255;137;54mSvelte - (https://svelte.dev/)\x1b[0m",
      TauriSubTemplate::Vanilla => "\x1b[33mVanilla\x1b[0m",
      TauriSubTemplate::Solid => "\x1b[38;2;68;206;246mSolid - (https://solidjs.com/)\x1b[0m",
      TauriSubTemplate::Preact => "\x1b[36mPreact - (https://preactjs.com/)\x1b[0m",
    }
  }
}

impl Display for TauriSubTemplate {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    fmt_sub_template(self, f)
  }
}

impl FromStr for TauriSubTemplate {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let map: HashMap<&str, TauriSubTemplate> = HashMap::from([
      ("react", TauriSubTemplate::React),
      ("vue", TauriSubTemplate::Vue),
      ("vanilla", TauriSubTemplate::Vanilla),
      ("svelte", TauriSubTemplate::Svelte),
      ("solid", TauriSubTemplate::Solid),
      ("preact", TauriSubTemplate::Preact),
    ]);
    match map.get(s) {
      Some(template) => Ok(*template),
      None => Err(format!("{s} is not a valid Electron template.")),
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
  Nestjs,
  Tauri2(Option<TauriSubTemplate>),
  Tauri(Option<TauriSubTemplate>),
  Electron(Option<ElectronSubTemplate>),
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
      Template::Vue3 => write!(f, "vue3"),
      Template::Vue2 => write!(f, "vue2"),
      Template::Svelte => write!(f, "svelte"),
      Template::Lit => write!(f, "lit"),
      Template::Solid => write!(f, "solid"),
      Template::Preact => write!(f, "preact"),
      Template::Nestjs => write!(f, "nestjs"),
      Template::Tauri2(None) => write!(f, "tauri2"),
      Template::Tauri2(Some(sub_template)) => write!(f, "tauri2-{sub_template}"),
      Template::Tauri(None) => write!(f, "tauri"),
      Template::Tauri(Some(sub_template)) => write!(f, "tauri-{sub_template}"),
      Template::Electron(None) => write!(f, "electron"),
      Template::Electron(Some(sub_template)) => write!(f, "electron-{sub_template}"),
    }
  }
}

impl FromStr for Template {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "vanilla" => Ok(Template::Vanilla),
      "react" => Ok(Template::React),
      "vue3" => Ok(Template::Vue3),
      "vue2" => Ok(Template::Vue2),
      "lit" => Ok(Template::Lit),
      "svelte" => Ok(Template::Svelte),
      "solid" => Ok(Template::Solid),
      "preact" => Ok(Template::Preact),
      "nestjs" => Ok(Template::Nestjs),
      "tauri2" => Ok(Template::Tauri2(None)),
      "tauri" => Ok(Template::Tauri(None)),
      "electron" => Ok(Template::Electron(None)),
      _ => Err(format!(
        "{YELLOW}{s}{RESET} is not a valid template. Valid templates are [{}]",
        Template::ALL_TOP_LEVEL
          .iter()
          .map(|e| format!("{GREEN}{e}{RESET}"))
          .collect::<Vec<_>>()
          .join(", ")
      )),
    }
  }
}

impl Displayable for Template {
  fn display_text(&self) -> &'static str {
    match self {
      Template::Vanilla => "\x1b[33mVanilla\x1b[0m",
      Template::React => "\x1b[36mReact - (https://react.dev/)\x1b[0m",
      Template::Vue3 => "\x1b[32mVue3 - (https://vuejs.org/)\x1b[0m",
      Template::Vue2 => "\x1b[32mVue2 - (https://v2.vuejs.org/)\x1b[0m",
      Template::Solid => "\x1b[38;2;68;206;246mSolid - (https://solidjs.com/)\x1b[0m",
      Template::Svelte => "\x1b[38;2;255;137;54mSvelte - (https://svelte.dev/)\x1b[0m",
      Template::Lit => "\x1b[33mLit - (https://lit.dev/)\x1b[0m",
      Template::Preact => "\x1b[36mPreact - (https://preactjs.com/)\x1b[0m",
      Template::Tauri2(None) => "\x1b[38;2;255;137;54mTauri2 - (https://tauri.app/)\x1b[0m",
      Template::Tauri(None) => "\x1b[38;2;255;137;54mTauri - (https://tauri.app/)\x1b[0m",
      Template::Tauri2(Some(sub_template)) => match sub_template {
        TauriSubTemplate::React => "\x1b[38;2;255;215;0mTauri2 with React\x1b[0m",
        TauriSubTemplate::Vue => "\x1b[38;2;255;215;0mTauri2 with Vue\x1b[0m",
        TauriSubTemplate::Vanilla => "\x1b[38;2;255;215;0mTauri2 with Vanilla\x1b[0m",
        TauriSubTemplate::Svelte => "\x1b[38;2;255;215;0mTauri2 with Svelte\x1b[0m",
        TauriSubTemplate::Solid => "\x1b[38;2;255;215;0mTauri2 with Solid\x1b[0m",
        TauriSubTemplate::Preact => "\x1b[38;2;255;215;0mTauri2 with Preact\x1b[0m",
      },
      Template::Tauri(Some(sub_template)) => match sub_template {
        TauriSubTemplate::React => "\x1b[38;2;255;215;0mTauri with React\x1b[0m",
        TauriSubTemplate::Vue => "\x1b[38;2;255;215;0mTauri with Vue\x1b[0m",
        TauriSubTemplate::Vanilla => "\x1b[38;2;255;215;0mTauri with Vanilla\x1b[0m",
        TauriSubTemplate::Svelte => "\x1b[38;2;255;215;0mTauri with Svelte\x1b[0m",
        TauriSubTemplate::Solid => "\x1b[38;2;255;215;0mTauri with Solid\x1b[0m",
        TauriSubTemplate::Preact => "\x1b[38;2;255;215;0mTauri with Preact\x1b[0m",
      },
      Template::Electron(None) => {
        "\x1b[38;2;255;215;0mElectron - (https://www.electronjs.org/)\x1b[0m"
      }
      Template::Electron(Some(sub_template)) => match sub_template {
        ElectronSubTemplate::React => "\x1b[38;2;255;215;0mElectron with React\x1b[0m",
        ElectronSubTemplate::Vue => "\x1b[38;2;255;215;0mElectron with Vue\x1b[0m",
        ElectronSubTemplate::Vanilla => "\x1b[38;2;255;215;0mElectron with Vanilla\x1b[0m",
        ElectronSubTemplate::Svelte => "\x1b[38;2;255;215;0mElectron with Svelte\x1b[0m",
        ElectronSubTemplate::Solid => "\x1b[38;2;255;215;0mElectron with Solid\x1b[0m",
        ElectronSubTemplate::Preact => "\x1b[38;2;255;215;0mElectron with Preact\x1b[0m",
      },
      Template::Nestjs => "\x1b[31mNestJS - (https://nestjs.com/)\x1b[0m",
    }
  }
}

impl<'a> Template {
  pub(crate) const ALL_TOP_LEVEL: &'a [Template] = &[
    Template::Vanilla,
    Template::React,
    Template::Vue3,
    Template::Vue2,
    Template::Lit,
    Template::Svelte,
    Template::Solid,
    Template::Preact,
    Template::Nestjs,
    Template::Tauri2(None),
    Template::Tauri(None),
    Template::Electron(None),
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

  pub(crate) fn render(
    &self,
    target_dir: &path::Path,
    _pkg_manager: PackageManager,
    project_name: &str,
    package_name: &str,
    lang: &Lang,
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
            let (mut _flags, _name) = (
              s.next().unwrap().split('-').collect::<Vec<_>>(),
              s.next().unwrap(),
            );

            // skip writing this file
            return Ok(());
          }
          name => name,
        };

        let (file_data, file_name) = if let Some(new_name) = file_name.strip_suffix(".lte") {
          let data = match *lang {
            Lang::Javascript => lte::render(
              EMBEDDED_JS_TEMPLATES::get(file).unwrap().data.to_vec(),
              &template_data,
            )?
            .replace("<FARM-TEMPLATE-NAME>", project_name),
            Lang::Typescript => lte::render(
              EMBEDDED_TEMPLATES::get(file).unwrap().data.to_vec(),
              &template_data,
            )?
            .replace("<FARM-TEMPLATE-NAME>", project_name),
          };
          (data.into_bytes(), new_name)
        } else {
          let plain_data = match *lang {
            Lang::Javascript => EMBEDDED_JS_TEMPLATES::get(file).unwrap().data.to_vec(),
            Lang::Typescript => EMBEDDED_TEMPLATES::get(file).unwrap().data.to_vec(),
          };
          let data = String::from_utf8(plain_data.clone())
            .map(|s| {
              s.replace("<FARM-TEMPLATE-NAME>", &project_name)
                .into_bytes()
            })
            .unwrap_or(plain_data);
          (data, file_name)
        };

        let file_name = lte::render(file_name, &template_data)?;

        let parent = p.parent().unwrap();
        fs::create_dir_all(parent)?;
        fs::write(parent.join(file_name), file_data)?;
        Ok(())
      };

    let current_template_name = match self {
      Template::Tauri2(None) => "tauri2".to_string(),
      Template::Tauri2(Some(sub_template)) => format!("tauri2/{}", sub_template.to_simple_string()),
      Template::Tauri(None) => "tauri".to_string(),
      Template::Tauri(Some(sub_template)) => format!("tauri/{}", sub_template.to_simple_string()),
      Template::Electron(None) => "electron".to_string(),
      Template::Electron(Some(sub_template)) => {
        format!("electron/{}", sub_template.to_simple_string())
      }
      _ => self.to_string(),
    };

    let skip_count = current_template_name.matches('/').count() + 1;

    let filter_fn = |e: &Cow<'static, str>| {
      let path = path::PathBuf::from(e.as_ref());
      let _components: Vec<_> = path.components().collect();
      let path_str = path.to_string_lossy();
      path_str.starts_with(&current_template_name)
    };

    match *lang {
      Lang::Javascript => {
        for file in EMBEDDED_JS_TEMPLATES::iter().filter(filter_fn) {
          write_file(file.as_ref(), template_data.clone(), skip_count)?;
        }
      }
      Lang::Typescript => {
        for file in EMBEDDED_TEMPLATES::iter().filter(filter_fn) {
          write_file(file.as_ref(), template_data.clone(), skip_count)?;
        }
      }
    }

    handle_brand_text("\n ✔️ Template copied Successfully! \n");
    Ok(())
  }
}

fn fmt_sub_template<T>(template: &T, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
where
  T: Copy + PartialEq + Eq + 'static,
{
  let type_id = TypeId::of::<T>();

  if type_id == TypeId::of::<ElectronSubTemplate>() {
    let electron_template: &ElectronSubTemplate = unsafe { transmute(template) };
    match electron_template {
      &ElectronSubTemplate::React => write!(f, "\x1b[36mReact - (https://react.dev/)\x1b[0m"),
      &ElectronSubTemplate::Vue => write!(f, "\x1b[32mVue3 - (https://vuejs.org/)\x1b[0m"),
      &ElectronSubTemplate::Svelte => write!(
        f,
        "\x1b[38;2;255;137;54mSvelte - (https://svelte.dev/)\x1b[0m"
      ),
      &ElectronSubTemplate::Vanilla => write!(f, "\x1b[33mVanilla\x1b[0m"),
      &ElectronSubTemplate::Solid => write!(
        f,
        "\x1b[38;2;68;206;246mSolid - (https://solidjs.com/)\x1b[0m"
      ),
      &ElectronSubTemplate::Preact => write!(f, "\x1b[36mPreact - (https://preactjs.com/)\x1b[0m"),
    }
  } else if type_id == TypeId::of::<TauriSubTemplate>() {
    let tauri_template: &TauriSubTemplate = unsafe { transmute(template) };
    match tauri_template {
      &TauriSubTemplate::React => write!(f, "\x1b[36mReact - (https://react.dev/)\x1b[0m"),
      &TauriSubTemplate::Vue => write!(f, "\x1b[32mVue3 - (https://vuejs.org/)\x1b[0m"),
      &TauriSubTemplate::Svelte => write!(
        f,
        "\x1b[38;2;255;137;54mSvelte - (https://svelte.dev/)\x1b[0m"
      ),
      &TauriSubTemplate::Vanilla => write!(f, "\x1b[33mVanilla\x1b[0m"),
      &TauriSubTemplate::Solid => write!(
        f,
        "\x1b[38;2;68;206;246mSolid - (https://solidjs.com/)\x1b[0m"
      ),
      &TauriSubTemplate::Preact => write!(f, "\x1b[36mPreact - (https://preactjs.com/)\x1b[0m"),
    }
  } else {
    Err(std::fmt::Error)
  }
}
