use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum BoolOrObj<T> {
  Bool(bool),
  Obj(T),
}

impl<T> BoolOrObj<T> {
  pub fn enabled(&self) -> bool {
    match self {
      BoolOrObj::Bool(b) => *b,
      BoolOrObj::Obj(_) => true,
    }
  }

  pub fn unwrap_as_option<F>(self, default: F) -> Option<T>
  where
    F: FnOnce(Option<bool>) -> Option<T>,
  {
    match self {
      BoolOrObj::Obj(v) => Some(v),
      BoolOrObj::Bool(b) => default(Some(b)),
    }
  }
}

impl<T> Default for BoolOrObj<T> {
  fn default() -> Self {
    Self::Bool(true)
  }
}

impl<T> From<bool> for BoolOrObj<T> {
  fn from(value: bool) -> Self {
    Self::Bool(value)
  }
}
