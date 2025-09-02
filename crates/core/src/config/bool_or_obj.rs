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

  pub fn as_obj(&self) -> Option<&T> {
    match self {
      BoolOrObj::Obj(v) => Some(v),
      BoolOrObj::Bool(_) => None,
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

  pub fn unwrap_or_default(self) -> T
  where
    T: Default,
  {
    match self {
      BoolOrObj::Obj(v) => v,
      _ => T::default(),
    }
  }

  pub fn map<R>(self, f: impl FnOnce(T) -> R) -> BoolOrObj<R> {
    match self {
      BoolOrObj::Obj(v) => BoolOrObj::Obj(f(v)),
      BoolOrObj::Bool(bool) => BoolOrObj::Bool(bool),
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

mod tests {
  #[test]
  fn test_bool_or_obj() {
    use super::BoolOrObj;
    use serde_json::json;

    let value = json!(true);
    let value: BoolOrObj<String> = serde_json::from_value(value).unwrap();
    assert!(matches!(value, BoolOrObj::Bool(true)));

    let value = json!("value");
    let value: BoolOrObj<String> = serde_json::from_value(value).unwrap();
    assert!(matches!(value, BoolOrObj::Obj(_)));
    println!("{value:?}");
  }
}
