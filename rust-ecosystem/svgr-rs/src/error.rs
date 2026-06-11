use thiserror::Error;

#[derive(Error, Debug)]
pub enum SvgrError {
  #[error("failed to parse SVG: {0}")]
  Parse(String),
  #[error("this is invalid SVG")]
  InvalidSvg,
  #[error("invalid configuration option: {0}")]
  Configuration(String),
}
