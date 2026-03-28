use std::io::Write;

use farmfe_core::{
  error::{CompilationError, Result},
  resource::resource_pot::ResourcePotType,
};

use crate::CompressAlgorithm;

pub fn compress_buffer(
  buffer: &[u8],
  algorithm: &CompressAlgorithm,
  level: u32,
) -> Result<Vec<u8>> {
  match algorithm {
    CompressAlgorithm::Brotli => brotli_compress(buffer, level),
    CompressAlgorithm::Gzip => gzip_compress(buffer, level),
    CompressAlgorithm::DeflateRaw => deflate_raw_compress(buffer, level),
    CompressAlgorithm::Deflate => deflate_compress(buffer, level),
  }
}

pub fn gzip_compress(buffer: &[u8], level: u32) -> Result<Vec<u8>> {
  let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::new(level));
  encoder
    .write_all(buffer)
    .map_err(|e| CompilationError::GenerateResourcesError {
      name: "gz".to_string(),
      ty: ResourcePotType::Custom("gz".to_string()),
      source: Some(Box::new(e)),
    })?;
  encoder
    .finish()
    .map_err(|e| CompilationError::GenerateResourcesError {
      name: "gz".to_string(),
      ty: ResourcePotType::Custom("gz".to_string()),
      source: Some(Box::new(e)),
    })
}

pub fn deflate_compress(buffer: &[u8], level: u32) -> Result<Vec<u8>> {
  let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::new(level));
  encoder
    .write_all(buffer)
    .map_err(|e| CompilationError::GenerateResourcesError {
      name: "deflate".to_string(),
      ty: ResourcePotType::Custom("deflate".to_string()),
      source: Some(Box::new(e)),
    })?;
  encoder
    .finish()
    .map_err(|e| CompilationError::GenerateResourcesError {
      name: "deflate".to_string(),
      ty: ResourcePotType::Custom("deflate".to_string()),
      source: Some(Box::new(e)),
    })
}

pub fn deflate_raw_compress(buffer: &[u8], level: u32) -> Result<Vec<u8>> {
  let mut encoder = flate2::write::DeflateEncoder::new(Vec::new(), flate2::Compression::new(level));
  encoder
    .write_all(buffer)
    .map_err(|e| CompilationError::GenerateResourcesError {
      name: "deflate".to_string(),
      ty: ResourcePotType::Custom("deflate".to_string()),
      source: Some(Box::new(e)),
    })?;
  encoder
    .finish()
    .map_err(|e| CompilationError::GenerateResourcesError {
      name: "deflate".to_string(),
      ty: ResourcePotType::Custom("deflate".to_string()),
      source: Some(Box::new(e)),
    })
}

pub fn brotli_compress(buffer: &[u8], level: u32) -> Result<Vec<u8>> {
  let mut encoder = brotli::CompressorWriter::new(Vec::new(), 4096, level, 22);
  // let mut encoder = brotli::CompressorWriter::with_params(Vec::new(), 4096, &BrotliEncoderInitParams());
  encoder
    .write_all(buffer)
    .map_err(|e| CompilationError::GenerateResourcesError {
      name: "br".to_string(),
      ty: ResourcePotType::Custom("br".to_string()),
      source: Some(Box::new(e)),
    })?;
  Ok(encoder.into_inner())
}

pub fn get_ext_name(algorithm: &CompressAlgorithm) -> &str {
  match algorithm {
    CompressAlgorithm::Brotli => "br",
    CompressAlgorithm::Gzip => "gz",
    CompressAlgorithm::DeflateRaw | CompressAlgorithm::Deflate => "deflate",
  }
}
