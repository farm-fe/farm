macro_rules! otr {
  ($e:expr, $err:expr) => {
    match $e {
      Some(v) => Ok(v),
      None => Err($err),
    }
  };
}

pub(super) use otr;
