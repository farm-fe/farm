use bincode::{config, Decode, Encode};
use cached::stores::DiskCache;
use loading::Loading;
use reqwest::{header::CACHE_CONTROL, Error, Response};
use std::time::{Duration, SystemTime};

#[derive(Encode, Decode, Debug)]
struct CacheValue {
  data: String,
  expiration: SystemTime,
}

pub struct HttpClient {
  cache: Option<DiskCache<String, Vec<u8>>>,
}

fn try_build_disk_cache(cache_name: &str, cache_dir: &str) -> Option<DiskCache<String, Vec<u8>>> {
  const MAX_RETRIES: u32 = 5;
  let mut delay = Duration::from_millis(50);

  for attempt in 0..=MAX_RETRIES {
    match DiskCache::new(cache_name)
      .set_disk_directory(cache_dir)
      .build()
    {
      Ok(cache) => return Some(cache),
      Err(e) => {
        if attempt < MAX_RETRIES {
          std::thread::sleep(delay);
          delay *= 2;
        } else {
          eprintln!(
            "[farm-plugin-icons] Warning: could not open disk cache after {MAX_RETRIES} retries: {e}. \
             Continuing without disk cache."
          );
        }
      }
    }
  }
  None
}

impl HttpClient {
  pub fn new(cache_name: &str, cache_dir: &str) -> Self {
    let cache = try_build_disk_cache(cache_name, cache_dir);
    HttpClient { cache }
  }

  pub async fn fetch_data(&self, url: &str) -> Result<String, Error> {
    let loading = Loading::default();
    let config = config::standard();

    if let Some(disk_cache) = &self.cache {
      if let Ok(Some(entry)) = disk_cache.connection().get(url) {
        if let Ok((cached_value, _)) = bincode::decode_from_slice::<CacheValue, _>(&entry, config) {
          if cached_value.expiration > SystemTime::now() {
            loading.success(format!("{url} icon fetched from cache"));
            loading.end();
            return Ok(cached_value.data);
          } else {
            // Remove expired cache entry; ignore errors
            let _ = disk_cache.connection().remove(url);
          }
        }
      }
    }

    loading.text(format!("Fetching {url} icon from network..."));
    let result = reqwest::get(url).await;
    match result {
      Ok(response) => {
        if response.status().is_success() {
          let cache_duration = get_cache_duration(&response).unwrap_or(Duration::from_secs(60));
          let text = response.text().await?;
          loading.success(format!("{url} icon fetched from network"));
          loading.end();

          if let Some(disk_cache) = &self.cache {
            let cache_value = CacheValue {
              data: text.clone(),
              expiration: SystemTime::now() + cache_duration,
            };
            if let Ok(serialized_data) = bincode::encode_to_vec(&cache_value, config) {
              // Ignore cache write errors — non-fatal
              let _ = disk_cache.connection().insert(url, serialized_data);
            }
          }

          Ok(text)
        } else {
          let status = response.status();
          loading.fail(format!("{url} icon fetch err: {status:?}"));
          loading.end();
          Err(response.error_for_status().unwrap_err())
        }
      }
      Err(e) => {
        loading.fail(format!("{url} icon fetch err: {e:?}"));
        loading.end();
        Err(e)
      }
    }
  }
}

fn get_cache_duration(response: &Response) -> Option<Duration> {
  if let Some(cache_control) = response.headers().get(CACHE_CONTROL) {
    if let Ok(cache_control_str) = cache_control.to_str() {
      for directive in cache_control_str.split(',') {
        let directive = directive.trim();
        if let Some(value) = directive.strip_prefix("max-age=") {
          if let Ok(seconds) = value.parse::<u64>() {
            return Some(Duration::from_secs(seconds));
          }
        }
      }
    }
  }
  None
}
