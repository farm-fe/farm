use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
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
