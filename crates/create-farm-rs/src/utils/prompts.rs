use anyhow::Result;

use crate::template::Displayable;

use super::theme::ColorfulTheme;

pub(crate) fn select<'t, T: Displayable>(
  prompt: &str,
  items: &'t [T],
  default: Option<usize>,
) -> Result<&'t T> {
  let theme = ColorfulTheme::default();
  let mut builder = dialoguer::Select::with_theme(&theme)
    .with_prompt(prompt)
    .items(
      &items
        .iter()
        .map(|i| i.display_text())
        .collect::<Vec<&str>>(),
    );
  if let Some(default) = default {
    builder = builder.default(default);
  }
  let selected = builder.interact()?;
  Ok(items.get(selected).unwrap())
}

pub(crate) fn multi_select<'t, I, T>(
  prompt: &str,
  items: I,
  default: Option<&[bool]>,
) -> Result<Vec<T>>
where
  I: IntoIterator<Item = T>,
  T: Displayable,
{
  let items = items.into_iter().collect::<Vec<T>>();
  let theme = ColorfulTheme::default();
  let mut builder = dialoguer::MultiSelect::with_theme(&theme)
    .with_prompt(prompt)
    .items(
      &items
        .iter()
        .map(|i| i.display_text())
        .collect::<Vec<&str>>(),
    );
  if let Some(default) = default {
    builder = builder.defaults(default);
  }
  let selected = builder.interact()?;
  Ok(
    items
      .into_iter()
      .enumerate()
      .filter_map(|(i, item)| {
        if selected.contains(&i) {
          Some(item)
        } else {
          None
        }
      })
      .collect(),
  )
}

pub(crate) fn input(prompt: &str, default: Option<&str>, allow_empty: bool) -> Result<String> {
  let theme = ColorfulTheme::default();
  let mut builder = dialoguer::Input::with_theme(&theme)
    .with_prompt(prompt)
    .allow_empty(allow_empty);
  if let Some(default) = default {
    builder = builder.default(default.to_string());
  }
  Ok(builder.interact_text()?)
}

pub(crate) fn confirm(prompt: &str, default: bool) -> Result<bool> {
  let theme = ColorfulTheme::default();
  let builder = dialoguer::Confirm::with_theme(&theme)
    .with_prompt(prompt)
    .default(default);
  Ok(builder.interact()?)
}
